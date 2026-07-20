use network_manager::{
    AccessPoint, AccessPointCredentials, Connection, DeviceType, NetworkManager, Security,
};
use network_manager::wifi::WiFiDevice;
use network_manager::errors::Error as NmLibError;

use super::cli;
use super::types::{
    ConnectionInfo, DeviceInfo, IpConfig, NetworkSnapshot, NmError, Result, WifiNetwork,
};

pub struct NetworkService {
    manager: NetworkManager,
}

impl NetworkService {
    pub fn new() -> Self {
        Self {
            manager: NetworkManager::with_method_timeout(30),
        }
    }

    pub fn snapshot(&self, rescan_wifi: bool) -> Result<NetworkSnapshot> {
        let wifi_enabled = self
            .manager
            .is_wireless_enabled()
            .map_err(map_err)?;
        let devices = self.manager.get_devices().map_err(map_err)?;
        let connections = self.manager.get_connections().map_err(map_err)?;
        let active = self.manager.get_active_connections().map_err(map_err)?;

        let active_names: Vec<String> = active
            .iter()
            .map(|c| c.settings().id.clone())
            .collect();

        let mut device_infos = Vec::new();
        let mut connected = false;
        let mut connection_name = String::new();
        let mut ip4 = String::new();
        let mut gateway = String::new();
        let mut device_name = String::new();
        let mut connection_type = String::new();

        for device in &devices {
            if device.interface() == "lo" {
                continue;
            }

            let state = format!("{:?}", device.get_state().map_err(map_err)?);
            let iface = device.interface().to_string();
            let ip = cli::device_ip4(&iface);

            let active_conn = active
                .iter()
                .find(|c| {
                    c.get_devices()
                        .map(|devs| devs.iter().any(|d| d.interface() == iface))
                        .unwrap_or(false)
                })
                .map(|c| c.settings().id.clone())
                .unwrap_or_default();

            if state.contains("Activated") || state.contains("Connected") {
                connected = true;
                connection_name = active_conn.clone();
                ip4 = ip.clone();
                gateway = cli::device_gateway(&iface);
                device_name = iface.clone();
                connection_type = format!("{:?}", device.device_type());
            }

            device_infos.push(DeviceInfo {
                name: iface,
                device_type: format!("{:?}", device.device_type()),
                state,
                connection: active_conn,
                ip4: ip,
            });
        }

        let saved_wifi: Vec<(String, String)> = connections
            .iter()
            .filter(|c| c.settings().kind == "802-11-wireless")
            .filter_map(|c| {
                c.settings()
                    .ssid
                    .as_str()
                    .ok()
                    .map(|ssid| (ssid.to_string(), c.settings().id.clone()))
            })
            .collect();

        let connection_infos: Vec<ConnectionInfo> = connections
            .iter()
            .filter(|c| c.settings().kind != "loopback" && c.settings().kind != "bridge")
            .map(|c| {
                let ssid = c
                    .settings()
                    .ssid
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                ConnectionInfo {
                    name: c.settings().id.clone(),
                    uuid: c.settings().uuid.clone(),
                    connection_type: c.settings().kind.clone(),
                    device: c
                        .get_devices()
                        .ok()
                        .and_then(|devs| devs.first().map(|d| d.interface().to_string()))
                        .unwrap_or_default(),
                    active: active_names.contains(&c.settings().id),
                    ssid,
                }
            })
            .collect();

        let wifi_networks = if wifi_enabled {
            self.with_wifi_device(|wifi| self.collect_wifi(wifi, rescan_wifi, &saved_wifi))?
        } else {
            Vec::new()
        };

        let hotspot_active = connection_infos.iter().any(|c| {
            c.connection_type == "802-11-wireless" && c.name.to_ascii_lowercase().contains("hotspot")
        });
        let hotspot_ssid = connection_infos
            .iter()
            .find(|c| c.active && c.connection_type == "802-11-wireless")
            .map(|c| c.ssid.clone())
            .unwrap_or_default();

        Ok(NetworkSnapshot {
            wifi_enabled,
            connected,
            connection_name,
            ip4,
            gateway,
            device: device_name,
            connection_type,
            devices: device_infos,
            connections: connection_infos,
            wifi_networks,
            hotspot_active,
            hotspot_ssid,
        })
    }

    fn with_wifi_device<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&WiFiDevice<'_>) -> Result<T>,
    {
        let devices = self.manager.get_devices().map_err(map_err)?;
        let device = devices
            .iter()
            .find(|d| *d.device_type() == DeviceType::WiFi)
            .ok_or_else(|| NmError::Message("No WiFi device found".into()))?;
        let wifi = device
            .as_wifi_device()
            .ok_or_else(|| NmError::Message("Device is not WiFi".into()))?;
        f(&wifi)
    }

    fn collect_wifi(
        &self,
        wifi: &WiFiDevice<'_>,
        rescan: bool,
        saved: &[(String, String)],
    ) -> Result<Vec<WifiNetwork>> {
        if rescan {
            let _ = wifi.request_scan();
        }

        let access_points = wifi.get_access_points().map_err(map_err)?;
        let active_ssid = self
            .manager
            .get_active_connections()
            .ok()
            .and_then(|conns| {
                conns.first().and_then(|c| c.settings().ssid.as_str().ok().map(str::to_string))
            })
            .unwrap_or_default();

        let mut networks: Vec<WifiNetwork> = access_points
            .iter()
            .filter_map(|ap| {
                let ssid = ap.ssid().as_str().ok()?.to_string();
                if ssid.is_empty() {
                    return None;
                }
                let saved_entry = saved.iter().find(|(s, _)| s == &ssid);
                Some(WifiNetwork {
                    saved: saved_entry.is_some(),
                    connection_name: saved_entry.map(|(_, name)| name.clone()),
                    ssid: ssid.clone(),
                    signal: ap.strength.min(100) as u8,
                    security: security_label(ap.security),
                    in_use: ssid == active_ssid,
                })
            })
            .collect();

        networks.sort_by(|a, b| b.signal.cmp(&a.signal));
        networks.dedup_by(|a, b| a.ssid == b.ssid);
        Ok(networks)
    }

    pub fn set_wifi_enabled(&self, enabled: bool) -> Result<()> {
        // network-manager crate lacks radio toggle; use nmcli
        let state = if enabled { "on" } else { "off" };
        cli::run_nmcli_simple(&["radio", "wifi", state])
    }

    pub fn connect_wifi(&self, ssid: &str, password: Option<&str>) -> Result<()> {
        if password.is_none() {
            let connections = self.manager.get_connections().map_err(map_err)?;
            if let Some(conn) = connections.iter().find(|c| {
                c.settings().kind == "802-11-wireless"
                    && c.settings().ssid.as_str().ok() == Some(ssid)
            }) {
                conn.activate().map_err(map_err)?;
                return Ok(());
            }
        }

        self.with_wifi_device(|wifi| {
            if password.is_none() {
                let _ = wifi.request_scan();
            }
            let access_points = wifi.get_access_points().map_err(map_err)?;
            let ap = access_points
                .iter()
                .find(|ap| ap.ssid().as_str().ok() == Some(ssid))
                .ok_or(NmError::NotFound)?;

            let credentials = credentials_for(ap, password)?;
            wifi.connect(ap, &credentials).map_err(map_err)?;
            Ok(())
        })
    }

    pub fn disconnect_wifi(&self) -> Result<()> {
        let devices = self.manager.get_devices().map_err(map_err)?;
        let device = devices
            .iter()
            .find(|d| *d.device_type() == DeviceType::WiFi)
            .ok_or_else(|| NmError::Message("No WiFi device".into()))?;
        device.disconnect().map_err(map_err)?;
        Ok(())
    }

    pub fn forget_wifi(&self, ssid: &str) -> Result<()> {
        let connections = self.manager.get_connections().map_err(map_err)?;
        let conn = connections
            .iter()
            .find(|c| {
                c.settings().kind == "802-11-wireless"
                    && c.settings().ssid.as_str().ok() == Some(ssid)
            })
            .ok_or(NmError::NotFound)?;
        conn.delete().map_err(map_err)?;
        Ok(())
    }

    pub fn activate_connection(&self, uuid: &str) -> Result<()> {
        let conn = self.find_connection(uuid)?;
        conn.activate().map_err(map_err)?;
        Ok(())
    }

    pub fn deactivate_connection(&self, uuid: &str) -> Result<()> {
        let conn = self.find_connection(uuid)?;
        conn.deactivate().map_err(map_err)?;
        Ok(())
    }

    pub fn read_ip_config(&self, uuid: &str) -> Result<IpConfig> {
        cli::read_ipv4_config(uuid)
    }

    pub fn save_ip_config(&self, uuid: &str, config: &IpConfig) -> Result<()> {
        cli::write_ipv4_config(uuid, config)?;
        self.activate_connection(uuid)
    }

    pub fn create_hotspot(&self, ssid: &str, password: &str) -> Result<()> {
        self.with_wifi_device(|wifi| {
            let pass = if password.is_empty() {
                None
            } else {
                Some(password)
            };
            wifi.create_hotspot(ssid, pass, None).map_err(map_err)?;
            Ok(())
        })
    }

    pub fn stop_hotspot(&self) -> Result<()> {
        self.disconnect_wifi()
    }

    fn find_connection(&self, uuid: &str) -> Result<Connection> {
        self.manager
            .get_connections()
            .map_err(map_err)?
            .into_iter()
            .find(|c| c.settings().uuid == uuid)
            .ok_or(NmError::NotFound)
    }
}

impl Default for NetworkService {
    fn default() -> Self {
        Self::new()
    }
}

fn security_label(security: Security) -> String {
    if security == Security::NONE {
        "Open".into()
    } else if security.contains(Security::WPA2) {
        "WPA2".into()
    } else if security.contains(Security::WPA) {
        "WPA".into()
    } else if security.contains(Security::WEP) {
        "WEP".into()
    } else {
        "Secured".into()
    }
}

fn credentials_for(ap: &AccessPoint, password: Option<&str>) -> Result<AccessPointCredentials> {
    match password {
        Some(pass) if !pass.is_empty() => Ok(AccessPointCredentials::Wpa {
            passphrase: pass.to_string(),
        }),
        None if ap.security == Security::NONE => Ok(AccessPointCredentials::None),
        None => Err(NmError::AuthRequired),
        Some(_) => Ok(AccessPointCredentials::None),
    }
}

fn map_err(error: NmLibError) -> NmError {
    let msg = format!("{error}");
    if msg.to_ascii_lowercase().contains("secret") || msg.to_ascii_lowercase().contains("password") {
        NmError::AuthRequired
    } else {
        NmError::Message(msg)
    }
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal: u8,
    pub security: String,
    pub in_use: bool,
    pub bssid: String,
    pub saved: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkDevice {
    pub name: String,
    pub device_type: String,
    pub state: String,
    pub connection: String,
    pub ip4: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedConnection {
    pub name: String,
    pub uuid: String,
    pub connection_type: String,
    pub device: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ipv4Config {
    pub method: String,
    pub addresses: String,
    pub gateway: String,
    pub dns: String,
    pub mtu: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ipv6Config {
    pub method: String,
    pub addresses: String,
    pub gateway: String,
    pub dns: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionDetails {
    pub uuid: String,
    pub name: String,
    pub connection_type: String,
    pub device: String,
    pub ipv4: Ipv4Config,
    pub ipv6: Ipv6Config,
    pub mtu: Option<u32>,
    pub autoconnect: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkStatus {
    pub wifi_enabled: bool,
    pub connected: bool,
    pub ssid: String,
    pub ip4: String,
    pub gateway: String,
    pub device: String,
    pub connection_type: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotspotStatus {
    pub active: bool,
    pub ssid: String,
    pub device: String,
}

fn run_nmcli(args: &[&str]) -> Result<String, String> {
    let output = Command::new("nmcli")
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run nmcli: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        Ok(stdout)
    } else if !stderr.is_empty() {
        Err(stderr)
    } else {
        Err(format!("nmcli failed: {stdout}"))
    }
}

fn parse_wifi_line(line: &str) -> Option<WifiNetwork> {
    let mut parts = line.splitn(5, ':');
    let ssid = parts.next()?.replace('\\', "");
    let signal = parts.next()?.parse().ok()?;
    let security = parts.next()?.to_string();
    let in_use = parts.next().map(|v| v == "*").unwrap_or(false);
    let bssid = parts
        .next()
        .map(|v| v.replace('\\', ""))
        .unwrap_or_default();

    if ssid.is_empty() {
        return None;
    }

    Some(WifiNetwork {
        ssid,
        signal,
        security,
        in_use,
        bssid,
        saved: false,
    })
}

fn saved_wifi_profiles() -> Result<HashMap<String, String>, String> {
    let output = run_nmcli(&["-t", "-f", "NAME,TYPE", "connection", "show"])?;
    let mut profiles = HashMap::new();

    for line in output.lines() {
        let mut parts = line.splitn(2, ':');
        let name = parts.next().unwrap_or_default().to_string();
        let connection_type = parts.next().unwrap_or_default();

        if connection_type != "wifi" {
            continue;
        }

        let ssid = run_nmcli(&["-g", "802-11-wireless.ssid", "connection", "show", &name])?;
        if !ssid.is_empty() {
            profiles.insert(ssid, name);
        }
    }

    Ok(profiles)
}

pub fn list_wifi() -> Result<Vec<WifiNetwork>, String> {
    let saved = saved_wifi_profiles()?;
    let output = run_nmcli(&[
        "-t",
        "-f",
        "SSID,SIGNAL,SECURITY,IN-USE,BSSID",
        "device",
        "wifi",
        "list",
    ])?;

    let mut networks = parse_wifi_output(&output)?;
    for network in &mut networks {
        network.saved = saved.contains_key(&network.ssid);
    }
    Ok(networks)
}

pub fn rescan_wifi() -> Result<(), String> {
    run_nmcli(&["device", "wifi", "rescan"]).map(|_| ())
}

pub fn scan_wifi() -> Result<Vec<WifiNetwork>, String> {
    rescan_wifi()?;
    list_wifi()
}

fn parse_wifi_output(output: &str) -> Result<Vec<WifiNetwork>, String> {
    let mut networks = Vec::new();
    for line in output.lines() {
        if let Some(network) = parse_wifi_line(line) {
            networks.push(network);
        }
    }

    networks.sort_by(|a, b| b.signal.cmp(&a.signal));
    networks.dedup_by(|a, b| a.ssid == b.ssid);
    Ok(networks)
}

pub fn connect_wifi(ssid: &str, password: Option<&str>) -> Result<(), String> {
    if let Some(pass) = password.filter(|p| !p.is_empty()) {
        return run_nmcli(&["device", "wifi", "connect", ssid, "password", pass]).map(|_| ());
    }

    let saved = saved_wifi_profiles()?;
    if let Some(connection_name) = saved.get(ssid) {
        return run_nmcli(&["connection", "up", connection_name]).map(|_| ());
    }

    run_nmcli(&["device", "wifi", "connect", ssid]).map(|_| ())
}

pub fn disconnect_wifi() -> Result<(), String> {
    if let Ok(device) = wifi_device_name() {
        run_nmcli(&["device", "disconnect", &device]).map(|_| ())
    } else {
        Err("No WiFi device found".into())
    }
}

pub fn forget_wifi(ssid: &str) -> Result<(), String> {
    run_nmcli(&["connection", "delete", ssid]).map(|_| ())
}

pub fn wifi_device_name() -> Result<String, String> {
    let output = run_nmcli(&["-t", "-f", "DEVICE,TYPE,STATE", "device", "status"])?;
    for line in output.lines() {
        let mut parts = line.splitn(3, ':');
        let device = parts.next().unwrap_or_default();
        let device_type = parts.next().unwrap_or_default();
        if device_type == "wifi" {
            return Ok(device.to_string());
        }
    }
    Err("No WiFi device found".into())
}

pub fn set_wifi_enabled(enabled: bool) -> Result<(), String> {
    let state = if enabled { "on" } else { "off" };
    run_nmcli(&["radio", "wifi", state]).map(|_| ())
}

pub fn get_devices() -> Result<Vec<NetworkDevice>, String> {
    let output = run_nmcli(&["-t", "-f", "DEVICE,TYPE,STATE,CONNECTION", "device"])?;
    let mut devices = Vec::new();

    for line in output.lines() {
        let mut parts = line.splitn(4, ':');
        let name = parts.next().unwrap_or_default().to_string();
        let device_type = parts.next().unwrap_or_default().to_string();
        let state = parts.next().unwrap_or_default().to_string();
        let connection = parts.next().unwrap_or_default().to_string();

        if device_type == "loopback" {
            continue;
        }

        let ip4 = run_nmcli(&["-g", "IP4.ADDRESS", "device", "show", &name])
            .unwrap_or_default()
            .lines()
            .next()
            .unwrap_or("")
            .to_string();

        devices.push(NetworkDevice {
            name,
            device_type,
            state,
            connection,
            ip4,
        });
    }

    Ok(devices)
}

pub fn get_status() -> Result<NetworkStatus, String> {
    let devices = get_devices()?;
    let active = devices
        .iter()
        .find(|d| d.state.contains("connected") && d.device_type != "loopback");

    let Some(device) = active else {
        return Ok(NetworkStatus {
            wifi_enabled: is_wifi_radio_on()?,
            connected: false,
            ssid: String::new(),
            ip4: String::new(),
            gateway: String::new(),
            device: String::new(),
            connection_type: String::new(),
        });
    };

    let detail = run_nmcli(&["-g", "IP4.ADDRESS,IP4.GATEWAY", "device", "show", &device.name])?;
    let mut lines = detail.lines();
    let ip4 = lines.next().unwrap_or("").to_string();
    let gateway = lines.next().unwrap_or("").to_string();

    Ok(NetworkStatus {
        wifi_enabled: is_wifi_radio_on()?,
        connected: true,
        ssid: device.connection.clone(),
        ip4,
        gateway,
        device: device.name.clone(),
        connection_type: device.device_type.clone(),
    })
}

fn is_wifi_radio_on() -> Result<bool, String> {
    let output = run_nmcli(&["radio", "wifi"])?;
    Ok(output.trim() == "enabled")
}

pub fn get_connections() -> Result<Vec<SavedConnection>, String> {
    let output = run_nmcli(&["-t", "-f", "NAME,UUID,TYPE,DEVICE", "connection", "show"])?;
    let active = run_nmcli(&["-t", "-f", "NAME", "connection", "show", "--active"])
        .unwrap_or_default();

    let mut connections = Vec::new();
    for line in output.lines() {
        let mut parts = line.splitn(4, ':');
        let name = parts.next().unwrap_or_default().to_string();
        let uuid = parts.next().unwrap_or_default().to_string();
        let connection_type = parts.next().unwrap_or_default().to_string();
        let device = parts.next().unwrap_or_default().to_string();

        if connection_type == "loopback" {
            continue;
        }

        connections.push(SavedConnection {
            active: active.lines().any(|a| a == name),
            name,
            uuid,
            connection_type,
            device,
        });
    }

    Ok(connections)
}

pub fn get_connection_details(uuid: &str) -> Result<ConnectionDetails, String> {
    let name = run_nmcli(&["-g", "connection.id", "connection", "show", uuid])?;
    let connection_type = run_nmcli(&["-g", "connection.type", "connection", "show", uuid])?;
    let device = run_nmcli(&["-g", "GENERAL.DEVICES", "connection", "show", uuid])
        .unwrap_or_default();
    let autoconnect = run_nmcli(&["-g", "connection.autoconnect", "connection", "show", uuid])
        .map(|v| v == "yes")
        .unwrap_or(true);

    let ipv4_method = run_nmcli(&["-g", "ipv4.method", "connection", "show", uuid])?;
    let ipv4_addresses = run_nmcli(&["-g", "ipv4.addresses", "connection", "show", uuid])
        .unwrap_or_default();
    let ipv4_gateway = run_nmcli(&["-g", "ipv4.gateway", "connection", "show", uuid])
        .unwrap_or_default();
    let ipv4_dns = run_nmcli(&["-g", "ipv4.dns", "connection", "show", uuid])
        .unwrap_or_default()
        .replace(',', " ");

    let ipv6_method = run_nmcli(&["-g", "ipv6.method", "connection", "show", uuid])?;
    let ipv6_addresses = run_nmcli(&["-g", "ipv6.addresses", "connection", "show", uuid])
        .unwrap_or_default();
    let ipv6_gateway = run_nmcli(&["-g", "ipv6.gateway", "connection", "show", uuid])
        .unwrap_or_default();
    let ipv6_dns = run_nmcli(&["-g", "ipv6.dns", "connection", "show", uuid])
        .unwrap_or_default()
        .replace(',', " ");

    let mtu = run_nmcli(&["-g", "802-11-wireless.mtu", "connection", "show", uuid])
        .ok()
        .and_then(|v| v.parse().ok())
        .or_else(|| {
            run_nmcli(&["-g", "ethernet.mtu", "connection", "show", uuid])
                .ok()
                .and_then(|v| v.parse().ok())
        });

    Ok(ConnectionDetails {
        uuid: uuid.to_string(),
        name,
        connection_type,
        device,
        autoconnect,
        mtu,
        ipv4: Ipv4Config {
            method: ipv4_method,
            addresses: ipv4_addresses,
            gateway: ipv4_gateway,
            dns: ipv4_dns,
            mtu,
        },
        ipv6: Ipv6Config {
            method: ipv6_method,
            addresses: ipv6_addresses,
            gateway: ipv6_gateway,
            dns: ipv6_dns,
        },
    })
}

pub fn update_connection(
    uuid: &str,
    ipv4: Ipv4Config,
    ipv6: Ipv6Config,
    mtu: Option<u32>,
    autoconnect: bool,
) -> Result<(), String> {
    let mut args: Vec<String> = vec![
        "connection".into(),
        "modify".into(),
        uuid.to_string(),
    ];

    let ipv4_method = if ipv4.method == "manual" {
        "manual"
    } else {
        "auto"
    };
    args.push("ipv4.method".into());
    args.push(ipv4_method.into());

    if ipv4_method == "manual" {
        if !ipv4.addresses.is_empty() {
            args.push("ipv4.addresses".into());
            args.push(ipv4.addresses);
        }
        if !ipv4.gateway.is_empty() {
            args.push("ipv4.gateway".into());
            args.push(ipv4.gateway);
        }
        if !ipv4.dns.is_empty() {
            args.push("ipv4.dns".into());
            args.push(ipv4.dns.replace(' ', ","));
        }
    } else {
        args.extend([
            "ipv4.addresses".into(),
            "".into(),
            "ipv4.gateway".into(),
            "".into(),
            "ipv4.dns".into(),
            "".into(),
        ]);
    }

    let ipv6_method = if ipv6.method == "manual" {
        "manual"
    } else if ipv6.method == "disabled" {
        "disabled"
    } else {
        "auto"
    };
    args.push("ipv6.method".into());
    args.push(ipv6_method.into());

    if ipv6_method == "manual" {
        if !ipv6.addresses.is_empty() {
            args.push("ipv6.addresses".into());
            args.push(ipv6.addresses);
        }
        if !ipv6.gateway.is_empty() {
            args.push("ipv6.gateway".into());
            args.push(ipv6.gateway);
        }
        if !ipv6.dns.is_empty() {
            args.push("ipv6.dns".into());
            args.push(ipv6.dns.replace(' ', ","));
        }
    }

    args.push("connection.autoconnect".into());
    args.push(if autoconnect { "yes" } else { "no" }.into());

    if let Some(mtu) = mtu {
        let conn_type = run_nmcli(&["-g", "connection.type", "connection", "show", uuid])?;
        if conn_type.contains("wireless") {
            args.push("802-11-wireless.mtu".into());
        } else {
            args.push("ethernet.mtu".into());
        }
        args.push(mtu.to_string());
    }

    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run_nmcli(&arg_refs).map(|_| ())
}

pub fn activate_connection(uuid: &str) -> Result<(), String> {
    run_nmcli(&["connection", "up", uuid]).map(|_| ())
}

pub fn deactivate_connection(uuid: &str) -> Result<(), String> {
    run_nmcli(&["connection", "down", uuid]).map(|_| ())
}

pub fn get_vpn_connections() -> Result<Vec<SavedConnection>, String> {
    Ok(get_connections()?
        .into_iter()
        .filter(|c| c.connection_type == "vpn")
        .collect())
}

pub fn create_hotspot(ssid: &str, password: &str) -> Result<String, String> {
    let device = wifi_device_name()?;
    let args = if password.is_empty() {
        vec![
            "device",
            "wifi",
            "hotspot",
            "ifname",
            &device,
            "ssid",
            ssid,
        ]
    } else {
        vec![
            "device",
            "wifi",
            "hotspot",
            "ifname",
            &device,
            "ssid",
            ssid,
            "password",
            password,
        ]
    };
    run_nmcli(&args)
}

pub fn stop_hotspot() -> Result<(), String> {
    let device = wifi_device_name()?;
    run_nmcli(&["device", "disconnect", &device]).map(|_| ())
}

pub fn hotspot_status() -> Result<HotspotStatus, String> {
    let output = run_nmcli(&["-t", "-f", "GENERAL.CONNECTION,GENERAL.TYPE", "device", "show"])?;
    for block in output.split("\n\n") {
        if block.contains("wifi-p2p") || block.contains("Hotspot") {
            let ssid = block
                .lines()
                .find(|l| l.contains("GENERAL.CONNECTION"))
                .and_then(|l| l.split(':').nth(1))
                .unwrap_or("Hotspot")
                .trim()
                .to_string();
            return Ok(HotspotStatus {
                active: true,
                ssid,
                device: wifi_device_name().unwrap_or_default(),
            });
        }
    }

    Ok(HotspotStatus {
        active: false,
        ssid: String::new(),
        device: wifi_device_name().unwrap_or_default(),
    })
}

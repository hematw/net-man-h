use std::collections::HashMap;
use std::process::Command;

use super::types::{
    ConnectionInfo, DeviceInfo, IpConfig, NetworkSnapshot, NmError, Result, WifiNetwork,
};

pub fn snapshot(include_wifi: bool, rescan_wifi: bool) -> Result<NetworkSnapshot> {
    let wifi_enabled = run(&["radio", "wifi"]).map(|v| v.trim() == "enabled")?;
    let devices = list_devices()?;
    let connections = list_connections()?;

    let active = devices
        .iter()
        .find(|d| d.state.contains("connected") && d.device_type != "loopback");

    let (connected, connection_name, ip4, gateway, device, connection_type) =
        if let Some(dev) = active {
            (
                true,
                dev.connection.clone(),
                dev.ip4.clone(),
                device_gateway(&dev.name),
                dev.name.clone(),
                dev.device_type.clone(),
            )
        } else {
            (
                false,
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            )
        };

    let wifi_networks = if wifi_enabled && include_wifi {
        list_wifi(rescan_wifi, &connections)?
    } else {
        Vec::new()
    };

    Ok(NetworkSnapshot {
        wifi_enabled,
        connected,
        connection_name,
        ip4,
        gateway,
        device,
        connection_type,
        devices,
        connections,
        wifi_networks,
    })
}

pub fn list_devices() -> Result<Vec<DeviceInfo>> {
    let output = run(&["-t", "-f", "DEVICE,TYPE,STATE,CONNECTION", "device"])?;
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

        devices.push(DeviceInfo {
            ip4: device_ip4(&name),
            name,
            device_type,
            state,
            connection,
        });
    }

    Ok(devices)
}

pub fn list_connections() -> Result<Vec<ConnectionInfo>> {
    let output = run(&["-t", "-f", "NAME,UUID,TYPE,DEVICE", "connection", "show"])?;
    let active = run(&["-t", "-f", "NAME", "connection", "show", "--active"]).unwrap_or_default();

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

        let ssid = if connection_type == "802-11-wireless" || connection_type == "wifi" {
            run(&["-g", "802-11-wireless.ssid", "connection", "show", &uuid]).unwrap_or_default()
        } else {
            String::new()
        };

        connections.push(ConnectionInfo {
            active: active.lines().any(|line| line == name),
            name,
            uuid,
            connection_type,
            device,
            ssid,
        });
    }

    Ok(connections)
}

pub fn list_wifi(rescan: bool, connections: &[ConnectionInfo]) -> Result<Vec<WifiNetwork>> {
    if rescan {
        let _ = run(&["device", "wifi", "rescan"]);
    }

    let saved: HashMap<String, ()> = connections
        .iter()
        .filter(|c| c.connection_type.contains("wireless") || c.connection_type == "wifi")
        .map(|c| {
            let key = if c.ssid.is_empty() {
                c.name.clone()
            } else {
                c.ssid.clone()
            };
            (key, ())
        })
        .collect();

    let output = run(&[
        "-t",
        "-f",
        "SSID,SIGNAL,SECURITY,IN-USE,BSSID",
        "device",
        "wifi",
        "list",
    ])?;

    let mut networks = Vec::new();
    for line in output.lines() {
        let mut parts = line.splitn(5, ':');
        let ssid = parts.next().unwrap_or_default().replace('\\', "");
        let signal = parts.next().unwrap_or("0").parse().unwrap_or(0);
        let security = parts.next().unwrap_or_default().to_string();
        let in_use = parts.next().map(|v| v == "*").unwrap_or(false);

        if ssid.is_empty() {
            continue;
        }

        networks.push(WifiNetwork {
            saved: saved.contains_key(&ssid),
            ssid,
            signal,
            security: if security.is_empty() || security == "--" {
                "Open".into()
            } else {
                security
            },
            in_use,
        });
    }

    networks.sort_by(|a, b| b.signal.cmp(&a.signal));
    networks.dedup_by(|a, b| a.ssid == b.ssid);
    Ok(networks)
}

pub fn set_wifi_enabled(enabled: bool) -> Result<()> {
    let state = if enabled { "on" } else { "off" };
    run(&["radio", "wifi", state]).map(|_| ())
}

pub fn connect_wifi(ssid: &str, password: Option<&str>) -> Result<()> {
    if let Some(pass) = password.filter(|p| !p.is_empty()) {
        return run(&["device", "wifi", "connect", ssid, "password", pass]).map(|_| ());
    }

    let connections = list_connections()?;
    if let Some(conn) = connections.iter().find(|c| c.ssid == ssid || c.name == ssid) {
        return run(&["connection", "up", &conn.uuid]).map(|_| ());
    }

    run(&["device", "wifi", "connect", ssid]).map(|_| ())
}

pub fn disconnect_wifi() -> Result<()> {
    let device = wifi_device()?;
    run(&["device", "disconnect", &device]).map(|_| ())
}

pub fn forget_wifi(ssid: &str) -> Result<()> {
    let connections = list_connections()?;
    let conn = connections
        .iter()
        .find(|c| c.ssid == ssid || c.name == ssid)
        .ok_or(NmError::NotFound)?;
    run(&["connection", "delete", &conn.uuid]).map(|_| ())
}

pub fn activate_connection(uuid: &str) -> Result<()> {
    run(&["connection", "up", uuid]).map(|_| ())
}

pub fn deactivate_connection(uuid: &str) -> Result<()> {
    run(&["connection", "down", uuid]).map(|_| ())
}

pub fn read_ipv4_config(uuid: &str) -> Result<IpConfig> {
    let method = run(&["-g", "ipv4.method", "connection", "show", uuid])?;
    let addresses = run(&["-g", "ipv4.addresses", "connection", "show", uuid]).unwrap_or_default();
    let gateway = run(&["-g", "ipv4.gateway", "connection", "show", uuid]).unwrap_or_default();
    let dns = run(&["-g", "ipv4.dns", "connection", "show", uuid])
        .unwrap_or_default()
        .replace(',', " ");
    let mtu = run(&["-g", "802-11-wireless.mtu", "connection", "show", uuid])
        .ok()
        .and_then(|v| v.parse().ok())
        .or_else(|| {
            run(&["-g", "ethernet.mtu", "connection", "show", uuid])
                .ok()
                .and_then(|v| v.parse().ok())
        });
    let autoconnect = run(&["-g", "connection.autoconnect", "connection", "show", uuid])
        .map(|v| v == "yes")
        .unwrap_or(true);

    Ok(IpConfig {
        method,
        addresses,
        gateway,
        dns,
        mtu,
        autoconnect,
    })
}

pub fn write_ipv4_config(uuid: &str, config: &IpConfig) -> Result<()> {
    let mut args = vec![
        "connection".to_string(),
        "modify".to_string(),
        uuid.to_string(),
    ];
    let method = if config.method == "manual" {
        "manual"
    } else {
        "auto"
    };
    args.push("ipv4.method".into());
    args.push(method.into());

    if method == "manual" {
        if !config.addresses.is_empty() {
            args.push("ipv4.addresses".into());
            args.push(config.addresses.clone());
        }
        if !config.gateway.is_empty() {
            args.push("ipv4.gateway".into());
            args.push(config.gateway.clone());
        }
        if !config.dns.is_empty() {
            args.push("ipv4.dns".into());
            args.push(config.dns.replace(' ', ","));
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

    args.push("connection.autoconnect".into());
    args.push(if config.autoconnect {
        "yes".into()
    } else {
        "no".into()
    });

    if let Some(mtu) = config.mtu {
        let conn_type = run(&["-g", "connection.type", "connection", "show", uuid])?;
        if conn_type.contains("wireless") {
            args.push("802-11-wireless.mtu".into());
        } else {
            args.push("ethernet.mtu".into());
        }
        args.push(mtu.to_string());
    }

    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run(&arg_refs).map(|_| ())
}

fn wifi_device() -> Result<String> {
    for device in list_devices()? {
        if device.device_type == "wifi" {
            return Ok(device.name);
        }
    }
    Err(NmError::Message("No WiFi device found".into()))
}

fn device_ip4(interface: &str) -> String {
    run(&["-g", "IP4.ADDRESS", "device", "show", interface])
        .unwrap_or_default()
        .lines()
        .next()
        .unwrap_or("")
        .to_string()
}

fn device_gateway(interface: &str) -> String {
    run(&["-g", "IP4.GATEWAY", "device", "show", interface]).unwrap_or_default()
}

fn run(args: &[&str]) -> Result<String> {
    let output = Command::new("nmcli")
        .args(args)
        .output()
        .map_err(|e| NmError::Message(format!("Failed to run nmcli: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        Ok(stdout)
    } else if stderr.to_ascii_lowercase().contains("secret")
        || stderr.to_ascii_lowercase().contains("password")
        || stderr.to_ascii_lowercase().contains("802-1x")
    {
        Err(NmError::AuthRequired)
    } else if !stderr.is_empty() {
        Err(NmError::Message(stderr))
    } else {
        Err(NmError::Message(format!("nmcli failed: {stdout}")))
    }
}

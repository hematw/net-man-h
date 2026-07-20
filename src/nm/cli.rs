use std::process::Command;

use super::types::{IpConfig, NmError, Result};

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
    let mut args = vec!["connection".to_string(), "modify".to_string(), uuid.to_string()];
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

pub fn device_ip4(interface: &str) -> String {
    run(&["-g", "IP4.ADDRESS", "device", "show", interface])
        .unwrap_or_default()
        .lines()
        .next()
        .unwrap_or("")
        .to_string()
}

pub fn device_gateway(interface: &str) -> String {
    run(&["-g", "IP4.GATEWAY", "device", "show", interface])
        .unwrap_or_default()
}

pub fn run_nmcli_simple(args: &[&str]) -> Result<()> {
    run(args).map(|_| ())
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
    } else if stderr.to_ascii_lowercase().contains("secrets")
        || stderr.to_ascii_lowercase().contains("password")
    {
        Err(NmError::AuthRequired)
    } else if !stderr.is_empty() {
        Err(NmError::Message(stderr))
    } else {
        Err(NmError::Message(format!("nmcli failed: {stdout}")))
    }
}

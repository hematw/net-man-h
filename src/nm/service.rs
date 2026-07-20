use super::cli;
use super::types::{IpConfig, NetworkSnapshot, Result};

/// Thread-safe NetworkManager facade (nmcli). Safe to call from worker threads.
#[derive(Clone, Default)]
pub struct NetworkService;

impl NetworkService {
    pub fn new() -> Self {
        Self
    }

    pub fn snapshot(&self, include_wifi: bool, rescan_wifi: bool) -> Result<NetworkSnapshot> {
        cli::snapshot(include_wifi, rescan_wifi)
    }

    pub fn set_wifi_enabled(&self, enabled: bool) -> Result<()> {
        cli::set_wifi_enabled(enabled)
    }

    pub fn connect_wifi(&self, ssid: &str, password: Option<&str>) -> Result<()> {
        cli::connect_wifi(ssid, password)
    }

    pub fn disconnect_wifi(&self) -> Result<()> {
        cli::disconnect_wifi()
    }

    pub fn forget_wifi(&self, ssid: &str) -> Result<()> {
        cli::forget_wifi(ssid)
    }

    pub fn activate_connection(&self, uuid: &str) -> Result<()> {
        cli::activate_connection(uuid)
    }

    pub fn deactivate_connection(&self, uuid: &str) -> Result<()> {
        cli::deactivate_connection(uuid)
    }

    pub fn read_ip_config(&self, uuid: &str) -> Result<IpConfig> {
        cli::read_ipv4_config(uuid)
    }

    pub fn save_ip_config(&self, uuid: &str, config: &IpConfig) -> Result<()> {
        cli::write_ipv4_config(uuid, config)?;
        cli::activate_connection(uuid)
    }
}

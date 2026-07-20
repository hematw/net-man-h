#[derive(Clone, Debug, Default)]
pub struct NetworkSnapshot {
    pub wifi_enabled: bool,
    pub connected: bool,
    pub connection_name: String,
    pub ip4: String,
    pub gateway: String,
    pub device: String,
    pub connection_type: String,
    pub devices: Vec<DeviceInfo>,
    pub connections: Vec<ConnectionInfo>,
    pub wifi_networks: Vec<WifiNetwork>,
}

#[derive(Clone, Debug)]
pub struct DeviceInfo {
    pub name: String,
    pub device_type: String,
    pub state: String,
    pub connection: String,
    pub ip4: String,
}

#[derive(Clone, Debug)]
pub struct ConnectionInfo {
    pub name: String,
    pub uuid: String,
    pub connection_type: String,
    pub device: String,
    pub active: bool,
    pub ssid: String,
}

#[derive(Clone, Debug)]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal: u8,
    pub security: String,
    pub in_use: bool,
    pub saved: bool,
}

#[derive(Clone, Debug, Default)]
pub struct IpConfig {
    pub method: String,
    pub addresses: String,
    pub gateway: String,
    pub dns: String,
    pub mtu: Option<u32>,
    pub autoconnect: bool,
}

#[derive(Clone, Debug)]
pub enum NmError {
    Message(String),
    AuthRequired,
    NotFound,
}

impl std::fmt::Display for NmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NmError::Message(msg) => write!(f, "{msg}"),
            NmError::AuthRequired => write!(f, "Authentication required"),
            NmError::NotFound => write!(f, "Not found"),
        }
    }
}

impl std::error::Error for NmError {}

pub type Result<T> = std::result::Result<T, NmError>;

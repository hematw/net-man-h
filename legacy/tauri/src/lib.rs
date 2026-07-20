mod nm;
mod theme;

use nm::{
    activate_connection, connect_wifi, create_hotspot, deactivate_connection, disconnect_wifi,
    forget_wifi, get_connection_details, get_connections, get_devices, get_status,
    get_vpn_connections, hotspot_status, list_wifi, rescan_wifi, scan_wifi, set_wifi_enabled,
    stop_hotspot, update_connection, ConnectionDetails, HotspotStatus, Ipv4Config, Ipv6Config,
    NetworkDevice, NetworkStatus, SavedConnection, WifiNetwork,
};
use theme::{load_theme, ThemeColors};

async fn run_network<F, T>(work: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String> + Send + 'static,
    T: Send + 'static,
{
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|error| format!("Network task failed: {error}"))
        .and_then(|result| result)
}

#[tauri::command]
fn get_theme_colors() -> ThemeColors {
    load_theme()
}

#[tauri::command]
async fn fetch_status() -> Result<NetworkStatus, String> {
    run_network(get_status).await
}

#[tauri::command]
async fn fetch_devices() -> Result<Vec<NetworkDevice>, String> {
    run_network(get_devices).await
}

#[tauri::command]
async fn fetch_wifi_networks(rescan: Option<bool>) -> Result<Vec<WifiNetwork>, String> {
    let should_rescan = rescan.unwrap_or(false);
    run_network(move || {
        if should_rescan {
            scan_wifi()
        } else {
            list_wifi()
        }
    })
    .await
}

#[tauri::command]
async fn wifi_rescan() -> Result<(), String> {
    run_network(rescan_wifi).await
}

#[tauri::command]
async fn wifi_connect(ssid: String, password: Option<String>) -> Result<(), String> {
    run_network(move || connect_wifi(&ssid, password.as_deref())).await
}

#[tauri::command]
async fn wifi_disconnect() -> Result<(), String> {
    run_network(disconnect_wifi).await
}

#[tauri::command]
async fn wifi_forget(ssid: String) -> Result<(), String> {
    run_network(move || forget_wifi(&ssid)).await
}

#[tauri::command]
async fn wifi_set_enabled(enabled: bool) -> Result<(), String> {
    run_network(move || set_wifi_enabled(enabled)).await
}

#[tauri::command]
async fn fetch_connections() -> Result<Vec<SavedConnection>, String> {
    run_network(get_connections).await
}

#[tauri::command]
async fn fetch_connection_details(uuid: String) -> Result<ConnectionDetails, String> {
    run_network(move || get_connection_details(&uuid)).await
}

#[tauri::command]
async fn save_connection(
    uuid: String,
    ipv4: Ipv4Config,
    ipv6: Ipv6Config,
    mtu: Option<u32>,
    autoconnect: bool,
) -> Result<(), String> {
    run_network(move || update_connection(&uuid, ipv4, ipv6, mtu, autoconnect)).await
}

#[tauri::command]
async fn connection_activate(uuid: String) -> Result<(), String> {
    run_network(move || activate_connection(&uuid)).await
}

#[tauri::command]
async fn connection_deactivate(uuid: String) -> Result<(), String> {
    run_network(move || deactivate_connection(&uuid)).await
}

#[tauri::command]
async fn fetch_vpn_connections() -> Result<Vec<SavedConnection>, String> {
    run_network(get_vpn_connections).await
}

#[tauri::command]
async fn hotspot_create(ssid: String, password: String) -> Result<String, String> {
    run_network(move || create_hotspot(&ssid, &password)).await
}

#[tauri::command]
async fn hotspot_stop() -> Result<(), String> {
    run_network(stop_hotspot).await
}

#[tauri::command]
async fn fetch_hotspot_status() -> Result<HotspotStatus, String> {
    run_network(hotspot_status).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_theme_colors,
            fetch_status,
            fetch_devices,
            fetch_wifi_networks,
            wifi_rescan,
            wifi_connect,
            wifi_disconnect,
            wifi_forget,
            wifi_set_enabled,
            fetch_connections,
            fetch_connection_details,
            save_connection,
            connection_activate,
            connection_deactivate,
            fetch_vpn_connections,
            hotspot_create,
            hotspot_stop,
            fetch_hotspot_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

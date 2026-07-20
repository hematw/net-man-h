import { invoke } from "@tauri-apps/api/core";
import type {
  ConnectionDetails,
  HotspotStatus,
  Ipv4Config,
  Ipv6Config,
  NetworkDevice,
  NetworkStatus,
  SavedConnection,
  ThemeColors,
  WifiNetwork,
} from "./types";

export const api = {
  getTheme: () => invoke<ThemeColors>("get_theme_colors"),
  getStatus: () => invoke<NetworkStatus>("fetch_status"),
  getDevices: () => invoke<NetworkDevice[]>("fetch_devices"),
  scanWifi: (rescan = false) =>
    invoke<WifiNetwork[]>("fetch_wifi_networks", { rescan }),
  rescanWifi: () => invoke<void>("wifi_rescan"),
  connectWifi: (ssid: string, password?: string) =>
    invoke<void>("wifi_connect", { ssid, password }),
  disconnectWifi: () => invoke<void>("wifi_disconnect"),
  forgetWifi: (ssid: string) => invoke<void>("wifi_forget", { ssid }),
  setWifiEnabled: (enabled: boolean) =>
    invoke<void>("wifi_set_enabled", { enabled }),
  getConnections: () => invoke<SavedConnection[]>("fetch_connections"),
  getConnectionDetails: (uuid: string) =>
    invoke<ConnectionDetails>("fetch_connection_details", { uuid }),
  saveConnection: (
    uuid: string,
    ipv4: Ipv4Config,
    ipv6: Ipv6Config,
    mtu: number | null,
    autoconnect: boolean,
  ) =>
    invoke<void>("save_connection", {
      uuid,
      ipv4,
      ipv6,
      mtu,
      autoconnect,
    }),
  activateConnection: (uuid: string) =>
    invoke<void>("connection_activate", { uuid }),
  deactivateConnection: (uuid: string) =>
    invoke<void>("connection_deactivate", { uuid }),
  getVpnConnections: () => invoke<SavedConnection[]>("fetch_vpn_connections"),
  createHotspot: (ssid: string, password: string) =>
    invoke<string>("hotspot_create", { ssid, password }),
  stopHotspot: () => invoke<void>("hotspot_stop"),
  getHotspotStatus: () => invoke<HotspotStatus>("fetch_hotspot_status"),
};

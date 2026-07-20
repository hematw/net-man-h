export interface ThemeColors {
  accent: string;
  background: string;
  foreground: string;
  lighterBg: string;
  darkBg: string;
  muted: string;
  green: string;
  red: string;
  yellow: string;
  blue: string;
  cyan: string;
}

export interface WifiNetwork {
  ssid: string;
  signal: number;
  security: string;
  inUse: boolean;
  bssid: string;
  saved: boolean;
}

export interface NetworkDevice {
  name: string;
  deviceType: string;
  state: string;
  connection: string;
  ip4: string;
}

export interface SavedConnection {
  name: string;
  uuid: string;
  connectionType: string;
  device: string;
  active: boolean;
}

export interface Ipv4Config {
  method: string;
  addresses: string;
  gateway: string;
  dns: string;
  mtu?: number;
}

export interface Ipv6Config {
  method: string;
  addresses: string;
  gateway: string;
  dns: string;
}

export interface ConnectionDetails {
  uuid: string;
  name: string;
  connectionType: string;
  device: string;
  ipv4: Ipv4Config;
  ipv6: Ipv6Config;
  mtu?: number;
  autoconnect: boolean;
}

export interface NetworkStatus {
  wifiEnabled: boolean;
  connected: boolean;
  ssid: string;
  ip4: string;
  gateway: string;
  device: string;
  connectionType: string;
}

export interface HotspotStatus {
  active: boolean;
  ssid: string;
  device: string;
}

export type TabId = "overview" | "wifi" | "ethernet" | "vpn" | "hotspot";

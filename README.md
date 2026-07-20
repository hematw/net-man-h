# Aether Net

A modern NetworkManager GUI for Omarchy / Hyprland. **Pure Rust** — GTK4 + Libadwaita + NetworkManager D-Bus.

## Features

- WiFi scan, connect (saved profiles reconnect without password), disconnect, forget
- Ethernet and VPN profile management
- IPv4 static configuration (address, gateway, DNS, MTU)
- WiFi hotspot
- Omarchy theme color sync
- Single-instance window (Waybar click focuses existing window)
- Background NM work (UI stays responsive)

## Requirements

- Arch Linux / Omarchy
- NetworkManager
- `nmcli` (used only for IPv4 profile edits and radio toggle)
- Rust toolchain
- GTK4, Libadwaita (`gtk4`, `libadwaita` packages)

## Build

```bash
cargo build --release
./scripts/install.sh
```

## Waybar

```jsonc
"on-click": "aether-net"
```

## Legacy

The original Tauri + React app is archived in `legacy/`.

## License

MIT

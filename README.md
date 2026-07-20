# net-man-h

Pure Rust NetworkManager GUI for Omarchy / Hyprland — **GTK4 + Libadwaita**.

## Features (core)

- Overview — connection status, devices, active profiles
- WiFi — scan, connect (saved networks reconnect without password), disconnect, forget, radio toggle
- Ethernet — connect/disconnect + static IPv4 editor (address, gateway, DNS, MTU, autoconnect)
- Omarchy theme colors from `~/.config/omarchy/current/theme/colors.toml`
- Single-instance window (Waybar click focuses existing window)
- Background NM work (UI stays responsive)

built with 🩵 by hematw

## Requirements

- Arch Linux / Omarchy
- NetworkManager + `nmcli`
- `gtk4`, `libadwaita`
- Rust toolchain

## Build & install

```bash
cargo build --release
./scripts/install.sh
```

## Waybar

```jsonc
"on-click": "net-man-h"
```

## Develop

```bash
cargo run
```

The previous Tauri prototype lives under `legacy/` for reference.

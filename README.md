# netman-h

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

## Arch / AUR

Packaging lives in [`aur/`](aur/). After a `vX.Y.Z` GitHub release tag:

```bash
# local package build
cd aur
makepkg -si

# publish / update AUR package
# git clone ssh://aur@aur.archlinux.org/netman-h.git
# copy PKGBUILD + .SRCINFO, then:
makepkg --printsrcinfo > .SRCINFO
git add PKGBUILD .SRCINFO
git commit -m "Update to $pkgver"
git push
```

Install once published:

```bash
yay -S netman-h
# or: paru -S netman-h
```

## Waybar

```jsonc
"on-click": "netman-h"
```

## Develop

```bash
cargo run
```

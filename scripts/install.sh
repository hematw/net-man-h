#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="${HOME}/.local/bin"
DESKTOP_DIR="${HOME}/.local/share/applications"
APP_BIN="${BIN_DIR}/aether-net"
RELEASE_BIN="${ROOT}/target/release/aether-net"

mkdir -p "$BIN_DIR" "$DESKTOP_DIR"

if [[ ! -x "${RELEASE_BIN}" ]]; then
  echo "Release binary not found. Run: cargo build --release"
  exit 1
fi

cat > "$APP_BIN" <<EOF
#!/usr/bin/env bash
exec "${RELEASE_BIN}" "\$@"
EOF
chmod +x "$APP_BIN"

cat > "${DESKTOP_DIR}/aether-net.desktop" <<EOF
[Desktop Entry]
Name=Aether Net
Comment=Modern NetworkManager GUI
Exec=${APP_BIN}
Terminal=false
Type=Application
Categories=Network;Settings;
StartupWMClass=io.github.hemat.AetherNet
EOF

echo "Installed launcher to ${APP_BIN}"

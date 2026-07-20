#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="${HOME}/.local/bin"
DESKTOP_DIR="${HOME}/.local/share/applications"
APP_BIN="${BIN_DIR}/net-man-h"
RELEASE_BIN="${ROOT}/target/release/net-man-h"

mkdir -p "$BIN_DIR" "$DESKTOP_DIR"

if [[ ! -x "${RELEASE_BIN}" ]]; then
  echo "Release binary not found. Run: cargo build --release"
  exit 1
fi

# Remove previous launcher name if present
rm -f "${BIN_DIR}/aether-net" "${DESKTOP_DIR}/aether-net.desktop"

cat > "$APP_BIN" <<EOF
#!/usr/bin/env bash
exec "${RELEASE_BIN}" "\$@"
EOF
chmod +x "$APP_BIN"

cat > "${DESKTOP_DIR}/net-man-h.desktop" <<EOF
[Desktop Entry]
Name=net-man-h
Comment=Modern NetworkManager GUI
Exec=${APP_BIN}
Terminal=false
Type=Application
Categories=Network;Settings;
StartupWMClass=io.github.hemat.NetManH
EOF

echo "Installed launcher to ${APP_BIN}"

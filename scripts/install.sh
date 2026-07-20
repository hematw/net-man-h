#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="${PREFIX}/bin"
DESKTOP_DIR="${PREFIX}/share/applications"
APP_BIN="${BIN_DIR}/netman-h"
RELEASE_BIN="${ROOT}/target/release/netman-h"
DESKTOP_SRC="${ROOT}/data/netman-h.desktop"

mkdir -p "$BIN_DIR" "$DESKTOP_DIR"

if [[ ! -x "${RELEASE_BIN}" ]]; then
  echo "Release binary not found. Run: cargo build --release"
  exit 1
fi

# Remove previous launcher names if present
rm -f "${BIN_DIR}/aether-net" "${BIN_DIR}/net-man-h"
rm -f "${DESKTOP_DIR}/aether-net.desktop" "${DESKTOP_DIR}/net-man-h.desktop"

install -Dm755 "$RELEASE_BIN" "$APP_BIN"

# Rewrite Exec for user-local installs; system packages use data/*.desktop as-is.
sed "s|^Exec=.*|Exec=${APP_BIN}|" "$DESKTOP_SRC" > "${DESKTOP_DIR}/netman-h.desktop"

echo "Installed ${APP_BIN}"
echo "Desktop entry: ${DESKTOP_DIR}/netman-h.desktop"

#!/usr/bin/env bash
# Refresh b2sums + .SRCINFO from the GitHub release tag matching pkgver.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")"

# shellcheck disable=SC1091
source ./PKGBUILD

tarball="${pkgname}-${pkgver}.tar.gz"
url="${source[0]##*::}"

echo "Downloading ${url}"
curl -fsSL -o "$tarball" "$url"

sum="$(b2sum "$tarball" | awk '{print $1}')"
sed -i "s/^b2sums=.*/b2sums=('${sum}')/" PKGBUILD
makepkg --printsrcinfo > .SRCINFO

echo "Updated b2sums and .SRCINFO for ${pkgname} ${pkgver}"
echo "Tarball kept at aur/${tarball} (safe to delete after push)"

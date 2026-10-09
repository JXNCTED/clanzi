#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
if [ "$(uname -m)" != x86_64 ]; then
  echo 'This package targets Linux x86_64.' >&2
  exit 1
fi
npm run build
cargo build --manifest-path src-tauri/Cargo.toml --release --features custom-protocol --locked
stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT HUP INT TERM
chmod 0755 "$stage"
install -Dm755 src-tauri/target/release/clanzi "$stage/usr/bin/clanzi"
install -Dm644 packaging/clanzi.desktop "$stage/usr/share/applications/clanzi.desktop"
install -Dm644 src-tauri/icons/128x128.png "$stage/usr/share/icons/hicolor/128x128/apps/clanzi.png"
install -Dm644 packaging/70-clanzi.rules "$stage/usr/share/clanzi/70-clanzi.rules"
install -Dm755 packaging/install-permissions.sh "$stage/usr/share/clanzi/install-permissions.sh"
install -Dm755 packaging/remove-permissions.sh "$stage/usr/share/clanzi/remove-permissions.sh"
install -Dm644 README.md "$stage/usr/share/doc/clanzi/README.md"
install -Dm644 README.zh.md "$stage/usr/share/doc/clanzi/README.zh.md"
for doc in docs/*.md; do
  install -Dm644 "$doc" "$stage/usr/share/doc/clanzi/$doc"
done
for doc in cross-platform-replication-spec.md cross-platform-replication-spec.zh.md; do
  install -Dm644 "$doc" "$stage/usr/share/doc/clanzi/$doc"
done
install -Dm644 examples/keymap.json "$stage/usr/share/doc/clanzi/examples/keymap.json"
mkdir -p "$stage/DEBIAN" artifacts
cat > "$stage/DEBIAN/control" <<'CONTROL'
Package: clanzi
Version: 0.1.0
Section: utils
Priority: optional
Architecture: amd64
Maintainer: Clanzi contributors
Depends: libwebkit2gtk-4.1-0, libgtk-3-0, libayatana-appindicator3-1, dbus-bin, udev, kmod
Description: Olanzi AU05 desktop key mapper
 Local Linux key mappings, background forwarding, and an offline editor.
CONTROL
dpkg-deb --root-owner-group --build "$stage" artifacts/clanzi_0.1.0_amd64.deb

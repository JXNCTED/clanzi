#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
if [ "$(uname -m)" != x86_64 ]; then
  echo 'This package targets Linux x86_64.' >&2
  exit 1
fi
version=$(node tools/release-version.mjs)
npm run build
cargo build --manifest-path src-tauri/Cargo.toml --release --features custom-protocol --locked
stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT HUP INT TERM
chmod 0755 "$stage"
install -Dm755 src-tauri/target/release/clanzi "$stage/usr/bin/clanzi"
install -Dm644 packaging/clanzi.desktop "$stage/usr/share/applications/clanzi.desktop"
install -Dm644 src-tauri/icons/128x128.png "$stage/usr/share/icons/hicolor/128x128/apps/clanzi.png"
install -Dm644 packaging/70-clanzi.rules "$stage/usr/lib/udev/rules.d/70-clanzi.rules"
install -Dm644 packaging/clanzi.conf "$stage/usr/lib/modules-load.d/clanzi.conf"
install -Dm755 packaging/postinst "$stage/DEBIAN/postinst"
install -Dm755 packaging/postrm "$stage/DEBIAN/postrm"
install -Dm644 README.md "$stage/usr/share/doc/clanzi/README.md"
install -Dm644 README.zh.md "$stage/usr/share/doc/clanzi/README.zh.md"
install -Dm644 LICENSE "$stage/usr/share/doc/clanzi/LICENSE"
install -Dm644 THIRD_PARTY_NOTICES.md "$stage/usr/share/doc/clanzi/THIRD_PARTY_NOTICES.md"
for doc in docs/*.md; do
  install -Dm644 "$doc" "$stage/usr/share/doc/clanzi/$doc"
done
for doc in cross-platform-replication-spec.md cross-platform-replication-spec.zh.md; do
  install -Dm644 "$doc" "$stage/usr/share/doc/clanzi/$doc"
done
install -Dm644 examples/keymap.json "$stage/usr/share/doc/clanzi/examples/keymap.json"
mkdir -p "$stage/DEBIAN" artifacts
cat > "$stage/DEBIAN/control" <<CONTROL
Package: clanzi
Version: $version
Section: utils
Priority: optional
Architecture: amd64
Maintainer: Clanzi contributors
Depends: libwebkit2gtk-4.1-0, libgtk-3-0, libayatana-appindicator3-1, dbus-bin, udev, kmod
Description: Olanzi AU05 desktop key mapper
 Local Linux key mappings, background forwarding, and an offline editor.
CONTROL
dpkg-deb --root-owner-group --build "$stage" "artifacts/clanzi_${version}_amd64.deb"

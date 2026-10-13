#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
if [ "$(uname -m)" != x86_64 ]; then
  echo 'This package targets Linux x86_64.' >&2
  exit 1
fi
version=$(node tools/release-version.mjs)
# Run the bundler's AppImage helpers without requiring a /dev/fuse mount.
export APPIMAGE_EXTRACT_AND_RUN=1
# Keep installed AppImageLauncher integration out of the build helpers.
export APPIMAGELAUNCHER_DISABLE=1
npm run tauri -- build --bundles appimage -- --locked
install -Dm755 "src-tauri/target/release/bundle/appimage/clanzi_${version}_amd64.AppImage" "artifacts/clanzi_${version}_amd64.AppImage"
install -Dm644 packaging/70-clanzi.rules artifacts/70-clanzi.rules
install -Dm644 packaging/clanzi.conf artifacts/clanzi.conf

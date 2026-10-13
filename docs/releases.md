# Release guide

[中文](releases.zh.md) · [README](../README.md) · [Contributing](../CONTRIBUTING.md)

## Before tagging

The [Release workflow](https://github.com/JXNCTED/clanzi/blob/main/.github/workflows/release.yml) builds Linux amd64 packages on Ubuntu 22.04 when a `vMAJOR.MINOR.PATCH` tag is pushed. It checks versions, runs core/frontend/UI tests, and publishes a GitHub Release containing the `.deb`, `.AppImage`, `70-clanzi.rules`, `clanzi.conf`, and `SHA256SUMS`, with generated release notes. Prerelease suffixes are not supported yet.

1. Keep versions in `Cargo.toml`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, and `package.json` aligned. Refresh both Cargo lockfiles and `package-lock.json` as needed.
2. Run the checks in [Contributing](../CONTRIBUTING.md#make-and-verify-a-change), `make deb`, and `make appimage`. Check the package version, architecture, and bundled files. AppImage builds download Tauri's bundling tools and run their helpers in extraction mode, so a FUSE mount is not required during builds.
3. Commit the workflow and all release files before tagging. Do not include build artifacts, logs, credentials, or local configuration.

Validate and tag the release commit (replace `0.1.0` with the intended version):

```sh
node tools/release-version.mjs v0.1.0
git tag -a v0.1.0 -m "clanzi v0.1.0"
git push origin v0.1.0
```

## Publish and inspect

In **Actions → Release → Run workflow**, an existing tag can also be selected for a manual run. Builds use that tag's source. The workflow uses GitHub's automatic `GITHUB_TOKEN`; no personal access token is needed. Only the publishing job has `contents: write`. An existing Release is not overwritten; use a new version if one has already been published.

After the workflow completes, confirm the Release contains both packages, the two permission files, and `SHA256SUMS`. Verify all checksums and review the generated notes. Add a concise description of changes and link to [AppImage permission setup](troubleshooting.md#appimage-permission-setup). Test both packages in a suitable Linux desktop environment, including installation/removal and AppImage device access after the one-time setup.

## Public repository preparation

Use this repository description:

> Open-source Linux key mapper for Ulanzi Vibe Key (AU05), with custom shortcuts, push-to-talk, and a bilingual desktop UI.

Suggested topics: `linux`, `ulanzi`, `vibekey`, `au05`, `key-mapping`, `tauri`, `rust`, `react`, `typescript`.

Before changing repository visibility, review the files and Git history for credentials or private data, confirm the MIT license and upstream notices are included, and check README links and screenshots. Changing visibility and publishing a release are separate from preparing the documentation.

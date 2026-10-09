# clanzi — AU05 key mapper for Linux

[中文](README.zh.md) · [Configuration](docs/configuration.md) · [Troubleshooting](docs/troubleshooting.md) · [Validation status](docs/acceptance.md)

A local Linux x86_64 desktop mapper for Ulanzi Vibe Key (AU05), built with **Tauri 2, React, TypeScript, and Rust**.

- Map six controls to keys and Ctrl/Shift/Alt/Super combinations.
- Buttons follow physical press and release; knob rotation sends short taps.
- Edit offline, save locally, and import/export JSON configurations.
- Keep mappings active in the background, with an English or Chinese interface.

The desktop UI displays the name **clanzi**. Closing the window keeps forwarding active; use **Settings → Quit clanzi** to exit. Linux desktop and hardware validation is still incomplete; see [validation status](docs/acceptance.md).

## Build and run

Install Rust, Node.js 24, and the [Tauri Linux prerequisites](https://v2.tauri.app/start/prerequisites/#linux). On Ubuntu:

```sh
sudo apt install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libxdo-dev build-essential
npm ci
cargo fetch --locked
cargo fetch --manifest-path src-tauri/Cargo.toml --locked
make deb
sudo apt install ./artifacts/clanzi_0.1.0_amd64.deb
clanzi
```

The `.deb` automatically installs permission rules and enables the uinput module. Reconnect the receiver after installation, then select **Settings → Retry connection**. Run the GUI as your normal desktop user and close other AU05 clients. The rules grant the active local session access to the AU05 vendor interface and `/dev/uinput`; they do not grant access to physical keyboard event nodes.

To update, quit the running instance first. After rebuilding, run `./src-tauri/target/release/clanzi`, or reinstall the package with `sudo apt install --reinstall ./artifacts/clanzi_0.1.0_amd64.deb`.

## Voice input with Handy

Assign an AU05 button to your Handy recording shortcut and save. Set Handy to push-to-talk: hold the button to record, release to stop. All three buttons and the knob press follow physical duration automatically, with no mode selector or long-press threshold. See [configuration](docs/configuration.md) for shared-key behavior and compatibility.

## Development and checks

```sh
make dev                 # Native desktop development
make demo                # Isolated native demo, no real device or key output
make check               # Formatting, core lint, frontend build, and tests
cargo clippy --manifest-path src-tauri/Cargo.toml --offline --all-targets -- -D warnings
npm exec playwright install chromium
node tools/ui-smoke.mjs
```

`make dev` invokes the Tauri CLI through npm. Alternatively, run `npm run dev` and `cargo run --manifest-path src-tauri/Cargo.toml` in separate terminals. Vite uses port 1420; browser-only preview stores data in memory. Demo success does not establish hardware compatibility.

| Path | Purpose |
|---|---|
| `ui/` | React editor, bilingual text, frontend validation, IPC |
| `src/` | Configuration, storage, protocol, runtime, hidraw/uinput, worker service |
| `src-tauri/` | Native window, tray, single instance, file dialogs |
| `tests/`, `tools/` | Deterministic tests, UI smoke test, package build |
| `packaging/` | Desktop entry, permission rules, and package lifecycle scripts |

The [reference specification](cross-platform-replication-spec.md) records the upstream design baseline. Current Linux behavior and deviations are documented in [configuration](docs/configuration.md) and [validation status](docs/acceptance.md).

## Releases

The [Release workflow](.github/workflows/release.yml) builds Linux amd64 packages on Ubuntu 24.04 when a `vMAJOR.MINOR.PATCH` tag is pushed. It checks versions, runs core/frontend/UI tests, and publishes a GitHub Release containing the `.deb` and `SHA256SUMS`, with generated release notes. Prerelease suffixes are not supported yet.

Before tagging, keep the versions in `Cargo.toml`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, and `package.json` aligned; refresh and commit both Cargo lockfiles and `package-lock.json` as needed. Commit the workflow and all release files, then validate and tag that commit:

```sh
node tools/release-version.mjs v0.1.0
git tag -a v0.1.0 -m "clanzi v0.1.0"
git push origin v0.1.0
```

Use the matching version for future releases. In **Actions → Release → Run workflow**, an existing tag can also be selected for a manual run. Builds use that tag’s source. The workflow uses GitHub’s automatic `GITHUB_TOKEN`; no personal access token is needed. Only the publishing job has `contents: write`. An existing Release is not overwritten; use a new version if one has already been published.

## Uninstall

```sh
sudo apt remove clanzi
```

The package manager automatically removes the packaged permission rules and boot-time module configuration. Quit clanzi before uninstalling; an already running process may retain open device handles. User mappings, preferences, and logs are retained. Their locations are documented in [configuration](docs/configuration.md).

## Acknowledgments and license

Thanks to **[MrCroxx/olanzi](https://github.com/MrCroxx/olanzi)** and its contributors for the original Olanzi project, AU05 protocol research, documentation, and reference application. Their work made this Linux implementation possible.

clanzi is licensed under the [MIT License](LICENSE). Upstream copyright and license notices are preserved in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

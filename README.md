# Olanzi for Linux

[中文](README.zh.md) · [Specification](cross-platform-replication-spec.md) · [Acceptance](docs/acceptance.md)

A local AU05 key mapper built with **Tauri 2, React, TypeScript, and Rust**. This implementation targets Linux x86_64 first, as requested. macOS is not implemented.

Select one of six controls, assign one ordinary key and optional Ctrl/Shift/Alt/Super modifiers, then save to this computer. Editing remains available without hardware or permissions. Closing the window keeps the runtime alive; reopening the application brings back the existing window and draft.

## Run

Install the generated package and explicitly install its device permissions:

```sh
sudo apt install ./artifacts/clanzi_0.1.0_amd64.deb
sudo sh /usr/share/clanzi/install-permissions.sh
clanzi
```

Reconnect the receiver after installing the rules. Use **Settings → Retry connection**. Run the application as your normal desktop user. Quit Ulanzi Studio and other AU05 clients before connecting. The rules grant the active local session access to the AU05 vendor interface and `/dev/uinput`; they do not open physical keyboard event nodes.

The target acceptance environments are Ubuntu 24.04 GNOME Wayland and Xorg. Builds in this workspace use Ubuntu 22.04 x86_64; that does not establish hardware compatibility on either target. See the [acceptance record](docs/acceptance.md) for tests actually performed and remaining limitations.

## Push-to-talk with Handy

Quit the previous clanzi instance from **Settings → Quit Olanzi**, then launch the updated binary or install the rebuilt package. Closing its window only hides it.

AU05 buttons now follow physical press and release automatically: a short press is short, and holding keeps the shortcut down until release. There is no mode selector or long-press threshold. Existing shortcuts work immediately after restarting the updated application. For Handy, assign its recording shortcut if needed and use Handy’s push-to-talk setting. The three buttons and knob press follow this behavior; rotation remains tap-only.

Existing button mappings also follow physical duration. See [button behavior and configuration compatibility](docs/hold-mode.md).

## Development

Install the [Tauri Linux prerequisites](https://v2.tauri.app/start/prerequisites/#linux), Rust, and Node.js 24. Dependencies are pinned in both Cargo lockfiles and `package-lock.json`.

```sh
sudo apt install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libxdo-dev build-essential
npm ci
cargo fetch --locked
cargo fetch --manifest-path src-tauri/Cargo.toml --locked
make dev
```

`make dev` uses the Tauri CLI through npm. To avoid fetching the CLI, run `npm run dev` and `cargo run --manifest-path src-tauri/Cargo.toml` in separate terminals. Native development uses the Vite server at port 1420.

```sh
make demo
# Alternatively, run the packaged binary:
clanzi --demo
# Browser-only, explicitly labeled preview:
npm run dev
```

The native demo uses a separate instance identifier and a temporary configuration directory. It never opens a real receiver or creates a virtual keyboard. Browser preview keeps data only in memory. Demo success is not hardware validation.

## Check and package

```sh
make check
cargo clippy --manifest-path src-tauri/Cargo.toml --offline --all-targets -- -D warnings
npm exec playwright install chromium
node tools/ui-smoke.mjs
make deb
```

`make deb` produces `artifacts/clanzi_0.1.0_amd64.deb` with the embedded frontend. Installation needs no Rust or Node toolchain. The separate validator accepts only the specified format:

```sh
cargo run --offline --bin clanzi-validate -- examples/keymap.json
```

## Configuration and behavior

- Configuration: `$XDG_CONFIG_HOME/olanzi-cross-platform/keymap.json`, or `~/.config/olanzi-cross-platform/keymap.json` when XDG is unset or relative. Preferences live beside it in `preferences.json`.
- No configuration file is created until an explicit save. Saving six “No action” bindings is a valid configuration that takes over and discards vendor actions.
- Imports are validated completely, then replace only the draft. Exports use the saved version. Native file dialogs confirm overwrites.
- Corrupt startup files are preserved. Explicit repair saves first create a unique `keymap.invalid-YYYYMMDD-HHmmss-N.json` backup.
- Saves run outside the input worker, freeze a complete draft, synchronize the temporary file, and atomically rename it. Save errors retain the draft and previous active mapping. Forwarding errors after a successful save are reported separately.
- Rotary taps last 40 ms, separated by at least 10 ms, in one FIFO with 32 pending slots. All mapped buttons follow physical edges immediately and release on physical release or runtime stop; they do not block the tap queue. Shared output keys stay pressed until their last owner releases them. A delayed scheduler never compresses multiple overdue edges. Cancellation clears pending actions and retains failed releases as cleanup debt.
- Only the documented online query, software online/offline handover, and heartbeat commands can be transmitted. No on-device key table, light, firmware, or identifier operations exist.
- Rotating diagnostics are local, at most two 5 MiB files under the configuration directory's `logs/`. Ordinary keyboard text and mapping contents are not logged. There is no runtime network service or account.

## Layout

| Path | Responsibility |
|---|---|
| `ui/` | Bilingual editor, settings, draft UI, isolated browser preview |
| `src/config.rs`, `src/storage.rs` | Strict format, duplicate rejection, persistence, repair backups |
| `src/protocol.rs` | Little-endian TEA, exact report boundaries, command allowlist |
| `src/runtime.rs` | Readiness, handover, edge filtering, FIFO, cleanup debt |
| `src/linux.rs` | Descriptor-verified hidraw access and uinput keyboard |
| `src/service.rs` | Background worker, asynchronous saves, model, logs, sleep observation |
| `src-tauri/` | Native window, single instance, tray, native file dialogs |
| `tests/` | Deterministic contract and failure-injection tests |

## Uninstall

```sh
sudo sh /usr/share/clanzi/remove-permissions.sh
sudo apt remove clanzi
```

Only this application's rules are removed. User mappings, preferences, and logs are retained. To erase them, remove the configuration directory explicitly after quitting Olanzi. See [troubleshooting](docs/troubleshooting.md).

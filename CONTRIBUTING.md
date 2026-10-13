# Contributing to clanzi

[中文](CONTRIBUTING.zh.md) · [README](README.md)

Bug reports, Linux/AU05 testing, documentation fixes, and focused pull requests are welcome. You can explore the editor and work on the frontend without owning a device.

## Set up your checkout

Follow [Build from source](README.md#build-from-source) to install Node.js 24, stable Rust, Linux native dependencies, and the locked npm/Cargo dependencies. Read [AGENTS.md](https://github.com/JXNCTED/clanzi/blob/main/AGENTS.md) and [configuration](docs/configuration.md) before changing behavior. The [reference specification](cross-platform-replication-spec.md) records the upstream baseline; the current configuration docs explain differences such as physical button duration.

Choose the environment that fits your work:

| Command | Environment | Hardware and system input |
|---|---|---|
| `npm run dev` | Browser preview at the Vite localhost URL (port 1420) | None; data stays in memory |
| `make demo` | Native Tauri window with isolated demo state | No real device access or system keys |
| `make dev` | Native Tauri development with the real backend | Can access AU05 and send mapped keys |

Native modes require a desktop display. `make dev` and `make demo` invoke the Tauri CLI through npm and may download it on first use. Alternatively, run `npm run dev` and `cargo run --manifest-path src-tauri/Cargo.toml -- --demo` in separate terminals for a native demo.

For real-device development, install the `.deb` to configure permissions, reconnect the receiver, and quit the installed instance before starting your development build. Run as a normal desktop user and close other AU05 clients. Closing the window leaves the process running; use Settings or the tray to quit.

## Find the relevant code

| Path | Responsibility |
|---|---|
| `ui/` | React editor, draft state, bilingual text, frontend validation, IPC |
| `src/config.rs`, `src/storage.rs` | Authoritative validation, atomic saves, damaged-file backups |
| `src/protocol.rs` | AU05 reports and permitted device commands |
| `src/runtime.rs` | Physical edges, rotation queue, shared output keys, cleanup |
| `src/linux.rs`, `src/service.rs` | hidraw/uinput, worker thread, saving, lifecycle |
| `src-tauri/` | Native window, tray, single instance, file dialogs |
| `tests/`, `tools/` | Core/frontend tests, browser checks, version validation, packaging |
| `packaging/` | Desktop entry, permissions, package lifecycle scripts |

## Make and verify a change

Keep changes focused on the reported problem. Preserve draft-versus-saved behavior, shared-key ownership, cleanup on stop paths, and the device command allowlist. Update English and Chinese UI text and user docs together. Avoid logging keyboard text or full mappings.

For code changes, run:

```sh
make check
cargo clippy --manifest-path src-tauri/Cargo.toml --offline --all-targets -- -D warnings
```

`make check` covers formatting, core Clippy, TypeScript/build, and core/frontend/version/permission-script tests. The desktop crate has its own Cargo manifest and lockfile, so its Clippy check is separate.

For UI changes, also run:

```sh
npm exec playwright install chromium
node tools/ui-smoke.mjs
```

The UI check uses the production `dist/` from `make check` or `npm run build`. A compatible existing browser can be selected with `PLAYWRIGHT_CHROMIUM_EXECUTABLE`. Browser checks do not establish native WebKit, tray, or hardware compatibility.

For packaging changes, run `make deb` and `make appimage`, then inspect both packages. AppImage builds download bundling tools; release builds use Ubuntu 22.04. For runtime changes, add behavior tests covering the affected edge or failure path. Documentation-only changes need link and instruction checks, not new implementation tests.

## Report a bug or hardware result

Check [troubleshooting](docs/troubleshooting.md) and existing [issues](https://github.com/JXNCTED/clanzi/issues) first. Include:

- clanzi version or commit, install method, Linux distribution, and kernel version.
- Desktop environment and Wayland/Xorg session; whether this is browser demo, native demo, or real hardware.
- AU05 firmware if available, number of receivers, and whether other AU05 clients were closed.
- Reproduction steps, expected/actual behavior, and visible status/error codes.
- Relevant diagnostic log excerpts with private details removed; see [configuration](docs/configuration.md) for log locations.

English and Chinese reports are welcome.

## Open a pull request

Describe the problem, the resulting behavior, and the checks you ran. Clearly identify checks you could not run. Keep generated output, dependency directories, logs, credentials, and local user configuration out of the diff. Preserve `package-lock.json` and both Cargo lockfiles; update them when dependencies actually change.

Release maintainers should follow the [release guide](docs/releases.md).

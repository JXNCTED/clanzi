# Linux implementation and acceptance record

[中文](acceptance.zh.md)

📚 [README](../README.md) · **Acceptance** · [Troubleshooting](troubleshooting.md)

## Scope and evidence

Date: 2026-10-09. Application: 0.1.0. Reference specification baseline: `eb78f88f973d4d28f1d6d3369cdfb65881b1e1c6`, as recorded in the supplied specification. Inputs were the bilingual specification and Olanzi protocol/runtime/application documentation. The implementation does not copy the Swift application.

The user narrowed implementation to Linux and explicitly excluded Qt and macOS for this round. The original cross-platform specifications remain unchanged as reference contracts. This record describes the Linux implementation and deviations; it is not a declaration of full specification acceptance.

Build environment: Ubuntu 22.04.5 x86_64, kernel 6.8, Rust 1.96.1, Node.js 24.14.1, WebKitGTK 2.50.4, GTK 3.24.33. The execution environment has no desktop display or accessible AU05 receiver. Device firmware is unavailable. No hardware writes or permission-rule installation were performed.

## Executed checks

| Check | Result | Evidence |
|---|---|---|
| Rust core deterministic suite | Passed, 30 tests | `cargo test --offline`; `tests/core.rs` |
| Frontend domain tests | Passed, 6 cases | `node tests/frontend.test.mjs` |
| Rust formatting and lint | Passed | `cargo fmt`; core and desktop Clippy with `-D warnings` |
| Frontend types and production assets | Passed | `npm run build` |
| Tauri native compilation | Passed | Desktop `cargo check` and release build with `custom-protocol` |
| Example exchange file | Passed | `clanzi-validate examples/keymap.json` |
| Desktop entry and shell syntax | Passed | `desktop-file-validate`; `sh -n` |
| Debian package | Built; not installed | `artifacts/clanzi_0.1.0_amd64.deb`; `dpkg-deb` inspection |
| UI rendering and interactions | Blocked; not passed | Chromium startup denied by sandbox IPC (`Operation not permitted`); browser tool exposes no browser |
| Native graphical startup | Blocked; not passed | GTK initialization fails without a display; attempted only `--demo` |

The Playwright script in `tools/ui-smoke.mjs` covers draft editing, modifier retention/clearing, saved exports, import cancellation, exit cancellation, language changes, compact layout, and keyboard focus. It must still be run in a normal environment. Do not infer successful UI verification from the script's presence.

## Specification matrix

“Automated” refers only to deterministic core/domain logic, not an actual input reaching an application. Both Ubuntu 24.04 target sessions remain untested.

| Case | Automated evidence | GNOME Wayland hardware/UI | GNOME Xorg hardware/UI |
|---|---|---|---|
| T01 First launch | No configuration, read-only probing, explicit empty save | Not run | Not run |
| T02 Six controls | Index decoding and Linux key mappings | Not run | Not run |
| T03 Edges | Held/repeated press suppression; independent rotary pulses | Not run | Not run |
| T04 Chords | Press/release ordering; complete mapping whitelist | Physical modifier interleaving not run | Physical modifier interleaving not run |
| T05 FIFO/load | 600 actions at simulated 5 Hz; queue overflow; delayed deadlines | Real load not run | Real load not run |
| T06 Draft | Service edit/undo/language preservation | Window/UI checks not run | Window/UI checks not run |
| T07 Save | Roundtrip, atomic replacement, failed writes, unchanged draft | Desktop save not run | Desktop save not run |
| T08 Exchange | Strict types, duplicate/unknown fields, limits, canonical modifiers | Native file dialogs not run | Native file dialogs not run |
| T09 No action | Empty saved mappings take over with no synthetic action | Standard-HID leakage not run | Standard-HID leakage not run |
| T10 Lifecycle | Pause latch, unplug/reconnect, sleep intent | Tray, single instance, actual sleep not run | Tray, single instance, actual sleep not run |
| T11 Cleanup | Cancellation, partial output, retained failed releases | Kernel/physical cleanup not run | Kernel/physical cleanup not run |
| T12 Handover | Observed-held set, quiet window, explicit resync | Release blocker; delayed reports not run | Release blocker; delayed reports not run |
| T13 Errors | Injected output/handover/query/device failures | Actual permissions and recovery not run | Actual permissions and recovery not run |
| T14 Codec | Zero vector, roundtrip, boundary lengths, invalid indices | Captured-device replay not run | Captured-device replay not run |
| T15 Usability | UI smoke script provided | Rendering, keyboard, 200% scale not run | Rendering, keyboard, 200% scale not run |
| T16 Install/privacy | Package inspection, limited command set, private config files | Clean install/uninstall not run | Clean install/uninstall not run |
| T17 Performance | Simulated load only | 100-action latency and one hour not run | 100-action latency and one hour not run |
| T18 Corruption/exit | Backups, quit choices, failed save-and-quit | Native dialogs/concurrent save exit not run | Native dialogs/concurrent save exit not run |

## Deviations and unresolved limits

1. macOS source, packaging, and verification are intentionally out of scope for this round, per the user's updated direction.
2. Linux uses the kernel hidraw and uinput APIs directly rather than HIDAPI. The descriptor is checked before opening the vendor node. All input injection uses a dedicated virtual keyboard, independent of X11 tools.
3. Linux hidraw does not offer mandatory exclusive ownership. The implementation takes a cooperative `flock`; it cannot detect/prevent a non-cooperating application from opening the same interface. Closing Studio and other clients is required. The stronger busy-detection requirement is not fully met.
4. The 100 ms quiet-window implementation is not proof that all old input has drained. No protocol sequence/timestamp is available to distinguish an arbitrarily delayed old press. T12 remains release-blocking; no claim of guaranteed duplicate-free physical handover is made.
5. Sleep notifications are observed through logind's `dbus-monitor`. No delay inhibitor is held; pre-sleep completion and notification-loss recovery require further verification. A monitor startup error is logged.
6. A directory-sync failure after successful atomic rename is logged as `CONFIG_DIRECTORY_SYNC`; rename remains the commit point. Crash durability after that unusual failure is not guaranteed, and the UI does not falsely report that old disk bytes are still current.
7. Actual Wayland/Xorg modifier coordination, device routing, virtual-keyboard registration time, UI rendering, and packaging installation are unverified. A one-second registration grace period is a scheduling precaution, not proof of compositor readiness.

## Hardware procedure

1. Record application version, session (`Wayland` or `Xorg`), AU05 firmware if available, wall-clock time, and local log paths. Close other AU05 applications.
2. Install the permission rules; reconnect one receiver. Import `examples/keymap.json`, then explicitly save. Verify status reaches Forwarding.
3. Focus an empty text editor or input event viewer. Press top, wait one second; middle, wait one second; bottom, wait one second; knob press, wait one second; rotate right one detent, wait one second; rotate left one detent. Repeat 20 times. Do not hold the power button.
4. Test Ctrl+C, Shift+Tab, Alt+Left and a suitable Super shortcut in applications where their results are observable, including interleaved physical modifiers. Check both event order and application effects.
5. While a synthetic chord is active, test pause, disconnect, configuration replacement, permissions loss and quit. Check no queued action is replayed and failed releases remain visible.
6. Test startup with held controls and delayed reports, then explicit resync. Any historical action replay or double output fails T12.
7. Close the window for an hour, exercise sleep/wake and no-tray reopening, measure 100 isolated actions, and repeat the basic sequence. Repeat the complete procedure under the other target session.

## Button duration update

The user requested physical press/release behavior for all buttons without mode selection. Ordinary version 1 configurations require no edits, and previous version 2 files remain compatible; see [button behavior](hold-mode.md). Tests cover physical durations, shared ownership, rotary coexistence, and cleanup/recovery. The UI script checks that no mode selector appears and ordinary version 1 configurations are exported. Earlier browser verification was blocked by sandbox IPC restrictions; UI and physical Handy verification remain incomplete.

# Troubleshooting

[中文](troubleshooting.zh.md)

📚 [README](../README.md) · [Acceptance](acceptance.md) · **Troubleshooting**

## Receiver and output are separate

`PERMISSION_REQUIRED` includes an area. `receiver` concerns the AU05 vendor hidraw node; `output` concerns `/dev/uinput`. Install the packaged rules, reconnect the receiver, and retry as your normal logged-in desktop user. An SSH session may not receive logind's active-seat ACL. Do not give everyone access to `/dev/input/*` or run the GUI as root.

The target receiver has VID `0xFFF1`, PID `0x00DD`, vendor Usage Page `0xFFFC`, and Report ID `0x55`. The runtime checks the actual report descriptor. The supplied udev rule targets the documented USB interface `03`; an altered descriptor/interface needs separate verification.

## Status and recovery

| Error | Meaning and next action |
|---|---|
| `DEVICE_MISSING` | No matching vendor interface. Reconnect; discovery retries every two seconds. |
| `DEVICE_MULTIPLE` | More than one candidate. Keep one and explicitly retry. |
| `DEVICE_BUSY` | Cooperative interface lock unavailable. Close the other client and retry. |
| `DEVICE_OFFLINE` | A valid query returned status `0x00`. Turn on AU05 and wait for the next check. |
| `DEVICE_STATUS_UNKNOWN` | Query timed out or returned an unsupported status. This is not proof of offline status. |
| `CONFIG_INVALID` | Invalid local/imported JSON. Imported failures keep the current draft; damaged local files require explicit repair save. |
| `CONFIG_SAVE_FAILED` | Persistence failed. Check free space and directory permissions; your draft remains. |
| `OUTPUT_FAILED` | Input output/cleanup failed. Resolve permissions and retry; pending releases are attempted before re-enabling. |
| `HANDOVER_FAILED` | Handover could not be transmitted. Control return is unconfirmed; reconnect and retry. |
| `INPUT_OVERFLOW` | More than 32 actions were pending. The newest was dropped; reduce the input rate. |

An error remains visible after hiding and reopening the window. Logs distinguish explicit offline reports, query timeout/invalid status, invalid reports, and output failures. Pausing stops heartbeats and periodic queries. Explicit retry performs one query without clearing pause; only Resume clears that intent.

## Synchronization

Held controls observed before or during handover are quarantined until released. Rotation pulses restart the 100 ms quiet window. If a held control prevents synchronization for more than three seconds, release every control and use the explicit resynchronize button. The application sends no test key.

This is not a complete physical-state snapshot. The protocol provides no timestamp or sequence number that distinguishes an arbitrarily delayed old press from a new press. Startup-with-held-controls and delayed reports are release-blocking hardware validation items; see the acceptance record.

## Development failures

| Symptom | Action |
|---|---|
| Missing `webkit2gtk-4.1` or appindicator | Install the Tauri Linux prerequisites. |
| Cargo cannot download | Download dependencies outside a restricted sandbox with `cargo fetch --manifest-path src-tauri/Cargo.toml --locked`. |
| Vite is unavailable in a debug native run | Start `npm run dev` in another terminal, or use the release binary with embedded assets. |
| UI test browser is missing | Run `npm exec playwright install chromium`. A compatible existing browser can be selected with `PLAYWRIGHT_CHROMIUM_EXECUTABLE`. |
| Browser exits with `Operation not permitted` | Run the UI test in an ordinary desktop/CI environment; this workspace's sandbox prohibits Chromium IPC. |
| No tray icon on GNOME | Start Olanzi again. The existing instance reopens its window. |

The application observes logind's `PrepareForSleep` through `dbus-monitor`; failed monitor startup is logged as `SLEEP_MONITOR_FAILED`. The current implementation does not hold a logind delay inhibitor. Pre-sleep cleanup timing therefore still requires real desktop verification.

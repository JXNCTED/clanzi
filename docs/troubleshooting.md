# Troubleshooting

[中文](troubleshooting.zh.md)

📚 [README](../README.md) · **Troubleshooting**

## Receiver and output are separate

`PERMISSION_REQUIRED` includes an area. `receiver` concerns the AU05 vendor hidraw node; `output` concerns `/dev/uinput`. Install or reinstall the `.deb`, or follow the AppImage setup below, reconnect the receiver, and retry as your normal logged-in desktop user. An SSH session may not receive logind's active-seat ACL. Do not give everyone access to `/dev/input/*` or run the GUI as root.

The target receiver has VID `0xFFF1`, PID `0x00DD`, vendor Usage Page `0xFFFC`, and Report ID `0x55`. The runtime checks the actual report descriptor. The supplied udev rule targets the documented USB interface `03`; an altered descriptor/interface needs separate verification.

When upgrading from the manual permission installer, unchanged copies in `/etc/udev/rules.d/70-clanzi.rules` and `/etc/modules-load.d/clanzi.conf` are removed in favor of package-managed files in `/usr/lib`. Modified files and symlinks in `/etc` are preserved as local overrides, including on uninstall. Development builds can use the permissions configured by an installed `.deb`. If installation reports that the module or rules could not be activated, restart and reconnect before retrying.

## AppImage permission setup

AppImage does not install system permission rules. If an installed clanzi `.deb` already configured access, reuse those permissions. Otherwise download `70-clanzi.rules`, `clanzi.conf`, and `SHA256SUMS` from the same Release as your AppImage. Check for existing destination files before installing; preserve local overrides. From that directory, on Ubuntu 22.04:

```sh
sha256sum --check --ignore-missing SHA256SUMS
sudo apt install libfuse2 dbus-bin udev kmod
sudo install -Dm644 70-clanzi.rules /etc/udev/rules.d/70-clanzi.rules
sudo install -Dm644 clanzi.conf /etc/modules-load.d/clanzi.conf
sudo modprobe uinput
sudo udevadm control --reload-rules
sudo udevadm trigger --subsystem-match=misc --sysname-match=uinput
sudo udevadm trigger --subsystem-match=hidraw
```

On other distributions, install the equivalent host packages. Reconnect the receiver, launch the AppImage as your normal desktop user, and choose **Settings → Retry connection**. This setup is needed once per computer, not for every AppImage update.

To remove the manually installed files, quit clanzi, remove only the two files you installed in `/etc`, then reload and trigger udev using the commands above. Do not remove package-managed files in `/usr/lib`. Mappings and preferences are retained.

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

This is not a complete physical-state snapshot. The protocol provides no timestamp or sequence number that distinguishes an arbitrarily delayed old press from a new press.

## Development failures

| Symptom | Action |
|---|---|
| Missing `webkit2gtk-4.1` or appindicator | Install the Tauri Linux prerequisites. |
| Cargo cannot download | Check network/proxy access, then download dependencies with `cargo fetch --manifest-path src-tauri/Cargo.toml --locked`. |
| Vite is unavailable in a debug native run | Start `npm run dev` in another terminal, or use the release binary with embedded assets. |
| UI test browser is missing | Run `npm exec playwright install chromium`. A compatible existing browser can be selected with `PLAYWRIGHT_CHROMIUM_EXECUTABLE`. |
| Browser exits with `Operation not permitted` | Run the UI test in an ordinary desktop/CI environment; some sandboxes block Chromium IPC. |
| No tray icon on GNOME | Start clanzi again. The existing instance reopens its window. |

The application observes logind's `PrepareForSleep` through `dbus-monitor`; failed monitor startup is logged as `SLEEP_MONITOR_FAILED`. The current implementation does not hold a logind delay inhibitor. Pre-sleep cleanup timing therefore still requires real desktop verification.

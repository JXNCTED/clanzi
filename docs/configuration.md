# Configuration and input behavior

[中文](configuration.zh.md) · [README](../README.md)

## Files and persistence

Configuration lives in `$XDG_CONFIG_HOME/olanzi-cross-platform/`, or `~/.config/olanzi-cross-platform/` when XDG is unset or relative:

- `keymap.json`: saved mappings; created only on explicit save.
- `preferences.json`: interface preferences, separate from mappings.
- `logs/`: local diagnostics, capped at two 5 MiB files. Ordinary keyboard text and full mappings are not logged.

Editing changes a draft. Imports replace only the draft; exports use the saved configuration. Saving validates and freezes the complete draft, writes and synchronizes a temporary file, then atomically renames it. A write failure preserves the draft and previous active mapping. Corrupt local files are preserved; an explicit repair save first creates a unique `keymap.invalid-YYYYMMDD-HHmmss-N.json` backup.

Six “No action” bindings are valid: saving them enables takeover while discarding the corresponding actions. To stop forwarding, use Pause or Quit. To delete personal data, quit the application before removing its configuration directory.

## Button and rotary behavior

The three buttons and knob press forward physical down/up edges immediately. There is no mode selector, minimum duration, or long-press threshold. Repeated down reports are ignored; even down/up reports read in one worker iteration produce both output edges. No application-level repeat timer is added.

Rotary actions use 40 ms taps with a minimum 10 ms gap, in a FIFO with up to 32 pending actions. Button holds do not block this queue. Shared output keys remain pressed until their last owner releases them; tapping a key already held does not create another down edge. Held modifiers can affect other shortcuts.

Pause, sleep, mapping replacement, quit, disconnection, offline/unknown status, and output/heartbeat faults release owned keys and clear queued actions. Failed releases remain cleanup debt and require successful cleanup and Retry before forwarding resumes. Observed held buttons must be released before resynchronization. If a release report is lost while the device remains online, Pause or Quit releases a stuck shortcut.

## JSON compatibility

Both supported versions use the `olanzi-cross-platform-keymap` format, device `AU05`, and exactly six bindings. Files are limited to 64 KiB. Unknown or duplicate fields, invalid usages/modifiers, and unsupported versions are rejected as a whole. Modifier order is Ctrl, Shift, Alt, Super.

- Version 1 accepts `key` and `none`. Button `key` actions follow physical duration; rotary `key` actions produce timed taps.
- Version 2 also accepts the legacy `hold` action on buttons only. It behaves identically to button `key` actions. Existing files require no startup rewrite or explicit save.
- New mappings use `key`. The editor preserves legacy `hold` actions when editing their key/modifiers and uses version 2 while any remain. Old tap-only builds may interpret version 1 button actions differently and reject version 2 files.

See [the example](../examples/keymap.json) for a complete file. Validate a file with:

```sh
cargo run --offline --bin clanzi-validate -- examples/keymap.json
```

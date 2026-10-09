# Buttons follow physical duration

[中文](hold-mode.zh.md) · [README](../README.md)

## Use with Handy

The top, middle, bottom, and knob-press controls automatically follow physical press and release. A short press produces a short keystroke; holding keeps the shortcut down until release. No mode selection or long-press threshold is needed. Rotary directions continue to produce 40 ms taps.

Existing mappings work immediately after restarting the updated application. Quit the old clanzi instance first: closing its window leaves it running. To use Handy, assign its recording shortcut if needed and use Handy’s push-to-talk setting. No CLI toggle command is needed.

## Runtime behavior

Button edges are forwarded immediately, independently of the rotary tap queue. Repeated down reports are ignored. Down/up reports read in the same worker iteration still generate both output edges. There is no minimum button duration or synthetic 40 ms release timer for buttons.

Multiple buttons and a running rotary tap may share a key or modifier. Each output key is pressed once and released only when its last owner releases it. A rotary tap of a key already held does not create another down edge. Held modifiers can affect other shortcuts, as with a physical keyboard. This change forwards press/release state; it does not add an application-level repeat timer.

Pause, sleep, configuration replacement, quit, receiver disconnection, offline/unknown device status, and output or heartbeat failures cancel held keys and queued taps. Failed releases retain cleanup debt and lock forwarding until cleanup succeeds and the user retries. Resume does not re-press an observed held button; release it first. Lost release reports while the device remains online cannot be detected by a timeout without cutting off intentional long holds; pause or quit to release a stuck shortcut.

## Configuration compatibility

This user-requested behavior supersedes the original specification’s fixed-duration button taps and the previous implementation’s per-button mode selection. Existing files are accepted without a startup rewrite or an explicit save.

- Version 1 `key` actions now follow physical duration on the four buttons. Rotary `key` actions remain timed taps. `none` still ignores the control.
- Version 2 files from the previous build remain accepted. Their button-only `hold` actions behave identically to button `key` actions. The editor retains these legacy actions when changing their key or modifiers; it offers no mode selector.
- New mappings use `key` and require no new field or version upgrade. Files containing legacy `hold` actions keep version 2 so that older applications reject unsupported actions rather than misinterpret them.
- Size limits, strict fields, duplicate rejection, modifier ordering, and rejection of rotary `hold` actions are unchanged.
- Export preserves the saved configuration. An older tap-only clanzi build may interpret version 1 button mappings as fixed taps; physical-duration behavior requires this updated build.

## Verification

Deterministic tests cover ordinary version 1 button mappings at 1, 39, 40, 41, 200, and 1000 ms; all four buttons held for several seconds; legacy version 2 compatibility; repeated and same-iteration edges; shared keys; rotary tap coexistence; cleanup failures; and stop/resume behavior. Actual AU05-to-Handy operation still needs desktop hardware verification.

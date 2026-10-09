# 11 Olanzi cross platform reference replication specification

> 🌐 [中文](11-cross-platform-replication-spec.zh.md)

> 📚 Documentation: [README](../README.md) · [02 Protocol](02-vibekey-protocol.md) · [09 Reference app](09-native-macos.md) · [10 Reference runtime](10-input-runtime.md) · **11 Cross platform reference replication specification**

## 1 Document contract

This is a complete product and behavioral specification for an independent implementation using the existing Olanzi macOS application as a reference. The target supports Linux and macOS, runs in the background, maps the six AU05 controls to basic keys or key combinations, and stores configuration locally. Implementers should be able to design, build, and verify it without reproducing the original class structure or porting SwiftUI.

This is a complete specification with an explicit functional boundary, not an MVP, an iteration roadmap, or a requirement to reproduce every reference feature. It replaces the earlier cross-platform MVP draft. Background operation, local configuration, and the absence of a full macOS parity requirement are user-confirmed; other scope details and defaults are design decisions proposed by this specification, not individually confirmed user choices.

Document date: 2026-10-09. Status: specification awaiting implementation. “Must” defines acceptance requirements; “recommended” permits alternatives. [CONFIRMED] identifies recorded reference facts, and [INFERRED] identifies inferences. Target design requirements are not labeled as verified facts.

This document uses a clean-room-style behavioral contract: reference material establishes external behavior and protocol facts, while implementation is written anew against this specification. It does not claim separation between specification authors and implementers; earlier feasibility work inspected repository source. It is not evidence of an executed separation process.

## 2 References and usage rules

The reference baseline is repository commit `eb78f88f973d4d28f1d6d3369cdfb65881b1e1c6`. The identifiers below provide traceability. Changes to reference files require recording a new baseline rather than silently changing target behavior.

| ID | Material | What it establishes | What it does not establish |
|---|---|---|---|
| R1 | [Native app guide](09-native-macos.md) | Editing, drafts and saving, background operation, permission presentation, and import/export semantics | Every described scenario has been verified on hardware |
| R2 | [Input runtime](10-input-runtime.md) | Control events, action semantics, cancellation, and failure cleanup boundaries | Linux already behaves identically |
| R3 | [Protocol](02-vibekey-protocol.md) | TEA, frame layout, interface identification, known commands and replies | Unverified write commands are safe to use |
| R4 | [Heartbeat investigation](07-heartbeat-investigation.md) and linked evidence | Switching between standard input and vendor events; observation and inference boundaries | A successful heartbeat proves the device is online or never sleeps |
| R5 | [Chinese keymap screenshot](images/keymap-zh.png), [English keymap screenshot](images/keymap-en.png), [settings screenshot](images/device-zh.png) | Visual hierarchy, device orientation, and the relationship between selection and editing areas | Demo screenshots prove hardware connection or working interactions |
| R6 | [Verification log](05-verification-log.md) | Specific experiments, failed attempts, and confirmed results | Historical partial success proves the whole new product works |

![Reference keymap screen with demo data](images/keymap-zh.png)

The screenshot supplies layout and visual reference only. Preserve the relationship of device and controls above, action editing below, and separate settings. Remove the layers, gesture columns, APP, and macro areas excluded by this specification. Pixel identity, original fonts, icons, and asset files are not required.

Implementation inputs are this specification, descriptions and evidence in R1–R6, protocol test vectors, and new black-box observations. If a strictly independent implementation workflow is required, production source and original test implementations are not implementation inputs; write tests anew from this specification's scenarios. Source-code links inside R3 do not extend the allowed implementation inputs.

When sources conflict, explicit target behavior here overrides reference behavior; direct evidence governs protocol bytes; screenshots do not override interaction descriptions. Record unresolved hardware differences as verification gaps rather than conclusions. New observations must identify version, platform, operation sequence, results, and evidence location.

## 3 Product scope and reference differences

| Capability | Reference application | Target product | Basis or reason |
|---|---|---|---|
| Six controls | Independently assignable | Required | R1, R2; basic device purpose |
| Keys and combinations | Multiple ordinary keys, modifiers, Fn, and more | One ordinary key plus zero to four modifiers | Explicitly smaller action model |
| Trigger behavior | Single/double/long press and several output modes | One tap per press or knob step | No gesture delay or implicit repetition |
| Layers, macros, APP actions | Supported | Absent | Advanced workflows are not replicated |
| Draft and local configuration | Editing and activation are separate | Required | R1; editing must not change live input |
| Import/export | Native profiles contain complex actions | Separate format defined here | No native-profile migration requirement |
| Background operation | Close window and reopen from menu bar | Required, including desktops without a tray | R1; user-confirmed |
| Permission UI | macOS welcome page gates the workspace | Offline editing remains available; forwarding is gated | Deliberate cross-platform UX difference |
| Pause/resume | Idle policies and automatic device-activity resume | Manual pause/resume and recovery after disconnection | No automatic idle policy |
| Battery, lighting, statistics | Related UI exists | Absent | Outside basic mapping responsibilities |
| Login autostart, multiple profiles, update service | Partially supported in reference | Absent | User launches the installed app explicitly |
| Fn, media, mouse, scripts, firmware updates | Support varies | All absent | Explicit capability boundary |

Support one AU05 and USB receiver only; the power key is not assignable. If multiple matching receivers are detected, stop automatic selection and ask the user to leave only one connected before retrying. Do not choose the first arbitrarily or terminate another program occupying the device.

The release acceptance baseline is Ubuntu 24.04 LTS x86_64 with GNOME Wayland and GNOME Xorg, plus macOS 14 on Apple Silicon. These are target test environments, not verified compatibility claims. Windows, Linux ARM, Intel macOS, and other desktops are outside this specification's guaranteed support.

## 4 Terms and data ownership

| Term | Meaning and boundary |
|---|---|
| Draft | The six mappings being edited in the window; never used for forwarding |
| Saved configuration | The last successfully persisted mappings, loaded on application restart |
| Active configuration | The saved revision currently used by the runtime; activation may wait for a device |
| Takeover | The app consumes vendor events and emits synthetic input; distinct from USB connection |
| Pause | A user request to stop takeover; does not guarantee physical device silence |
| Ready | Valid configuration, online device, usable permissions/output, and no user pause |
| Tap | One synthetic press followed by release, independent of physical hold duration |
| No action | Discard a control action during takeover; does not restore its hardware mapping |

Draft, saved configuration, active configuration, and the device's own key table must remain distinct. Connection, queries, and reconnection cannot overwrite draft or saved mappings. Ordinary editing and saving must not write the device key table; after exit, the device may use its original configuration.

## 5 UI and interaction specification

### 5.1 Page structure

| Area | Required content | Interaction rules |
|---|---|---|
| Header | Olanzi name, Keymap and Settings navigation, textual runtime status | Color cannot be the only status indicator; background errors open an explanation |
| Device area | Three vertically arranged keys, knob press, left/right rotation, and all six assignments | Click to select, clearly mark selection, support keyboard focus and readable names |
| Action editor | Selected control, key categories, search, modifier toggles, combination preview, No action | Selections change the draft only |
| Footer | Unsaved state, Revert changes, Save to this computer | Disable revert/save when a saved configuration exists and is unchanged; lock mapping edits and duplicate submissions while saving |
| Settings | Connection details, permission help, Retry, Pause/Resume, import/export, language, Quit | Do not expose raw protocol, developer paths, or heartbeat counts as everyday controls |
| Tray menu | Status, Open window, Pause/Resume, Quit | Without a tray, relaunch must reopen the existing instance |

On first launch, select the top key and leave all six actions empty. Editing, saving, and import/export remain available without hardware or permissions, with forwarding explicitly shown as inactive. An absent configuration differs from explicitly saving six No action assignments: the former does not take over; the latter takes over when ready and consumes all six controls without output.

Without a valid saved configuration, six empty actions are a new configuration that can be submitted, so Save is enabled; Revert is disabled until editing occurs. A damaged startup configuration also produces six empty draft actions with a persistent error. Do not take over or overwrite the original file until the user explicitly saves a repair.

### 5.2 Editing rules

The ordinary-key catalog must include A–Z, 0–9, Space, Enter, Esc, Tab, Backspace, Delete, Insert, Home, End, PageUp, PageDown, four arrows, F1–F12, and the US physical positions for minus, equals, brackets, backslash, semicolon, quote, grave accent, comma, period, and slash. Punctuation labels describe physical positions, not guaranteed characters under the current layout.

Modifiers are Ctrl, Shift, Alt, and Super; display the latter two as Option and Command on macOS. Allow each once, without left/right selection, and emit left-side usages. An ordinary key is required. Modifier-only actions, multiple ordinary keys, Fn, lock keys, and keypad-specific keys are unsupported. Search matches localized names and common English names; an empty result does not replace the selected action.

Choosing an ordinary key replaces the current ordinary key while retaining modifiers. Choosing No action clears that control's key and modifiers. Selecting an ordinary key afterward starts without modifiers. Changing controls, pages, or language preserves the draft. Selecting a control never sends a test key.

### 5.3 Saving and file operations

| User action | Preconditions | Result |
|---|---|---|
| Save to this computer | Valid draft, with no valid saved configuration or differing from it | Atomically persist the full configuration; activate when ready, otherwise show “Saved, waiting for device or permissions” |
| Revert changes | Unsaved changes exist | Restore the entire saved configuration, or six empty actions before the first save |
| Export configuration | A valid saved configuration exists | Export the saved revision; explain that unsaved draft changes are excluded |
| Import configuration | User selected a file that passes validation | Replace the draft without saving or activating; failure/cancellation preserves the draft |
| Close window | Application is not quitting | Retain the in-memory draft and hide the window; forwarding continues; no discard or save prompt merely for closing |
| Quit application | No changes, or unsaved-change prompt resolved | Release input, relinquish control, and exit; next launch loads saved configuration only |

Before an import replaces a dirty draft, offer “Discard draft and import” and “Cancel”; replace only after file validation succeeds. On quit, offer “Save and quit,” “Quit without saving,” and “Cancel.” Save failure keeps the application open. Revert is an explicit discard action and needs no further confirmation.

A successful save notification reflects persistence; a forwarding-start failure is reported separately and does not claim the save failed or restore old disk contents. A file-write failure preserves the previous file, active configuration, and current draft. Distinguish “Save failed” from “Saved but forwarding unavailable.”

Freeze the complete draft when saving begins. Disable key editing, Revert, Import, and another Save until completion; navigation, language changes, and hiding the window remain available. Success makes the snapshot saved and clears dirty state; failure unlocks and retains the draft. A quit request during saving waits for that operation: success proceeds to exit cleanup, failure restores the window and cancels exit. Do not start a second concurrent save or force a normal exit while the write remains unfinished.

### 5.4 Language and visual requirements

Provide Chinese and English UI and user-facing errors. Initially use Chinese for a Chinese system language, otherwise English; allow an explicit persistent override. Do not translate key codes, paths, error identifiers, or JSON fields. Language changes apply immediately without changing mappings.

Preserve R5's clear device illustration, assignment, and action-selection hierarchy while allowing platform-specific window decoration and themes. Long translations must not obscure buttons; persistent errors cannot rely only on transient notifications. All editing and saving operations need keyboard navigation, visible focus, and accessible names. Verify at 1024×768 logical resolution and 200% system scaling; scrolling is allowed, but primary actions cannot be clipped.

## 6 State machine and lifecycle

Derive state from receiver availability, device online status, configuration, permissions, output backend, and user pause intent. A single “Connected” flag is insufficient. List concurrent failures individually; choose the first applicable primary state from the following table.

| Primary state | Condition | Behavior and recovery |
|---|---|---|
| Quitting | Quit confirmed | Reject new actions, clean up, exit, and do not reconnect |
| Forwarding fault | Output failure, handover failure, or incomplete release | Stop takeover, retain the error, require user Retry before enabling again |
| Paused | Explicit user pause | Stop heartbeat and periodic queries; keep window/configuration; saving or reconnection does not clear pause |
| Permission required | Device or output access unavailable | Editing remains available; show platform permission steps and Retry |
| Configuration unavailable | Never saved or file invalid | Allow editing/repair; do not automatically create a file before first save |
| Device unavailable | Missing, occupied, or multiple receivers | Allow editing; watch insertion when missing; wait for Retry when occupied or multiple |
| Device offline or unknown | Most recently completed query explicitly offline, timed out, or returned an invalid reply | No takeover; distinguish offline from query failure; an unfinished first query is not this state |
| Connecting | Opening interface, checking online state, or handing over | Reject actions; failure enters the relevant state |
| Forwarding | All conditions ready and handover complete | Own one device connection and process new actions with the active configuration |

Primary state identifies the leading forwarding blocker; show connection progress independently in details. For example, without configuration while querying the device, primary state is Configuration unavailable and details may say Connecting. Initial open or user Retry clears old query results and marks the query in progress. Routine background probes retain the last completed result and timestamp until the new query finishes.

Cold start loads saved mappings without preserving the previous process's temporary pause state, then connects and takes over if all conditions are satisfied. Closing the window does not stop connection, scheduling, or error handling. Repeated launch activates the existing instance; competing versions report occupation rather than seize the device.

Pause, quit, sleep, receiver removal, confirmed device offline state, permission loss, output failure, and configuration replacement must cancel pending actions and attempt to release app-owned keys. Failed releases remain cleanup obligations and block forwarding; retry cleanup first when output returns. Quit uses bounded cleanup, records incomplete releases on timeout, and destroys output resources rather than preventing exit indefinitely.

Receiver reinsertion and system wake recheck all readiness conditions and may restore normal connections, but cannot clear explicit pause or a forwarding fault. Permission recovery updates the display automatically; an output-fault latch still requires Retry. Missing callbacks alone never prove device offline state.

Input during takeover transitions is not guaranteed to execute; do not display Forwarding before completion. After beginning handover, drain old reports and collect edges into an observed-held set. Initialize it empty on cold start; preserve previously observed unreleased controls during reconnection within the process. This is an observation set, not a complete physical-state snapshot. Transition presses only add to the set and releases remove from it; neither produces an action.

Accept new actions only when the set is empty and at least 100 ms passes without device activity. Discard rotation pulses during this interval and restart the quiet timer; historical presses never become actions after takeover. If a nonempty set blocks progress for more than three seconds, show “Release all controls, then resynchronize” without automatically clearing it. User confirmation that controls are released may clear isolation state, but must repeat the quiet window, send no test keys, and never clear explicit pause.

A 100 ms quiet interval proves neither physical release nor absence of delayed reports. This is a target handover policy, not a firmware guarantee. T12 must cover controls held before startup and delayed reports. If the reference protocol cannot distinguish delayed presses from new ones and stale actions or duplicate output result, this policy fails acceptance. Change synchronization and update the specification rather than treating the issue as solved.

## 7 Input behavior contract

### 7.1 Controls and triggers

| Control ID | Physical position | Vendor index | Trigger rule |
|---|---|---|---|
| key_top | Top key | 0 | First press triggers; release rearms |
| key_middle | Middle key | 1 | First press triggers; release rearms |
| key_bottom | Bottom key | 2 | First press triggers; release rearms |
| knob_press | Knob press | 3 | First press triggers; release rearms |
| knob_right | Clockwise twist | 4 | Each valid press pulse triggers once without waiting for release |
| knob_left | Counterclockwise twist | 5 | Each valid press pulse triggers once without waiting for release |

Repeated press edges on an unreleased pressable control do not retrigger; an isolated release only clears state. Unknown indices, invalid states, and short events produce no input and create diagnostics; never guess a key code. Without a sequence number suitable for deduplication, do not merge rotation pulses merely because they arrive close together.

### 7.2 Timing and conflicts

For each action, press selected modifiers in the fixed order Ctrl, Shift, Alt, Super, then press the ordinary key. After 40 ms, release the ordinary key and then the action-owned modifiers in reverse order. Never actively release or rewrite physical modifiers not owned by the application. Physical keyboard modifiers may combine with synthetic output; the system and target application determine interpretation.

Execute actions through one global FIFO sequence, starting the next only after the previous action releases, so independent combinations do not accidentally merge. Leave at least 10 ms between a release and the next action's press. Queue at most 32 waiting actions, excluding one executing action. On overflow, discard the newest action and expose a diagnosable overload error without truncating an existing press/release pair.

Use monotonic time and do not block device events or heartbeat. After a scheduling delay, do not burst out multiple overdue presses; advance one due edge at a time and schedule later edges from the actual send time. Fault recovery, configuration replacement, pause, and reconnection clear the queue without replay. An action captures the active configuration revision at its trigger.

Saved actions are physical combinations, not cross-platform semantic commands such as Copy or Paste. Super+C becomes Command+C on macOS but remains Super+C on Linux; it does not automatically become Ctrl+C. Character output follows the current keyboard layout; fixed text output and input-method commits are not provided.

## 8 Configuration exchange format

Use UTF-8 JSON with format identifier `olanzi-cross-platform-keymap` and version 1. Limit files to 64 KiB. Reject duplicate properties, unknown fields, incorrect types, unsupported versions, and missing or extra controls. Never silently truncate or partially import.

```json
{
  "format": "olanzi-cross-platform-keymap",
  "version": 1,
  "device": "AU05",
  "bindings": {
    "key_top": {"kind": "key", "usage": 6, "modifiers": ["ctrl"]},
    "key_middle": {"kind": "key", "usage": 40, "modifiers": []},
    "key_bottom": {"kind": "none"},
    "knob_press": {"kind": "key", "usage": 43, "modifiers": ["shift"]},
    "knob_right": {"kind": "key", "usage": 79, "modifiers": []},
    "knob_left": {"kind": "key", "usage": 80, "modifiers": []}
  }
}
```

| Field | Rule |
|---|---|
| format, version, device | Must equal the example values; version must be an integer |
| bindings | Exactly the six IDs in §7, with one action each |
| kind | Only key or none; none cannot contain usage or modifiers |
| usage | Required for key; an integer HID Keyboard/Keypad Usage Page ordinary key from the §5.2 allowlist; booleans are not integers |
| modifiers | Required for key; ctrl, shift, alt, super only, unique, at most four; export in §7.2 order |

The example values represent HID usages for C, Enter, Tab, RightArrow, and LeftArrow. Configuration contains no device-unique identifier, platform virtual key code, pause state, application path, or executable content. Displayed combination names are derived, not stored as a second authority.

On Linux, use `$XDG_CONFIG_HOME/olanzi-cross-platform/keymap.json`, falling back to `~/.config/olanzi-cross-platform/keymap.json` when the variable is absent or not absolute. On macOS, use `~/Library/Application Support/Olanzi Cross Platform/keymap.json`. Store language preference in `preferences.json` alongside it, excluded from mapping exports. Do not read or overwrite the existing native application's configuration location.

Validate fully before writing a temporary file in the same directory, completing persistence, and atomically replacing the target. Failure preserves the old file and removes the temporary file. Invalid startup configuration must be preserved and reported, not overwritten with defaults. When the user explicitly saves a valid repair, first back up the damaged file's original bytes alongside it as `keymap.invalid-YYYYMMDD-HHmmss-N.json` without overwriting an existing backup, then replace the main file. Backup failure makes the save fail and preserves the original. Default exports to `olanzi-keymap-YYYYMMDD-HHmmss.json`; confirm overwrite of an existing export. Restart must load the latest successfully saved complete revision, never a mixed revision.

Old macOS HostProfile files, device-keymap exports, and future versions are outside this format and must be rejected as an unsupported format or version. Format changes require an explicit migration specification rather than guessed field semantics.

## 9 Device communication contract

### 9.1 Identification and codec

The following wire constants come from R3. Never match devices by the example serial number.

| Item | Contract |
|---|---|
| USB identification | VID `0xFFF1`, PID `0x00DD` |
| Vendor interface | Usage Page `0xFFFC`, Report ID `0x55`; reference descriptor uses interface 3, but verify the actual interface rather than enumeration order |
| Encryption | TEA, ECB, 32 rounds, 8-byte blocks, no IV; unsigned 32-bit arithmetic with modular overflow |
| Key | `ca ba a5 ca 6d 8a 2a bc ba 9e 5a ca ca 8b b8 9b` |
| Round constants | delta `0x9E3779B9`, initial decryption sum `0xC6EF3720` |
| Sending | Zero-fill commands to 64 bytes, encrypt, and use the first 63 bytes as payload; do not encrypt the Report ID |
| Receiving | Separate the Report ID and decrypt only the first 56 payload bytes; the final 7 incomplete-block bytes are not valid plaintext |
| API boundary | Normalize whether the underlying API supplies the Report ID separately, preventing duplicate prefixes or a missing byte |

Interpret key and data words as little-endian. An independent TEA calculation during specification authoring matched R3's zero-block vector with little-endian words. This checks the algorithm vector, not hardware communication.

```text
TEA_Enc(00 00 00 00 00 00 00 00) = 38 90 c4 99 a3 60 aa ad
```

### 9.2 Minimal commands and events

| Purpose | Plaintext prefix or field | Required handling |
|---|---|---|
| Query device online state | `06 03 0a 01` | Status following reply prefix `06 03 0a 11`: `0x01` online, `0x00` offline, other values unknown |
| Software-online handover | `01 01 10 00 03` | Attempt handover after output and configuration are ready, then enable heartbeat; based on R1, without treating write success as a wireless acknowledgment |
| Takeover heartbeat | `06 01 23 00 01` | Send every second while forwarding; no confirmed heartbeat ACK contract exists, so do not wait for an invented ACK |
| Software-offline handover | `01 01 10 00 00` | Attempt while the connection remains usable, after stopping heartbeat and releasing output |
| Control-event identification | `(frame[0] & 0x1F) == 0x0B` and `frame[1] == 0x10` | `frame[4]` is the control index; `frame[3]` is `0x01` press or `0x00` release |
| Logical action number | `frame[2]` | Never emit it as a keyboard usage |

The handover prefixes are protocol operations described by the reference application's documentation. Complete Linux handover still requires hardware acceptance; do not invent ACK assumptions or configuration writes. While connected and unpaused, query online state every two seconds, with a one-second deadline and at most one outstanding query. Timeout means unknown state, stopped takeover, and input cleanup. Continue read-only probing and resume only after a valid online reply and no latched fault.

Queries cannot block input decoding or heartbeat scheduling. Retry receiver discovery every two seconds when absent; permission, occupation, and multiple-receiver problems must not produce recurring dialogs. While paused, stop heartbeat and periodic queries but retain removal detection. Retry connection may perform one explicit read-only check without clearing pause. Resume forwarding clears pause intent and rechecks readiness.

Send only commands required by the table. Do not write key tables, lighting, standby parameters, or firmware, and do not query full device-unique identifiers. Command-set expansion requires separate protocol evidence and a requirement.

## 10 Platform and implementation boundaries

The product contract does not mandate a language, GUI framework, thread count, or internal class names. Tauri, Web UI, and a Rust core remain a recommended implementation direction, not an acceptance requirement. Alternatives are valid if they meet this specification; reference use of native APIs does not require copying the whole architecture.

| Boundary | Linux | macOS | Shared requirement |
|---|---|---|---|
| Device access | Recommended HIDAPI hidraw backend and targeted udev rules | HIDAPI or per-device IOKit access | Own only the intended interface and handle removal/occupation correctly |
| Keyboard output | Recommended uinput virtual keyboard | Recommended CoreGraphics events | Map HID usages to platform codes with complete press/release and cleanup |
| Permission presentation | Distinguish receiver access from virtual-input access | Distinguish necessary input access from event-posting authorization | Check the running process's actual capability, not merely a settings toggle |
| Background entry | Tray when available, otherwise relaunch opens window | Menu-bar entry | Single instance and window-independent input handling |
| Installation | deb package and rule installation/removal instructions | App bundle and DMG with signing/permission instructions | No development toolchain or external service for normal use |

Missing permissions cannot require running the entire GUI as root. Installing access rules may request administrator authorization, limited to necessary device/output resources rather than granting everyone access to all input nodes. Uninstall removes only this application's rules; retain user configuration by default and document separate cleanup.

Do not rely on X11-only key tools as the sole Linux output backend. Both Wayland and Xorg must pass real system-input acceptance. Application focus control is out of scope, so compositor-specific window management is unnecessary. Do not continuously record ordinary keyboard text to implement basic combinations; handle only necessary metadata if physical-modifier coordination is needed.

## 11 Error and recovery contract

Errors include a stable identifier, localized explanation, forwarding impact, and next action. Diagnostic detail is allowed, but a stack trace cannot replace user guidance. Background errors remain visible on reopening; clear the current error only after successful recovery. Do not collapse distinct problems into “Device disconnected.”

| Error ID | Trigger | Forwarding impact | User recovery |
|---|---|---|---|
| DEVICE_MISSING | No matching receiver | No takeover | Insert receiver; automatic rediscovery |
| DEVICE_MULTIPLE | Multiple matching receivers | No takeover | Leave one connected and Retry |
| DEVICE_BUSY | Target interface occupied | No takeover | Exit competing program and Retry |
| DEVICE_OFFLINE | Valid status reply explicitly offline | Stop takeover | Switch on AU05 and wait for status check |
| DEVICE_STATUS_UNKNOWN | Query timeout or invalid reply | Stop takeover | Check device; automatic probe or Retry |
| PERMISSION_REQUIRED | Device or output authorization missing | Stop takeover | Follow platform authorization steps and recheck |
| CONFIG_INVALID | Invalid local file or import | No takeover without a valid active configuration; failed import leaves existing forwarding intact | Repair configuration or edit and save again |
| CONFIG_SAVE_FAILED | Persistence fails | Old mappings continue; draft retained | Fix directory permissions or disk issue and save again |
| OUTPUT_FAILED | Synthetic press or release fails | Clean up and latch forwarding fault | Restore output capability and Retry |
| HANDOVER_FAILED | Software-online/offline send fails | Stop takeover and latch fault | Check connection and Retry; do not claim device control was returned |
| INPUT_OVERFLOW | Action queue full | Drop newest action; finish others in order | Slow input; diagnostic must not block device reads |

Malformed vendor reports are diagnosed and discarded, not interpreted as offline state. Logs must distinguish device-reported offline, query timeout, event parsing failure, and output failure. Never swallow exceptions while continuing to display normal forwarding.

## 12 Nonfunctional requirements

| Area | Acceptance requirement |
|---|---|
| Response latency | For 100 nonqueued actions, p95 from receiving a control event to submitting the first synthetic press is at most 50 ms; excludes wireless transit, 40 ms hold, and target-app rendering |
| Normal load | Five actions per second for two minutes without loss or duplication; input scheduling must not starve heartbeat |
| Stability | Run with the window closed for one hour, repeat the six-control case before and after; no crash, stuck key, or continuously growing backlog |
| Exit | Complete cleanup attempts and exit within three seconds after user confirmation; log cleanup failures rather than claim release succeeded |
| Privacy | No account or network needed; no key/configuration uploads; redact full deviceSn and flashId in published logs, preserving raw byte lengths |
| Diagnostics | Log time, app version, platform, state changes, and error IDs; no ordinary keyboard text or full configuration by default; rotate logs with a 10 MiB total limit |
| Testability | Verify configuration, UI, and deterministic timing without hardware; explicitly label simulation and isolate it from real configuration/devices |
| Maintainability | Isolate platform differences at device, output, permission, and desktop-integration boundaries; window lifetime cannot control input runtime lifetime |

Latency and load values are target requirements, not current measurements. Performance reports must identify machine, system session, app version, and measurement boundaries. Static screenshots cannot establish response time.

## 13 Acceptance cases and traceability

### 13.1 Test method

Record target app version, reference baseline, OS/session, device firmware when available, operation times, and actual results. Use reference screenshots for information hierarchy and this specification for interactions; use R3 vectors and R4 reports for protocol checks. Excluded features do not require reference-equivalent output.

Hardware baseline sequence: map the six controls respectively to A, B, C, D, RightArrow, and LeftArrow; save and focus a blank text editor or a test window that explicitly displays keyboard events. Press the top key, wait one second, middle key, wait one second, bottom key, wait one second, knob press, wait one second, one right step, wait one second, one left step. Repeat the entire sequence 20 times. Do not long-press the power key or mistake an intercepted system shortcut for absent application output.

Test combinations only after basic input succeeds, using a target application with observable behavior for that combination. Record output events separately from target-app effects; the latter cannot establish correctness of every protocol stage. Demo reference screenshots do not replace hardware acceptance below.

| Case | Source and specification | Procedure | Required observation |
|---|---|---|---|
| T01 First launch | R1; §5, §6 | Launch without configuration/device, then edit and save | Six empty draft actions and offline editing; no takeover before save, explicit device wait afterward |
| T02 Six controls | R2, R4; §7 | Execute the baseline sequence | Exactly 20 actions per control, correct directions, no duplication or stuck keys |
| T03 Tap and edges | R2; §7 | Hold a key for two seconds, release, press again; inject repeated presses and isolated releases | One action per independent press, no hold repeat, no extra output from invalid edges |
| T04 Combinations | R1; §5, §7 | Use Ctrl+C, Shift+Tab, Alt+Left, and a platform-appropriate Super combination; interleave physical modifiers | Correct event order and release, no active release of physical modifiers, no residual state |
| T05 Serialization and load | Target difference; §7, §12 | Alternate combinations at five actions per second for two minutes, then simulate overflow | No normal-load loss or merged combinations; overflow drops only newest action with diagnostics |
| T06 Draft | R1; §4, §5 | Edit, change control/language, close/reopen window, revert | Draft retained, saved mappings continue forwarding, revert restores the whole saved revision |
| T07 Save | R1; §5, §8 | Save/restart; simulate write failure and forwarding-start failure | Restart preserves values; failed write preserves old values; failed start reports saved but unavailable |
| T08 File exchange | R1; §8 | Export/import across platforms; try unknown versions, duplicate fields, oversized files, old native profiles | Valid files become drafts only; no implicit Ctrl/Super rewrite; invalid files preserve draft and active mappings |
| T09 No action | Target difference; §5, §7 | Save six No action mappings and exercise controls during takeover | No synthetic or original hardware output; pause does not promise physical silence |
| T10 Lifecycle | R1, R2; §6 | Close, relaunch, suspend/wake, unplug/replug, then replug while explicitly paused | One running instance, background forwarding, no automatic pause clearance or stale replay |
| T11 Cancellation and release | R2; §6, §7 | After synthetic press, simulate disconnection, permission revocation, configuration save, quit, and release failure | Queue cancelled, releases recorded, failed cleanup obligations retained, no replay after recovery |
| T12 Handover | R4; §6, §9 | Take over/pause while held, rotating, idle; hold before startup and delay the press report beyond the quiet window; exercise manual resynchronization after waiting | Quiet-window and observed-set rules hold, no stale action or path duplication; indistinguishable delayed input fails acceptance; no false successful-handover claim |
| T13 Error recovery | R1, R6; §11 | Produce each tabulated fault and recover | Correct identifier, guidance, impact, recovery; unknown is not offline, offline is not a permission problem |
| T14 Codec | R3; §9 | Check zero block, round trip, 63-byte truncation, short reports, Report ID boundary, unknown events | Known vector matches, only complete decrypted fields read, no output from malformed input |
| T15 UI usability | R5; §5 | Both languages, keyboard navigation, minimum resolution/high scaling, no-tray case | No lost draft or clipped primary action, visible focus, relaunch reopens the window |
| T16 Installation and privacy | §10, §12 | Clean install, permissions, offline operation, uninstall, inspect logs | No toolchain/root needed for normal operation, no network dependency, only own rules removed, configuration retained |
| T17 Performance and endurance | §12 | Measure 100 isolated actions; run one hour with window closed and baseline sequence before/after | Meet latency, stability, and exit limits, with recorded data |
| T18 Invalid configuration and quit | §5, §8 | Start with damaged configuration, repair/save, try each dirty-quit option, then quit during saving | Startup preserves the bad file; explicit repair backs it up before replacement; all three quit choices and waiting during save follow the contract |

### 13.2 Decision rules

Configuration, scheduling, and fault injection can use deterministic automated tests. UI cases require rendered interaction checks. Device-to-system behavior in T02, T04, T09, T10, T12, and T17 requires real AU05 hardware. Simulated events establish local logic only, not the hardware path.

The complete matrix covers Linux Wayland, Linux Xorg, and macOS. Mark each applicable case passed, failed, or unexecuted with a reason; unexecuted is not passed. Do not erase known defects by rewriting reference facts. If target behavior changes, update both language specifications and corresponding cases first.

## 14 Deliverables and verification obligations

Complete delivery includes independently written source, installable artifacts for both platforms, a configuration validator and examples, automated tests derived from this document, bilingual installation/troubleshooting documentation, case-by-case acceptance records, and an explicit list of deviations. Completion means meeting this specification, not merely migrating a framework.

| Verification obligation | Current boundary | Closure condition |
|---|---|---|
| Linux takeover and release | R4 establishes routing changes in reference scenarios, not the full Linux path | T02, T09, T12 pass in both Linux sessions |
| Modifier interleaving | Reference has partial measurements; target backends differ | T04, T11 verify physical/synthetic interleaving without stuck keys |
| Sleep and firmware fallback | No promise of immediate post-exit fallback or a particular sleep duration | Record actual T10/T12 fallback behavior and limits; failed handover is visible |
| Desktops without a tray | A status icon cannot be assumed everywhere | T15 verifies relaunch restores the window in a single instance |
| Platform support claims | Target environments have not passed new-implementation acceptance | Complete the matrix before declaring support |

These are verification obligations, not a plan to defer functionality to another release. Questions unanswered by reference material require controlled observations and evidence. Specification changes must update reference differences, behavioral clauses, affected data formats, and acceptance cases in both languages.

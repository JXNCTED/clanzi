use clanzi_core::{
    config::{allowed_usage, Action, Keymap, Modifier, LIMIT},
    linux::{linux_key, vendor_descriptor},
    protocol::{self, Event},
    runtime::{Issue, Output, Runtime, Scheduler, Transport},
    service::{Command, Service},
    storage::{atomic_write, Store},
};
use std::{
    collections::VecDeque,
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

static TEMP_ID: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "clanzi-test-{}-{}",
            std::process::id(),
            TEMP_ID.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn key(usage: u16, modifiers: Vec<Modifier>) -> Action {
    Action::Key { usage, modifiers }
}
fn map() -> Keymap {
    let mut m = Keymap::default();
    m.bindings.key_top = key(4, vec![Modifier::Ctrl]);
    m.bindings.key_middle = key(5, vec![]);
    m.bindings.knob_right = key(79, vec![]);
    m.bindings.knob_left = key(80, vec![]);
    m
}

fn hold_map() -> Keymap {
    let mut m = map();
    m.version = 2;
    m.bindings.key_top = Action::Hold {
        usage: 4,
        modifiers: vec![Modifier::Ctrl],
    };
    m
}

#[test]
fn both_configuration_versions_roundtrip_without_rewriting() {
    let tmp = Temp::new();
    let store = Store::new(tmp.0.clone());
    store.save(&map()).unwrap();
    assert_eq!(store.load().unwrap(), Some(map()));
    store.save(&hold_map()).unwrap();
    assert_eq!(store.load().unwrap(), Some(hold_map()));
    let mut invalid = hold_map();
    invalid.version = 1;
    assert!(Keymap::parse(&invalid.bytes()).is_err());
    for index in [4, 5] {
        let mut invalid = hold_map();
        let hold = invalid.bindings.key_top.clone();
        if index == 4 {
            invalid.bindings.knob_right = hold;
        } else {
            invalid.bindings.knob_left = hold;
        }
        assert!(Keymap::parse(&invalid.bytes()).is_err());
    }
    let valid = String::from_utf8(hold_map().bytes()).unwrap();
    for invalid in [
        valid.replace("\"hold\"", "\"toggle\""),
        valid.replace("\"usage\": 4", "\"usage\": 57"),
        valid.replace("\"ctrl\"", "\"ctrl\", \"ctrl\""),
        valid.replace("\"kind\": \"hold\"", "\"kind\": \"hold\", \"extra\": true"),
        valid.replace(
            "\"kind\": \"hold\"",
            "\"kind\": \"hold\", \"kind\": \"key\"",
        ),
    ] {
        assert!(Keymap::parse(invalid.as_bytes()).is_err());
    }
}

#[test]
fn strict_configuration_and_modifier_normalization() {
    let mut m = map();
    m.bindings.key_top = key(
        4,
        vec![
            Modifier::Super,
            Modifier::Ctrl,
            Modifier::Alt,
            Modifier::Shift,
        ],
    );
    assert_eq!(
        Keymap::parse(&m.bytes()).unwrap().bindings.key_top.keys(),
        vec![224, 225, 226, 227, 4]
    );
    let valid = String::from_utf8(map().bytes()).unwrap();
    for broken in [
        valid.replace("\"version\": 1", "\"version\": true"),
        valid.replace("\"version\": 1", "\"version\": 1.0"),
        valid.replace("\"version\": 1", "\"version\": 3"),
        valid.replace("\"version\": 1", "\"version\": 1, \"version\": 1"),
        valid.replace("\"usage\": 4", "\"usage\": 4, \"usage\": 5"),
        valid.replace("\"usage\": 4", "\"usage\": false"),
        valid.replace("\"usage\": 4", "\"usage\": 57"),
        valid.replace("\"ctrl\"", "\"ctrl\", \"ctrl\""),
        valid.replace("\"ctrl\"", "\"fn\""),
        valid.replace("\"kind\": \"none\"", "\"kind\": \"none\", \"usage\": 4"),
        valid.replace("\"key_bottom\"", "\"unknown_control\""),
        valid.replace("\"device\": \"AU05\"", "\"device\": \"AU05\", \"extra\": 1"),
        valid.replace("\"kind\": \"key\"", "\"kind\": \"key\", \"kind\": \"none\""),
    ] {
        assert!(
            Keymap::parse(broken.as_bytes()).is_err(),
            "accepted invalid JSON: {broken}"
        );
    }
    assert!(Keymap::parse(&vec![b' '; LIMIT + 1]).is_err());
    assert!(Keymap::parse(b"{\"bindings\": {}}").is_err());
    assert!(Keymap::parse(&[0xff, 0xfe]).is_err());
}

#[test]
fn all_and_only_specified_keys_have_platform_mapping() {
    for usage in 0..256 {
        assert_eq!(
            linux_key(usage).is_some(),
            allowed_usage(usage) || (224..=227).contains(&usage),
            "usage {usage}"
        );
    }
    assert_eq!(linux_key(4), Some(30));
    assert_eq!(linux_key(39), Some(11));
    assert_eq!(linux_key(79), Some(106));
}

#[test]
fn tea_vector_roundtrip_and_report_boundaries() {
    assert_eq!(
        protocol::tea([0; 8], false),
        [0x38, 0x90, 0xc4, 0x99, 0xa3, 0x60, 0xaa, 0xad]
    );
    for n in 0u8..=255 {
        let block = [n, 1, 2, 3, 4, 5, 6, 7];
        assert_eq!(protocol::tea(protocol::tea(block, false), true), block);
    }
    let frame: Vec<_> = (0..64).collect();
    let report = protocol::encode_frame(&frame);
    assert_eq!(report.len(), 64);
    assert_eq!(protocol::decode(&report).unwrap().as_slice(), &frame[..56]);
    for length in [0, 1, 5, 56, 63, 65] {
        assert!(protocol::decode(&vec![0x55; length]).is_err());
    }
    let mut wrong = report;
    wrong[0] = 0;
    assert!(protocol::decode(&wrong).is_err());
    assert!(protocol::encode(&[1, 6, 0x50, 4]).is_err());
    assert!(protocol::encode(protocol::HEARTBEAT).is_ok());
}

#[test]
fn events_use_physical_index_and_strict_state() {
    assert_eq!(
        protocol::parse(&[0x8b, 0x10, 0xff, 1, 4]),
        Ok(Event::Control {
            index: 4,
            down: true
        })
    );
    assert_eq!(
        protocol::parse(&[6, 3, 10, 17, 0]),
        Ok(Event::Status(Some(false)))
    );
    assert_eq!(
        protocol::parse(&[0x86, 3, 10, 17, 1]),
        Ok(Event::Status(Some(true)))
    );
    assert_eq!(protocol::parse(&[6, 3, 10, 17, 2]), Ok(Event::Status(None)));
    assert!(protocol::parse(&[0x8b, 0x10, 0, 2, 0]).is_err());
    assert!(protocol::parse(&[0x8b, 0x10, 0, 1, 6]).is_err());
    assert!(protocol::parse(&[0x8b, 0x10, 0, 1]).is_err());
}

#[test]
fn matches_vendor_report_descriptor_not_interface_order() {
    let descriptor = [
        0x06, 0xfc, 0xff, 0x09, 1, 0xa1, 1, 0x09, 2, 0x85, 0x55, 0x75, 8, 0x95, 0x3f, 0x81, 2,
        0x09, 3, 0x75, 8, 0x95, 0x3f, 0x91, 2, 0xc0,
    ];
    assert!(vendor_descriptor(&descriptor));
    let mut wrong = descriptor;
    wrong[1] = 1;
    assert!(!vendor_descriptor(&wrong));
    let mut wrong = descriptor;
    wrong[10] = 0x54;
    assert!(!vendor_descriptor(&wrong));
    for len in 0..24 {
        assert!(!vendor_descriptor(&descriptor[..len]));
    }
}

#[test]
fn atomic_store_roundtrip_and_corrupt_backup() {
    let tmp = Temp::new();
    let store = Store::new(tmp.0.clone());
    assert_eq!(store.load().unwrap(), None);
    assert!(!tmp.0.join("keymap.json").exists());
    let corrupt = b"{damaged \xff";
    fs::write(tmp.0.join("keymap.json"), corrupt).unwrap();
    assert!(store.load().is_err());
    assert_eq!(fs::read(tmp.0.join("keymap.json")).unwrap(), corrupt);
    store.save(&map()).unwrap();
    assert_eq!(store.load().unwrap(), Some(map()));
    let backups: Vec<_> = fs::read_dir(&tmp.0)
        .unwrap()
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("keymap.invalid-")
        })
        .collect();
    assert_eq!(backups.len(), 1);
    assert_eq!(fs::read(backups[0].path()).unwrap(), corrupt);
    fs::write(tmp.0.join("keymap.json"), b"broken again").unwrap();
    store.save(&Keymap::default()).unwrap();
    assert_eq!(
        fs::read_dir(&tmp.0)
            .unwrap()
            .flatten()
            .filter(|e| e
                .file_name()
                .to_string_lossy()
                .starts_with("keymap.invalid-"))
            .count(),
        2
    );
    use std::os::unix::fs::PermissionsExt;
    assert_eq!(
        fs::metadata(tmp.0.join("keymap.json"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
}

#[test]
fn write_failure_leaves_existing_bytes_and_no_temp_files() {
    let tmp = Temp::new();
    let destination = tmp.0.join("keymap.json");
    fs::create_dir(&destination).unwrap();
    fs::write(destination.join("sentinel"), "old").unwrap();
    assert!(atomic_write(&destination, b"new").is_err());
    assert_eq!(fs::read(destination.join("sentinel")).unwrap(), b"old");
    assert!(!fs::read_dir(&tmp.0)
        .unwrap()
        .flatten()
        .any(|e| e.file_name().to_string_lossy().ends_with(".tmp")));
    assert!(Store::new(tmp.0.clone()).save(&map()).is_err());
    assert_eq!(fs::read(destination.join("sentinel")).unwrap(), b"old");
}

#[derive(Default)]
struct OutputData {
    events: Vec<(u16, bool)>,
    fail_press: bool,
    fail_release: bool,
    denied: bool,
}
#[derive(Clone, Default)]
struct FakeOutput(Arc<Mutex<OutputData>>);
impl Output for FakeOutput {
    fn ensure(&mut self) -> Result<(), Issue> {
        if self.0.lock().unwrap().denied {
            Err(Issue::new("PERMISSION_REQUIRED", "output", "denied"))
        } else {
            Ok(())
        }
    }
    fn key(&mut self, usage: u16, down: bool) -> Result<(), Issue> {
        let mut data = self.0.lock().unwrap();
        if (down && data.fail_press) || (!down && data.fail_release) {
            return Err(Issue::new("OUTPUT_FAILED", "output", "injected"));
        }
        data.events.push((usage, down));
        Ok(())
    }
    fn close(&mut self) {}
}

#[test]
fn serial_chords_release_in_reverse_order_and_obey_actual_time() {
    let mut output = FakeOutput::default();
    let mut s = Scheduler::default();
    s.enqueue(&key(6, vec![Modifier::Ctrl, Modifier::Shift]));
    s.enqueue(&key(43, vec![Modifier::Alt]));
    s.tick(&mut output, 0).unwrap();
    assert_eq!(
        output.0.lock().unwrap().events,
        [(224, true), (225, true), (6, true)]
    );
    s.tick(&mut output, 39).unwrap();
    assert_eq!(output.0.lock().unwrap().events.len(), 3);
    s.tick(&mut output, 1000).unwrap();
    assert_eq!(
        output.0.lock().unwrap().events[3..],
        [(6, false), (225, false), (224, false)]
    );
    s.tick(&mut output, 1009).unwrap();
    assert_eq!(output.0.lock().unwrap().events.len(), 6);
    s.tick(&mut output, 1010).unwrap();
    assert_eq!(
        output.0.lock().unwrap().events[6..],
        [(226, true), (43, true)]
    );
    s.tick(&mut output, 1049).unwrap();
    assert!(s.debt());
    s.tick(&mut output, 1050).unwrap();
    assert!(!s.debt());
}

#[test]
fn queue_limit_excludes_active_action_and_drops_newest() {
    let mut output = FakeOutput::default();
    let mut s = Scheduler::default();
    s.enqueue(&key(4, vec![]));
    s.tick(&mut output, 0).unwrap();
    for _ in 0..32 {
        assert!(s.enqueue(&key(5, vec![])));
    }
    assert!(!s.enqueue(&key(6, vec![])));
    assert_eq!(s.queued(), 32);
    s.tick(&mut output, 40).unwrap();
    assert_eq!(output.0.lock().unwrap().events, [(4, true), (4, false)]);
    s.cancel(&mut output, 50).unwrap();
    assert_eq!(s.queued(), 0);
}

#[test]
fn release_failure_keeps_debt_and_partial_press_gets_cleanup() {
    let mut output = FakeOutput::default();
    let mut s = Scheduler::default();
    s.enqueue(&key(4, vec![Modifier::Ctrl]));
    s.tick(&mut output, 0).unwrap();
    output.0.lock().unwrap().fail_release = true;
    assert!(s.cancel(&mut output, 10).is_err());
    assert!(s.debt());
    s.enqueue(&key(5, vec![]));
    s.tick(&mut output, 1000).unwrap();
    assert_eq!(output.0.lock().unwrap().events.len(), 2);
    output.0.lock().unwrap().fail_release = false;
    s.cancel(&mut output, 1001).unwrap();
    assert!(!s.debt());
    output.0.lock().unwrap().fail_press = true;
    s.enqueue(&key(4, vec![]));
    assert!(s.tick(&mut output, 1100).is_err());
    assert!(s.debt());
    s.cancel(&mut output, 1110).unwrap();
    assert!(!s.debt());
}

struct DeviceData {
    reports: VecDeque<Vec<u8>>,
    commands: Vec<Vec<u8>>,
    connected: bool,
    missing: bool,
    status: Option<u8>,
    handover_fails: bool,
    multiple: bool,
}
impl Default for DeviceData {
    fn default() -> Self {
        Self {
            reports: VecDeque::new(),
            commands: vec![],
            connected: false,
            missing: false,
            status: Some(1),
            handover_fails: false,
            multiple: false,
        }
    }
}
#[derive(Clone, Default)]
struct FakeDevice(Arc<Mutex<DeviceData>>);
impl Transport for FakeDevice {
    fn connect(&mut self) -> Result<(), Issue> {
        let mut data = self.0.lock().unwrap();
        if data.missing {
            return Err(Issue::new("DEVICE_MISSING", "receiver", "missing"));
        }
        if data.multiple {
            return Err(Issue::new("DEVICE_MULTIPLE", "receiver", "multiple"));
        }
        data.connected = true;
        Ok(())
    }
    fn read(&mut self) -> Result<Option<Vec<u8>>, Issue> {
        let mut data = self.0.lock().unwrap();
        if data.missing {
            return Err(Issue::new("DEVICE_MISSING", "receiver", "removed"));
        }
        Ok(data.reports.pop_front())
    }
    fn send(&mut self, command: &[u8]) -> Result<(), Issue> {
        let mut data = self.0.lock().unwrap();
        if data.missing || (data.handover_fails && command != protocol::QUERY) {
            return Err(Issue::new("HANDOVER_FAILED", "receiver", "injected"));
        }
        data.commands.push(command.to_vec());
        if command == protocol::QUERY {
            if let Some(status) = data.status {
                data.reports
                    .push_back(protocol::encode_frame(&[6, 3, 10, 17, status]).to_vec());
            }
        }
        Ok(())
    }
    fn close(&mut self) {
        self.0.lock().unwrap().connected = false;
    }
}
fn runtime(saved: Option<Keymap>) -> (Runtime, FakeDevice, FakeOutput) {
    let device = FakeDevice::default();
    let output = FakeOutput::default();
    (
        Runtime::new(
            Box::new(device.clone()),
            Box::new(output.clone()),
            saved,
            None,
            true,
        ),
        device,
        output,
    )
}
fn edge(runtime: &mut Runtime, index: u8, down: bool, now: u64) {
    runtime.receive(
        &protocol::encode_frame(&[0x8b, 0x10, 0xfe, u8::from(down), index]),
        now,
    );
}
fn ready(runtime: &mut Runtime) {
    runtime.tick(0);
    runtime.tick(100);
    assert!(runtime.snapshot(100).forwarding);
}

#[test]
fn all_button_holds_follow_release_without_timer_or_duplicate_presses() {
    for index in 0..4 {
        let mut m = Keymap::default();
        let action = map().bindings.key_top;
        match index {
            0 => m.bindings.key_top = action,
            1 => m.bindings.key_middle = action,
            2 => m.bindings.key_bottom = action,
            _ => m.bindings.knob_press = action,
        }
        let (mut r, _, o) = runtime(Some(m));
        ready(&mut r);
        edge(&mut r, index, true, 110);
        for now in (120..10_000).step_by(10) {
            edge(&mut r, index, true, now);
            r.tick(now);
        }
        assert!(r.snapshot(9990).forwarding);
        assert!(!r.snapshot(9990).cleanup_pending);
        assert_eq!(o.0.lock().unwrap().events, [(224, true), (4, true)]);
        edge(&mut r, index, false, 10_000);
        edge(&mut r, index, false, 10_001);
        assert_eq!(
            o.0.lock().unwrap().events,
            [(224, true), (4, true), (4, false), (224, false)]
        );
        assert!(!r.scheduler.debt());
    }
}

#[test]
fn button_duration_is_physical_for_existing_and_legacy_hold_configurations() {
    for saved in [map(), hold_map()] {
        for duration in [1, 39, 40, 41, 200, 1000] {
            let (mut r, _, o) = runtime(Some(saved.clone()));
            ready(&mut r);
            edge(&mut r, 0, true, 110);
            // Press is immediate, with no long-press detection delay.
            assert_eq!(o.0.lock().unwrap().events, [(224, true), (4, true)]);
            r.tick(110 + duration - 1);
            assert_eq!(o.0.lock().unwrap().events.len(), 2);
            edge(&mut r, 0, false, 110 + duration);
            // Release follows the physical edge, including taps below 40 ms.
            assert_eq!(
                o.0.lock().unwrap().events,
                [(224, true), (4, true), (4, false), (224, false)]
            );
            r.tick(111 + duration);
            assert!(!r.scheduler.debt());
            assert_eq!(r.saved, Some(saved.clone()));
        }
    }
}

#[test]
fn short_hold_edges_in_one_read_batch_are_not_lost() {
    let (mut r, d, o) = runtime(Some(map()));
    ready(&mut r);
    for down in [true, false] {
        d.0.lock()
            .unwrap()
            .reports
            .push_back(protocol::encode_frame(&[0x8b, 0x10, 0xfe, u8::from(down), 0]).to_vec());
    }
    r.tick(110);
    assert_eq!(
        o.0.lock().unwrap().events,
        [(224, true), (4, true), (4, false), (224, false)]
    );
}

#[test]
fn concurrent_holds_and_taps_release_only_their_own_keys() {
    // Two hold buttons share Ctrl+A; a tap shares Ctrl, but uses B.
    let mut m = map();
    m.bindings.key_middle = m.bindings.key_top.clone();
    m.bindings.knob_right = key(5, vec![Modifier::Ctrl]);
    let (mut r, _, o) = runtime(Some(m));
    ready(&mut r);
    edge(&mut r, 0, true, 110);
    edge(&mut r, 1, true, 120);
    edge(&mut r, 4, true, 130);
    r.tick(130);
    r.tick(170);
    edge(&mut r, 0, false, 180);
    assert_eq!(
        o.0.lock().unwrap().events,
        [(224, true), (4, true), (5, true), (5, false)]
    );
    edge(&mut r, 1, false, 190);
    assert_eq!(o.0.lock().unwrap().events[4..], [(4, false), (224, false)]);
    assert!(!r.scheduler.debt());

    // A hold that starts during the same tap takes ownership past its timer.
    let mut m = map();
    m.bindings.knob_right = key(4, vec![Modifier::Ctrl]);
    let (mut r, _, o) = runtime(Some(m));
    ready(&mut r);
    edge(&mut r, 4, true, 110);
    r.tick(110);
    edge(&mut r, 0, true, 120);
    r.tick(150);
    assert_eq!(o.0.lock().unwrap().events, [(224, true), (4, true)]);
    edge(&mut r, 0, false, 160);
    assert_eq!(o.0.lock().unwrap().events[2..], [(4, false), (224, false)]);
    assert!(!r.scheduler.debt());
}

#[test]
fn held_shortcut_is_cleaned_up_on_every_stop_path() {
    for reason in [
        "pause",
        "sleep",
        "quit",
        "replace",
        "unplug",
        "permission",
        "offline",
        "timeout",
        "heartbeat",
        "retry",
    ] {
        let (mut r, d, o) = runtime(Some(map()));
        ready(&mut r);
        edge(&mut r, 0, true, 110);
        match reason {
            "pause" => r.pause(true, 120),
            "sleep" => r.sleep(true, 120),
            "quit" => r.quit(120),
            "replace" => r.replace(map(), 120),
            "retry" => r.retry(120),
            "unplug" => {
                d.0.lock().unwrap().missing = true;
                r.tick(120);
            }
            "permission" => {
                o.0.lock().unwrap().denied = true;
                r.tick(1000);
            }
            "offline" => {
                d.0.lock().unwrap().status = Some(0);
                r.tick(2000);
                r.tick(2001);
            }
            "timeout" => {
                d.0.lock().unwrap().status = None;
                r.tick(2000);
                r.tick(3000);
            }
            "heartbeat" => {
                d.0.lock().unwrap().handover_fails = true;
                r.tick(1000);
            }
            _ => unreachable!(),
        }
        assert_eq!(
            o.0.lock().unwrap().events,
            [(224, true), (4, true), (4, false), (224, false)],
            "{reason}"
        );
        assert!(!r.scheduler.debt(), "{reason}");
    }
}

#[test]
fn hold_release_failure_locks_forwarding_until_cleanup_and_retry() {
    let (mut r, _, o) = runtime(Some(map()));
    ready(&mut r);
    edge(&mut r, 0, true, 110);
    o.0.lock().unwrap().fail_release = true;
    edge(&mut r, 0, false, 120);
    assert_eq!(r.snapshot(120).state, "fault");
    assert!(r.snapshot(120).cleanup_pending);
    r.retry(130);
    assert_eq!(r.snapshot(130).state, "fault");
    o.0.lock().unwrap().fail_release = false;
    r.retry(140);
    r.tick(140);
    r.tick(240);
    assert!(r.snapshot(240).forwarding);
    assert!(!r.scheduler.debt());
    assert_eq!(
        o.0.lock().unwrap().events,
        [(224, true), (4, true), (4, false), (224, false)]
    );

    o.0.lock().unwrap().fail_press = true;
    edge(&mut r, 0, true, 250);
    assert_eq!(r.snapshot(250).state, "fault");
    assert!(!r.scheduler.debt());
    assert_eq!(o.0.lock().unwrap().events.last(), Some(&(224, false)));
}

#[test]
fn held_button_does_not_reactivate_after_resume_until_released() {
    let (mut r, _, o) = runtime(Some(map()));
    ready(&mut r);
    edge(&mut r, 0, true, 110);
    r.pause(true, 120);
    r.pause(false, 130);
    r.tick(130);
    r.tick(240);
    assert!(!r.snapshot(240).forwarding);
    edge(&mut r, 0, true, 250);
    edge(&mut r, 0, false, 260);
    r.tick(360);
    assert!(r.snapshot(360).forwarding);
    assert_eq!(o.0.lock().unwrap().events.len(), 4);
    edge(&mut r, 0, true, 370);
    edge(&mut r, 0, false, 380);
    assert_eq!(o.0.lock().unwrap().events.len(), 8);
}

#[test]
fn first_launch_queries_but_never_takes_over_until_explicit_empty_save() {
    let (mut r, d, o) = runtime(None);
    r.tick(0);
    r.tick(2000);
    assert_eq!(r.snapshot(2000).state, "configuration");
    assert!(d
        .0
        .lock()
        .unwrap()
        .commands
        .iter()
        .all(|c| c == protocol::QUERY));
    r.replace(Keymap::default(), 2100);
    r.tick(2100);
    r.tick(2200);
    assert!(r.snapshot(2200).forwarding);
    edge(&mut r, 0, true, 2210);
    r.tick(2210);
    assert!(o.0.lock().unwrap().events.is_empty());
}

#[test]
fn repeated_press_suppressed_rotations_remain_distinct() {
    let (mut r, _, o) = runtime(Some(map()));
    ready(&mut r);
    edge(&mut r, 0, true, 110);
    r.tick(110);
    edge(&mut r, 0, true, 120);
    r.tick(150);
    assert_eq!(o.0.lock().unwrap().events.len(), 2);
    edge(&mut r, 0, false, 160);
    edge(&mut r, 0, true, 170);
    r.tick(170);
    r.tick(210);
    edge(&mut r, 4, true, 220);
    edge(&mut r, 4, true, 220);
    r.tick(220);
    r.tick(260);
    r.tick(270);
    r.tick(310);
    let data = o.0.lock().unwrap();
    assert_eq!(data.events.iter().filter(|e| **e == (4, true)).count(), 2);
    assert_eq!(data.events.iter().filter(|e| **e == (79, true)).count(), 2);
}

#[test]
fn held_inputs_quarantined_and_manual_resync_requires_silence() {
    let (mut r, _, o) = runtime(Some(map()));
    edge(&mut r, 0, true, 0);
    r.tick(0);
    r.tick(3100);
    assert!(!r.snapshot(3100).forwarding);
    assert!(r.snapshot(3100).resync_required);
    r.resync(3110);
    r.tick(3209);
    assert!(!r.snapshot(3209).forwarding);
    edge(&mut r, 4, true, 3210);
    r.tick(3309);
    assert!(!r.snapshot(3309).forwarding);
    r.tick(3310);
    assert!(r.snapshot(3310).forwarding);
    assert!(o.0.lock().unwrap().events.is_empty());
}

#[test]
fn pause_survives_retry_save_and_reconnect_without_heartbeats() {
    let (mut r, d, _) = runtime(Some(map()));
    ready(&mut r);
    r.pause(true, 110);
    let count = d.0.lock().unwrap().commands.len();
    r.tick(5000);
    assert_eq!(d.0.lock().unwrap().commands.len(), count);
    r.retry(5010);
    r.tick(5010);
    assert_eq!(d.0.lock().unwrap().commands.len(), count + 1);
    r.replace(Keymap::default(), 5020);
    r.tick(8000);
    assert_eq!(r.snapshot(8000).state, "paused");
    r.sleep(true, 8100);
    r.sleep(false, 9000);
    r.tick(9000);
    assert!(r.snapshot(9000).paused);
    assert!(!r.snapshot(9000).forwarding);
}

#[test]
fn query_timeout_is_unknown_not_offline_and_valid_reply_recovers() {
    let (mut r, d, _) = runtime(Some(map()));
    ready(&mut r);
    d.0.lock().unwrap().status = None;
    r.tick(2000);
    assert!(r.snapshot(2000).forwarding);
    r.tick(3000);
    let snap = r.snapshot(3000);
    assert!(!snap.forwarding);
    assert!(snap
        .issues
        .iter()
        .any(|e| e.code == "DEVICE_STATUS_UNKNOWN"));
    d.0.lock().unwrap().status = Some(0);
    r.tick(4000);
    r.tick(4001);
    assert!(r
        .snapshot(4001)
        .issues
        .iter()
        .any(|e| e.code == "DEVICE_OFFLINE"));
    d.0.lock().unwrap().status = Some(1);
    r.tick(6000);
    r.tick(6001);
    r.tick(6101);
    assert!(r.snapshot(6101).forwarding);
}

#[test]
fn output_fault_cancels_queue_and_requires_explicit_retry() {
    let (mut r, _, o) = runtime(Some(map()));
    ready(&mut r);
    o.0.lock().unwrap().fail_press = true;
    edge(&mut r, 4, true, 110);
    edge(&mut r, 0, true, 110);
    r.tick(110);
    assert_eq!(r.snapshot(110).state, "fault");
    assert_eq!(r.snapshot(110).queued, 0);
    o.0.lock().unwrap().fail_press = false;
    r.tick(2000);
    r.tick(2100);
    assert_eq!(r.snapshot(2100).state, "fault");
    edge(&mut r, 0, false, 2110);
    r.retry(2200);
    r.tick(2200);
    r.tick(2300);
    assert!(r.snapshot(2300).forwarding);
}

#[test]
fn configuration_replacement_cancels_old_action_and_preserves_observed_hold() {
    let (mut r, _, o) = runtime(Some(map()));
    ready(&mut r);
    edge(&mut r, 0, true, 110);
    r.tick(110);
    edge(&mut r, 4, true, 115);
    r.replace(Keymap::default(), 120);
    assert_eq!(r.snapshot(120).queued, 0);
    assert_eq!(
        o.0.lock().unwrap().events,
        [(224, true), (4, true), (4, false), (224, false)]
    );
    r.tick(130);
    r.tick(500);
    assert!(!r.snapshot(500).forwarding);
    edge(&mut r, 0, false, 510);
    r.tick(610);
    assert!(r.snapshot(610).forwarding);
}

#[test]
fn unplug_releases_and_auto_recovers_without_fault_latch() {
    let (mut r, d, o) = runtime(Some(map()));
    ready(&mut r);
    edge(&mut r, 0, true, 110);
    r.tick(110);
    d.0.lock().unwrap().missing = true;
    r.tick(120);
    assert!(!r.snapshot(120).forwarding);
    assert_eq!(o.0.lock().unwrap().events.last(), Some(&(224, false)));
    d.0.lock().unwrap().missing = false;
    r.tick(2200);
    edge(&mut r, 0, false, 2201);
    r.tick(2301);
    assert!(r.snapshot(2301).forwarding, "{:?}", r.snapshot(2301));
}

#[test]
fn handover_failure_locks_and_multiple_devices_wait_for_retry() {
    let (mut r, d, _) = runtime(Some(map()));
    d.0.lock().unwrap().handover_fails = true;
    r.tick(0);
    assert_eq!(r.snapshot(0).state, "fault");
    d.0.lock().unwrap().handover_fails = false;
    r.tick(2000);
    assert_eq!(r.snapshot(2000).state, "fault");
    r.retry(2010);
    r.tick(2010);
    r.tick(2110);
    assert!(r.snapshot(2110).forwarding);
    let (mut r, d, _) = runtime(Some(map()));
    d.0.lock().unwrap().multiple = true;
    r.tick(0);
    d.0.lock().unwrap().multiple = false;
    r.tick(5000);
    assert!(!r.snapshot(5000).connected);
    r.retry(5010);
    r.tick(5010);
    r.tick(5110);
    assert!(r.snapshot(5110).forwarding);
}

#[test]
fn five_per_second_for_two_minutes_does_not_starve_heartbeats() {
    let (mut r, d, o) = runtime(Some(map()));
    ready(&mut r);
    for n in 0..600 {
        let now = 200 + n * 200;
        edge(&mut r, 4, true, now);
        r.tick(now);
        r.tick(now + 40);
        r.tick(now + 100);
    }
    assert_eq!(
        o.0.lock().unwrap().events.iter().filter(|e| e.1).count(),
        600
    );
    assert!(
        d.0.lock()
            .unwrap()
            .commands
            .iter()
            .filter(|c| c.as_slice() == protocol::HEARTBEAT)
            .count()
            >= 119
    );
}

fn wait_for(service: &Service, predicate: impl Fn(&clanzi_core::service::Model) -> bool) {
    let start = Instant::now();
    while !predicate(&service.snapshot()) {
        assert!(start.elapsed() < Duration::from_secs(3));
        thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn service_draft_import_save_undo_and_quit() {
    let tmp = Temp::new();
    let service = Service::start(true, Some(tmp.0.clone()));
    service
        .command(Command::Edit {
            keymap: Box::new(map()),
        })
        .unwrap();
    assert!(service.snapshot().dirty);
    assert!(service.snapshot().saved.is_none());
    assert!(service
        .command(Command::Import {
            content: "{}".into(),
            discard: true
        })
        .is_err());
    assert_eq!(service.snapshot().draft, map());
    service.command(Command::RequestQuit).unwrap();
    assert!(service.snapshot().quit_prompt);
    service.command(Command::CancelQuit).unwrap();
    assert!(!service.snapshot().quit_prompt);
    service.command(Command::Save).unwrap();
    wait_for(&service, |m| !m.saving);
    assert_eq!(service.snapshot().saved, Some(map()));
    assert_eq!(Store::new(tmp.0.clone()).load().unwrap(), Some(map()));
    service
        .command(Command::Edit {
            keymap: Box::new(Keymap::default()),
        })
        .unwrap();
    service
        .command(Command::Language {
            language: "zh".into(),
        })
        .unwrap();
    assert!(service.snapshot().dirty);
    service.command(Command::Undo).unwrap();
    assert_eq!(service.snapshot().draft, map());
    assert!(!service.snapshot().dirty);
    service.command(Command::RequestQuit).unwrap();
    wait_for(&service, |m| m.exited);
}

#[test]
fn save_failure_cancels_exit_and_keeps_draft() {
    let tmp = Temp::new();
    let path = tmp.0.join("not-a-directory");
    fs::write(&path, "blocker").unwrap();
    let service = Service::start(true, Some(path));
    service
        .command(Command::Edit {
            keymap: Box::new(map()),
        })
        .unwrap();
    service.command(Command::SaveAndQuit).unwrap();
    wait_for(&service, |m| !m.saving);
    let model = service.snapshot();
    assert!(!model.exited);
    assert!(model.dirty);
    assert_eq!(model.draft, map());
    assert_eq!(model.error.unwrap().code, "CONFIG_SAVE_FAILED");
    service.command(Command::QuitWithoutSaving).unwrap();
    wait_for(&service, |m| m.exited);
}

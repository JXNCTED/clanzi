use crate::{
    config::{Action, Keymap},
    protocol::{self, Event},
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Issue {
    pub code: String,
    pub area: String,
    pub detail: String,
}
impl Issue {
    pub fn new(code: &str, area: &str, detail: impl ToString) -> Self {
        Self {
            code: code.into(),
            area: area.into(),
            detail: detail.to_string(),
        }
    }
}
pub trait Output: Send {
    fn ensure(&mut self) -> Result<(), Issue>;
    fn ready(&self) -> bool {
        true
    }
    fn key(&mut self, usage: u16, down: bool) -> Result<(), Issue>;
    fn close(&mut self);
}
pub trait Transport: Send {
    fn connect(&mut self) -> Result<(), Issue>;
    fn read(&mut self) -> Result<Option<Vec<u8>>, Issue>;
    fn send(&mut self, command: &[u8]) -> Result<(), Issue>;
    fn close(&mut self);
}

#[derive(Default)]
pub struct Scheduler {
    queue: VecDeque<Vec<u16>>,
    tap: Vec<u16>,
    holds: BTreeMap<u8, Vec<u16>>,
    held: Vec<u16>,
    release_at: Option<u64>,
    next_at: u64,
}
impl Scheduler {
    pub fn enqueue(&mut self, action: &Action) -> bool {
        let keys = action.keys();
        if keys.is_empty() {
            return true;
        }
        if self.queue.len() >= 32 {
            return false;
        }
        self.queue.push_back(keys);
        true
    }
    pub fn debt(&self) -> bool {
        !self.held.is_empty()
    }
    pub fn queued(&self) -> usize {
        self.queue.len()
    }
    pub fn cancel(&mut self, output: &mut dyn Output, now: u64) -> Result<(), Issue> {
        self.queue.clear();
        self.tap.clear();
        self.holds.clear();
        self.release_at = None;
        self.next_at = now + 10;
        self.release(output)
    }
    pub fn hold(
        &mut self,
        index: u8,
        action: &Action,
        output: &mut dyn Output,
    ) -> Result<(), Issue> {
        if self.holds.contains_key(&index) {
            return Ok(());
        }
        let keys = action.keys();
        self.holds.insert(index, keys.clone());
        self.press(&keys, output)
    }
    pub fn release_control(&mut self, index: u8, output: &mut dyn Output) -> Result<(), Issue> {
        self.holds.remove(&index);
        self.release(output)
    }
    fn needed(&self, usage: u16) -> bool {
        self.tap.contains(&usage) || self.holds.values().any(|keys| keys.contains(&usage))
    }
    fn press(&mut self, keys: &[u16], output: &mut dyn Output) -> Result<(), Issue> {
        for &usage in keys {
            if !self.held.contains(&usage) {
                // 即使写操作部分成功后报错，也必须安排对应的松开。
                self.held.push(usage);
                output.key(usage, true)?;
            }
        }
        Ok(())
    }
    fn release(&mut self, output: &mut dyn Output) -> Result<(), Issue> {
        let mut failure = None;
        // 所有松开均尝试；失败的键继续保留为清理债务。
        for i in (0..self.held.len()).rev() {
            // 多个控件共用的键，等最后一个使用者松开后再释放。
            if self.needed(self.held[i]) {
                continue;
            }
            match output.key(self.held[i], false) {
                Ok(()) => {
                    self.held.remove(i);
                }
                Err(e) => failure = Some(e),
            }
        }
        failure.map_or(Ok(()), Err)
    }
    pub fn tick(&mut self, output: &mut dyn Output, now: u64) -> Result<(), Issue> {
        if let Some(deadline) = self.release_at {
            if now >= deadline {
                self.release_at = None;
                self.tap.clear();
                self.next_at = now + 10;
                self.release(output)?;
            }
            return Ok(());
        }
        if !self.tap.is_empty()
            || self.held.iter().any(|&usage| !self.needed(usage))
            || now < self.next_at
        {
            return Ok(());
        }
        if let Some(keys) = self.queue.pop_front() {
            self.tap = keys.clone();
            self.press(&keys, output)?;
            self.release_at = Some(now + 40);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Online {
    Querying,
    Online,
    Offline,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Snapshot {
    pub state: String,
    pub connected: bool,
    pub online: Online,
    pub paused: bool,
    pub forwarding: bool,
    pub synchronizing: bool,
    pub resync_required: bool,
    pub issues: Vec<Issue>,
    pub queued: usize,
    pub cleanup_pending: bool,
    pub query_completed_ms: Option<u64>,
    pub demo: bool,
}

pub struct Runtime {
    pub saved: Option<Keymap>,
    pub config_error: Option<Issue>,
    pub paused: bool,
    pub sleeping: bool,
    pub quitting: bool,
    pub scheduler: Scheduler,
    transport: Box<dyn Transport>,
    output: Box<dyn Output>,
    connected: bool,
    online: Online,
    device_error: Option<Issue>,
    output_error: Option<Issue>,
    fault: Option<Issue>,
    diagnostics: Vec<Issue>,
    observed: BTreeSet<u8>,
    takeover: bool,
    forwarding: bool,
    sync_started: u64,
    last_activity: u64,
    next_discovery: u64,
    next_query: u64,
    query_deadline: Option<u64>,
    query_completed: Option<u64>,
    query_failure: &'static str,
    next_heartbeat: u64,
    next_permission: u64,
    output_ready: bool,
    demo: bool,
}

impl Runtime {
    pub fn new(
        transport: Box<dyn Transport>,
        output: Box<dyn Output>,
        saved: Option<Keymap>,
        config_error: Option<Issue>,
        demo: bool,
    ) -> Self {
        Self {
            saved,
            config_error,
            paused: false,
            sleeping: false,
            quitting: false,
            scheduler: Scheduler::default(),
            transport,
            output,
            connected: false,
            online: Online::Querying,
            device_error: None,
            output_error: None,
            fault: None,
            diagnostics: vec![],
            observed: BTreeSet::new(),
            takeover: false,
            forwarding: false,
            sync_started: 0,
            last_activity: 0,
            next_discovery: 0,
            next_query: 0,
            query_deadline: None,
            query_completed: None,
            query_failure: "query pending",
            next_heartbeat: 0,
            next_permission: 0,
            output_ready: false,
            demo,
        }
    }
    fn diagnostic(&mut self, issue: Issue) {
        self.diagnostics.retain(|e| e.code != issue.code);
        self.diagnostics.push(issue);
        if self.diagnostics.len() > 8 {
            self.diagnostics.remove(0);
        }
    }
    fn stop(&mut self, now: u64) {
        self.forwarding = false;
        if let Err(e) = self.scheduler.cancel(self.output.as_mut(), now) {
            self.fault = Some(e);
        }
        if self.takeover && self.connected {
            if let Err(e) = self.transport.send(protocol::HAND_BACK) {
                self.fault = Some(Issue::new("HANDOVER_FAILED", "device", e.detail));
            }
        }
        self.takeover = false;
    }
    fn disconnected(&mut self, issue: Issue, now: u64) {
        // 已拔出的接收器无法交还；仍清理输出，但不把正常拔插锁成交接故障。
        if issue.code == "DEVICE_MISSING" {
            self.connected = false;
        }
        self.stop(now);
        self.transport.close();
        self.connected = false;
        self.device_error = Some(issue);
        self.online = Online::Querying;
        self.query_deadline = None;
        self.query_completed = None;
        self.next_discovery = now + 2000;
    }
    pub fn replace(&mut self, map: Keymap, now: u64) {
        self.stop(now);
        self.saved = Some(map);
        self.config_error = None;
    }
    pub fn pause(&mut self, paused: bool, now: u64) {
        self.stop(now);
        self.paused = paused;
        self.query_deadline = None;
        if !paused {
            self.online = Online::Querying;
            self.query_completed = None;
            self.next_query = now;
            self.next_discovery = now;
        }
    }
    pub fn sleep(&mut self, sleeping: bool, now: u64) {
        self.stop(now);
        self.sleeping = sleeping;
        self.transport.close();
        self.connected = false;
        self.query_deadline = None;
        self.online = Online::Querying;
        self.query_completed = None;
        self.next_discovery = now;
    }
    pub fn retry(&mut self, now: u64) {
        self.stop(now);
        // 先恢复输出并清偿释放债务，才能解除故障锁。
        match self
            .output
            .ensure()
            .and_then(|_| self.scheduler.cancel(self.output.as_mut(), now))
        {
            Ok(()) => {
                self.fault = None;
                self.output_error = None;
                self.output_ready = self.output.ready();
            }
            Err(e) => {
                self.output_error = Some(e);
                self.output_ready = false;
            }
        }
        self.diagnostics.clear();
        self.transport.close();
        self.connected = false;
        self.device_error = None;
        self.online = Online::Querying;
        self.query_completed = None;
        self.query_deadline = None;
        self.next_discovery = now;
        // 暂停期间也允许这一次明确的只读检查。
        if !self.sleeping && !self.quitting {
            self.connect(now, true);
        }
    }
    pub fn resync(&mut self, now: u64) {
        if self.takeover && !self.forwarding && now.saturating_sub(self.sync_started) >= 3000 {
            self.observed.clear();
            self.last_activity = now;
            self.sync_started = now;
        }
    }
    fn connect(&mut self, now: u64, explicit: bool) {
        self.next_discovery = now + 2000;
        match self.transport.connect() {
            Ok(()) => {
                self.connected = true;
                self.device_error = None;
                self.online = Online::Querying;
                self.query_completed = None;
                self.next_query = now;
                if explicit || !self.paused {
                    self.query(now);
                }
            }
            Err(e) => self.device_error = Some(e),
        }
    }
    fn query(&mut self, now: u64) {
        match self.transport.send(protocol::QUERY) {
            Ok(()) => {
                self.query_deadline = Some(now + 1000);
                self.next_query = now + 2000;
            }
            Err(e) => self.disconnected(e, now),
        }
    }
    pub fn receive(&mut self, report: &[u8], now: u64) {
        match protocol::decode(report).and_then(|f| protocol::parse(&f)) {
            Ok(Event::Status(status)) if self.query_deadline.is_some() => {
                self.query_deadline = None;
                self.query_completed = Some(now);
                self.online = match status {
                    Some(true) => Online::Online,
                    Some(false) => Online::Offline,
                    None => Online::Unknown,
                };
                self.query_failure = if status.is_none() {
                    "invalid status reply"
                } else {
                    ""
                };
                if status != Some(true) {
                    self.stop(now);
                }
            }
            Ok(Event::Control { index, down }) => {
                self.last_activity = now;
                let first = if index < 4 {
                    if down {
                        self.observed.insert(index)
                    } else {
                        self.observed.remove(&index);
                        false
                    }
                } else {
                    down
                };
                if self.forwarding {
                    let result = if !down && index < 4 {
                        self.scheduler.release_control(index, self.output.as_mut())
                    } else if down && first {
                        if let Some(action) =
                            self.saved.as_ref().and_then(|m| m.bindings.get(index))
                        {
                            if index < 4 && !matches!(action, Action::None) {
                                self.scheduler.hold(index, action, self.output.as_mut())
                            } else {
                                if !self.scheduler.enqueue(action) {
                                    self.diagnostic(Issue::new(
                                        "INPUT_OVERFLOW",
                                        "input",
                                        "pending action limit reached",
                                    ));
                                }
                                Ok(())
                            }
                        } else {
                            Ok(())
                        }
                    } else {
                        Ok(())
                    };
                    if let Err(e) = result {
                        self.fault = Some(e);
                        self.stop(now);
                    }
                }
            }
            Ok(_) => (),
            Err(e) => self.diagnostic(Issue::new("REPORT_INVALID", "device", e)),
        }
    }
    pub fn tick(&mut self, now: u64) {
        if self.quitting || self.sleeping {
            return;
        }
        if now >= self.next_permission {
            self.next_permission = now + 1000;
            match self.output.ensure() {
                Ok(()) => {
                    self.output_ready = self.output.ready();
                    self.output_error = None;
                    if self.scheduler.debt() && self.fault.is_some() {
                        let _ = self.scheduler.cancel(self.output.as_mut(), now);
                    }
                }
                Err(e) => {
                    self.output_ready = false;
                    self.output_error = Some(e);
                    self.stop(now);
                }
            }
        }
        let auto_discover = self
            .device_error
            .as_ref()
            .is_none_or(|e| matches!(e.code.as_str(), "DEVICE_MISSING" | "PERMISSION_REQUIRED"));
        if !self.connected && now >= self.next_discovery && auto_discover && !self.paused {
            self.connect(now, false);
        }
        if !self.connected {
            return;
        }
        // 每轮有限读取，避免报告洪水饿死查询、心跳和松开调度。
        let mut drained = false;
        for _ in 0..64 {
            match self.transport.read() {
                Ok(Some(report)) => self.receive(&report, now),
                Ok(None) => {
                    drained = true;
                    break;
                }
                Err(e) => {
                    self.disconnected(e, now);
                    return;
                }
            }
        }
        if self.query_deadline.is_some_and(|deadline| now >= deadline) {
            self.query_deadline = None;
            self.query_completed = Some(now);
            self.online = Online::Unknown;
            self.query_failure = "status query timed out";
            self.stop(now);
        }
        if !self.paused && now >= self.next_query && self.query_deadline.is_none() {
            self.query(now);
        }
        let ready = self.connected
            && !self.paused
            && self.saved.is_some()
            && self.online == Online::Online
            && self.output_ready
            && self.fault.is_none()
            && !self.scheduler.debt();
        // 持键期间的 scheduler.debt 是正常状态，仅阻止重新接管。
        if ready && !self.takeover {
            match self.transport.send(protocol::TAKEOVER) {
                Ok(()) => {
                    self.takeover = true;
                    self.sync_started = now;
                    self.last_activity = now;
                    self.next_heartbeat = now;
                }
                Err(e) => {
                    self.fault = Some(Issue::new("HANDOVER_FAILED", "device", e.detail));
                    self.stop(now);
                }
            }
        }
        if self.takeover && now >= self.next_heartbeat {
            self.next_heartbeat = now + 1000;
            if let Err(e) = self.transport.send(protocol::HEARTBEAT) {
                self.fault = Some(Issue::new("HANDOVER_FAILED", "device", e.detail));
                self.stop(now);
                return;
            }
        }
        if self.takeover
            && !self.forwarding
            && drained
            && self.observed.is_empty()
            && now.saturating_sub(self.last_activity) >= 100
        {
            self.forwarding = true;
        }
        if self.forwarding {
            if let Err(e) = self.scheduler.tick(self.output.as_mut(), now) {
                self.fault = Some(e);
                self.stop(now);
            }
        }
    }
    pub fn quit(&mut self, now: u64) {
        self.quitting = true;
        self.stop(now);
    }
    pub fn cleanup(&mut self, now: u64) -> bool {
        self.scheduler.cancel(self.output.as_mut(), now).is_ok()
    }
    pub fn close(&mut self) {
        self.transport.close();
        self.output.close();
    }
    pub fn snapshot(&self, now: u64) -> Snapshot {
        let mut issues: Vec<Issue> = [
            &self.fault,
            &self.output_error,
            &self.config_error,
            &self.device_error,
        ]
        .into_iter()
        .flatten()
        .cloned()
        .collect();
        if self.online == Online::Offline {
            issues.push(Issue::new("DEVICE_OFFLINE", "device", "status=0"));
        }
        if self.online == Online::Unknown {
            issues.push(Issue::new(
                "DEVICE_STATUS_UNKNOWN",
                "device",
                self.query_failure,
            ));
        }
        issues.extend(self.diagnostics.clone());
        let state = if self.quitting {
            "quitting"
        } else if self.fault.is_some() {
            "fault"
        } else if self.paused {
            "paused"
        } else if self.output_error.is_some()
            || self
                .device_error
                .as_ref()
                .is_some_and(|e| e.code == "PERMISSION_REQUIRED")
        {
            "permission"
        } else if self.saved.is_none() {
            "configuration"
        } else if self.device_error.is_some() {
            "device"
        } else if matches!(self.online, Online::Offline | Online::Unknown) {
            "offline"
        } else if self.forwarding {
            "forwarding"
        } else {
            "connecting"
        };
        Snapshot {
            state: state.into(),
            connected: self.connected,
            online: self.online,
            paused: self.paused,
            forwarding: self.forwarding,
            synchronizing: self.takeover && !self.forwarding,
            resync_required: self.takeover
                && !self.forwarding
                && !self.observed.is_empty()
                && now.saturating_sub(self.sync_started) >= 3000,
            issues,
            queued: self.scheduler.queued(),
            cleanup_pending: self.scheduler.debt() && self.fault.is_some(),
            query_completed_ms: self.query_completed,
            demo: self.demo,
        }
    }
}

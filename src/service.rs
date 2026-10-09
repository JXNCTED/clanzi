use crate::{
    config::Keymap,
    linux::{DemoOutput, DemoTransport, Hidraw, UInput},
    runtime::{Issue, Runtime, Snapshot},
    storage::{self, Store},
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    os::unix::fs::OpenOptionsExt,
    path::PathBuf,
    process::{Child, Command as ProcessCommand, Stdio},
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Serialize)]
pub struct Model {
    pub draft: Keymap,
    pub saved: Option<Keymap>,
    pub language: String,
    pub saving: bool,
    pub dirty: bool,
    pub quit_prompt: bool,
    pub exited: bool,
    pub error: Option<Issue>,
    pub notice: Option<String>,
    pub runtime: Snapshot,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    Edit { keymap: Box<Keymap> },
    Save,
    Undo,
    Import { content: String, discard: bool },
    Language { language: String },
    Pause { paused: bool },
    Retry,
    Resync,
    RequestQuit,
    CancelQuit,
    QuitWithoutSaving,
    SaveAndQuit,
}
enum Message {
    Command(Command, mpsc::SyncSender<Result<(), String>>),
    Saved(Result<Keymap, String>),
    Sleep(bool),
}

pub struct Service {
    pub model: Arc<Mutex<Model>>,
    sender: mpsc::Sender<Message>,
    sleep_monitor: Option<Child>,
}
impl Service {
    pub fn start(demo: bool, directory: Option<PathBuf>) -> Self {
        let directory = directory.unwrap_or_else(|| {
            if demo {
                std::env::temp_dir().join(format!("clanzi-demo-{}", std::process::id()))
            } else {
                storage::config_dir()
            }
        });
        let store = Store::new(directory.clone());
        let language = store.language();
        let loaded = store.load();
        let error = loaded
            .as_ref()
            .err()
            .map(|e| Issue::new("CONFIG_INVALID", "config", e));
        let saved = loaded.ok().flatten();
        let mut runtime = Runtime::new(
            if demo {
                Box::new(DemoTransport::default())
            } else {
                Box::new(Hidraw::default())
            },
            if demo {
                Box::new(DemoOutput)
            } else {
                Box::new(UInput::default())
            },
            saved.clone(),
            error,
            demo,
        );
        let model = Arc::new(Mutex::new(Model {
            draft: saved.clone().unwrap_or_default(),
            saved,
            language,
            saving: false,
            dirty: false,
            quit_prompt: false,
            exited: false,
            error: None,
            notice: None,
            runtime: runtime.snapshot(0),
        }));
        let (sender, receiver) = mpsc::channel();
        let worker_sender = sender.clone();
        let worker_model = model.clone();
        let sleep_monitor = if demo {
            None
        } else {
            monitor_sleep(sender.clone())
        };
        let monitoring = sleep_monitor.is_some();
        thread::spawn(move || {
            let origin = Instant::now();
            let mut quit_after_save = false;
            let mut stopping: Option<Instant> = None;
            let mut last_snapshot = None;
            let mut logger = Logger::new(directory.join("logs"));
            logger.write("START version=0.1.0 platform=linux");
            if !demo && !monitoring {
                logger.write("SLEEP_MONITOR_FAILED dbus-monitor unavailable");
            }
            loop {
                let now = origin.elapsed().as_millis() as u64;
                // 每轮只处理有限命令；文件持久化在另一个线程完成。
                for _ in 0..16 {
                    let Ok(message) = receiver.try_recv() else {
                        break;
                    };
                    let mut model = worker_model.lock().unwrap();
                    match message {
                        Message::Sleep(sleeping) => runtime.sleep(sleeping, now),
                        Message::Saved(result) => {
                            model.saving = false;
                            match result {
                                Ok(map) => {
                                    runtime.replace(map.clone(), now);
                                    model.draft = map.clone();
                                    model.saved = Some(map);
                                    model.dirty = false;
                                    model.error = None;
                                    model.notice = Some("saved".into());
                                    if quit_after_save {
                                        runtime.quit(now);
                                        stopping = Some(Instant::now());
                                    }
                                }
                                Err(e) => {
                                    model.error =
                                        Some(Issue::new("CONFIG_SAVE_FAILED", "config", e));
                                    model.quit_prompt = false;
                                    quit_after_save = false;
                                }
                            }
                        }
                        Message::Command(command, reply) => {
                            let result = (|| -> Result<(), String> {
                                if stopping.is_some() {
                                    return Err("application is quitting".into());
                                }
                                match command {
                                    Command::Edit { keymap } => {
                                        if model.saving {
                                            return Err("save in progress".into());
                                        }
                                        model.draft = (*keymap).validate()?;
                                        model.notice = None;
                                    }
                                    Command::Undo => {
                                        if model.saving {
                                            return Err("save in progress".into());
                                        }
                                        model.draft = model.saved.clone().unwrap_or_default();
                                        model.error = None;
                                        model.notice = None;
                                    }
                                    Command::Import { content, discard } => {
                                        if model.saving {
                                            return Err("save in progress".into());
                                        }
                                        // 验证成功后才考虑替换；失败不改变草稿及运行配置。
                                        let imported = Keymap::parse(content.as_bytes())
                                            .inspect_err(|e| {
                                                model.error =
                                                    Some(Issue::new("CONFIG_INVALID", "import", e));
                                            })?;
                                        if model.dirty && !discard {
                                            return Err("discard confirmation required".into());
                                        }
                                        model.draft = imported;
                                        model.error = None;
                                        model.notice = Some("imported".into());
                                    }
                                    Command::Language { language } => {
                                        store.set_language(&language)?;
                                        model.language = language;
                                    }
                                    Command::Pause { paused } => runtime.pause(paused, now),
                                    Command::Retry => runtime.retry(now),
                                    Command::Resync => runtime.resync(now),
                                    Command::CancelQuit => model.quit_prompt = false,
                                    Command::RequestQuit => {
                                        if model.saving {
                                            quit_after_save = true;
                                        } else if model.dirty {
                                            model.quit_prompt = true;
                                        } else {
                                            runtime.quit(now);
                                            stopping = Some(Instant::now());
                                        }
                                    }
                                    Command::QuitWithoutSaving => {
                                        if model.saving {
                                            quit_after_save = true;
                                        } else {
                                            model.quit_prompt = false;
                                            runtime.quit(now);
                                            stopping = Some(Instant::now());
                                        }
                                    }
                                    Command::Save | Command::SaveAndQuit => {
                                        let exit = matches!(command, Command::SaveAndQuit);
                                        if model.saving {
                                            if exit {
                                                quit_after_save = true;
                                            } else {
                                                return Err("save in progress".into());
                                            }
                                        } else if model.saved.as_ref() == Some(&model.draft) {
                                            if exit {
                                                runtime.quit(now);
                                                stopping = Some(Instant::now());
                                            }
                                        } else {
                                            let map = model.draft.clone().validate()?;
                                            model.saving = true;
                                            model.error = None;
                                            model.notice = None;
                                            model.quit_prompt = false;
                                            quit_after_save = exit;
                                            let directory = directory.clone();
                                            let sender = worker_sender.clone();
                                            thread::spawn(move || {
                                                let result =
                                                    Store::new(directory).save(&map).map(|_| map);
                                                let _ = sender.send(Message::Saved(result));
                                            });
                                        }
                                    }
                                }
                                model.dirty =
                                    model.draft != model.saved.clone().unwrap_or_default();
                                Ok(())
                            })();
                            if let Err(ref detail) = result {
                                if model.error.is_none() {
                                    model.error =
                                        Some(Issue::new("OPERATION_FAILED", "application", detail));
                                }
                            }
                            let _ = reply.send(result);
                        }
                    }
                }
                if let Some(started) = stopping {
                    if runtime.cleanup(now) || started.elapsed() >= Duration::from_millis(2500) {
                        if runtime.scheduler.debt() {
                            logger.write("OUTPUT_FAILED cleanup incomplete at exit");
                        }
                        runtime.close();
                        let mut model = worker_model.lock().unwrap();
                        model.runtime = runtime.snapshot(now);
                        model.exited = true;
                        logger.write("EXIT");
                        if demo {
                            let _ = fs::remove_dir_all(&directory);
                        }
                        break;
                    }
                } else {
                    runtime.tick(now);
                }
                let snapshot = runtime.snapshot(now);
                if last_snapshot.as_ref() != Some(&snapshot) {
                    logger.write(&format!(
                        "state={} issues={}",
                        snapshot.state,
                        snapshot
                            .issues
                            .iter()
                            .map(|e| format!(
                                "{}:{}:{}",
                                e.code,
                                e.area,
                                e.detail.replace(['\n', '\r'], " ")
                            ))
                            .collect::<Vec<_>>()
                            .join(",")
                    ));
                    last_snapshot = Some(snapshot.clone());
                }
                worker_model.lock().unwrap().runtime = snapshot;
                thread::sleep(Duration::from_millis(2));
            }
        });
        Self {
            model,
            sender,
            sleep_monitor,
        }
    }
    pub fn snapshot(&self) -> Model {
        self.model.lock().unwrap().clone()
    }
    pub fn command(&self, command: Command) -> Result<(), String> {
        let (tx, rx) = mpsc::sync_channel(1);
        self.sender
            .send(Message::Command(command, tx))
            .map_err(|_| "runtime stopped")?;
        rx.recv_timeout(Duration::from_secs(3))
            .map_err(|_| "runtime command timeout")?
    }
}
impl Drop for Service {
    fn drop(&mut self) {
        if let Some(child) = &mut self.sleep_monitor {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn monitor_sleep(sender: mpsc::Sender<Message>) -> Option<Child> {
    let mut child = ProcessCommand::new("dbus-monitor")
        .args([
            "--system",
            "type='signal',interface='org.freedesktop.login1.Manager',member='PrepareForSleep'",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let out = child.stdout.take()?;
    thread::spawn(move || {
        for line in BufReader::new(out).lines().map_while(Result::ok) {
            match line.trim() {
                "boolean true" => {
                    let _ = sender.send(Message::Sleep(true));
                }
                "boolean false" => {
                    let _ = sender.send(Message::Sleep(false));
                }
                _ => (),
            }
        }
    });
    Some(child)
}

struct Logger {
    directory: PathBuf,
}
impl Logger {
    fn new(directory: PathBuf) -> Self {
        let _ = storage::private_directory(&directory);
        Self { directory }
    }
    fn write(&mut self, message: &str) {
        let mut end = message.len().min(3072);
        while !message.is_char_boundary(end) {
            end -= 1;
        }
        let message = &message[..end];
        let path = self.directory.join("app.log");
        if fs::metadata(&path).is_ok_and(|m| m.len() > 5 * 1024 * 1024 - 4096) {
            let _ = fs::rename(&path, self.directory.join("app.1.log"));
        }
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .mode(0o600)
            .open(path)
        {
            let _ = writeln!(file, "{} {message}", storage::timestamp());
        }
    }
}

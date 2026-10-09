use clanzi_core::{
    config::{Keymap, LIMIT},
    service::{Command, Model, Service},
    storage::{atomic_write, read_limited, timestamp},
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::Duration,
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};
use tauri_plugin_dialog::DialogExt;

struct Desktop {
    service: Mutex<Service>,
    exiting: AtomicBool,
}

#[tauri::command]
fn snapshot(state: tauri::State<Desktop>) -> Model {
    state.service.lock().unwrap().snapshot()
}

#[tauri::command]
fn command(state: tauri::State<Desktop>, command: Command) -> Result<(), String> {
    state.service.lock().unwrap().command(command)
}

#[tauri::command]
async fn import_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let selected = app
            .dialog()
            .file()
            .add_filter("Olanzi keymap", &["json"])
            .blocking_pick_file();
        let Some(path) = selected else {
            return Ok(None);
        };
        let path = path.into_path().map_err(|e| e.to_string())?;
        let bytes = read_limited(&path, LIMIT).map_err(|e| e.to_string())?;
        Keymap::parse(&bytes)?;
        String::from_utf8(bytes)
            .map(Some)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn export_file(app: tauri::AppHandle) -> Result<bool, String> {
    let saved = app
        .state::<Desktop>()
        .service
        .lock()
        .unwrap()
        .snapshot()
        .saved
        .ok_or("no saved configuration")?;
    tauri::async_runtime::spawn_blocking(move || {
        let selected = app
            .dialog()
            .file()
            .add_filter("Olanzi keymap", &["json"])
            .set_file_name(format!("olanzi-keymap-{}.json", timestamp()))
            .blocking_save_file();
        let Some(path) = selected else {
            return Ok(false);
        };
        let path = path.into_path().map_err(|e| e.to_string())?;
        // 原生保存对话框负责现有文件的覆盖确认。
        atomic_write(&path, &saved.bytes()).map_err(|e| e.to_string())?;
        Ok(true)
    })
    .await
    .map_err(|e| e.to_string())?
}

fn show(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn request_quit(app: &tauri::AppHandle) {
    let state = app.state::<Desktop>();
    let _ = state.service.lock().unwrap().command(Command::RequestQuit);
    show(app);
}

fn main() {
    let demo = std::env::args().any(|a| a == "--demo");
    let mut context = tauri::generate_context!();
    if demo {
        context.config_mut().identifier.push_str(".demo");
    }
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| show(app)))
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            snapshot,
            command,
            import_file,
            export_file
        ])
        .setup(move |app| {
            // 单实例插件检查结束后才启动运行时，重复启动不能短暂争用设备。
            app.manage(Desktop {
                service: Mutex::new(Service::start(demo, None)),
                exiting: AtomicBool::new(false),
            });
            let open = MenuItem::with_id(app, "open", "Open Olanzi", true, None::<&str>)?;
            let pause = MenuItem::with_id(app, "pause", "Pause", true, None::<&str>)?;
            let status = MenuItem::with_id(app, "status", "Starting", false, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&status, &open, &pause, &quit])?;
            let mut tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("Olanzi")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => show(app),
                    "pause" => {
                        let service = app.state::<Desktop>();
                        let service = service.service.lock().unwrap();
                        let paused = service.snapshot().runtime.paused;
                        let _ = service.command(Command::Pause { paused: !paused });
                    }
                    "quit" => request_quit(app),
                    _ => (),
                });
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            // 无托盘环境仍保留进程；单实例回调可重新打开窗口。
            if let Err(e) = tray.build(app) {
                eprintln!("TRAY_UNAVAILABLE: {e}");
            }
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let mut attention_was_needed = false;
                loop {
                    std::thread::sleep(Duration::from_millis(200));
                    let state = handle.state::<Desktop>();
                    let model = state.service.lock().unwrap().snapshot();
                    if model.exited {
                        state.exiting.store(true, Ordering::SeqCst);
                        handle.exit(0);
                        break;
                    }
                    let zh = model.language == "zh";
                    let _ = open.set_text(if zh { "打开 Olanzi" } else { "Open Olanzi" });
                    let _ = quit.set_text(if zh { "退出" } else { "Quit" });
                    let _ = pause.set_text(match (zh, model.runtime.paused) {
                        (true, true) => "恢复转发",
                        (true, false) => "暂停转发",
                        (false, true) => "Resume",
                        (false, false) => "Pause",
                    });
                    let label = match (zh, model.runtime.state.as_str()) {
                        (true, "forwarding") => "转发中",
                        (true, "paused") => "已暂停",
                        (true, "configuration") => "配置不可用",
                        (true, "permission") => "权限不足",
                        (true, "fault") => "转发故障",
                        (true, "device") => "设备不可用",
                        (true, "offline") => "设备离线或未知",
                        (true, _) => "正在连接",
                        (false, "forwarding") => "Forwarding",
                        (false, "paused") => "Paused",
                        (false, "configuration") => "Configuration needed",
                        (false, "permission") => "Permission needed",
                        (false, "fault") => "Forwarding fault",
                        (false, "device") => "Device unavailable",
                        (false, "offline") => "Offline or unknown",
                        _ => "Connecting",
                    };
                    let _ = status.set_text(label);
                    let attention_needed = model.quit_prompt
                        || model
                            .error
                            .as_ref()
                            .is_some_and(|e| e.code == "CONFIG_SAVE_FAILED");
                    if attention_needed && !attention_was_needed {
                        show(&handle);
                    }
                    attention_was_needed = attention_needed;
                }
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .build(context)
        .expect("failed to start Olanzi desktop");
    app.run(|handle, event| {
        if let tauri::RunEvent::ExitRequested { api, .. } = event {
            if !handle.state::<Desktop>().exiting.load(Ordering::SeqCst) {
                api.prevent_exit();
                request_quit(handle);
            }
        }
    });
}

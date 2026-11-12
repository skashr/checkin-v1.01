use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use serde_json::Value;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, WindowEvent};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_dialog::DialogExt;

// 托盘“退出”会置 true，从而放行窗口关闭事件
static EXITING: AtomicBool = AtomicBool::new(false);

const DATA_FILE: &str = "checkin-data.json";
const CONFIG_FILE: &str = "config.json";

fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    Ok(dir.join(CONFIG_FILE))
}

fn default_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path().app_data_dir().map_err(|e| e.to_string())
}

fn read_custom_data_dir(app: &AppHandle) -> Option<PathBuf> {
    let cfg = config_path(app).ok()?;
    let s = fs::read_to_string(cfg).ok()?;
    let v: Value = serde_json::from_str(&s).ok()?;
    v.get("data_dir").and_then(|x| x.as_str()).map(PathBuf::from)
}

fn current_data_dir(app: &AppHandle) -> PathBuf {
    read_custom_data_dir(app)
        .unwrap_or_else(|| default_data_dir(app).unwrap_or_else(|_| PathBuf::from(".")))
}

fn data_file(app: &AppHandle) -> PathBuf {
    current_data_dir(app).join(DATA_FILE)
}

fn load_icon() -> tauri::image::Image<'static> {
    tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png")).expect("tray icon")
}

fn focus_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "打开", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;

    TrayIconBuilder::with_id("tray")
        .icon(load_icon())
        .tooltip("每日打卡")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, e| match e.id.as_ref() {
            "open" => focus_main(app),
            "quit" => {
                EXITING.store(true, Ordering::SeqCst);
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, e| {
            if let TrayIconEvent::Click { .. } = e {
                focus_main(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// 数据命令：数据存为 JSON 文件，路径可由用户改到网盘同步目录
// ---------------------------------------------------------------------------

#[tauri::command]
fn load_data(app: AppHandle) -> Result<Option<String>, String> {
    let p = data_file(&app);
    if !p.exists() {
        return Ok(None);
    }
    fs::read_to_string(&p).map(Some).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_data(app: AppHandle, data: String) -> Result<(), String> {
    let p = data_file(&app);
    if let Some(dir) = p.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    // 先写临时文件再重命名，尽量避免网盘同步到半截文件
    let tmp = p.with_extension("json.tmp");
    fs::write(&tmp, data.as_bytes()).map_err(|e| e.to_string())?;
    if fs::rename(&tmp, &p).is_err() {
        let _ = fs::remove_file(&p);
        fs::rename(&tmp, &p).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn get_data_dir(app: AppHandle) -> String {
    current_data_dir(&app).to_string_lossy().to_string()
}

#[tauri::command]
fn set_data_dir(app: AppHandle, dir: String) -> Result<(), String> {
    let dir = PathBuf::from(dir);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let cfg = config_path(&app)?;
    if let Some(parent) = cfg.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let v = serde_json::json!({ "data_dir": dir.to_string_lossy().to_string() });
    fs::write(&cfg, v.to_string()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn pick_data_dir(app: AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel::<Option<std::path::PathBuf>>();
    app.dialog().file().pick_folder(move |f| {
        let _ = tx.send(f.and_then(|fp| fp.into_path().ok()));
    });
    match rx.recv() {
        Ok(Some(p)) => Ok(Some(p.to_string_lossy().to_string())),
        _ => Ok(None),
    }
}

// ---------------------------------------------------------------------------
// 开机自启
// ---------------------------------------------------------------------------

#[tauri::command]
fn get_autostart(app: AppHandle) -> bool {
    app.autolaunch().is_enabled().unwrap_or(false)
}

#[tauri::command]
fn set_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    let m = app.autolaunch();
    if enabled {
        m.enable().map_err(|e| e.to_string())
    } else {
        m.disable().map_err(|e| e.to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            focus_main(app);
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            setup_tray(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_data,
            save_data,
            get_data_dir,
            set_data_dir,
            pick_data_dir,
            get_autostart,
            set_autostart
        ])
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if !EXITING.load(Ordering::SeqCst) {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

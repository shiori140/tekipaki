mod capture;
mod overlay;
mod session;
mod source_app;
mod store;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, WindowEvent};
use tauri_plugin_global_shortcut::{Shortcut, ShortcutState};

use session::Session;
use store::{Priority, Store, Task};

/// 2キーの組み合わせは他のアプリと衝突しやすいため、3キーにしている。
const REGISTER_SHORTCUT: &str = "CommandOrControl+Shift+Space";

#[tauri::command]
fn list_tasks(store: tauri::State<Store>) -> Vec<Task> {
    store.list()
}

#[tauri::command]
fn save_manual(
    app: AppHandle,
    store: tauri::State<Store>,
    text: String,
    priority: Priority,
    source_app: Option<String>,
) -> Result<(), String> {
    let task = Task::from_text(&text, priority, source_app).ok_or("内容が空です")?;
    store.add(task).map_err(|e| e.to_string())?;
    let _ = app.emit("tasks:changed", ());
    hide_input(app);
    Ok(())
}

#[tauri::command]
fn hide_input(app: AppHandle) {
    if let Some(w) = app.get_webview_window(overlay::INPUT) {
        let _ = w.hide();
    }
    // 窓を隠すだけでは tekipaki が手前に残り、次の登録で自分を発生元として拾ってしまう。
    #[cfg(target_os = "macos")]
    let _ = app.hide();
}

fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let register: Shortcut = REGISTER_SHORTCUT.parse().expect("ショートカットの書式が不正です");
    let escape: Shortcut = overlay::ESCAPE.parse().expect("ショートカットの書式が不正です");

    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcut(register)
                .expect("ショートカットを登録できませんでした")
                .with_handler(move |app, shortcut, event| {
                    if event.state != ShortcutState::Pressed {
                        return;
                    }
                    let session = app.state::<Session>();
                    if *shortcut == register {
                        session.on_press(app);
                    } else if *shortcut == escape {
                        session.cancel(app);
                    }
                })
                .build(),
        )
        .manage(Session::default())
        .setup(|app| {
            // 常駐ツールなので Dock には出さず、メニューバーにだけ置く。
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let store = Store::open(app.path().app_data_dir()?)?;
            app.manage(store);

            overlay::build_windows(app.handle())?;

            let open = MenuItem::with_id(app, "open", "一覧を開く", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "終了", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &quit])?;
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("tekipaki")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => show_main(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            // 閉じるボタンで終了せず、常駐を続ける。終了はトレイメニューから行う。
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![list_tasks, save_manual, hide_input])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

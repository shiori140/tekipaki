mod capture;
mod overlay;
mod session;
mod settings;
mod source_app;
mod store;
mod tray;

use tauri::{AppHandle, Emitter, Manager, WindowEvent};
use tauri_plugin_global_shortcut::{Shortcut, ShortcutState};

use session::Session;
use settings::{Lang, Settings};
use store::{Priority, Status, Store, Task};

/// 2キーの組み合わせは他のアプリと衝突しやすいため、3キーにしている。
const REGISTER_SHORTCUT: &str = "CommandOrControl+Shift+Space";

#[tauri::command]
fn list_tasks(store: tauri::State<Store>) -> Vec<Task> {
    store.list()
}

/// 優先度や完了状態を変え、「元に戻す」用に変更前の内容を返す。
#[tauri::command]
fn update_task(
    app: AppHandle,
    store: tauri::State<Store>,
    id: String,
    priority: Option<Priority>,
    status: Option<Status>,
) -> Result<Option<Task>, String> {
    let before = store
        .update(&id, |t| {
            if let Some(p) = priority {
                t.priority = p;
            }
            if let Some(s) = status {
                t.set_status(s);
            }
        })
        .map_err(|e| e.to_string())?;
    let _ = app.emit("tasks:changed", ());
    Ok(before)
}

#[tauri::command]
fn restore_task(app: AppHandle, store: tauri::State<Store>, task: Task) -> Result<(), String> {
    store.restore(task).map_err(|e| e.to_string())?;
    let _ = app.emit("tasks:changed", ());
    Ok(())
}

#[tauri::command]
fn save_manual(
    app: AppHandle,
    store: tauri::State<Store>,
    text: String,
    priority: Priority,
    source_app: Option<String>,
) -> Result<(), String> {
    // エラーの文言は画面側で翻訳するので、種類を表す文字列を返す。
    let task = Task::from_text(&text, priority, source_app).ok_or("empty")?;
    store.add(task).map_err(|e| e.to_string())?;
    let _ = app.emit("tasks:changed", ());
    hide_input(app);
    Ok(())
}

/// 画面側はこの言語で表示する。OS の言語ではなく、利用者が選んだ設定を優先する。
#[tauri::command]
fn get_language(settings: tauri::State<Settings>) -> Lang {
    settings.language().resolve()
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

pub(crate) fn show_main(app: &AppHandle) {
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

            let data_dir = app.path().app_data_dir()?;
            app.manage(Store::open(data_dir.clone())?);
            app.manage(Settings::open(data_dir)?);

            overlay::build_windows(app.handle())?;

            tray::create(app.handle())?;
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
        .invoke_handler(tauri::generate_handler![
            list_tasks,
            update_task,
            restore_task,
            save_manual,
            get_language,
            hide_input
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

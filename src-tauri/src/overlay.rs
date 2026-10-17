//! 判定表示（オーバーレイ）と手入力欄のウィンドウ操作。

use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use serde::Serialize;

use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

use crate::session::{InputRequest, OverlayState};

pub const OVERLAY: &str = "overlay";
pub const INPUT: &str = "input";
pub const ESCAPE: &str = "Escape";

/// フェードアウトの CSS アニメーションと揃える。
const FADE: Duration = Duration::from_millis(350);
const ERROR_VISIBLE: Duration = Duration::from_millis(2000);
/// カーソルとオーバーレイの間隔（論理ピクセル）。
const CURSOR_OFFSET: f64 = 18.0;

/// 遅れて実行される hide が、その後に出した新しい表示を消さないための世代番号。
static SHOWN: AtomicU64 = AtomicU64::new(0);

pub fn build_windows(app: &AppHandle) -> tauri::Result<()> {
    WebviewWindowBuilder::new(app, OVERLAY, WebviewUrl::App("overlay.html".into()))
        .title("tekipaki")
        .inner_size(280.0, 64.0)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .visible_on_all_workspaces(true)
        .skip_taskbar(true)
        .focused(false)
        .focusable(false)
        .visible(false)
        .build()?;

    WebviewWindowBuilder::new(app, INPUT, WebviewUrl::App("input.html".into()))
        .title("tekipaki")
        .inner_size(440.0, 132.0)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .center()
        .visible(false)
        .build()?;
    Ok(())
}

fn window(app: &AppHandle, label: &str) -> Option<WebviewWindow> {
    app.get_webview_window(label)
}

pub fn show(app: &AppHandle) {
    SHOWN.fetch_add(1, Ordering::SeqCst);
    let Some(w) = window(app, OVERLAY) else { return };
    place_near_cursor(app, &w);
    let _ = w.set_ignore_cursor_events(true);
    let _ = w.emit("overlay:show", ());
    let _ = w.show();
}

pub fn update(app: &AppHandle, state: &OverlayState) {
    let _ = app.emit_to(OVERLAY, "overlay:update", state);
}

pub fn commit(app: &AppHandle) {
    let _ = app.emit_to(OVERLAY, "overlay:commit", ());
    hide_after(app, FADE);
}

pub fn cancel(app: &AppHandle) {
    let _ = app.emit_to(OVERLAY, "overlay:cancel", ());
    hide_after(app, FADE);
}

/// 画面に出す文言は画面側の辞書（i18n.js の error.*）で翻訳するので、ここでは種類だけを送る。
#[derive(Clone, Serialize)]
pub struct OverlayError {
    pub code: &'static str,
    pub detail: Option<String>,
}

pub fn error(app: &AppHandle, code: &'static str, detail: Option<String>) {
    let _ = app.emit_to(OVERLAY, "overlay:error", OverlayError { code, detail });
    hide_after(app, ERROR_VISIBLE);
}

pub fn hide(app: &AppHandle) {
    if let Some(w) = window(app, OVERLAY) {
        let _ = w.hide();
    }
}

fn hide_after(app: &AppHandle, delay: Duration) {
    let generation = SHOWN.load(Ordering::SeqCst);
    let app = app.clone();
    thread::spawn(move || {
        thread::sleep(delay);
        if SHOWN.load(Ordering::SeqCst) == generation {
            hide(&app);
        }
    });
}

/// 判定中だけ Esc を横取りして取り消しに使う。常時登録すると他のアプリで Esc が効かなくなる。
pub fn set_escape_enabled(app: &AppHandle, enabled: bool) {
    let shortcuts = app.global_shortcut();
    let registered = shortcuts.is_registered(ESCAPE);
    let result = match (enabled, registered) {
        (true, false) => shortcuts.register(ESCAPE),
        (false, true) => shortcuts.unregister(ESCAPE),
        _ => Ok(()),
    };
    if let Err(e) = result {
        eprintln!("Esc の登録を切り替えられませんでした: {e}");
    }
}

pub fn open_input(app: &AppHandle, request: InputRequest) {
    let Some(w) = window(app, INPUT) else { return };
    let _ = w.center();
    let _ = w.emit("input:open", request);
    let _ = w.show();
    let _ = w.set_focus();
}

fn place_near_cursor(app: &AppHandle, w: &WebviewWindow) {
    let Ok(cursor) = app.cursor_position() else { return };
    let Ok(size) = w.outer_size() else { return };
    let scale = w.scale_factor().unwrap_or(1.0);
    let offset = CURSOR_OFFSET * scale;

    let mut x = cursor.x + offset;
    let mut y = cursor.y + offset;
    // 画面の右端・下端からはみ出す場合は、カーソルの反対側に出す。
    if let Ok(Some(monitor)) = app.monitor_from_point(cursor.x, cursor.y) {
        let origin = monitor.position();
        let area = monitor.size();
        let right = origin.x as f64 + area.width as f64;
        let bottom = origin.y as f64 + area.height as f64;
        if x + size.width as f64 > right {
            x = cursor.x - offset - size.width as f64;
        }
        if y + size.height as f64 > bottom {
            y = cursor.y - offset - size.height as f64;
        }
    }
    let _ = w.set_position(PhysicalPosition::new(x, y));
}

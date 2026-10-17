//! ショートカットの連打を数えて優先度を決め、タスクとして登録するまでの流れ。
//!
//! ロックを持ったままウィンドウ操作やショートカット登録を呼ぶと、
//! メインスレッドとの待ち合わせでデッドロックするため、
//! ロック中は状態の更新だけを行い、副作用はロックを外してから実行する。

use std::sync::{Condvar, Mutex};
use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::capture::{self, CaptureError};
use crate::overlay;
use crate::source_app;
use crate::store::{Priority, Store, Task};

/// 最後の押下からこの時間が経ったら優先度を確定する。
pub const JUDGE_DELAY: Duration = Duration::from_millis(400);
/// 選択文字の取得がこれ以上かかったら、取得できなかったものとして扱う。
const CAPTURE_WAIT: Duration = Duration::from_secs(3);

type CaptureResult = Result<Option<String>, CaptureError>;

struct Active {
    id: u64,
    count: u32,
    press_seq: u64,
    source_app: Option<String>,
    capture: Option<CaptureResult>,
}

#[derive(Default)]
pub struct Session {
    next_id: Mutex<u64>,
    active: Mutex<Option<Active>>,
    captured: Condvar,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayState {
    pub priority: Priority,
    pub preview: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct InputRequest {
    pub priority: Priority,
    pub source_app: Option<String>,
}

impl Session {
    pub fn on_press(&self, app: &AppHandle) {
        let mut started = None;
        let (id, seq, state) = {
            let mut active = self.active.lock().unwrap();
            if active.is_none() {
                let id = {
                    let mut next = self.next_id.lock().unwrap();
                    *next += 1;
                    *next
                };
                // オーバーレイを出す前に取らないと、自分のウィンドウを拾うおそれがある。
                let source_app = source_app::frontmost_app_name();
                *active = Some(Active {
                    id,
                    count: 0,
                    press_seq: 0,
                    source_app,
                    capture: None,
                });
                started = Some(id);
            }
            let a = active.as_mut().unwrap();
            a.count += 1;
            a.press_seq += 1;
            (a.id, a.press_seq, overlay_state(a))
        };

        if let Some(id) = started {
            let capture_app = app.clone();
            thread::spawn(move || {
                let result = capture::capture_selection();
                capture_app
                    .state::<Session>()
                    .on_captured(&capture_app, id, result);
            });
            overlay::show(app);
            overlay::set_escape_enabled(app, true);
        }
        overlay::update(app, &state);

        let app = app.clone();
        thread::spawn(move || {
            thread::sleep(JUDGE_DELAY);
            app.state::<Session>().finalize(&app, id, seq);
        });
    }

    pub fn cancel(&self, app: &AppHandle) {
        let cancelled = self.active.lock().unwrap().take().is_some();
        self.captured.notify_all();
        if cancelled {
            overlay::set_escape_enabled(app, false);
            overlay::cancel(app);
        }
    }

    fn on_captured(&self, app: &AppHandle, id: u64, result: CaptureResult) {
        let state = {
            let mut active = self.active.lock().unwrap();
            let Some(a) = active.as_mut().filter(|a| a.id == id) else {
                return;
            };
            a.capture = Some(result);
            overlay_state(a)
        };
        self.captured.notify_all();
        overlay::update(app, &state);
    }

    fn finalize(&self, app: &AppHandle, id: u64, seq: u64) {
        let done = {
            let is_latest =
                |a: &Option<Active>| matches!(a, Some(a) if a.id == id && a.press_seq == seq);
            let active = self.active.lock().unwrap();
            if !is_latest(&active) {
                return;
            }
            // 修飾キーを押したまま連打していると、取得が確定より後になる。
            let (mut active, _) = self
                .captured
                .wait_timeout_while(active, CAPTURE_WAIT, |a| {
                    is_latest(a) && a.as_ref().unwrap().capture.is_none()
                })
                .unwrap();
            // 待っている間に押し直されたり取り消されたりしたら、後の判定に任せる。
            if !is_latest(&active) {
                return;
            }
            active.take().unwrap()
        };
        overlay::set_escape_enabled(app, false);

        let priority = Priority::from_count(done.count);
        match done.capture {
            Some(Ok(Some(text))) => {
                let Some(task) = Task::from_text(&text, priority, done.source_app) else {
                    overlay::cancel(app);
                    return;
                };
                match app.state::<Store>().add(task) {
                    Ok(()) => {
                        overlay::commit(app);
                        let _ = app.emit("tasks:changed", ());
                    }
                    Err(e) => overlay::error(app, &format!("保存に失敗しました: {e}")),
                }
            }
            Some(Err(CaptureError::NoPermission)) => {
                overlay::error(app, "アクセシビリティの許可が必要です");
            }
            other => {
                if let Some(Err(CaptureError::Other(e))) = other {
                    eprintln!("選択文字の取得に失敗しました: {e}");
                }
                overlay::hide(app);
                overlay::open_input(
                    app,
                    InputRequest {
                        priority,
                        source_app: done.source_app,
                    },
                );
            }
        }
    }
}

fn overlay_state(a: &Active) -> OverlayState {
    OverlayState {
        priority: Priority::from_count(a.count),
        preview: match &a.capture {
            Some(Ok(Some(text))) => text.trim().lines().next().map(|l| l.chars().take(40).collect()),
            _ => None,
        },
    }
}

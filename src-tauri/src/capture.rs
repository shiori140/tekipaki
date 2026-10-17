use std::thread::sleep;
use std::time::{Duration, Instant};

use arboard::{Clipboard, ImageData};
use enigo::{Direction, Enigo, Key, Keyboard, Settings};

/// コピー操作を送ってから、クリップボードが更新されるのを待つ最大時間。
const COPY_TIMEOUT: Duration = Duration::from_millis(400);
/// ショートカットの修飾キーが離されるのを待つ最大時間。
/// 押しっぱなしで連打されても、この時間を過ぎたらコピーを送る。
const RELEASE_TIMEOUT: Duration = Duration::from_millis(1500);
const POLL: Duration = Duration::from_millis(15);

#[derive(Debug)]
pub enum CaptureError {
    /// macOS でアクセシビリティ権限がなく、キー操作を送れない。
    NoPermission,
    Other(String),
}

/// 手前のアプリで選択されている文字を取得する。
/// 何も選択されていなければ Ok(None)。元のクリップボードの中身は復元する。
pub fn capture_selection() -> Result<Option<String>, CaptureError> {
    wait_for_modifiers_released();

    let mut clipboard = Clipboard::new().map_err(|e| CaptureError::Other(e.to_string()))?;
    let saved = Saved::take(&mut clipboard);
    let before = clipboard_seq();

    send_copy()?;

    let deadline = Instant::now() + COPY_TIMEOUT;
    let mut copied = None;
    while Instant::now() < deadline {
        if clipboard_seq() != before {
            // 更新直後は中身がまだ書き込まれていないことがあるので、少しだけ待つ。
            sleep(POLL);
            copied = clipboard.get_text().ok();
            break;
        }
        sleep(POLL);
    }

    // クリップボードが変わっていないなら上書きもしていないので、復元は不要。
    if clipboard_seq() != before {
        saved.restore(&mut clipboard);
    }

    Ok(copied.filter(|s| !s.trim().is_empty()))
}

/// 復元用に退避したクリップボードの中身。文字と画像のみ対応し、
/// それ以外の形式（ファイルやリッチテキストの書式など）は失われる。
enum Saved {
    Text(String),
    Image(ImageData<'static>),
    Empty,
}

impl Saved {
    fn take(clipboard: &mut Clipboard) -> Self {
        if let Ok(text) = clipboard.get_text() {
            return Saved::Text(text);
        }
        if let Ok(image) = clipboard.get_image() {
            return Saved::Image(image.to_owned_img());
        }
        Saved::Empty
    }

    fn restore(self, clipboard: &mut Clipboard) {
        let result = match self {
            Saved::Text(text) => clipboard.set_text(text),
            Saved::Image(image) => clipboard.set_image(image),
            Saved::Empty => clipboard.clear(),
        };
        if let Err(e) = result {
            eprintln!("クリップボードの復元に失敗しました: {e}");
        }
    }
}

fn send_copy() -> Result<(), CaptureError> {
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| match e {
        enigo::NewConError::NoPermission => CaptureError::NoPermission,
        other => CaptureError::Other(other.to_string()),
    })?;

    // Unicode('c') はキー配列から C キーを探すが、日本語入力がオンだと配列が取れず
    // A キーを押してしまう。さらにメインスレッド以外では強制終了するため、物理キー番号で送る。
    // kVK_ANSI_C = 8。Dvorak など C の位置が違う配列では別のキーになる。
    #[cfg(target_os = "macos")]
    let (modifier, c) = (Key::Meta, Key::Other(8));
    // Windows では Unicode 指定だと文字入力扱いになり Ctrl+C として届かないため、仮想キーで送る。
    #[cfg(windows)]
    let (modifier, c) = (Key::Control, Key::C);
    #[cfg(not(any(target_os = "macos", windows)))]
    let (modifier, c) = (Key::Control, Key::Unicode('c'));

    let result = enigo
        .key(modifier, Direction::Press)
        .and_then(|_| enigo.key(c, Direction::Click));
    // 途中で失敗しても修飾キーが押しっぱなしにならないよう、必ず離す。
    let release = enigo.key(modifier, Direction::Release);
    result
        .and(release)
        .map_err(|e| CaptureError::Other(e.to_string()))
}

fn wait_for_modifiers_released() {
    let deadline = Instant::now() + RELEASE_TIMEOUT;
    while modifiers_held() && Instant::now() < deadline {
        sleep(POLL);
    }
}

#[cfg(target_os = "macos")]
fn modifiers_held() -> bool {
    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGEventSourceFlagsState(state_id: i32) -> u64;
    }
    const HID_SYSTEM_STATE: i32 = 1;
    const SHIFT: u64 = 0x0002_0000;
    const CONTROL: u64 = 0x0004_0000;
    const OPTION: u64 = 0x0008_0000;
    const COMMAND: u64 = 0x0010_0000;
    let flags = unsafe { CGEventSourceFlagsState(HID_SYSTEM_STATE) };
    flags & (SHIFT | CONTROL | OPTION | COMMAND) != 0
}

#[cfg(windows)]
fn modifiers_held() -> bool {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
    };
    [VK_SHIFT, VK_CONTROL, VK_MENU, VK_LWIN, VK_RWIN]
        .iter()
        .any(|&vk| unsafe { GetAsyncKeyState(vk as i32) } as u16 & 0x8000 != 0)
}

#[cfg(not(any(target_os = "macos", windows)))]
fn modifiers_held() -> bool {
    false
}

/// クリップボードが更新されるたびに変わる番号。
#[cfg(target_os = "macos")]
fn clipboard_seq() -> i64 {
    use objc2_app_kit::NSPasteboard;
    #[allow(unused_unsafe)]
    unsafe {
        NSPasteboard::generalPasteboard().changeCount() as i64
    }
}

#[cfg(windows)]
fn clipboard_seq() -> i64 {
    unsafe { windows_sys::Win32::System::DataExchange::GetClipboardSequenceNumber() as i64 }
}

#[cfg(not(any(target_os = "macos", windows)))]
fn clipboard_seq() -> i64 {
    0
}

/// いま手前にあるアプリの表示名を返す。取れなかった場合は None。
#[cfg(target_os = "macos")]
pub fn frontmost_app_name() -> Option<String> {
    use objc2_app_kit::NSWorkspace;

    #[allow(unused_unsafe)]
    unsafe {
        let workspace = NSWorkspace::sharedWorkspace();
        let app = workspace.frontmostApplication()?;
        let name = app.localizedName()?;
        Some(name.to_string())
    }
}

#[cfg(windows)]
pub fn frontmost_app_name() -> Option<String> {
    use std::os::windows::ffi::OsStringExt;
    use std::path::Path;
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
        QueryFullProcessImageNameW,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return None;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 {
            return None;
        }
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if process.is_null() {
            return None;
        }
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, buf.as_mut_ptr(), &mut len);
        CloseHandle(process);
        if ok == 0 {
            return None;
        }
        let path = std::ffi::OsString::from_wide(&buf[..len as usize]);
        Path::new(&path)
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
    }
}

#[cfg(not(any(target_os = "macos", windows)))]
pub fn frontmost_app_name() -> Option<String> {
    None
}

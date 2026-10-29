//! Window enumeration. One EnumWindows pass, no per-window UIA or process queries.

use windows::core::BOOL;
use windows::Win32::Foundation::{HWND, LPARAM, RECT};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowRect, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, IsIconic,
    IsWindowVisible,
};

use crate::platform::{Rect, WindowInfo, WindowQuery};

unsafe extern "system" fn cb(h: HWND, l: LPARAM) -> BOOL {
    let v = &mut *(l.0 as *mut Vec<WindowInfo>);
    if !IsWindowVisible(h).as_bool() {
        return BOOL(1);
    }
    let mut cloaked = 0u32;
    let _ = DwmGetWindowAttribute(h, DWMWA_CLOAKED, &mut cloaked as *mut _ as *mut _, 4);
    if cloaked != 0 {
        return BOOL(1);
    }
    // Skip our own windows (the agent cursor): querying a window owned by another thread of this
    // process sends it a message, and that thread may be asleep, stalling the whole enumeration.
    let mut pid = 0u32;
    GetWindowThreadProcessId(h, Some(&mut pid));
    if pid == std::process::id() {
        return BOOL(1);
    }
    let len = GetWindowTextLengthW(h);
    if len == 0 {
        return BOOL(1);
    }
    let mut buf = vec![0u16; len as usize + 1];
    let n = GetWindowTextW(h, &mut buf) as usize;
    let mut r = RECT::default();
    let _ = GetWindowRect(h, &mut r);
    v.push(WindowInfo {
        id: h.0 as isize as i64,
        pid,
        title: String::from_utf16_lossy(&buf[..n]),
        rect: Rect { left: r.left, top: r.top, right: r.right, bottom: r.bottom },
        minimized: IsIconic(h).as_bool(),
    });
    BOOL(1)
}

pub fn list() -> Vec<WindowInfo> {
    let mut v: Vec<WindowInfo> = Vec::with_capacity(32);
    unsafe {
        let _ = EnumWindows(Some(cb), LPARAM(&mut v as *mut _ as isize));
    }
    v
}

pub fn exe_name(pid: u32) -> String {
    unsafe {
        let Ok(h) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
            return String::new();
        };
        let mut buf = [0u16; 512];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, windows::core::PWSTR(buf.as_mut_ptr()), &mut len);
        let _ = windows::Win32::Foundation::CloseHandle(h);
        if ok.is_err() {
            return String::new();
        }
        let s = String::from_utf16_lossy(&buf[..len as usize]);
        s.rsplit('\\').next().unwrap_or("").to_string()
    }
}

pub fn find(q: &WindowQuery) -> Option<WindowInfo> {
    let all = list();
    if let Some(t) = &q.title {
        let t = t.to_lowercase();
        let mut m: Vec<WindowInfo> = all.into_iter().filter(|w| w.title.to_lowercase().contains(&t)).collect();
        m.sort_by_key(|w| w.minimized);
        return m.into_iter().next();
    }
    if let Some(id) = q.id {
        return all.into_iter().find(|w| w.id == id);
    }
    let pid = q.pid?;
    all.into_iter().filter(|w| w.pid == pid).max_by_key(|w| w.rect.w() as i64 * w.rect.h() as i64)
}

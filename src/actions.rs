//! Input actions. Background-first: UIA patterns and posted window messages never touch the
//! user's cursor or focus. SendInput is used only when explicitly requested (foreground) or for
//! hotkeys with modifiers, which posted messages cannot express reliably.

use std::mem::size_of;

use windows::core::BSTR;
use windows::Win32::Foundation::{HWND, LPARAM, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::ScreenToClient;
use windows::Win32::UI::Accessibility::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

fn h(hwnd: isize) -> HWND {
    HWND(hwnd as *mut _)
}

/// Bring `top` to the foreground and confirm it. Foreground input must never land in another window.
pub fn raise(top: isize) -> bool {
    unsafe {
        for _ in 0..20 {
            if GetForegroundWindow() == h(top) {
                return true;
            }
            let _ = SetForegroundWindow(h(top));
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        GetForegroundWindow() == h(top)
    }
}

/// Run the first UIA pattern the element supports that "does the click".
pub fn invoke(el: &IUIAutomationElement) -> Result<&'static str, String> {
    unsafe {
        if let Ok(p) = el.GetCurrentPatternAs::<IUIAutomationInvokePattern>(UIA_InvokePatternId) {
            p.Invoke().map_err(|e| e.to_string())?;
            return Ok("InvokePattern");
        }
        if let Ok(p) = el.GetCurrentPatternAs::<IUIAutomationTogglePattern>(UIA_TogglePatternId) {
            p.Toggle().map_err(|e| e.to_string())?;
            return Ok("TogglePattern");
        }
        if let Ok(p) = el.GetCurrentPatternAs::<IUIAutomationSelectionItemPattern>(UIA_SelectionItemPatternId) {
            p.Select().map_err(|e| e.to_string())?;
            return Ok("SelectionItemPattern");
        }
        if let Ok(p) = el.GetCurrentPatternAs::<IUIAutomationExpandCollapsePattern>(UIA_ExpandCollapsePatternId) {
            p.Expand().map_err(|e| e.to_string())?;
            return Ok("ExpandCollapsePattern");
        }
    }
    Err("element supports no click-like UIA pattern; retry with x,y".into())
}

pub fn set_value(el: &IUIAutomationElement, v: &str) -> Result<(), String> {
    unsafe {
        let p = el
            .GetCurrentPatternAs::<IUIAutomationValuePattern>(UIA_ValuePatternId)
            .map_err(|_| "element has no ValuePattern".to_string())?;
        p.SetValue(&BSTR::from(v)).map_err(|e| e.to_string())
    }
}

/// Window the keyboard goes to inside `top`: its focused control if any, else `top`.
pub fn focus_target(top: isize) -> HWND {
    unsafe {
        let tid = GetWindowThreadProcessId(h(top), None);
        let mut gi = GUITHREADINFO { cbSize: size_of::<GUITHREADINFO>() as u32, ..Default::default() };
        if GetGUIThreadInfo(tid, &mut gi).is_ok() && !gi.hwndFocus.0.is_null() {
            return gi.hwndFocus;
        }
        h(top)
    }
}

pub fn element_hwnd(el: &IUIAutomationElement) -> Option<HWND> {
    unsafe { el.CurrentNativeWindowHandle().ok().filter(|w| !w.0.is_null()) }
}

/// Post WM_CHAR for every UTF-16 unit. No sleeps: this is the whole point.
pub fn post_text(target: HWND, s: &str) -> usize {
    let mut n = 0;
    for u in s.replace("\r\n", "\r").replace('\n', "\r").encode_utf16() {
        unsafe {
            let _ = PostMessageW(Some(target), WM_CHAR, WPARAM(u as usize), LPARAM(1));
        }
        n += 1;
    }
    n
}

fn key_input(vk: u16, scan: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT { wVk: VIRTUAL_KEY(vk), wScan: scan, dwFlags: flags, time: 0, dwExtraInfo: 0 },
        },
    }
}

fn send(inputs: &[INPUT]) {
    unsafe {
        SendInput(inputs, size_of::<INPUT>() as i32);
    }
}

pub fn send_text_foreground(s: &str) -> usize {
    let mut v = Vec::with_capacity(s.len() * 2);
    for u in s.replace("\r\n", "\n").encode_utf16() {
        if u == b'\n' as u16 {
            v.push(key_input(VK_RETURN.0, 0, KEYBD_EVENT_FLAGS(0)));
            v.push(key_input(VK_RETURN.0, 0, KEYEVENTF_KEYUP));
        } else {
            v.push(key_input(0, u, KEYEVENTF_UNICODE));
            v.push(key_input(0, u, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP));
        }
    }
    let n = v.len() / 2;
    send(&v);
    n
}

pub fn vk(name: &str) -> Option<u16> {
    let n = name.to_ascii_lowercase();
    Some(match n.as_str() {
        "ctrl" | "control" => VK_CONTROL.0,
        "shift" => VK_SHIFT.0,
        "alt" => VK_MENU.0,
        "win" | "meta" | "cmd" | "super" => VK_LWIN.0,
        "enter" | "return" => VK_RETURN.0,
        "tab" => VK_TAB.0,
        "esc" | "escape" => VK_ESCAPE.0,
        "space" => VK_SPACE.0,
        "backspace" => VK_BACK.0,
        "delete" | "del" => VK_DELETE.0,
        "insert" => VK_INSERT.0,
        "home" => VK_HOME.0,
        "end" => VK_END.0,
        "pageup" | "pgup" => VK_PRIOR.0,
        "pagedown" | "pgdn" => VK_NEXT.0,
        "up" => VK_UP.0,
        "down" => VK_DOWN.0,
        "left" => VK_LEFT.0,
        "right" => VK_RIGHT.0,
        s if s.len() == 1 => {
            let c = s.as_bytes()[0];
            match c {
                b'a'..=b'z' => (c - 32) as u16,
                b'0'..=b'9' => c as u16,
                _ => return None,
            }
        }
        s if s.starts_with('f') && s[1..].parse::<u16>().map(|k| (1..=24).contains(&k)).unwrap_or(false) => {
            VK_F1.0 + s[1..].parse::<u16>().unwrap() - 1
        }
        _ => return None,
    })
}

fn is_ext(vk: u16) -> bool {
    matches!(
        VIRTUAL_KEY(vk),
        VK_INSERT | VK_DELETE | VK_HOME | VK_END | VK_PRIOR | VK_NEXT | VK_UP | VK_DOWN | VK_LEFT | VK_RIGHT
    )
}

/// Single key to a window without touching focus. Modifiers are NOT supported here.
pub fn post_key(target: HWND, vkey: u16) {
    unsafe {
        let scan = MapVirtualKeyW(vkey as u32, MAPVK_VK_TO_VSC) as isize;
        let ext = if is_ext(vkey) { 1isize << 24 } else { 0 };
        let down = 1 | (scan << 16) | ext;
        let up = down | (1isize << 30) | (1isize << 31);
        let _ = PostMessageW(Some(target), WM_KEYDOWN, WPARAM(vkey as usize), LPARAM(down));
        let _ = PostMessageW(Some(target), WM_KEYUP, WPARAM(vkey as usize), LPARAM(up));
    }
}

/// Chord like "ctrl+shift+t" via SendInput. Raises `top` first; this is a visible focus change.
pub fn hotkey(top: isize, keys: &str) -> Result<(), String> {
    let parts: Vec<&str> = keys.split('+').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    let vks: Option<Vec<u16>> = parts.iter().map(|p| vk(p)).collect();
    let vks = vks.ok_or_else(|| format!("unknown key in '{keys}'"))?;
    if vks.is_empty() {
        return Err("empty hotkey".into());
    }
    if !raise(top) {
        return Err("could not bring the window to the foreground; keys not sent".into());
    }
    let mut v = Vec::new();
    for &k in &vks {
        let f = if is_ext(k) { KEYEVENTF_EXTENDEDKEY } else { KEYBD_EVENT_FLAGS(0) };
        v.push(key_input(k, 0, f));
    }
    for &k in vks.iter().rev() {
        let f = if is_ext(k) { KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP } else { KEYEVENTF_KEYUP };
        v.push(key_input(k, 0, f));
    }
    send(&v);
    Ok(())
}

/// Click at window-local pixel (x, y). Background = posted mouse messages to the deepest child.
pub fn click_xy(top: isize, left: i32, topy: i32, x: i32, y: i32, double: bool, right: bool, foreground: bool) -> Result<String, String> {
    let (sx, sy) = (left + x, topy + y);
    unsafe {
        if foreground {
            if !raise(top) {
                return Err("could not bring the window to the foreground; click not sent".into());
            }
            let mut old = POINT::default();
            let _ = GetCursorPos(&mut old);
            let _ = SetCursorPos(sx, sy);
            let (d, u) = if right {
                (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP)
            } else {
                (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP)
            };
            let mk = |f| INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 { mi: MOUSEINPUT { dx: 0, dy: 0, mouseData: 0, dwFlags: f, time: 0, dwExtraInfo: 0 } },
            };
            let mut v = vec![mk(d), mk(u)];
            if double {
                v.push(mk(d));
                v.push(mk(u));
            }
            send(&v);
            let _ = SetCursorPos(old.x, old.y);
            return Ok("SendInput (foreground)".into());
        }
        // descend to the deepest child under the point
        let mut target = h(top);
        loop {
            let mut pt = POINT { x: sx, y: sy };
            let _ = ScreenToClient(target, &mut pt);
            let c = ChildWindowFromPointEx(target, pt, CWP_SKIPINVISIBLE | CWP_SKIPDISABLED | CWP_SKIPTRANSPARENT);
            if c.0.is_null() || c == target {
                break;
            }
            target = c;
        }
        let mut pt = POINT { x: sx, y: sy };
        let _ = ScreenToClient(target, &mut pt);
        let lp = LPARAM((((pt.y as isize) & 0xffff) << 16) | ((pt.x as isize) & 0xffff));
        let (dn, up, mk) = if right { (WM_RBUTTONDOWN, WM_RBUTTONUP, 2usize) } else { (WM_LBUTTONDOWN, WM_LBUTTONUP, 1usize) };
        let _ = PostMessageW(Some(target), WM_MOUSEMOVE, WPARAM(0), lp);
        let _ = PostMessageW(Some(target), dn, WPARAM(mk), lp);
        let _ = PostMessageW(Some(target), up, WPARAM(0), lp);
        if double {
            let _ = PostMessageW(Some(target), WM_LBUTTONDBLCLK, WPARAM(mk), lp);
            let _ = PostMessageW(Some(target), up, WPARAM(0), lp);
        }
        Ok("PostMessage (background)".into())
    }
}

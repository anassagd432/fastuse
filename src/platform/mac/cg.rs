//! Window list through CoreGraphics (`CGWindowListCopyWindowInfo`). Titles of other apps' windows
//! are only returned when Screen Recording permission is granted; without it we fall back to the
//! owning app's name, which is still enough to find a window.

use std::ffi::c_void;
use std::ptr;
use std::time::Instant;

use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use core_foundation_sys::array::{CFArrayGetCount, CFArrayGetValueAtIndex, CFArrayRef};
use core_foundation_sys::base::{CFGetTypeID, CFRelease};
use core_foundation_sys::dictionary::{CFDictionaryGetValue, CFDictionaryRef};
use core_foundation_sys::number::{kCFNumberFloat64Type, CFNumberGetTypeID, CFNumberGetValue, CFNumberRef};
use core_foundation_sys::string::{CFStringGetTypeID, CFStringRef};

use crate::imaging::Frame;
use crate::platform::{CaptureTarget, ClickKind, Rect, WindowInfo};

#[repr(C)]
#[derive(Clone, Copy)]
struct CGPoint {
    x: f64,
    y: f64,
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGWindowListCopyWindowInfo(option: u32, relative_to: u32) -> CFArrayRef;
    pub fn CGPreflightScreenCaptureAccess() -> bool;
    fn CGEventCreateKeyboardEvent(source: *const c_void, virtual_key: u16, key_down: bool) -> *mut c_void;
    fn CGEventKeyboardSetUnicodeString(event: *mut c_void, string_length: usize, unicode_string: *const u16);
    fn CGEventSetFlags(event: *mut c_void, flags: u64);
    fn CGEventPostToPid(pid: i32, event: *mut c_void);
    fn CGEventPost(tap: u32, event: *mut c_void);
    fn CGEventCreateMouseEvent(source: *const c_void, mouse_type: u32, mouse_cursor_position: CGPoint, mouse_button: u32) -> *mut c_void;
    fn CGEventSetIntegerValueField(event: *mut c_void, field: u32, value: i64);
    fn CGEventCreateScrollWheelEvent2(source: *const c_void, units: u32, wheel_count: u32, wheel1: i32, wheel2: i32, wheel3: i32) -> *mut c_void;
}

const ON_SCREEN_ONLY: u32 = 1;
const EXCLUDE_DESKTOP: u32 = 16;

pub struct Win {
    pub info: WindowInfo,
    pub owner: String,
}

unsafe fn get(d: CFDictionaryRef, key: &str) -> *const c_void {
    let k = CFString::new(key);
    CFDictionaryGetValue(d, k.as_concrete_TypeRef() as *const c_void)
}

unsafe fn num(v: *const c_void) -> Option<f64> {
    if v.is_null() || CFGetTypeID(v) != CFNumberGetTypeID() {
        return None;
    }
    let mut out = 0f64;
    if CFNumberGetValue(v as CFNumberRef, kCFNumberFloat64Type, &mut out as *mut f64 as *mut c_void) {
        Some(out)
    } else {
        None
    }
}

unsafe fn string(v: *const c_void) -> Option<String> {
    if v.is_null() || CFGetTypeID(v) != CFStringGetTypeID() {
        return None;
    }
    Some(CFString::wrap_under_get_rule(v as CFStringRef).to_string())
}

/// On-screen, normal-layer windows, front to back. Our own windows are skipped.
pub fn list(self_pid: u32) -> Vec<Win> {
    let mut out = Vec::new();
    unsafe {
        let arr = CGWindowListCopyWindowInfo(ON_SCREEN_ONLY | EXCLUDE_DESKTOP, 0);
        if arr.is_null() {
            return out;
        }
        for i in 0..CFArrayGetCount(arr) {
            let d = CFArrayGetValueAtIndex(arr, i) as CFDictionaryRef;
            if d.is_null() {
                continue;
            }
            if num(get(d, "kCGWindowLayer")).map_or(true, |l| l != 0.0) {
                continue;
            }
            let (Some(id), Some(pid)) = (num(get(d, "kCGWindowNumber")), num(get(d, "kCGWindowOwnerPID"))) else { continue };
            if pid as u32 == self_pid {
                continue;
            }
            let b = get(d, "kCGWindowBounds") as CFDictionaryRef;
            if b.is_null() {
                continue;
            }
            let (x, y) = (num(get(b, "X")).unwrap_or(0.0), num(get(b, "Y")).unwrap_or(0.0));
            let (w, h) = (num(get(b, "Width")).unwrap_or(0.0), num(get(b, "Height")).unwrap_or(0.0));
            if w < 1.0 || h < 1.0 {
                continue;
            }
            let owner = string(get(d, "kCGWindowOwnerName")).unwrap_or_default();
            let name = string(get(d, "kCGWindowName")).unwrap_or_default();
            let title = match (owner.is_empty(), name.is_empty()) {
                (false, false) => format!("{owner} — {name}"),
                (false, true) => owner.clone(),
                (true, _) => name,
            };
            out.push(Win {
                info: WindowInfo {
                    id: id as i64,
                    pid: pid as u32,
                    title,
                    rect: Rect { left: x as i32, top: y as i32, right: (x + w) as i32, bottom: (y + h) as i32 },
                    minimized: false,
                },
                owner,
            });
        }
        CFRelease(arr as *const c_void);
    }
    let _ = ptr::null::<u8>();
    out
}

// ---- CoreGraphics input (M2) and screenshot capture (M3) -------------------------------------

const K_CG_HID_EVENT_TAP: u32 = 0;
const K_CG_EVENT_LEFT_MOUSE_DOWN: u32 = 1;
const K_CG_EVENT_LEFT_MOUSE_UP: u32 = 2;
const K_CG_EVENT_RIGHT_MOUSE_DOWN: u32 = 3;
const K_CG_EVENT_RIGHT_MOUSE_UP: u32 = 4;
const K_CG_EVENT_MOUSE_MOVED: u32 = 5;
const K_CG_MOUSE_BUTTON_LEFT: u32 = 0;
const K_CG_MOUSE_BUTTON_RIGHT: u32 = 1;
const K_CG_MOUSE_EVENT_CLICK_STATE: u32 = 1;
const K_CG_SCROLL_EVENT_UNIT_LINE: u32 = 1;

const FLAG_SHIFT: u64 = 1 << 17;
const FLAG_CTRL: u64 = 1 << 18;
const FLAG_ALT: u64 = 1 << 19;
const FLAG_CMD: u64 = 1 << 20;

pub fn vk(name: &str) -> Option<u16> {
    let n = name.trim().to_ascii_lowercase();
    Some(match n.as_str() {
        "return" | "enter" => 36,
        "tab" => 48,
        "space" => 49,
        "backspace" => 51,
        "esc" | "escape" => 53,
        "cmd" | "command" | "meta" | "super" | "win" => 55,
        "shift" => 56,
        "alt" | "option" | "opt" => 58,
        "ctrl" | "control" => 59,
        "delete" | "del" => 117,
        "home" => 115,
        "end" => 119,
        "pageup" | "pgup" => 116,
        "pagedown" | "pgdn" => 121,
        "left" | "arrowleft" => 123,
        "right" | "arrowright" => 124,
        "down" | "arrowdown" => 125,
        "up" | "arrowup" => 126,
        "f1" => 122,
        "f2" => 120,
        "f3" => 99,
        "f4" => 118,
        "f5" => 96,
        "f6" => 97,
        "f7" => 98,
        "f8" => 100,
        "f9" => 101,
        "f10" => 109,
        "f11" => 103,
        "f12" => 111,
        "a" => 0,
        "s" => 1,
        "d" => 2,
        "f" => 3,
        "h" => 4,
        "g" => 5,
        "z" => 6,
        "x" => 7,
        "c" => 8,
        "v" => 9,
        "b" => 11,
        "q" => 12,
        "w" => 13,
        "e" => 14,
        "r" => 15,
        "y" => 16,
        "t" => 17,
        "1" => 18,
        "2" => 19,
        "3" => 20,
        "4" => 21,
        "6" => 22,
        "5" => 23,
        "=" => 24,
        "9" => 25,
        "7" => 26,
        "-" => 27,
        "8" => 28,
        "0" => 29,
        "]" => 30,
        "o" => 31,
        "u" => 32,
        "[" => 33,
        "i" => 34,
        "p" => 35,
        "l" => 37,
        "j" => 38,
        "'" => 39,
        "k" => 40,
        ";" => 41,
        "\\" => 42,
        "," => 43,
        "/" => 44,
        "n" => 45,
        "m" => 46,
        "." => 47,
        _ => return None,
    })
}

unsafe fn post(pid: i32, foreground: bool, ev: *mut c_void) {
    if ev.is_null() {
        return;
    }
    if foreground {
        CGEventPost(K_CG_HID_EVENT_TAP, ev);
    } else {
        CGEventPostToPid(pid, ev);
    }
    CFRelease(ev);
}

pub fn post_key(pid: i32, keycode: u16, flags: u64, foreground: bool) {
    unsafe {
        for down in [true, false] {
            let ev = CGEventCreateKeyboardEvent(ptr::null(), keycode, down);
            if !ev.is_null() && flags != 0 {
                CGEventSetFlags(ev, flags);
            }
            post(pid, foreground, ev);
        }
    }
}

pub fn type_text(pid: i32, text: &str, foreground: bool) -> usize {
    let mut n = 0;
    for ch in text.replace("\r\n", "\n").chars() {
        if ch == '\n' || ch == '\r' {
            post_key(pid, 36, 0, foreground);
            n += 1;
            continue;
        }
        let mut buf = [0u16; 2];
        let units = ch.encode_utf16(&mut buf);
        unsafe {
            for down in [true, false] {
                let ev = CGEventCreateKeyboardEvent(ptr::null(), 0, down);
                if !ev.is_null() {
                    CGEventKeyboardSetUnicodeString(ev, units.len(), units.as_ptr());
                }
                post(pid, foreground, ev);
            }
        }
        n += 1;
    }
    n
}

pub fn hotkey(pid: i32, keys: &str, foreground: bool) -> Result<(), String> {
    let parts: Vec<&str> = keys.split('+').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        return Err("empty hotkey".into());
    }
    let mut flags = 0u64;
    let mut main_vk = None;
    for p in &parts {
        match p.to_ascii_lowercase().as_str() {
            "cmd" | "command" | "meta" | "super" | "win" => flags |= FLAG_CMD,
            "shift" => flags |= FLAG_SHIFT,
            "alt" | "option" | "opt" => flags |= FLAG_ALT,
            "ctrl" | "control" => flags |= FLAG_CTRL,
            other => {
                main_vk = Some(vk(other).ok_or_else(|| format!("unknown key '{other}' in '{keys}'"))?);
            }
        }
    }
    let k = main_vk.ok_or_else(|| format!("hotkey '{keys}' needs a non-modifier key"))?;
    post_key(pid, k, flags, foreground);
    Ok(())
}

pub fn click_point(pid: i32, sx: i32, sy: i32, kind: ClickKind, foreground: bool) -> Result<String, String> {
    let pt = CGPoint { x: sx as f64, y: sy as f64 };
    let (dn, up, btn) = match kind {
        ClickKind::Right => (K_CG_EVENT_RIGHT_MOUSE_DOWN, K_CG_EVENT_RIGHT_MOUSE_UP, K_CG_MOUSE_BUTTON_RIGHT),
        _ => (K_CG_EVENT_LEFT_MOUSE_DOWN, K_CG_EVENT_LEFT_MOUSE_UP, K_CG_MOUSE_BUTTON_LEFT),
    };
    let clicks = if kind == ClickKind::Double { 2 } else { 1 };
    unsafe {
        let mv = CGEventCreateMouseEvent(ptr::null(), K_CG_EVENT_MOUSE_MOVED, pt, K_CG_MOUSE_BUTTON_LEFT);
        post(pid, foreground, mv);
        for c in 1..=clicks {
            for ty in [dn, up] {
                let ev = CGEventCreateMouseEvent(ptr::null(), ty, pt, btn);
                if !ev.is_null() {
                    CGEventSetIntegerValueField(ev, K_CG_MOUSE_EVENT_CLICK_STATE, c);
                }
                post(pid, foreground, ev);
            }
        }
    }
    Ok(if foreground { "CGEventPost (foreground)".into() } else { "CGEventPostToPid (background)".into() })
}

pub fn scroll(pid: i32, direction: &str, amount: i32) -> Result<String, String> {
    let ticks = amount.max(1);
    let (dy, dx) = match direction.to_ascii_lowercase().as_str() {
        "up" => (ticks, 0),
        "down" => (-ticks, 0),
        "left" => (0, ticks),
        "right" => (0, -ticks),
        other => return Err(format!("unknown scroll direction '{other}' (use up, down, left, right)")),
    };
    unsafe {
        let ev = CGEventCreateScrollWheelEvent2(ptr::null(), K_CG_SCROLL_EVENT_UNIT_LINE, 2, dy, dx, 0);
        post(pid, false, ev);
    }
    Ok(format!("scroll {direction} {ticks} via CGEvent (background)"))
}

pub fn grab(t: CaptureTarget, screen_mode: bool) -> Result<Frame, String> {
    if !unsafe { CGPreflightScreenCaptureAccess() } {
        return Err("Screen Recording permission is not granted. Run `fastuse doctor`.".into());
    }
    let tmp = std::env::temp_dir().join(format!("fastuse-cap-{}.bmp", std::process::id()));
    let mut cmd = std::process::Command::new("/usr/sbin/screencapture");
    cmd.args(["-x", "-t", "bmp"]);
    match t {
        CaptureTarget::Desktop => {}
        CaptureTarget::Window(id) => {
            if screen_mode {
                let w = list(std::process::id())
                    .into_iter()
                    .find(|w| w.info.id == id)
                    .ok_or("window not found for capture")?;
                let r = w.info.rect;
                cmd.args(["-R", &format!("{},{},{},{}", r.left, r.top, r.w().max(1), r.h().max(1))]);
            } else {
                cmd.args(["-l", &id.to_string()]);
            }
        }
    }
    cmd.arg(&tmp);
    let ok = cmd.output().map(|o| o.status.success()).unwrap_or(false);
    let bytes = std::fs::read(&tmp).ok();
    let _ = std::fs::remove_file(&tmp);
    let data = bytes.filter(|_| ok).ok_or("screencapture failed")?;
    parse_bmp(&data)
}

fn parse_bmp(data: &[u8]) -> Result<Frame, String> {
    if data.len() < 54 || &data[0..2] != b"BM" {
        return Err("invalid BMP from screencapture".into());
    }
    let offset = u32::from_le_bytes(data[10..14].try_into().unwrap()) as usize;
    let w = i32::from_le_bytes(data[18..22].try_into().unwrap());
    let h_raw = i32::from_le_bytes(data[22..26].try_into().unwrap());
    let bpp = u16::from_le_bytes(data[28..30].try_into().unwrap());
    if w <= 0 || h_raw == 0 || (bpp != 32 && bpp != 24) {
        return Err(format!("unsupported BMP format ({w}x{h_raw}, {bpp} bpp)"));
    }
    let (w, h, top_down) = (w as usize, h_raw.unsigned_abs() as usize, h_raw < 0);
    let bpp_bytes = (bpp / 8) as usize;
    let stride = (w * bpp_bytes + 3) & !3;
    if offset + stride * h > data.len() {
        return Err("truncated BMP data".into());
    }
    let mut bgra = vec![255u8; w * h * 4];
    for y in 0..h {
        let sy = if top_down { y } else { h - 1 - y };
        let src = &data[offset + sy * stride..offset + sy * stride + w * bpp_bytes];
        let dst = &mut bgra[y * w * 4..(y + 1) * w * 4];
        if bpp == 32 {
            dst.copy_from_slice(src);
        } else {
            for x in 0..w {
                dst[x * 4] = src[x * 3];
                dst[x * 4 + 1] = src[x * 3 + 1];
                dst[x * 4 + 2] = src[x * 3 + 2];
            }
        }
    }
    Ok(Frame { w, h, bgra, at: Instant::now() })
}

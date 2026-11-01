//! Window list through CoreGraphics (`CGWindowListCopyWindowInfo`). Titles of other apps' windows
//! are only returned when Screen Recording permission is granted; without it we fall back to the
//! owning app's name, which is still enough to find a window.

use std::ffi::c_void;
use std::ptr;

use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use core_foundation_sys::array::{CFArrayGetCount, CFArrayGetValueAtIndex, CFArrayRef};
use core_foundation_sys::base::{CFGetTypeID, CFRelease};
use core_foundation_sys::dictionary::{CFDictionaryGetValue, CFDictionaryRef};
use core_foundation_sys::number::{kCFNumberFloat64Type, CFNumberGetTypeID, CFNumberGetValue, CFNumberRef};
use core_foundation_sys::string::{CFStringGetTypeID, CFStringRef};

use crate::platform::{Rect, WindowInfo};

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGWindowListCopyWindowInfo(option: u32, relative_to: u32) -> CFArrayRef;
    pub fn CGPreflightScreenCaptureAccess() -> bool;
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

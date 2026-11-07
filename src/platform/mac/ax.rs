//! Accessibility API (AXUIElement): window lookup, tree walk with one batched attribute call per
//! element, and the two actions that need no input events (AXPress, set AXValue).
//!
//! Coordinates are macOS points with the origin at the top-left of the primary display.

use std::ffi::c_void;
use std::ptr;

use core_foundation::array::CFArray;
use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use core_foundation_sys::array::{CFArrayGetCount, CFArrayGetValueAtIndex, CFArrayRef};
use core_foundation_sys::base::{CFGetTypeID, CFRelease, CFRetain, CFTypeID, CFTypeRef};
use core_foundation_sys::number::{kCFNumberFloat64Type, CFNumberGetTypeID, CFNumberGetValue, CFNumberRef};
use core_foundation_sys::string::{CFStringGetTypeID, CFStringRef};

use crate::platform::{tree_row, ElementMeta, Rect};

type AxRef = *const c_void;

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct CGPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct CGSize {
    w: f64,
    h: f64,
}

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    static kCFBooleanTrue: CFTypeRef;
    fn AXIsProcessTrusted() -> u8;
    fn AXUIElementCreateApplication(pid: i32) -> AxRef;
    fn AXUIElementCopyAttributeValue(el: AxRef, attr: CFStringRef, value: *mut CFTypeRef) -> i32;
    fn AXUIElementCopyMultipleAttributeValues(el: AxRef, attrs: CFArrayRef, options: u32, values: *mut CFArrayRef) -> i32;
    fn AXUIElementPerformAction(el: AxRef, action: CFStringRef) -> i32;
    fn AXUIElementSetAttributeValue(el: AxRef, attr: CFStringRef, value: CFTypeRef) -> i32;
    fn AXUIElementSetMessagingTimeout(el: AxRef, seconds: f32) -> i32;
    fn AXValueGetType(v: CFTypeRef) -> u32;
    fn AXValueGetValue(v: CFTypeRef, kind: u32, out: *mut c_void) -> u8;
    fn AXValueGetTypeID() -> CFTypeID;
}

const AX_POINT: u32 = 1;
const AX_SIZE: u32 = 2;
const AX_ERROR_VALUE: u32 = 5;

pub fn trusted() -> bool {
    unsafe { AXIsProcessTrusted() != 0 }
}

/// An owned (retained) CoreFoundation object.
pub struct Owned(pub CFTypeRef);

impl Owned {
    fn new(r: CFTypeRef) -> Option<Self> {
        if r.is_null() {
            None
        } else {
            Some(Self(r))
        }
    }
    /// Retain a borrowed reference.
    fn retain(r: CFTypeRef) -> Option<Self> {
        if r.is_null() {
            return None;
        }
        unsafe { CFRetain(r) };
        Some(Self(r))
    }
}

impl Drop for Owned {
    fn drop(&mut self) {
        unsafe { CFRelease(self.0) }
    }
}

impl Clone for Owned {
    fn clone(&self) -> Self {
        unsafe { CFRetain(self.0) };
        Self(self.0)
    }
}

fn copy_attr(el: AxRef, name: &str) -> Option<Owned> {
    let n = CFString::new(name);
    let mut v: CFTypeRef = ptr::null();
    let e = unsafe { AXUIElementCopyAttributeValue(el, n.as_concrete_TypeRef(), &mut v) };
    if e == 0 {
        Owned::new(v)
    } else {
        None
    }
}

fn is_ax_error(v: CFTypeRef) -> bool {
    unsafe { CFGetTypeID(v) == AXValueGetTypeID() && AXValueGetType(v) == AX_ERROR_VALUE }
}

fn to_string(v: &Owned) -> Option<String> {
    unsafe {
        if CFGetTypeID(v.0) == CFStringGetTypeID() {
            Some(CFString::wrap_under_get_rule(v.0 as CFStringRef).to_string())
        } else {
            None
        }
    }
}

fn to_point(v: &Owned) -> Option<(f64, f64)> {
    unsafe {
        if CFGetTypeID(v.0) != AXValueGetTypeID() {
            return None;
        }
        let mut p = CGPoint::default();
        if AXValueGetValue(v.0, AX_POINT, &mut p as *mut CGPoint as *mut c_void) != 0 {
            Some((p.x, p.y))
        } else {
            None
        }
    }
}

fn to_size(v: &Owned) -> Option<(f64, f64)> {
    unsafe {
        if CFGetTypeID(v.0) != AXValueGetTypeID() {
            return None;
        }
        let mut s = CGSize::default();
        if AXValueGetValue(v.0, AX_SIZE, &mut s as *mut CGSize as *mut c_void) != 0 {
            Some((s.w, s.h))
        } else {
            None
        }
    }
}

#[allow(dead_code)]
fn to_f64(v: &Owned) -> Option<f64> {
    unsafe {
        if CFGetTypeID(v.0) != CFNumberGetTypeID() {
            return None;
        }
        let mut out = 0f64;
        if CFNumberGetValue(v.0 as CFNumberRef, kCFNumberFloat64Type, &mut out as *mut f64 as *mut c_void) {
            Some(out)
        } else {
            None
        }
    }
}

/// The application element for `pid`, with a short messaging timeout so a hung app cannot hang us.
pub fn app_element(pid: i32) -> Option<Owned> {
    let r = unsafe { AXUIElementCreateApplication(pid) };
    let o = Owned::new(r)?;
    unsafe { AXUIElementSetMessagingTimeout(o.0, 2.0) };
    Some(o)
}

fn frame_of(el: AxRef) -> Option<[f64; 4]> {
    let p = to_point(&copy_attr(el, "AXPosition")?)?;
    let s = to_size(&copy_attr(el, "AXSize")?)?;
    Some([p.0, p.1, s.0, s.1])
}

/// The AX window of `pid` whose frame best matches `rect` (the CoreGraphics bounds).
pub fn find_window(pid: i32, rect: Rect) -> Result<Owned, String> {
    if !trusted() {
        return Err("Accessibility permission is not granted. Run `fastuse doctor`.".into());
    }
    let app = app_element(pid).ok_or("could not create an accessibility element for that app")?;
    let wins = copy_attr(app.0, "AXWindows").ok_or("the app exposes no windows through accessibility (is it running, and is the permission granted to the app that launched fastuse?)")?;
    let arr = wins.0 as CFArrayRef;
    let mut best: Option<(f64, Owned)> = None;
    unsafe {
        for i in 0..CFArrayGetCount(arr) {
            let w = CFArrayGetValueAtIndex(arr, i);
            let Some(f) = frame_of(w) else { continue };
            let d = (f[0] - rect.left as f64).abs()
                + (f[1] - rect.top as f64).abs()
                + (f[2] - rect.w() as f64).abs()
                + (f[3] - rect.h() as f64).abs();
            if best.as_ref().map_or(true, |(bd, _)| d < *bd) {
                if let Some(o) = Owned::retain(w) {
                    best = Some((d, o));
                }
            }
        }
    }
    best.map(|(_, o)| o).ok_or_else(|| "no accessibility window matches that window".into())
}

const ATTRS: [&str; 9] = [
    "AXRole",
    "AXTitle",
    "AXDescription",
    "AXValue",
    "AXIdentifier",
    "AXPosition",
    "AXSize",
    "AXChildren",
    "AXPlaceholderValue",
];

/// (our role name, interactive)
fn role_name(ax: &str) -> (&'static str, bool) {
    match ax {
        "AXButton" | "AXDisclosureTriangle" => ("Button", true),
        "AXCheckBox" => ("CheckBox", true),
        "AXRadioButton" => ("RadioButton", true),
        "AXTextField" | "AXTextArea" | "AXSearchField" | "AXSecureTextField" => ("Edit", true),
        "AXComboBox" | "AXPopUpButton" | "AXMenuButton" => ("ComboBox", true),
        "AXMenuItem" | "AXMenuBarItem" => ("MenuItem", true),
        "AXLink" => ("Hyperlink", true),
        "AXSlider" | "AXIncrementor" => ("Slider", true),
        "AXStaticText" => ("Text", false),
        "AXGroup" | "AXSplitGroup" => ("Group", false),
        "AXWindow" | "AXSheet" | "AXDrawer" => ("Window", false),
        "AXScrollArea" | "AXSplitter" | "AXLayoutArea" => ("Pane", false),
        "AXToolbar" => ("ToolBar", false),
        "AXList" => ("List", false),
        "AXTable" | "AXOutline" => ("Table", false),
        "AXRow" => ("ListItem", false),
        "AXMenu" => ("Menu", false),
        "AXMenuBar" => ("MenuBar", false),
        "AXImage" => ("Image", false),
        "AXTabGroup" => ("Tab", false),
        "AXWebArea" => ("Document", true),
        _ => ("Element", false),
    }
}

pub struct Walk {
    pub elements: Vec<Owned>,
    pub meta: Vec<ElementMeta>,
    pub text: String,
    pub count: usize,
    pub truncated: bool,
}

pub fn walk_window(win: &Owned, max_elements: usize, max_depth: usize) -> Walk {
    let names: Vec<CFString> = ATTRS.iter().map(|a| CFString::new(a)).collect();
    let attrs = CFArray::from_CFTypes(&names);
    let mut w = Walk { elements: Vec::new(), meta: Vec::new(), text: String::with_capacity(4096), count: 0, truncated: false };
    walk(win.0, &attrs, 0, max_depth, max_elements, &mut w);
    w.text.push('\n');
    w
}

fn walk(el: AxRef, attrs: &CFArray<CFString>, depth: usize, max_depth: usize, max: usize, w: &mut Walk) {
    if w.count >= max {
        w.truncated = true;
        return;
    }
    let mut vals: CFArrayRef = ptr::null();
    let e = unsafe { AXUIElementCopyMultipleAttributeValues(el, attrs.as_concrete_TypeRef(), 0, &mut vals) };
    if e != 0 || vals.is_null() {
        return;
    }
    let vals = Owned(vals as CFTypeRef);
    let arr = vals.0 as CFArrayRef;
    let item = |i: usize| -> Option<Owned> {
        unsafe {
            if (i as isize) >= CFArrayGetCount(arr) {
                return None;
            }
            let v = CFArrayGetValueAtIndex(arr, i as isize);
            if v.is_null() || is_ax_error(v) {
                return None;
            }
            Owned::retain(v)
        }
    };
    let role_ax = item(0).and_then(|v| to_string(&v)).unwrap_or_default();
    let title = item(1).and_then(|v| to_string(&v)).unwrap_or_default();
    let desc = item(2).and_then(|v| to_string(&v)).unwrap_or_default();
    let value = item(3).and_then(|v| to_string(&v)).unwrap_or_default();
    let ident = item(4).and_then(|v| to_string(&v)).unwrap_or_default();
    let pos = item(5).and_then(|v| to_point(&v));
    let size = item(6).and_then(|v| to_size(&v));
    let placeholder = item(8).and_then(|v| to_string(&v)).unwrap_or_default();

    let (role, interactive) = role_name(&role_ax);
    let name = if !title.is_empty() {
        title
    } else if !desc.is_empty() {
        desc
    } else if !placeholder.is_empty() {
        placeholder
    } else if role == "Text" {
        value.clone()
    } else {
        String::new()
    };
    let rect = match (pos, size) {
        (Some(p), Some(s)) => Some(Rect { left: p.0 as i32, top: p.1 as i32, right: (p.0 + s.0) as i32, bottom: (p.1 + s.1) as i32 }),
        _ => None,
    };
    let show_value = if role == "Edit" || role == "Document" { value } else { String::new() };

    // Collapse unnamed structural wrappers so deep Cocoa/Electron/WebKit trees do not exhaust max_elements
    let is_wrapper = matches!(role, "Group" | "Pane" | "Element");
    if depth > 0 && !interactive && is_wrapper && name.is_empty() && show_value.is_empty() && ident.is_empty() {
        if let Some(kids) = item(7) {
            unsafe {
                let ka = kids.0 as CFArrayRef;
                if CFGetTypeID(kids.0) == core_foundation_sys::array::CFArrayGetTypeID() {
                    for i in 0..CFArrayGetCount(ka) {
                        walk(CFArrayGetValueAtIndex(ka, i), attrs, depth, max_depth, max, w);
                    }
                }
            }
        }
        return;
    }

    w.count += 1;
    let mut idx = None;
    let has_bounds = rect.map_or(true, |r| r.w() > 0 && r.h() > 0);
    if interactive && has_bounds {
        let i = w.elements.len();
        if let Some(o) = Owned::retain(el) {
            w.elements.push(o);
            let r = rect.unwrap_or_default();
            w.meta.push(ElementMeta { name: name.clone(), id: ident.clone(), role, rect: [r.left, r.top, r.right, r.bottom] });
            idx = Some(i);
        }
    }
    tree_row(&mut w.text, depth, idx, role, &name, &show_value, &ident, if idx.is_some() { rect } else { None });
    if depth + 1 >= max_depth {
        return;
    }
    if let Some(kids) = item(7) {
        unsafe {
            let ka = kids.0 as CFArrayRef;
            if CFGetTypeID(kids.0) == core_foundation_sys::array::CFArrayGetTypeID() {
                for i in 0..CFArrayGetCount(ka) {
                    walk(CFArrayGetValueAtIndex(ka, i), attrs, depth + 1, max_depth, max, w);
                }
            }
        }
    }
}

fn ax_err(code: i32) -> String {
    let what = match code {
        -25200 => "failure",
        -25201 => "illegal argument",
        -25202 => "invalid element",
        -25204 => "cannot complete (app busy or blocked)",
        -25205 => "action unsupported",
        -25206 => "notification unsupported",
        -25211 => "API disabled: Accessibility permission missing",
        -25212 => "no value",
        _ => "error",
    };
    format!("accessibility error {code} ({what})")
}

/// Press an element. Tries AXPress, then AXConfirm and AXPick; returns which one worked.
pub fn press(el: &Owned) -> Result<&'static str, String> {
    let mut last = 0;
    for action in ["AXPress", "AXConfirm", "AXPick"] {
        let a = CFString::new(action);
        let e = unsafe { AXUIElementPerformAction(el.0, a.as_concrete_TypeRef()) };
        if e == 0 {
            return Ok(action);
        }
        last = e;
    }
    Err(ax_err(last))
}

/// Focus an accessibility element inside its window without activating the app.
pub fn focus(el: &Owned) {
    let a = CFString::new("AXFocused");
    unsafe {
        let _ = AXUIElementSetAttributeValue(el.0, a.as_concrete_TypeRef(), kCFBooleanTrue);
    }
}

/// Bring the app owning `pid` to the foreground via AXFrontmost.
pub fn raise_app(pid: i32) -> bool {
    let Some(app) = app_element(pid) else { return false };
    let a = CFString::new("AXFrontmost");
    let ok = unsafe { AXUIElementSetAttributeValue(app.0, a.as_concrete_TypeRef(), kCFBooleanTrue) == 0 };
    std::thread::sleep(std::time::Duration::from_millis(30));
    ok
}

/// Set the element's AXValue to a string (text fields, text areas).
pub fn set_value(el: &Owned, value: &str) -> Result<(), String> {
    focus(el);
    let a = CFString::new("AXValue");
    let v = CFString::new(value);
    let e = unsafe { AXUIElementSetAttributeValue(el.0, a.as_concrete_TypeRef(), v.as_CFTypeRef()) };
    if e == 0 {
        Ok(())
    } else {
        Err(ax_err(e))
    }
}

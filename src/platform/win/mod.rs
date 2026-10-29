//! Windows backend: UI Automation for the tree and actions, window messages and SendInput for
//! input, PrintWindow/GDI for capture, a layered window for the agent cursor.

mod capture;
mod cursor;
mod enumerate;
mod input;
mod uia;

use crate::imaging::Frame;
use crate::platform::{
    Capabilities, CaptureTarget, ClickKind, Platform, TreeRead, WindowInfo, WindowQuery,
};

pub struct WinPlatform {
    uia: uia::Uia,
}

impl WinPlatform {
    pub fn new() -> Result<Self, String> {
        unsafe {
            let _ = windows::Win32::UI::HiDpi::SetProcessDpiAwarenessContext(
                windows::Win32::UI::HiDpi::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
            );
        }
        cursor::init();
        let uia = uia::Uia::new().map_err(|e| format!("UI Automation init failed: {e}"))?;
        Ok(Self { uia })
    }
}

impl Platform for WinPlatform {
    fn name(&self) -> &'static str {
        "Windows"
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            background_input: true,
            foreground_input: true,
            window_capture: true,
            desktop_capture: true,
            overlay_cursor: cursor::enabled(),
        }
    }

    fn list_windows(&self) -> Vec<WindowInfo> {
        enumerate::list()
    }

    fn exe_name(&self, pid: u32) -> String {
        enumerate::exe_name(pid)
    }

    fn find_window(&self, q: &WindowQuery) -> Option<WindowInfo> {
        enumerate::find(q)
    }

    fn read_tree(&mut self, w: &WindowInfo, max_elements: usize, max_depth: usize) -> Result<TreeRead, String> {
        self.uia.read(w.id as isize, max_elements, max_depth)
    }

    fn element_count(&self) -> usize {
        self.uia.element_count()
    }

    fn resolve_target(&self, target: &str) -> Result<usize, String> {
        self.uia.resolve_target(target)
    }

    fn element_center(&self, i: usize) -> Option<(i32, i32)> {
        self.uia.center_of(i)
    }

    fn click_element(&mut self, i: usize) -> Result<String, String> {
        let el = self.uia.element(i).ok_or_else(|| format!("no element [{i}] in snapshot"))?;
        input::invoke(el).map(|s| s.to_string())
    }

    fn set_value(&mut self, i: usize, value: &str) -> Result<(), String> {
        let el = self.uia.element(i).ok_or_else(|| format!("no element [{i}] in snapshot"))?;
        input::set_value(el, value)
    }

    fn type_text(&mut self, w: Option<&WindowInfo>, el: Option<usize>, text: &str, foreground: bool) -> Result<String, String> {
        if foreground {
            let w = w.ok_or("foreground typing needs a window")?;
            if !input::raise(w.id as isize) {
                return Err("could not bring the window to the foreground; text not sent".into());
            }
            let n = input::send_text_foreground(text);
            return Ok(format!("type_text {n} chars via SendInput (foreground)"));
        }
        let target = el
            .and_then(|i| self.uia.element(i))
            .and_then(input::element_hwnd)
            .or_else(|| w.map(|w| input::focus_target(w.id as isize)))
            .ok_or("type_text needs a window (window or window_id) when the element has no window of its own")?;
        let n = input::post_text(target, text);
        Ok(format!("type_text {n} chars via WM_CHAR (background)"))
    }

    fn press_key(&mut self, w: &WindowInfo, key: &str) -> Result<String, String> {
        let vk = input::vk(key).ok_or_else(|| format!("unknown key '{key}'"))?;
        input::post_key(input::focus_target(w.id as isize), vk);
        Ok(format!("press_key {key} via WM_KEYDOWN/UP (background, no modifiers)"))
    }

    fn hotkey(&mut self, w: &WindowInfo, keys: &str) -> Result<String, String> {
        input::hotkey(w.id as isize, keys)?;
        Ok(format!("hotkey {keys} via SendInput (foreground)"))
    }

    fn click_point(&mut self, w: &WindowInfo, x: i32, y: i32, kind: ClickKind, foreground: bool) -> Result<String, String> {
        input::click_xy(
            w.id as isize,
            w.rect.left,
            w.rect.top,
            x,
            y,
            kind == ClickKind::Double,
            kind == ClickKind::Right,
            foreground,
        )
    }

    fn grab(&mut self, t: CaptureTarget, screen_mode: bool) -> Result<Frame, String> {
        capture::grab(t, screen_mode)
    }

    fn move_cursor(&self, x: i32, y: i32, click: bool) {
        cursor::go(x, y, click);
    }
}

//! The platform boundary. Everything the MCP server needs from an operating system lives behind
//! the `Platform` trait: windows, the accessibility tree, input, capture and the agent cursor.
//! The core (server.rs, tools.rs, imaging.rs, runner.rs) never calls an OS API directly.
//!
//! Adding a platform = implementing `Platform` in `platform/<name>/` and selecting it in `default()`.

use std::fmt::Write;

use crate::imaging::Frame;

#[cfg(windows)]
pub mod win;

#[derive(Clone, Copy, Default, Debug)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub fn w(&self) -> i32 {
        self.right - self.left
    }
    pub fn h(&self) -> i32 {
        self.bottom - self.top
    }
}

#[derive(Clone, Debug)]
pub struct WindowInfo {
    /// Opaque, platform-defined handle (an HWND on Windows). Stable while the window lives.
    pub id: i64,
    pub pid: u32,
    pub title: String,
    pub rect: Rect,
    pub minimized: bool,
}

/// How a window is looked up. Precedence: title substring, then id, then the largest window of pid.
#[derive(Default, Clone, Debug)]
pub struct WindowQuery {
    pub id: Option<i64>,
    pub pid: Option<u32>,
    pub title: Option<String>,
}

pub struct TreeRead {
    pub text: String,
    pub count: usize,
    pub truncated: bool,
    pub build_us: u128,
    pub walk_us: u128,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClickKind {
    Single,
    Double,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CaptureTarget {
    Desktop,
    Window(i64),
}

/// What this platform (and session) can do. Tools report unavailable features instead of failing
/// mysteriously, so the model does not retry something that cannot work.
#[derive(Clone, Copy, Debug)]
pub struct Capabilities {
    pub background_input: bool,
    pub foreground_input: bool,
    pub window_capture: bool,
    pub desktop_capture: bool,
    pub overlay_cursor: bool,
}

/// Element indices are positions in the interactive-element list of the latest `read_tree`.
/// The core owns snapshot ids and rejects stale tokens; a platform only sees indices.
pub trait Platform {
    fn name(&self) -> &'static str;
    fn capabilities(&self) -> Capabilities;

    fn list_windows(&self) -> Vec<WindowInfo>;
    fn exe_name(&self, pid: u32) -> String;
    fn find_window(&self, q: &WindowQuery) -> Option<WindowInfo>;

    /// Read the window's UI tree as text (use `tree_row` so the format is identical everywhere)
    /// and remember its interactive elements for the calls below.
    fn read_tree(&mut self, w: &WindowInfo, max_elements: usize, max_depth: usize) -> Result<TreeRead, String>;
    fn element_count(&self) -> usize;
    /// "Go", "Button:Go" or an automation id -> element index in the latest tree.
    fn resolve_target(&self, target: &str) -> Result<usize, String>;
    fn element_center(&self, i: usize) -> Option<(i32, i32)>;

    /// Activate an element (UIA Invoke / AT-SPI DoAction). Returns how, e.g. "InvokePattern".
    fn click_element(&mut self, i: usize) -> Result<String, String>;
    fn set_value(&mut self, i: usize, value: &str) -> Result<(), String>;
    /// Returns a one-line description including the delivery method.
    fn type_text(&mut self, w: Option<&WindowInfo>, el: Option<usize>, text: &str, foreground: bool) -> Result<String, String>;
    fn press_key(&mut self, w: &WindowInfo, key: &str) -> Result<String, String>;
    fn hotkey(&mut self, w: &WindowInfo, keys: &str) -> Result<String, String>;
    /// (x, y) in window-local pixels. Returns how, e.g. "PostMessage (background)".
    fn click_point(&mut self, w: &WindowInfo, x: i32, y: i32, kind: ClickKind, foreground: bool) -> Result<String, String>;

    /// BGRA pixels. `screen_mode` asks for a screen-region grab instead of per-window rendering.
    fn grab(&mut self, t: CaptureTarget, screen_mode: bool) -> Result<Frame, String>;
    /// Glide the agent cursor to a screen point. No-op where unsupported.
    fn move_cursor(&self, x: i32, y: i32, click: bool);
}

/// One row of the text tree. Shared so the output format is identical on every platform:
/// `  - [N] Role "name" value="..." id=x @(left,top,w,h)` (`[N]` and `@` only for interactive rows).
pub fn tree_row(out: &mut String, depth: usize, idx: Option<usize>, role: &str, name: &str, value: &str, id: &str, rect: Option<Rect>) {
    for _ in 0..depth {
        out.push_str("  ");
    }
    out.push_str("- ");
    if let Some(i) = idx {
        let _ = write!(out, "[{i}] ");
    }
    out.push_str(role);
    if !name.is_empty() {
        let _ = write!(out, " \"{}\"", name.replace('"', "'").chars().take(80).collect::<String>());
    }
    if !value.is_empty() {
        let _ = write!(out, " value=\"{}\"", value.replace('"', "'").chars().take(120).collect::<String>());
    }
    if !id.is_empty() {
        let _ = write!(out, " id={id}");
    }
    if let Some(r) = rect {
        let _ = write!(out, " @({},{},{},{})", r.left, r.top, r.w(), r.h());
    }
    out.push('\n');
}

#[cfg(windows)]
pub fn default() -> Result<Box<dyn Platform>, String> {
    Ok(Box::new(win::WinPlatform::new()?))
}

#[cfg(not(windows))]
pub fn default() -> Result<Box<dyn Platform>, String> {
    Err("fastuse currently supports Windows only; Linux support is planned (see docs/CROSS_PLATFORM.md)".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_row_format() {
        let mut s = String::new();
        let r = Rect { left: 10, top: 20, right: 85, bottom: 43 };
        tree_row(&mut s, 1, Some(3), "Button", "Go \"now\"", "", "go", Some(r));
        assert_eq!(s, "  - [3] Button \"Go 'now'\" id=go @(10,20,75,23)\n");
        let mut s = String::new();
        tree_row(&mut s, 0, None, "Edit", "input", "hi", "", None);
        assert_eq!(s, "- Edit \"input\" value=\"hi\"\n");
    }
}

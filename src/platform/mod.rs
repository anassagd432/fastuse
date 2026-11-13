//! The platform boundary. Everything the MCP server needs from an operating system lives behind
//! the `Platform` trait: windows, the accessibility tree, input, capture and the agent cursor.
//! The core (server.rs, tools.rs, imaging.rs, runner.rs) never calls an OS API directly.
//!
//! Adding a platform = implementing `Platform` in `platform/<name>/` and selecting it in `default()`.

use std::fmt::Write;

use crate::imaging::Frame;

#[cfg(windows)]
pub mod win;
#[cfg(target_os = "macos")]
pub mod mac;

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
    /// Scroll `direction` ("up", "down", "left", "right") by `amount` ticks on element `el` or window `w`.
    fn scroll(&mut self, _w: &WindowInfo, _el: Option<usize>, _direction: &str, _amount: i32) -> Result<String, String> {
        Err(format!("scroll is not implemented on {}", self.name()))
    }

    /// BGRA pixels. `screen_mode` asks for a screen-region grab instead of per-window rendering.
    fn grab(&mut self, t: CaptureTarget, screen_mode: bool) -> Result<Frame, String>;
    /// Glide the agent cursor to a screen point. No-op where unsupported.
    fn move_cursor(&self, x: i32, y: i32, click: bool);
    /// Keep the agent overlay (glow and banner) visible while an action or session is active.
    fn ping_overlay(&self, _active: bool) {}

    /// Human-readable health report for `fastuse doctor` (permissions, accessibility switches...).
    fn doctor(&self) -> String {
        format!("{}: no extra checks.", self.name())
    }
}

/// What a backend remembers about each interactive element of its latest tree read.
#[derive(Clone, Debug)]
pub struct ElementMeta {
    pub name: String,
    pub id: String,
    pub role: &'static str,
    /// left, top, right, bottom in screen pixels
    pub rect: [i32; 4],
}

/// Screen-space center of element `i`, if it has bounds.
pub fn center_in(meta: &[ElementMeta], i: usize) -> Option<(i32, i32)> {
    let r = meta.get(i)?.rect;
    if r[2] <= r[0] || r[3] <= r[1] {
        return None;
    }
    Some(((r[0] + r[2]) / 2, (r[1] + r[3]) / 2))
}

/// Resolve a human target ("Go", "Button:Go", "[3]", "s00000001:3", or an automation id) to an index into `meta`.
/// Exact name/id matches win over substring matches; ambiguity is an error that lists candidates.
pub fn resolve_in(meta: &[ElementMeta], target: &str) -> Result<usize, String> {
    if meta.is_empty() {
        return Err("no snapshot yet; the window has no interactive elements or was not read".into());
    }
    let t = target.trim();
    if let Some(idx_str) = t.strip_prefix('[').and_then(|s| s.strip_suffix(']')).or_else(|| t.strip_prefix('#')) {
        if let Ok(i) = idx_str.trim().parse::<usize>() {
            return if i < meta.len() {
                Ok(i)
            } else {
                Err(format!("no element [{i}] in snapshot"))
            };
        }
    }
    if let Some((sid, idx_str)) = t.split_once(':') {
        if sid.starts_with('s') && sid.len() > 1 && sid[1..].chars().all(|c| c.is_ascii_digit()) {
            if let Ok(i) = idx_str.trim().parse::<usize>() {
                return if i < meta.len() {
                    Ok(i)
                } else {
                    Err(format!("no element [{i}] in snapshot"))
                };
            }
        }
    }
    let (role, want) = match t.split_once(':') {
        Some((r, n)) if meta.iter().any(|m| m.role.eq_ignore_ascii_case(r)) => (Some(r), n),
        _ => (None, t),
    };
    let w = want.trim().to_lowercase();
    let ok_role = |i: usize| role.map_or(true, |r| meta[i].role.eq_ignore_ascii_case(r));
    let exact: Vec<usize> = (0..meta.len())
        .filter(|&i| ok_role(i) && (meta[i].name.to_lowercase() == w || meta[i].id.to_lowercase() == w))
        .collect();
    let hits: Vec<usize> = if exact.is_empty() {
        (0..meta.len())
            .filter(|&i| ok_role(i) && (meta[i].name.to_lowercase().contains(&w) || meta[i].id.to_lowercase().contains(&w)))
            .collect()
    } else {
        exact
    };
    match hits.len() {
        1 => Ok(hits[0]),
        0 => Err(format!("no element matches target '{target}'")),
        _ => Err(format!(
            "target '{target}' is ambiguous: {}. Use Role:Name or an id.",
            hits.iter()
                .take(6)
                .map(|&i| format!("[{i}] {} \"{}\"", meta[i].role, meta[i].name))
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

/// Choose a window from `all` for a query. Precedence: title substring (non-minimized first),
/// then exact id, then the largest window of `pid`.
pub fn pick_window(all: Vec<WindowInfo>, q: &WindowQuery) -> Option<WindowInfo> {
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

fn strip_trailing_rect(s: &str) -> &str {
    if let Some(pos) = s.rfind(" @(") {
        if s.ends_with(')') {
            let inner = &s[pos + 3..s.len() - 1];
            if !inner.is_empty() && inner.bytes().all(|b| b.is_ascii_digit() || b == b',' || b == b'-') {
                return &s[..pos];
            }
        }
    }
    s
}

fn compact_line(line: &str) -> String {
    let trimmed = line.trim_start();
    let indent = if line.len() > trimmed.len() { "  " } else { "" };
    let no_rect = strip_trailing_rect(trimmed);
    let body = if no_rect.contains('"') {
        if let Some(pos) = no_rect.rfind(" id=") {
            let after_quote = no_rect.rfind('"').unwrap_or(0);
            if pos > after_quote {
                &no_rect[..pos]
            } else {
                no_rect
            }
        } else {
            no_rect
        }
    } else {
        no_rect
    };
    format!("{indent}{body}")
}

fn diff_key(line: &str) -> &str {
    let t = line.trim_start().strip_prefix("- ").unwrap_or(line.trim_start());
    if let Some(rest) = t.strip_prefix('[') {
        if let Some(end) = rest.find("] ") {
            return &rest[end + 2..];
        }
    }
    t
}

/// Shrinks UI/DOM tree output for LLM token efficiency (omits coordinates & redundant IDs unless
/// `details` is true) and optionally filters rows by case-insensitive substring.
pub fn format_tree_compact(text: &str, details: bool, filter: Option<&str>) -> String {
    let q = filter.map(|s| s.trim().to_ascii_lowercase()).filter(|s| !s.is_empty());
    let mut out = String::with_capacity(text.len() / 2 + 32);
    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        if let Some(ref needle) = q {
            if !line.to_ascii_lowercase().contains(needle) {
                continue;
            }
        }
        if details {
            out.push_str(line);
        } else {
            out.push_str(&compact_line(line));
        }
        out.push('\n');
    }
    if out.is_empty() {
        if let Some(needle) = q {
            return format!("(no elements match \"{needle}\")\n");
        }
    }
    out
}

/// Returns only added or changed tree lines compared to `prev`, keeping live `[N]` indices.
pub fn diff_tree(prev: &str, cur: &str) -> String {
    use std::collections::HashSet;
    let prev_keys: HashSet<&str> = prev.lines().filter(|l| !l.is_empty()).map(diff_key).collect();
    let mut changed = Vec::new();
    let mut unchanged = 0usize;
    for line in cur.lines().filter(|l| !l.is_empty()) {
        if prev_keys.contains(diff_key(line)) {
            unchanged += 1;
        } else {
            changed.push(line);
        }
    }
    if changed.is_empty() {
        return format!("(no UI change; {unchanged} unchanged)\n");
    }
    let mut out = if unchanged > 0 {
        format!("({unchanged} unchanged omitted)\n")
    } else {
        String::new()
    };
    for l in changed {
        out.push_str(l);
        out.push('\n');
    }
    out
}

#[cfg(windows)]
fn default() -> Result<Box<dyn Platform>, String> {
    Ok(Box::new(win::WinPlatform::new()?))
}

#[cfg(not(any(windows, target_os = "macos")))]
fn default() -> Result<Box<dyn Platform>, String> {
    Err("fastuse currently supports Windows only; macOS is next, then Linux (see docs/MACOS.md)".into())
}

/// Run the application with this OS's backend.
///
/// The backend is created and used on whichever thread `f` runs on. Windows runs `f` right here
/// on the main thread. macOS (see mac::run_app) runs `f` on a worker thread and gives the main
/// thread to AppKit, which the agent overlay windows need.
#[cfg(target_os = "macos")]
pub use mac::run_app;

#[cfg(not(target_os = "macos"))]
pub fn run_app<F>(f: F) -> i32
where
    F: FnOnce(Box<dyn Platform>) -> i32 + Send + 'static,
{
    match default() {
        Ok(p) => f(p),
        Err(e) => {
            eprintln!("fastuse: {e}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    fn meta(name: &str, id: &str, role: &'static str) -> ElementMeta {
        ElementMeta { name: name.into(), id: id.into(), role, rect: [0, 0, 10, 10] }
    }

    #[test]
    fn resolve_exact_beats_substring_and_role_prefix_disambiguates() {
        let m = vec![meta("Save", "", "Button"), meta("Save As", "", "Button"), meta("Save", "", "MenuItem")];
        assert!(resolve_in(&m, "Save").unwrap_err().contains("ambiguous")); // two exact matches
        assert_eq!(resolve_in(&m, "Button:Save").unwrap(), 0);
        assert_eq!(resolve_in(&m, "MenuItem:Save").unwrap(), 2);
        assert_eq!(resolve_in(&m, "as").unwrap(), 1); // unique substring
        assert_eq!(resolve_in(&m, "[1]").unwrap(), 1);
        assert_eq!(resolve_in(&m, "#2").unwrap(), 2);
        assert_eq!(resolve_in(&m, "s00000001:0").unwrap(), 0);
        assert!(resolve_in(&m, "[99]").is_err());
        assert!(resolve_in(&m, "nope").is_err());
        assert!(resolve_in(&[], "x").is_err());
    }

    #[test]
    fn pick_window_precedence() {
        let w = |id, pid, title: &str, w, mini| WindowInfo { id, pid, title: title.into(), rect: Rect { left: 0, top: 0, right: w, bottom: 10 }, minimized: mini };
        let all = || vec![w(1, 7, "Notes - a", 100, true), w(2, 7, "Notes - b", 50, false), w(3, 8, "Other", 500, false)];
        let by_title = pick_window(all(), &WindowQuery { title: Some("notes".into()), ..Default::default() }).unwrap();
        assert_eq!(by_title.id, 2); // non-minimized first
        assert_eq!(pick_window(all(), &WindowQuery { id: Some(3), ..Default::default() }).unwrap().pid, 8);
        assert_eq!(pick_window(all(), &WindowQuery { pid: Some(7), ..Default::default() }).unwrap().id, 1); // largest
        assert!(pick_window(all(), &WindowQuery::default()).is_none());
    }
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

    #[test]
    fn compact_and_diff_shrink_tree_cleanly() {
        let raw = "      - [0] Button \"Save\" id=btn-save @(10,20,80,30)\n  - [1] Button id=close @(90,20,24,24)\n";
        assert_eq!(
            format_tree_compact(raw, false, None),
            "  - [0] Button \"Save\"\n  - [1] Button id=close\n"
        );
        assert_eq!(
            format_tree_compact(raw, false, Some("save")),
            "  - [0] Button \"Save\"\n"
        );
        let prev = "  - [0] Button \"Save\"\n  - [1] Edit \"Name\" value=\"old\"\n";
        let cur = "  - [0] Button \"Save\"\n  - [1] Edit \"Name\" value=\"new\"\n";
        assert_eq!(
            diff_tree(prev, cur),
            "(1 unchanged omitted)\n  - [1] Edit \"Name\" value=\"new\"\n"
        );
    }
}

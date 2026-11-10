//! macOS backend (M1 plus the agent overlay).
//!
//! Working: window list (CoreGraphics), UI tree and element actions through the Accessibility API,
//! the agent cursor / screen-edge glow / banner (AppKit), `fastuse doctor`. Not yet: typing, keys,
//! coordinate clicks (M2) and screenshots (M3). Those return clear "not implemented yet" errors
//! and capabilities report them.

mod ax;
mod cg;
mod overlay;

use std::time::Instant;

use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};

use crate::imaging::Frame;
use crate::platform::{
    center_in, pick_window, resolve_in, Capabilities, CaptureTarget, ClickKind, ElementMeta, Platform, TreeRead,
    WindowInfo, WindowQuery,
};

pub struct MacPlatform {
    elements: Vec<ax::Owned>,
    meta: Vec<ElementMeta>,
    last_pid: u32,
}

impl MacPlatform {
    pub fn new() -> Result<Self, String> {
        crate::overlay::init(overlay::MacSurface::new);
        Ok(Self { elements: Vec::new(), meta: Vec::new(), last_pid: 0 })
    }
}

fn raise_pid(pid: u32) -> bool {
    ax::raise_app(pid as i32)
}

impl Platform for MacPlatform {
    fn name(&self) -> &'static str {
        "macOS"
    }

    fn capabilities(&self) -> Capabilities {
        let sr = unsafe { cg::CGPreflightScreenCaptureAccess() };
        Capabilities {
            background_input: true,
            foreground_input: true,
            window_capture: sr,
            desktop_capture: sr,
            overlay_cursor: crate::overlay::enabled(),
        }
    }

    fn list_windows(&self) -> Vec<WindowInfo> {
        cg::list(std::process::id()).into_iter().map(|w| w.info).collect()
    }

    fn exe_name(&self, pid: u32) -> String {
        cg::list(std::process::id()).into_iter().find(|w| w.info.pid == pid).map(|w| w.owner).unwrap_or_default()
    }

    fn find_window(&self, q: &WindowQuery) -> Option<WindowInfo> {
        pick_window(self.list_windows(), q)
    }

    fn read_tree(&mut self, w: &WindowInfo, max_elements: usize, max_depth: usize) -> Result<TreeRead, String> {
        let t0 = Instant::now();
        let win = ax::find_window(w.pid as i32, w.rect)?;
        let build_us = t0.elapsed().as_micros();
        let t1 = Instant::now();
        let walk = ax::walk_window(&win, max_elements, max_depth);
        self.elements = walk.elements;
        self.meta = walk.meta;
        self.last_pid = w.pid;
        Ok(TreeRead {
            text: walk.text,
            count: walk.count,
            truncated: walk.truncated,
            build_us,
            walk_us: t1.elapsed().as_micros(),
        })
    }

    fn element_count(&self) -> usize {
        self.elements.len()
    }

    fn resolve_target(&self, target: &str) -> Result<usize, String> {
        resolve_in(&self.meta, target)
    }

    fn element_center(&self, i: usize) -> Option<(i32, i32)> {
        center_in(&self.meta, i)
    }

    fn click_element(&mut self, i: usize) -> Result<String, String> {
        let center = self.element_center(i);
        let pid = self.last_pid;
        let el = self.elements.get(i).ok_or_else(|| format!("no element [{i}] in snapshot"))?;
        match ax::press(el) {
            Ok(how) => Ok(how.to_string()),
            Err(e) => {
                if pid != 0 {
                    if let Some((cx, cy)) = center {
                        if let Ok(how) = cg::click_point(pid as i32, cx, cy, ClickKind::Single, false) {
                            return Ok(format!("{how} (AX fallback)"));
                        }
                    }
                }
                Err(e)
            }
        }
    }

    fn set_value(&mut self, i: usize, value: &str) -> Result<(), String> {
        let el = self.elements.get(i).ok_or_else(|| format!("no element [{i}] in snapshot"))?;
        ax::set_value(el, value)
    }

    fn type_text(&mut self, w: Option<&WindowInfo>, el: Option<usize>, text: &str, foreground: bool) -> Result<String, String> {
        if let Some(e) = el.and_then(|i| self.elements.get(i)) {
            ax::focus(e);
        }
        let pid = w.map(|w| w.pid).or(if self.last_pid != 0 { Some(self.last_pid) } else { None }).ok_or("type_text needs a window")?;
        if foreground && !raise_pid(pid) {
            return Err("could not activate the target app for foreground typing".into());
        }
        let n = cg::type_text(pid as i32, text, foreground);
        Ok(format!("type_text {n} chars via CGEvent ({})", if foreground { "foreground" } else { "background" }))
    }

    fn press_key(&mut self, w: &WindowInfo, key: &str) -> Result<String, String> {
        let k = cg::vk(key).ok_or_else(|| format!("unknown key '{key}'"))?;
        cg::post_key(w.pid as i32, k, 0, false);
        Ok(format!("press_key {key} via CGEventPostToPid (background)"))
    }

    fn hotkey(&mut self, w: &WindowInfo, keys: &str) -> Result<String, String> {
        let fg = raise_pid(w.pid);
        cg::hotkey(w.pid as i32, keys, fg)?;
        Ok(format!("hotkey {keys} via CGEvent ({})", if fg { "foreground" } else { "background" }))
    }

    fn click_point(&mut self, w: &WindowInfo, x: i32, y: i32, kind: ClickKind, foreground: bool) -> Result<String, String> {
        if foreground && !raise_pid(w.pid) {
            return Err("could not activate the target app for foreground click".into());
        }
        cg::click_point(w.pid as i32, w.rect.left + x, w.rect.top + y, kind, foreground)
    }

    fn scroll(&mut self, w: &WindowInfo, _el: Option<usize>, direction: &str, amount: i32) -> Result<String, String> {
        cg::scroll(w.pid as i32, direction, amount)
    }

    fn grab(&mut self, t: CaptureTarget, screen_mode: bool) -> Result<Frame, String> {
        cg::grab(t, screen_mode)
    }

    fn move_cursor(&self, x: i32, y: i32, click: bool) {
        crate::overlay::go(x, y, click);
    }

    fn ping_overlay(&self, active: bool) {
        crate::overlay::set_active(active);
    }

    fn doctor(&self) -> String {
        let ax_ok = ax::trusted();
        let sr_ok = unsafe { cg::CGPreflightScreenCaptureAccess() };
        let exe = std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_else(|_| "?".into());
        let wins = cg::list(std::process::id());
        let titled = wins.iter().filter(|w| w.info.title.contains(" â€” ")).count();
        let mut s = String::new();
        s += &format!("fastuse {} on macOS\nbinary: {exe}\n\n", env!("CARGO_PKG_VERSION"));
        s += &format!(
            "Accessibility:    {}\n",
            if ax_ok { "granted" } else { "NOT granted: needed to read and press elements" }
        );
        s += &format!(
            "Screen Recording: {}\n",
            if sr_ok { "granted" } else { "NOT granted: window titles stay hidden (app names still work); screenshots need it (M3)" }
        );
        s += &format!("Visible windows:  {} ({} with titles)\n\n", wins.len(), titled);
        if !ax_ok || !sr_ok {
            s += "macOS attaches these permissions to the app that LAUNCHED fastuse (Terminal, iTerm,\n";
            s += "Antigravity, or agy's host app), not to fastuse itself. Open System Settings > Privacy &\n";
            s += "Security > Accessibility (and Screen Recording), add that app, turn it on, then QUIT AND\n";
            s += "REOPEN that app: a running process does not pick up a new grant.\n";
        } else {
            s += "Permissions look good.\n";
        }
        s
    }
}

/// AppKit must own the main thread: it runs the application loop (which serves the overlay
/// windows) while the MCP server runs on a worker thread and exits the process when it is done.
/// The app is an "accessory": no Dock icon, no menu bar, and it never takes keyboard focus.
pub fn run_app<F>(f: F) -> i32
where
    F: FnOnce(Box<dyn Platform>) -> i32 + Send + 'static,
{
    let Some(mtm) = MainThreadMarker::new() else {
        eprintln!("fastuse: must start on the main thread");
        return 1;
    };
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    std::thread::spawn(move || {
        let code = match MacPlatform::new() {
            Ok(p) => f(Box::new(p)),
            Err(e) => {
                eprintln!("fastuse: {e}");
                1
            }
        };
        std::process::exit(code);
    });
    app.run();
    0
}

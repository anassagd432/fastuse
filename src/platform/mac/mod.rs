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
}

impl MacPlatform {
    pub fn new() -> Result<Self, String> {
        crate::overlay::init(overlay::MacSurface::new);
        Ok(Self { elements: Vec::new(), meta: Vec::new() })
    }
}

fn todo(what: &str, milestone: &str) -> String {
    format!("{what} is not implemented on macOS yet (planned for {milestone}, see docs/MACOS.md)")
}

impl Platform for MacPlatform {
    fn name(&self) -> &'static str {
        "macOS"
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            background_input: true,
            foreground_input: false,
            window_capture: false,
            desktop_capture: false,
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
        let el = self.elements.get(i).ok_or_else(|| format!("no element [{i}] in snapshot"))?;
        ax::press(el).map(|s| s.to_string())
    }

    fn set_value(&mut self, i: usize, value: &str) -> Result<(), String> {
        let el = self.elements.get(i).ok_or_else(|| format!("no element [{i}] in snapshot"))?;
        ax::set_value(el, value)
    }

    fn type_text(&mut self, _w: Option<&WindowInfo>, _el: Option<usize>, _text: &str, _foreground: bool) -> Result<String, String> {
        Err(todo("type_text", "M2") + ". Use set_value for text fields.")
    }

    fn press_key(&mut self, _w: &WindowInfo, _key: &str) -> Result<String, String> {
        Err(todo("press_key", "M2"))
    }

    fn hotkey(&mut self, _w: &WindowInfo, _keys: &str) -> Result<String, String> {
        Err(todo("hotkey", "M2"))
    }

    fn click_point(&mut self, _w: &WindowInfo, _x: i32, _y: i32, _kind: ClickKind, _foreground: bool) -> Result<String, String> {
        Err(todo("clicking at x,y", "M2") + ". Click by element target instead.")
    }

    fn grab(&mut self, _t: CaptureTarget, _screen_mode: bool) -> Result<Frame, String> {
        Err(todo("screenshots", "M3"))
    }

    fn move_cursor(&self, x: i32, y: i32, click: bool) {
        crate::overlay::go(x, y, click);
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

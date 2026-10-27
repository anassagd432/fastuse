//! UI Automation tree read: one cross-process BuildUpdatedCache call, then walk the local cache.

use std::fmt::Write;

use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED};
use windows::Win32::UI::Accessibility::*;

pub struct Uia {
    auto: IUIAutomation,
    cache: IUIAutomationCacheRequest,
    pub snap: u64,
    /// live elements of the last snapshot, indexed by [N]; used by actions later
    pub elements: Vec<IUIAutomationElement>,
    /// (name, automation id, control type) per entry of `elements`, for resolving `target`
    meta: Vec<(String, String, i32, [i32; 4])>,
}

pub struct Read {
    pub text: String,
    pub count: usize,
    pub truncated: bool,
    pub snapshot_id: String,
    pub build_us: u128,
    pub walk_us: u128,
}

fn type_name(id: i32) -> &'static str {
    match id {
        50000 => "Button",
        50002 => "CheckBox",
        50003 => "ComboBox",
        50004 => "Edit",
        50005 => "Hyperlink",
        50006 => "Image",
        50007 => "ListItem",
        50008 => "List",
        50009 => "Menu",
        50010 => "MenuBar",
        50011 => "MenuItem",
        50013 => "RadioButton",
        50015 => "Slider",
        50018 => "Tab",
        50019 => "TabItem",
        50020 => "Text",
        50021 => "ToolBar",
        50023 => "Tree",
        50024 => "TreeItem",
        50026 => "Group",
        50028 => "Table",
        50030 => "Document",
        50032 => "Window",
        50033 => "Pane",
        50037 => "TitleBar",
        _ => "Element",
    }
}

fn interactive(id: i32) -> bool {
    matches!(
        id,
        50000 | 50002 | 50003 | 50004 | 50005 | 50007 | 50011 | 50013 | 50015 | 50019 | 50024 | 50030
    )
}

impl Uia {
    pub fn new() -> windows::core::Result<Self> {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let auto: IUIAutomation = CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)?;
            let cache = auto.CreateCacheRequest()?;
            for p in [
                UIA_NamePropertyId,
                UIA_ControlTypePropertyId,
                UIA_BoundingRectanglePropertyId,
                UIA_AutomationIdPropertyId,
                UIA_IsEnabledPropertyId,
                UIA_IsOffscreenPropertyId,
                UIA_ValueValuePropertyId,
            ] {
                cache.AddProperty(p)?;
            }
            cache.AddPattern(UIA_ValuePatternId)?;
            cache.SetTreeScope(TreeScope_Subtree)?;
            cache.SetTreeFilter(&auto.ControlViewCondition()?)?;
            Ok(Self { auto, cache, snap: 0, elements: Vec::new(), meta: Vec::new() })
        }
    }

    /// Resolve "sNNNNNNNN:N" against the latest snapshot only; older snapshots are stale.
    pub fn token(&self, tok: &str) -> Result<&IUIAutomationElement, String> {
        let (sid, idx) = tok.split_once(':').ok_or("bad element_token (expected sNNNNNNNN:N)")?;
        if sid != format!("s{:08}", self.snap) {
            return Err(format!("stale element_token {tok}: latest snapshot is s{:08}; call get_window_state again", self.snap));
        }
        let i: usize = idx.parse().map_err(|_| "bad element index")?;
        self.elements.get(i).ok_or_else(|| format!("no element [{i}] in snapshot"))
    }

    /// Screen-space center of element `tok` ("sNNNNNNNN:N") in the latest snapshot, if it has bounds.
    pub fn center_of(&self, tok: &str) -> Option<(i32, i32)> {
        let i: usize = tok.split_once(':')?.1.parse().ok()?;
        let r = self.meta.get(i)?.3;
        if r[2] <= r[0] || r[3] <= r[1] {
            return None;
        }
        Some(((r[0] + r[2]) / 2, (r[1] + r[3]) / 2))
    }

    /// Resolve a human target ("Go", "Button:Go", an automation id) to an index of the latest snapshot.
    pub fn resolve_target(&self, target: &str) -> Result<usize, String> {
        if self.meta.is_empty() {
            return Err("no snapshot yet; the window has no interactive elements or was not read".into());
        }
        let (role, want) = match target.split_once(':') {
            Some((r, n)) if self.meta.iter().any(|m| type_name(m.2).eq_ignore_ascii_case(r)) => (Some(r), n),
            _ => (None, target),
        };
        let w = want.trim().to_lowercase();
        let ok_role = |i: usize| role.map_or(true, |r| type_name(self.meta[i].2).eq_ignore_ascii_case(r));
        let exact: Vec<usize> = (0..self.meta.len())
            .filter(|&i| ok_role(i) && (self.meta[i].0.to_lowercase() == w || self.meta[i].1.to_lowercase() == w))
            .collect();
        let hits = if exact.is_empty() {
            (0..self.meta.len())
                .filter(|&i| ok_role(i) && (self.meta[i].0.to_lowercase().contains(&w) || self.meta[i].1.to_lowercase().contains(&w)))
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
                    .map(|&i| format!("[{i}] {} \"{}\"", type_name(self.meta[i].2), self.meta[i].0))
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }
    }

    pub fn read(&mut self, hwnd: isize, max_elements: usize, max_depth: usize) -> Result<Read, String> {
        use std::time::Instant;
        unsafe {
            let t0 = Instant::now();
            let root = self
                .auto
                .ElementFromHandleBuildCache(HWND(hwnd as *mut _), &self.cache)
                .map_err(|e| e.to_string())?;
            let build_us = t0.elapsed().as_micros();
            let t1 = Instant::now();
            self.snap += 1;
            let sid = format!("s{:08}", self.snap);
            self.elements.clear();
            self.meta.clear();
            let mut out = String::with_capacity(4096);
            let mut count = 0usize;
            let mut truncated = false;
            self.walk(&root, 0, max_depth, max_elements, &mut count, &mut truncated, &mut out);
            let _ = writeln!(out);
            Ok(Read {
                text: out,
                count,
                truncated,
                snapshot_id: sid,
                build_us,
                walk_us: t1.elapsed().as_micros(),
            })
        }
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn walk(
        &mut self,
        el: &IUIAutomationElement,
        depth: usize,
        max_depth: usize,
        max: usize,
        count: &mut usize,
        truncated: &mut bool,
        out: &mut String,
    ) {
        if *count >= max {
            *truncated = true;
            return;
        }
        *count += 1;
        let ct = el.CachedControlType().map(|c| c.0).unwrap_or(0);
        let name = el.CachedName().map(|b| b.to_string()).unwrap_or_default();
        let aid = el.CachedAutomationId().map(|b| b.to_string()).unwrap_or_default();
        let indent = "  ".repeat(depth);
        let _ = write!(out, "{indent}- ");
        if interactive(ct) {
            let idx = self.elements.len();
            self.elements.push(el.clone());
            let rect = el
                .CachedBoundingRectangle()
                .map(|r| [r.left, r.top, r.right, r.bottom])
                .unwrap_or([0; 4]);
            self.meta.push((name.clone(), aid.clone(), ct, rect));
            let _ = write!(out, "[{idx}] ");
        }
        let _ = write!(out, "{}", type_name(ct));
        if !name.is_empty() {
            let _ = write!(out, " \"{}\"", name.replace('"', "'").chars().take(80).collect::<String>());
        }
        if ct == 50004 || ct == 50030 {
            if let Ok(p) = el.GetCachedPatternAs::<IUIAutomationValuePattern>(UIA_ValuePatternId) {
                if let Ok(v) = p.CachedValue() {
                    let v = v.to_string();
                    if !v.is_empty() {
                        let _ = write!(out, " value=\"{}\"", v.replace('"', "'").chars().take(120).collect::<String>());
                    }
                }
            }
        }
        if !aid.is_empty() {
            let _ = write!(out, " id={aid}");
        }
        if let Ok(r) = el.CachedBoundingRectangle() {
            if interactive(ct) {
                let _ = write!(out, " @({},{},{},{})", r.left, r.top, r.right - r.left, r.bottom - r.top);
            }
        }
        let _ = writeln!(out);
        if depth + 1 >= max_depth {
            return;
        }
        if let Ok(kids) = el.GetCachedChildren() {
            let n = kids.Length().unwrap_or(0);
            for i in 0..n {
                if let Ok(k) = kids.GetElement(i) {
                    self.walk(&k, depth + 1, max_depth, max, count, truncated, out);
                }
            }
        }
    }
}

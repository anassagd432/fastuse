//! UI Automation tree read: one cross-process BuildUpdatedCache call, then walk the local cache.

use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED};
use windows::Win32::UI::Accessibility::*;

use crate::platform::{center_in, resolve_in, tree_row, ElementMeta, Rect, TreeRead};

pub struct Uia {
    auto: IUIAutomation,
    cache: IUIAutomationCacheRequest,
    /// live interactive elements of the last read, indexed by [N]
    elements: Vec<IUIAutomationElement>,
    /// (name, automation id, control type, bounds) per entry of `elements`, for resolving `target`
    meta: Vec<ElementMeta>,
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
            Ok(Self { auto, cache, elements: Vec::new(), meta: Vec::new() })
        }
    }

    pub fn element(&self, i: usize) -> Option<&IUIAutomationElement> {
        self.elements.get(i)
    }

    pub fn element_count(&self) -> usize {
        self.elements.len()
    }

    /// Screen-space center of element `i`, if it has bounds.
    pub fn center_of(&self, i: usize) -> Option<(i32, i32)> {
        center_in(&self.meta, i)
    }

    /// Resolve a human target ("Go", "Button:Go", an automation id) to an index of the latest read.
    pub fn resolve_target(&self, target: &str) -> Result<usize, String> {
        resolve_in(&self.meta, target)
    }

    pub fn read(&mut self, hwnd: isize, max_elements: usize, max_depth: usize) -> Result<TreeRead, String> {
        use std::time::Instant;
        unsafe {
            let t0 = Instant::now();
            let root = self
                .auto
                .ElementFromHandleBuildCache(HWND(hwnd as *mut _), &self.cache)
                .map_err(|e| e.to_string())?;
            let build_us = t0.elapsed().as_micros();
            let t1 = Instant::now();
            self.elements.clear();
            self.meta.clear();
            let mut out = String::with_capacity(4096);
            let mut count = 0usize;
            let mut truncated = false;
            self.walk(&root, 0, max_depth, max_elements, &mut count, &mut truncated, &mut out);
            out.push('\n');
            Ok(TreeRead { text: out, count, truncated, build_us, walk_us: t1.elapsed().as_micros() })
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
        let ct = el.CachedControlType().map(|c| c.0).unwrap_or(0);
        let name = el.CachedName().map(|b| b.to_string()).unwrap_or_default();
        let aid = el.CachedAutomationId().map(|b| b.to_string()).unwrap_or_default();
        let offscreen = el.CachedIsOffscreen().map(|b| b.as_bool()).unwrap_or(false);
        let rect = el
            .CachedBoundingRectangle()
            .ok()
            .map(|r| Rect { left: r.left, top: r.top, right: r.right, bottom: r.bottom });
        let is_int = interactive(ct);
        let mut value = String::new();
        if ct == 50004 || ct == 50030 {
            if let Ok(p) = el.GetCachedPatternAs::<IUIAutomationValuePattern>(UIA_ValuePatternId) {
                if let Ok(v) = p.CachedValue() {
                    value = v.to_string();
                }
            }
        }
        let kids = el.GetCachedChildren().ok();
        let n_kids = kids.as_ref().and_then(|k| k.Length().ok()).unwrap_or(0);

        // Skip offscreen leaf nodes so hidden items do not exhaust the element budget or shadow visible controls
        if offscreen && n_kids == 0 && depth > 0 {
            return;
        }
        // Collapse unnamed structural wrappers (Pane, Group, Custom, generic Element) so deep DOM/WPF trees stay compact
        let is_wrapper = matches!(ct, 50025 | 50026 | 50031 | 50033 | 0) || type_name(ct) == "Element" || type_name(ct) == "Custom";
        if depth > 0 && !is_int && is_wrapper && name.is_empty() && value.is_empty() && aid.is_empty() {
            if let Some(kids) = kids {
                for i in 0..n_kids {
                    if let Ok(k) = kids.GetElement(i) {
                        self.walk(&k, depth, max_depth, max, count, truncated, out);
                    }
                }
            }
            return;
        }

        *count += 1;
        let mut idx = None;
        if is_int && !offscreen {
            let i = self.elements.len();
            self.elements.push(el.clone());
            let b = rect.unwrap_or_default();
            self.meta.push(ElementMeta { name: name.clone(), id: aid.clone(), role: type_name(ct), rect: [b.left, b.top, b.right, b.bottom] });
            idx = Some(i);
        }
        tree_row(out, depth, idx, type_name(ct), &name, &value, &aid, if idx.is_some() { rect } else { None });
        if depth + 1 >= max_depth {
            return;
        }
        if let Some(kids) = kids {
            for i in 0..n_kids {
                if let Ok(k) = kids.GetElement(i) {
                    self.walk(&k, depth + 1, max_depth, max, count, truncated, out);
                }
            }
        }
    }
}

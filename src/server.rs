//! Tool logic, independent of the operating system. Talks to the OS only through `Platform`.

use std::time::Instant;

use base64::Engine;
use serde_json::{json, Value};

use crate::imaging::{self, Cache};
use crate::platform::{CaptureTarget, ClickKind, Platform, TreeRead, WindowInfo, WindowQuery};

pub struct Server {
    p: Box<dyn Platform>,
    shots: Cache,
    /// Bumped on every tree read; element tokens are "s<snap>:<index>" and only the latest is valid.
    snap: u64,
}

fn text(s: String) -> Value {
    json!({"content":[{"type":"text","text":s}]})
}

fn err(s: String) -> Value {
    json!({"content":[{"type":"text","text":s}],"isError":true})
}

fn arg_i(a: &Value, k: &str) -> Option<i64> {
    a.get(k).and_then(|v| v.as_i64())
}

fn arg_s<'a>(a: &'a Value, k: &str) -> Option<&'a str> {
    a.get(k).and_then(|v| v.as_str())
}

fn shot_opts(a: &Value) -> imaging::Opts {
    imaging::Opts {
        region: a.get("region").and_then(|r| r.as_array()).and_then(|r| {
            let g = |i: usize| r.get(i).and_then(|v| v.as_u64()).map(|v| v as usize);
            Some([g(0)?, g(1)?, g(2)?, g(3)?])
        }),
        max_dim: arg_i(a, "max_dim").unwrap_or(1280) as usize,
        quality: arg_i(a, "quality").unwrap_or(70) as u8,
        max_age_ms: arg_i(a, "max_age_ms").unwrap_or(0) as u128,
        screen: arg_s(a, "mode") == Some("screen"),
    }
}

fn image(o: &imaging::Out) -> Value {
    json!({"type":"image","data":base64::engine::general_purpose::STANDARD.encode(&o.jpeg),"mimeType":"image/jpeg"})
}

impl Server {
    pub fn new(p: Box<dyn Platform>) -> Self {
        Self { p, shots: Cache::default(), snap: 0 }
    }

    pub fn platform_name(&self) -> &'static str {
        self.p.name()
    }

    fn resolve(&self, a: &Value) -> Result<WindowInfo, String> {
        let q = WindowQuery {
            title: arg_s(a, "window").map(String::from),
            id: arg_i(a, "window_id"),
            pid: arg_i(a, "pid").map(|p| p as u32),
        };
        self.p.find_window(&q).ok_or_else(|| match &q.title {
            Some(t) => format!("no window title contains '{}'", t.to_lowercase()),
            None => "window not found (pass window or window_id from list_windows)".to_string(),
        })
    }

    /// Read the tree and start a new snapshot.
    fn read(&mut self, w: &WindowInfo, max_elements: usize, max_depth: usize) -> Result<(TreeRead, String), String> {
        let r = self.p.read_tree(w, max_elements, max_depth)?;
        self.snap += 1;
        Ok((r, format!("s{:08}", self.snap)))
    }

    /// "sNNNNNNNN:N" -> element index, only for the latest snapshot.
    fn tok_index(&self, tok: &str) -> Result<usize, String> {
        let (sid, idx) = tok.split_once(':').ok_or("bad element_token (expected sNNNNNNNN:N)")?;
        if sid != format!("s{:08}", self.snap) {
            return Err(format!("stale element_token {tok}: latest snapshot is s{:08}; call get_window_state again", self.snap));
        }
        let i: usize = idx.parse().map_err(|_| "bad element index")?;
        if i >= self.p.element_count() {
            return Err(format!("no element [{i}] in snapshot"));
        }
        Ok(i)
    }

    fn state_text(&mut self, w: &WindowInfo, a: &Value) -> Result<String, String> {
        let max_el = arg_i(a, "max_elements").unwrap_or(250) as usize;
        let max_depth = arg_i(a, "max_depth").unwrap_or(25) as usize;
        let (r, sid) = self.read(w, max_el, max_depth)?;
        Ok(format!(
            "window_id={} pid={} elements={} snapshot_id={}{}\ntiming_ms: uia_build={:.1} walk={:.1}\nelement_token for row [N] = {}:N\n\n{}",
            w.id,
            w.pid,
            r.count,
            sid,
            if r.truncated { " truncated=true" } else { "" },
            r.build_us as f64 / 1000.0,
            r.walk_us as f64 / 1000.0,
            sid,
            r.text
        ))
    }

    /// Action tools. Returns a one-line description of what happened.
    fn act(&mut self, name: &str, a: &Value) -> Result<String, String> {
        let fg = arg_s(a, "delivery_mode") == Some("foreground");
        let caps = self.p.capabilities();
        if (fg || name == "hotkey") && !caps.foreground_input {
            return Err(format!("foreground input is not available on {} in this session", self.p.name()));
        }
        if !fg && name != "hotkey" && name != "set_value" && !caps.background_input {
            return Err("background input is not available here; retry with delivery_mode:\"foreground\"".into());
        }
        let patched;
        let a = match (arg_s(a, "target"), arg_s(a, "element_token")) {
            (Some(t), None) => {
                let i = self.p.resolve_target(t)?;
                let mut v = a.clone();
                v["element_token"] = json!(format!("s{:08}:{}", self.snap, i));
                patched = v;
                &patched
            }
            _ => a,
        };
        match name {
            "click" | "double_click" | "right_click" => {
                if let Some(tok) = arg_s(a, "element_token") {
                    let i = self.tok_index(tok)?;
                    if let Some((cx, cy)) = self.p.element_center(i) {
                        self.p.move_cursor(cx, cy, true);
                    }
                    let how = self.p.click_element(i)?;
                    return Ok(format!("{name} {tok} via {how}"));
                }
                let (Some(x), Some(y)) = (arg_i(a, "x"), arg_i(a, "y")) else {
                    return Err("pass element_token, or x and y (window-local pixels)".into());
                };
                let w = self.resolve(a)?;
                self.p.move_cursor(w.rect.left + x as i32, w.rect.top + y as i32, true);
                let kind = match name {
                    "double_click" => ClickKind::Double,
                    "right_click" => ClickKind::Right,
                    _ => ClickKind::Single,
                };
                let how = self.p.click_point(&w, x as i32, y as i32, kind, fg)?;
                Ok(format!("{name} ({x},{y}) via {how}"))
            }
            "set_value" => {
                let tok = arg_s(a, "element_token").ok_or("set_value needs element_token")?;
                let v = arg_s(a, "value").ok_or("set_value needs value")?;
                let i = self.tok_index(tok)?;
                if let Some((cx, cy)) = self.p.element_center(i) {
                    self.p.move_cursor(cx, cy, false);
                }
                self.p.set_value(i, v)?;
                Ok(format!("set_value {tok} ({} chars)", v.chars().count()))
            }
            "type_text" => {
                let t = arg_s(a, "text").ok_or("type_text needs text")?;
                let el = match arg_s(a, "element_token") {
                    Some(tok) => {
                        let i = self.tok_index(tok)?;
                        if let Some((cx, cy)) = self.p.element_center(i) {
                            self.p.move_cursor(cx, cy, false);
                        }
                        Some(i)
                    }
                    None => None,
                };
                let w = self.resolve(a).ok();
                self.p.type_text(w.as_ref(), el, t, fg)
            }
            "press_key" => {
                let k = arg_s(a, "key").ok_or("press_key needs key")?;
                let w = self.resolve(a)?;
                self.p.press_key(&w, k)
            }
            "hotkey" => {
                let k = arg_s(a, "keys").ok_or("hotkey needs keys, e.g. \"ctrl+a\"")?;
                let w = self.resolve(a)?;
                self.p.hotkey(&w, k)
            }
            _ => Err(format!("unknown tool {name}")),
        }
    }

    fn run_steps(&mut self, a: &Value) -> Result<String, String> {
        let steps = a.get("steps").and_then(|s| s.as_array()).ok_or("steps is required")?;
        let t0 = Instant::now();
        let tw = self.resolve(a)?;
        if steps.iter().any(|s| s.get("target").is_some()) {
            self.read(&tw, 250, 25)?;
        }
        let mut log = String::new();
        let mut failed = false;
        for (i, s) in steps.iter().enumerate() {
            let tool = arg_s(s, "tool").ok_or_else(|| format!("step {} has no tool", i + 1))?;
            let mut args = s.clone();
            if let Some(o) = args.as_object_mut() {
                o.remove("tool");
                if let Some(Value::Object(inner)) = o.remove("args") {
                    o.extend(inner);
                }
            }
            args["window_id"] = json!(tw.id);
            match self.act(tool, &args) {
                Ok(m) => log += &format!("{}. ok: {m}\n", i + 1),
                Err(e) => {
                    log += &format!("{}. FAILED {tool}: {e}\n", i + 1);
                    failed = true;
                    break;
                }
            }
        }
        let steps_ms = t0.elapsed().as_secs_f64() * 1000.0;
        let mut out = format!("run_steps: {} step(s), {steps_ms:.1} ms{}\n{log}", steps.len(), if failed { " (stopped at failure)" } else { "" });
        if !matches!(a.get("observe"), Some(Value::Bool(false))) {
            let oa = match a.get("observe") {
                Some(o) if o.is_object() => o.clone(),
                _ => json!({}),
            };
            out += "\n";
            out += &self.state_text(&tw, &oa)?;
        }
        if failed {
            Err(out)
        } else {
            Ok(out)
        }
    }

    fn capture(&mut self, target: CaptureTarget, a: &Value) -> Result<imaging::Out, String> {
        let caps = self.p.capabilities();
        match target {
            CaptureTarget::Desktop if !caps.desktop_capture => {
                return Err(format!("desktop capture is not available on {} in this session", self.p.name()))
            }
            CaptureTarget::Window(_) if !caps.window_capture => {
                return Err(format!("window capture is not available on {} in this session", self.p.name()))
            }
            _ => {}
        }
        let opts = shot_opts(a);
        let id = match target {
            CaptureTarget::Desktop => 0,
            CaptureTarget::Window(id) => id,
        };
        let screen = opts.screen;
        let p = &mut self.p;
        self.shots.capture(id, &opts, &mut || p.grab(target, screen))
    }

    pub fn call(&mut self, name: &str, a: &Value) -> Value {
        match name {
            "list_windows" => {
                let with_exe = a.get("exe").and_then(|v| v.as_bool()).unwrap_or(false);
                let ws = self.p.list_windows();
                let mut s = format!("Found {} window(s)\n", ws.len());
                let c = self.p.capabilities();
                let missing: Vec<&str> = [
                    ("background input", c.background_input),
                    ("foreground input", c.foreground_input),
                    ("window screenshots", c.window_capture),
                    ("desktop screenshots", c.desktop_capture),
                    ("agent cursor", c.overlay_cursor),
                ]
                .iter()
                .filter(|(_, ok)| !ok)
                .map(|(n, _)| *n)
                .collect();
                if !missing.is_empty() {
                    s += &format!("Not available on {} here: {}\n", self.p.name(), missing.join(", "));
                }
                for w in ws {
                    let exe = if with_exe { format!("{} ", self.p.exe_name(w.pid)) } else { String::new() };
                    s += &format!(
                        "- {exe}(pid {}) \"{}\" [window_id: {}] {}x{} @({},{}){}\n",
                        w.pid,
                        w.title,
                        w.id,
                        w.rect.w(),
                        w.rect.h(),
                        w.rect.left,
                        w.rect.top,
                        if w.minimized { " [minimized]" } else { "" }
                    );
                }
                text(s)
            }
            "get_window_state" => {
                let w = match self.resolve(a) {
                    Ok(w) => w,
                    Err(e) => return err(e),
                };
                let t = match self.state_text(&w, a) {
                    Ok(t) => t,
                    Err(e) => return err(e),
                };
                let mut content = vec![json!({"type":"text","text": t})];
                if a.get("screenshot").and_then(|v| v.as_bool()).unwrap_or(false) {
                    match self.capture(CaptureTarget::Window(w.id), a) {
                        Ok(o) => content.push(image(&o)),
                        Err(e) => content.push(json!({"type":"text","text":format!("screenshot error: {e}")})),
                    }
                }
                json!({"content": content})
            }
            "screenshot" => {
                let desktop = a.get("screen").and_then(|v| v.as_bool()).unwrap_or(false)
                    || (a.get("window").is_none() && a.get("window_id").is_none() && a.get("pid").is_none());
                let target = if desktop {
                    CaptureTarget::Desktop
                } else {
                    match self.resolve(a) {
                        Ok(w) => CaptureTarget::Window(w.id),
                        Err(e) => return err(e),
                    }
                };
                match self.capture(target, a) {
                    Ok(o) => json!({"content":[
                        {"type":"text","text":format!("{}x{} jpeg={}B cached={} capture_ms={:.1} encode_ms={:.1}",
                            o.w, o.h, o.jpeg.len(), o.from_cache, o.capture_us as f64/1000.0, o.encode_us as f64/1000.0)},
                        image(&o)]}),
                    Err(e) => err(e),
                }
            }
            "act" | "run_steps" => match self.run_steps(a) {
                Ok(s) => text(s),
                Err(e) => err(e),
            },
            other => match self.act(other, a) {
                Ok(s) => text(s),
                Err(e) => err(e),
            },
        }
    }
}

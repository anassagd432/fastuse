mod actions;
mod cursor;
mod runner;
mod shot;
mod uia;
mod win;

use std::io::{BufRead, Write};
use std::time::Instant;

use base64::Engine;
use serde_json::{json, Value};

struct Server {
    uia: uia::Uia,
    shots: shot::Cache,
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

fn resolve(a: &Value) -> Result<win::Win, String> {
    if let Some(t) = arg_s(a, "window") {
        let t = t.to_lowercase();
        let mut m: Vec<win::Win> = win::list().into_iter().filter(|w| w.title.to_lowercase().contains(&t)).collect();
        m.sort_by_key(|w| w.minimized);
        return m.into_iter().next().ok_or_else(|| format!("no window title contains '{t}'"));
    }
    win::find(arg_i(a, "window_id"), arg_i(a, "pid").map(|p| p as u32))
        .ok_or_else(|| "window not found (pass window_id or pid from list_windows)".to_string())
}

fn shot_opts(a: &Value) -> shot::Opts {
    shot::Opts {
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

fn image(o: &shot::Out) -> Value {
    json!({"type":"image","data":base64::engine::general_purpose::STANDARD.encode(&o.jpeg),"mimeType":"image/jpeg"})
}

impl Server {
    fn state_text(&mut self, w: &win::Win, a: &Value) -> Result<String, String> {
        let max_el = arg_i(a, "max_elements").unwrap_or(250) as usize;
        let max_depth = arg_i(a, "max_depth").unwrap_or(25) as usize;
        let r = self.uia.read(w.hwnd, max_el, max_depth)?;
        Ok(format!(
            "window_id={} pid={} elements={} snapshot_id={}{}\ntiming_ms: uia_build={:.1} walk={:.1}\nelement_token for row [N] = {}:N\n\n{}",
            w.hwnd,
            w.pid,
            r.count,
            r.snapshot_id,
            if r.truncated { " truncated=true" } else { "" },
            r.build_us as f64 / 1000.0,
            r.walk_us as f64 / 1000.0,
            r.snapshot_id,
            r.text
        ))
    }

    /// Action tools. Returns a one-line description of what happened.
    fn act(&mut self, name: &str, a: &Value) -> Result<String, String> {
        let fg = arg_s(a, "delivery_mode") == Some("foreground");
        let patched;
        let a = match (arg_s(a, "target"), arg_s(a, "element_token")) {
            (Some(t), None) => {
                let i = self.uia.resolve_target(t)?;
                let mut v = a.clone();
                v["element_token"] = json!(format!("s{:08}:{}", self.uia.snap, i));
                patched = v;
                &patched
            }
            _ => a,
        };
        match name {
            "click" | "double_click" | "right_click" => {
                if let Some(tok) = arg_s(a, "element_token") {
                    let el = self.uia.token(tok)?;
                    if let Some((cx, cy)) = self.uia.center_of(tok) {
                        cursor::go(cx, cy, true);
                    }
                    let how = actions::invoke(el)?;
                    return Ok(format!("{name} {tok} via {how}"));
                }
                let (Some(x), Some(y)) = (arg_i(a, "x"), arg_i(a, "y")) else {
                    return Err("pass element_token, or x and y (window-local pixels)".into());
                };
                let w = resolve(a)?;
                cursor::go(w.rect.left + x as i32, w.rect.top + y as i32, true);
                let how = actions::click_xy(
                    w.hwnd, w.rect.left, w.rect.top, x as i32, y as i32,
                    name == "double_click", name == "right_click", fg,
                )?;
                Ok(format!("{name} ({x},{y}) via {how}"))
            }
            "set_value" => {
                let tok = arg_s(a, "element_token").ok_or("set_value needs element_token")?;
                let v = arg_s(a, "value").ok_or("set_value needs value")?;
                if let Some((cx, cy)) = self.uia.center_of(tok) {
                    cursor::go(cx, cy, false);
                }
                actions::set_value(self.uia.token(tok)?, v)?;
                Ok(format!("set_value {tok} ({} chars)", v.chars().count()))
            }
            "type_text" => {
                let t = arg_s(a, "text").ok_or("type_text needs text")?;
                if fg {
                    let w = resolve(a)?;
                    if !actions::raise(w.hwnd) {
                        return Err("could not bring the window to the foreground; text not sent".into());
                    }
                    let n = actions::send_text_foreground(t);
                    return Ok(format!("type_text {n} chars via SendInput (foreground)"));
                }
                let target = match arg_s(a, "element_token") {
                    Some(tok) => {
                        if let Some((cx, cy)) = self.uia.center_of(tok) {
                            cursor::go(cx, cy, false);
                        }
                        actions::element_hwnd(self.uia.token(tok)?)
                    }
                    None => None,
                };
                let target = match target {
                    Some(t) => t,
                    None => actions::focus_target(resolve(a)?.hwnd),
                };
                let n = actions::post_text(target, t);
                Ok(format!("type_text {n} chars via WM_CHAR (background)"))
            }
            "press_key" => {
                let k = arg_s(a, "key").ok_or("press_key needs key")?;
                let vk = actions::vk(k).ok_or_else(|| format!("unknown key '{k}'"))?;
                let target = actions::focus_target(resolve(a)?.hwnd);
                actions::post_key(target, vk);
                Ok(format!("press_key {k} via WM_KEYDOWN/UP (background, no modifiers)"))
            }
            "hotkey" => {
                let k = arg_s(a, "keys").ok_or("hotkey needs keys, e.g. \"ctrl+a\"")?;
                actions::hotkey(resolve(a)?.hwnd, k)?;
                Ok(format!("hotkey {k} via SendInput (foreground)"))
            }
            _ => Err(format!("unknown tool {name}")),
        }
    }

    fn run_steps(&mut self, a: &Value) -> Result<String, String> {
        let steps = a.get("steps").and_then(|s| s.as_array()).ok_or("steps is required")?;
        let t0 = Instant::now();
        let tw = resolve(a)?;
        if steps.iter().any(|s| s.get("target").is_some()) {
            self.uia.read(tw.hwnd, 250, 25)?;
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
            args["window_id"] = json!(tw.hwnd as i64);
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

    fn call(&mut self, name: &str, a: &Value) -> Value {
        match name {
            "list_windows" => {
                let with_exe = a.get("exe").and_then(|v| v.as_bool()).unwrap_or(false);
                let ws = win::list();
                let mut s = format!("Found {} window(s)\n", ws.len());
                for w in ws {
                    let exe = if with_exe { format!("{} ", win::exe_name(w.pid)) } else { String::new() };
                    s += &format!(
                        "- {exe}(pid {}) \"{}\" [window_id: {}] {}x{} @({},{}){}\n",
                        w.pid,
                        w.title,
                        w.hwnd,
                        w.rect.right - w.rect.left,
                        w.rect.bottom - w.rect.top,
                        w.rect.left,
                        w.rect.top,
                        if w.minimized { " [minimized]" } else { "" }
                    );
                }
                text(s)
            }
            "get_window_state" => {
                let w = match resolve(a) {
                    Ok(w) => w,
                    Err(e) => return err(e),
                };
                let t = match self.state_text(&w, a) {
                    Ok(t) => t,
                    Err(e) => return err(e),
                };
                let mut content = vec![json!({"type":"text","text": t})];
                if a.get("screenshot").and_then(|v| v.as_bool()).unwrap_or(false) {
                    match self.shots.capture(w.hwnd, &shot_opts(a)) {
                        Ok(o) => content.push(image(&o)),
                        Err(e) => content.push(json!({"type":"text","text":format!("screenshot error: {e}")})),
                    }
                }
                json!({"content": content})
            }
            "screenshot" => {
                let desktop = a.get("screen").and_then(|v| v.as_bool()).unwrap_or(false)
                    || (a.get("window").is_none() && a.get("window_id").is_none() && a.get("pid").is_none());
                let hwnd = if desktop {
                    0
                } else {
                    match resolve(a) {
                        Ok(w) => w.hwnd,
                        Err(e) => return err(e),
                    }
                };
                match self.shots.capture(hwnd, &shot_opts(a)) {
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

fn tools() -> Value {
    let tool = |n: &str, d: &str, p: Value| json!({"name":n,"description":d,"inputSchema":{"type":"object","properties":p}});
    let win = json!({"type":"string","description":"part of the window title"});
    json!({"tools":[
        tool("act",
            "Do a whole UI task in ONE call: find the window by title, run the steps in order, and return the resulting UI state. Prefer this over any other tool. Each step: {tool, target, ...}. tool is one of click, double_click, right_click, set_value (value), type_text (text), press_key (key), hotkey (keys). target is an element name or id from the UI state, e.g. \"Go\" or \"Button:Go\"; or use x,y window pixels. Stops at the first failure.",
            json!({"window":win,"window_id":{"type":"integer"},
                   "steps":{"type":"array","items":{"type":"object","properties":{
                       "tool":{"type":"string","enum":["click","double_click","right_click","set_value","type_text","press_key","hotkey"]},
                       "target":{"type":"string"},"value":{"type":"string"},"text":{"type":"string"},
                       "key":{"type":"string"},"keys":{"type":"string"},"x":{"type":"integer"},"y":{"type":"integer"},
                       "delivery_mode":{"type":"string","enum":["background","foreground"]}},"required":["tool"]}},
                   "observe":{"type":"boolean","description":"return the UI state after the steps (default true)"}})),
        tool("get_window_state","Read a window's UI tree as text. Only needed to explore; act already returns the new state.",
            json!({"window":win,"window_id":{"type":"integer"},"max_elements":{"type":"integer"},"screenshot":{"type":"boolean"}})),
        tool("list_windows","List visible windows: title, pid, window_id.", json!({})),
        tool("screenshot","Screenshot as JPEG: a window if window/window_id is given, otherwise the whole desktop (all monitors). region is [x,y,w,h] in that image's pixels.",
            json!({"window":win,"window_id":{"type":"integer"},"screen":{"type":"boolean","description":"force whole-desktop capture"},"region":{"type":"array","items":{"type":"integer"},"description":"[x,y,w,h]"},
                   "max_dim":{"type":"integer"},"max_age_ms":{"type":"integer","description":"reuse a frame younger than this"}}))
    ]})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tools_are_well_formed() {
        let t = tools();
        let names: Vec<&str> = t["tools"].as_array().unwrap().iter().map(|x| x["name"].as_str().unwrap()).collect();
        assert!(names.contains(&"act"));
        for tool in t["tools"].as_array().unwrap() {
            assert!(tool["description"].as_str().map_or(false, |d| !d.is_empty()));
            assert_eq!(tool["inputSchema"]["type"], "object");
        }
    }

    #[test]
    fn keys_map() {
        assert!(actions::vk("ctrl").is_some());
        assert_eq!(actions::vk("a"), actions::vk("A"));
        assert!(actions::vk("f5").is_some());
        assert!(actions::vk("f25").is_none());
        assert!(actions::vk("nonsense").is_none());
    }
}

fn main() {
    if std::env::args().nth(1).as_deref() == Some("--version") {
        println!("fastuse {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    unsafe {
        let _ = windows::Win32::UI::HiDpi::SetProcessDpiAwarenessContext(
            windows::Win32::UI::HiDpi::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        );
    }
    cursor::init();
    let mut srv = Server { uia: uia::Uia::new().expect("UI Automation init failed"), shots: Default::default() };
    let argv: Vec<String> = std::env::args().collect();
    if argv.get(1).map(|s| s.as_str()) == Some("run") {
        std::process::exit(runner::run(&mut srv, &argv[2..]));
    }
    let stdin = std::io::stdin();
    let mut out = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let Ok(req): Result<Value, _> = serde_json::from_str(&line) else { continue };
        let Some(id) = req.get("id").cloned() else { continue }; // notifications
        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let params = req.get("params").cloned().unwrap_or(Value::Null);
        let result = match method {
            "initialize" => json!({
                "protocolVersion": params.get("protocolVersion").cloned().unwrap_or(json!("2025-03-26")),
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "fastuse", "version": env!("CARGO_PKG_VERSION")},
                "instructions": "Drives Windows app windows fast and in the background. Call the `act` tool directly; no need to list files or read tool docs first. Example arguments: {\"window\":\"<part of window title>\",\"steps\":[{\"tool\":\"set_value\",\"target\":\"<element name>\",\"value\":\"text\"},{\"tool\":\"click\",\"target\":\"<element name>\"}]}. One call finds the window, runs all steps and returns the new UI tree, so you rarely need get_window_state or list_windows. If a target name is wrong the reply lists the elements. Step tools: click, double_click, right_click, set_value (value), type_text (text), press_key (key), hotkey (keys)."
            }),
            "tools/list" => tools(),
            "tools/call" => {
                let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let args = params.get("arguments").cloned().unwrap_or(json!({}));
                srv.call(name, &args)
            }
            "ping" => json!({}),
            _ => {
                let e = json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":"method not found"}});
                let _ = writeln!(out, "{e}");
                let _ = out.flush();
                continue;
            }
        };
        let _ = writeln!(out, "{}", json!({"jsonrpc":"2.0","id":id,"result":result}));
        let _ = out.flush();
    }
}

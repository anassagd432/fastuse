//! Browser use: lets the agent read and (later) operate the user's own Chrome / Brave / Edge.
//!
//!   agent --MCP--> fastuse --local pipe--> fastuse (native host, started by the browser)
//!                                              --native messaging--> extension --> your tabs
//!
//! The same binary plays both roles. The browser starts it as the native host with the extension
//! origin as its first argument (see `is_host_invocation`); every other run is the MCP server,
//! which finds a running host through the files in `hosts_dir()` and talks to it over the pipe.

mod frame;
mod install;
mod ipc;

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

pub use install::{extension_dir, install, open_browser, uninstall};

pub fn no_browser_err() -> String {
    format!(
        "No connected Chromium browser extension host found.\n\
         - Auto-connect: run `fastuse browser-open` (or `--relaunch` with user confirmation to keep existing tabs/logins).\n\
         - Existing open browser: use desktop `act`/`get_window_state` (`window: \"Brave\"` or `\"Chrome\"` or `\"Edge\"`) with mouse/keyboard directly, OR use `act` inside chrome://extensions / brave://extensions -> Developer mode -> Load unpacked -> \"{}\".",
        extension_dir().display()
    )
}

pub fn data_dir() -> PathBuf {
    #[cfg(windows)]
    {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            return PathBuf::from(local).join("fastuse");
        }
        PathBuf::from(std::env::var("USERPROFILE").unwrap_or_else(|_| ".".into()))
            .join("AppData")
            .join("Local")
            .join("fastuse")
    }
    #[cfg(not(windows))]
    {
        PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into())).join(".fastuse")
    }
}

fn hosts_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![data_dir().join("hosts")];
    #[cfg(windows)]
    if let Ok(up) = std::env::var("USERPROFILE") {
        let p = PathBuf::from(up).join("AppData").join("Local").join("fastuse").join("hosts");
        if !dirs.contains(&p) {
            dirs.push(p);
        }
    }
    dirs
}

fn hosts_dir() -> PathBuf {
    data_dir().join("hosts")
}

/// Browsers start native hosts with the calling extension's origin as the first argument.
pub fn is_host_invocation(args: &[String]) -> bool {
    args.get(1).map_or(false, |a| a.starts_with("chrome-extension://") || a == "--native-host")
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

// ---- the native host (started by the browser) ------------------------------------------------

type Pending = Arc<Mutex<HashMap<u64, mpsc::Sender<Value>>>>;

/// Runs until the browser closes the pipe. Nothing but protocol frames may go to stdout.
pub fn host_main() -> i32 {
    let pid = std::process::id();
    let endpoint = ipc::endpoint_name(pid);
    let listener = match ipc::Listener::bind(&endpoint) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("fastuse host: cannot listen on {endpoint}: {e}");
            return 1;
        }
    };
    let out = Arc::new(Mutex::new(std::io::stdout()));
    let pending: Pending = Arc::new(Mutex::new(HashMap::new()));
    let next = Arc::new(AtomicU64::new(1));

    {
        let (out, pending, next) = (out.clone(), pending.clone(), next.clone());
        std::thread::spawn(move || loop {
            match listener.accept() {
                Ok(stream) => {
                    let (out, pending, next) = (out.clone(), pending.clone(), next.clone());
                    std::thread::spawn(move || serve(stream, out, pending, next));
                }
                Err(_) => std::thread::sleep(Duration::from_millis(200)),
            }
        });
    }

    let info_path = hosts_dir().join(format!("{pid}.json"));
    let mut stdin = std::io::stdin().lock();
    loop {
        match frame::read_frame(&mut stdin) {
            Ok(Some(v)) => {
                if v.get("event").and_then(|e| e.as_str()) == Some("hello") {
                    let info = json!({
                        "pid": pid, "endpoint": endpoint, "started": now_secs(),
                        "browser": v.get("browser"), "extension": v.get("extension"), "extension_id": v.get("id"),
                    });
                    let _ = std::fs::create_dir_all(hosts_dir());
                    let _ = std::fs::write(&info_path, info.to_string());
                } else if let Some(id) = v.get("id").and_then(|i| i.as_u64()) {
                    if let Some(tx) = pending.lock().unwrap().remove(&id) {
                        let _ = tx.send(v);
                    }
                }
            }
            Ok(None) | Err(_) => break,
        }
    }
    let _ = std::fs::remove_file(&info_path);
    0
}

/// One MCP-side client: strict request line, then response line.
fn serve(stream: ipc::Stream, out: Arc<Mutex<std::io::Stdout>>, pending: Pending, next: Arc<AtomicU64>) {
    let mut rd = BufReader::new(&stream);
    let mut line = String::new();
    loop {
        line.clear();
        match rd.read_line(&mut line) {
            Ok(0) | Err(_) => return,
            Ok(_) => {}
        }
        let Ok(req) = serde_json::from_str::<Value>(&line) else { continue };
        let client_id = req.get("id").cloned().unwrap_or(Value::Null);
        let hid = next.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = mpsc::channel();
        pending.lock().unwrap().insert(hid, tx);
        let msg = json!({"id": hid, "method": req.get("method"), "params": req.get("params")});
        let sent = frame::write_frame(&mut *out.lock().unwrap(), &msg);
        let mut reply = match sent {
            Err(e) => json!({"error": format!("could not reach the browser: {e}")}),
            Ok(()) => match rx.recv_timeout(Duration::from_secs(60)) {
                Ok(v) => v,
                Err(_) => {
                    pending.lock().unwrap().remove(&hid);
                    json!({"error": "the browser did not answer within 60 s"})
                }
            },
        };
        reply["id"] = client_id;
        if (&stream).write_all(format!("{reply}\n").as_bytes()).is_err() {
            return;
        }
    }
}

// ---- the MCP side ----------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct HostInfo {
    endpoint: String,
    browser: String,
    started: u64,
    path: PathBuf,
}

/// Running hosts across all candidate host directories, newest first.
fn hosts() -> Vec<HostInfo> {
    let mut seen = std::collections::HashSet::new();
    let mut v: Vec<HostInfo> = Vec::new();
    for dir in hosts_dirs() {
        for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let Some(j) = std::fs::read_to_string(e.path())
                .ok()
                .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            else {
                continue;
            };
            let Some(endpoint) = j.get("endpoint").and_then(|x| x.as_str()).map(String::from) else {
                continue;
            };
            if !seen.insert(endpoint.clone()) {
                continue;
            }
            v.push(HostInfo {
                endpoint,
                browser: j.get("browser").and_then(|b| b.as_str()).unwrap_or("browser").to_string(),
                started: j.get("started").and_then(|s| s.as_u64()).unwrap_or(0),
                path: e.path(),
            });
        }
    }
    v.sort_by(|a, b| b.started.cmp(&a.started));
    v
}

/// Prune stale host files whose IPC endpoint is no longer listening, returning live hosts.
fn live_hosts() -> Vec<HostInfo> {
    let mut live = Vec::new();
    for h in hosts() {
        if ipc::connect(&h.endpoint).is_ok() {
            live.push(h);
        } else {
            let _ = std::fs::remove_file(&h.path);
        }
    }
    live
}

enum CallErr {
    NoHost,
    Msg(String),
}

fn call_raw(method: &str, params: Value, timeout: Duration) -> Result<Value, CallErr> {
    let hs = hosts();
    if hs.is_empty() {
        return Err(CallErr::NoHost);
    }
    for h in hs {
        let stream = match ipc::connect(&h.endpoint) {
            Ok(s) => s,
            Err(_) => {
                let _ = std::fs::remove_file(&h.path);
                continue;
            }
        };
        let req = format!("{}\n", json!({"id": 1, "method": method, "params": params}));
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let r = (|| -> std::io::Result<String> {
                (&stream).write_all(req.as_bytes())?;
                let mut line = String::new();
                BufReader::new(&stream).read_line(&mut line)?;
                Ok(line)
            })();
            let _ = tx.send(r);
        });
        return match rx.recv_timeout(timeout) {
            Ok(Ok(line)) => {
                let v: Value = serde_json::from_str(&line)
                    .map_err(|e| CallErr::Msg(format!("bad reply from the browser host: {e}")))?;
                match v.get("error") {
                    Some(e) => Err(CallErr::Msg(e.as_str().map(String::from).unwrap_or_else(|| e.to_string()))),
                    None => Ok(v.get("result").cloned().unwrap_or(Value::Null)),
                }
            }
            Ok(Err(e)) => Err(CallErr::Msg(format!("browser host connection failed: {e}"))),
            Err(_) => Err(CallErr::Msg("the browser did not answer in time".to_string())),
        };
    }
    Err(CallErr::NoHost)
}

/// Ask the browser extension to run `method`. Auto-launches a connected browser if none is attached.
pub fn call_auto(method: &str, params: Value, timeout: Duration) -> Result<(Value, Option<String>), String> {
    match call_raw(method, params.clone(), timeout) {
        Ok(v) => Ok((v, None)),
        Err(CallErr::Msg(e)) => Err(e),
        Err(CallErr::NoHost) => {
            if cfg!(test) || std::env::var_os("FASTUSE_NO_AUTOLAUNCH").is_some() {
                return Err(no_browser_err());
            }
            let Ok(launch_msg) = open_browser(None, false, false) else {
                return Err(no_browser_err());
            };
            let deadline = Instant::now() + Duration::from_secs(6);
            while Instant::now() < deadline {
                if !live_hosts().is_empty() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(200));
            }
            let note = if launch_msg.contains("isolated fastuse profile") {
                Some(format!(
                    "note: auto-opened an isolated browser because your existing browser has not loaded the extension yet. To act on your existing tabs/sessions directly, use desktop `act`/`get_window_state` on that browser window (with mouse/keyboard), or load `{}` in chrome://extensions / brave://extensions (`Load unpacked`), or run `fastuse browser-open --relaunch`.",
                    extension_dir().display()
                ))
            } else {
                None
            };
            match call_raw(method, params, timeout) {
                Ok(v) => Ok((v, note)),
                Err(CallErr::Msg(e)) => Err(match note {
                    Some(n) => format!("{n}\n{e}"),
                    None => e,
                }),
                Err(CallErr::NoHost) => Err(no_browser_err()),
            }
        }
    }
}

pub fn call(method: &str, params: Value, timeout: Duration) -> Result<Value, String> {
    match call_raw(method, params, timeout) {
        Ok(v) => Ok(v),
        Err(CallErr::Msg(e)) => Err(e),
        Err(CallErr::NoHost) => Err(no_browser_err()),
    }
}

pub struct Browser {
    snap: u64,
    last_trees: HashMap<i64, String>,
}

impl Browser {
    pub fn new() -> Self {
        Self { snap: 0, last_trees: HashMap::new() }
    }

    /// `browser_state`: compact tree of the active/selected tab (plus tabs when helpful).
    pub fn state(&mut self, a: &Value) -> Result<String, String> {
        let mut params = json!({});
        for k in ["tab", "text"] {
            if let Some(v) = a.get(k) {
                params[k] = v.clone();
            }
        }
        if let Some(n) = a.get("max_elements") {
            params["max"] = n.clone();
        }
        let details = a.get("details").and_then(|v| v.as_bool()).unwrap_or(false);
        let diff = a.get("diff").and_then(|v| v.as_bool()).unwrap_or(false);
        let filter = a.get("filter").and_then(|v| v.as_str());
        let (st, auto_note) = call_auto("page.state", params, Duration::from_secs(30))?;
        let tabs = match st.get("tabs") {
            Some(t) if t.is_object() => t.clone(),
            _ => call("tabs.list", json!({}), Duration::from_secs(10))?,
        };
        self.snap += 1;
        let sid = format!("s{:08}", self.snap);
        let prefix = match auto_note {
            Some(n) => format!("{n}\n"),
            None => String::new(),
        };
        if details && filter.is_none() && !diff {
            let tab_id = st["tab"]["id"].as_i64().unwrap_or(0);
            let compact = crate::platform::format_tree_compact(st["page"]["text"].as_str().unwrap_or(""), false, None);
            self.last_trees.insert(tab_id, compact);
            return Ok(format!("{prefix}{}", format_state(&tabs, &st, &sid)));
        }
        let tab_id = st["tab"]["id"].as_i64().unwrap_or(0);
        let raw = st["page"]["text"].as_str().unwrap_or("");
        let compact = crate::platform::format_tree_compact(raw, details, filter);
        let prev = if diff { self.last_trees.get(&tab_id).cloned() } else { None };
        if filter.is_none() {
            self.last_trees.insert(tab_id, compact.clone());
        }
        let body = match prev {
            Some(ref p) => crate::platform::diff_tree(p, &compact),
            None => compact,
        };
        Ok(format!("{prefix}{}", format_state_compact(Some(&tabs), &st, &sid, &body)))
    }

    /// `browser_act`: run steps in the agent's own tabs, then return the compact new page state.
    pub fn act(&mut self, a: &Value) -> Result<String, String> {
        let steps = a.get("steps").and_then(|s| s.as_array()).ok_or("steps is required")?;
        let t0 = Instant::now();
        let mut params = json!({
            "steps": steps,
            "snap": format!("s{:08}", self.snap),
            "snap_next": format!("s{:08}", self.snap + 1),
        });
        for k in ["tab", "observe", "text"] {
            if let Some(v) = a.get(k) {
                params[k] = v.clone();
            }
        }
        if let Some(n) = a.get("max_elements") {
            params["max"] = n.clone();
        }
        let details = a.get("details").and_then(|v| v.as_bool()).unwrap_or(false);
        let diff = a.get("diff").and_then(|v| v.as_bool()).unwrap_or(false);
        let filter = a.get("filter").and_then(|v| v.as_str());
        let has_tab_step = steps.iter().any(|s| {
            matches!(
                s.get("tool").and_then(|v| v.as_str()),
                Some("new_tab" | "switch_tab" | "close_tab")
            )
        });

        let (r, auto_note) = call_auto("act.run", params, Duration::from_secs(120))?;
        let failed = r.get("failed").and_then(|f| f.as_bool()).unwrap_or(false);
        let mut out = format!(
            "browser_act: {} step(s), {:.0} ms{}\n",
            steps.len(),
            t0.elapsed().as_secs_f64() * 1000.0,
            if failed { " (stopped at failure)" } else { "" }
        );
        if let Some(n) = auto_note {
            out += &format!("{n}\n");
        }
        for l in r.get("log").and_then(|l| l.as_array()).into_iter().flatten() {
            out += l.as_str().unwrap_or("");
            out.push('\n');
        }
        if let Some(n) = r.get("note").and_then(|n| n.as_str()) {
            out += &format!("note: {n}\n");
        }
        if let Some(st) = r.get("state") {
            self.snap += 1;
            let sid = format!("s{:08}", self.snap);
            let tabs = if details || has_tab_step {
                match r.get("tabs").or_else(|| st.get("tabs")) {
                    Some(t) if t.is_object() => Some(t.clone()),
                    _ => call("tabs.list", json!({}), Duration::from_secs(10)).ok(),
                }
            } else {
                None
            };
            let st_view = json!({"tab": st["tab"], "page": st["page"], "browser": r["browser"]});
            let tab_id = st["tab"]["id"].as_i64().unwrap_or(0);
            let raw = st["page"]["text"].as_str().unwrap_or("");
            let compact = crate::platform::format_tree_compact(raw, details, filter);
            let prev = if diff { self.last_trees.get(&tab_id).cloned() } else { None };
            if filter.is_none() {
                self.last_trees.insert(tab_id, compact.clone());
            }
            out += "\n";
            if details && filter.is_none() && !diff {
                if let Some(ref t) = tabs {
                    out += &format_state(t, &st_view, &sid);
                } else {
                    out += &format_state_compact(None, &st_view, &sid, &compact);
                }
            } else {
                let body = match prev {
                    Some(ref p) => crate::platform::diff_tree(p, &compact),
                    None => compact,
                };
                out += &format_state_compact(tabs.as_ref(), &st_view, &sid, &body);
            }
        }
        if failed {
            Err(out)
        } else {
            Ok(out)
        }
    }
}

fn format_tabs(tabs: &Value, selected: Option<i64>) -> String {
    let list = tabs.get("tabs").and_then(|t| t.as_array()).cloned().unwrap_or_default();
    let hidden = tabs.get("hidden").and_then(|h| h.as_i64()).unwrap_or(0);
    let mut s = format!("tabs: {} usable by the agent, {} others hidden from it\n", list.len(), hidden);
    for t in list {
        let id = t.get("id").and_then(|i| i.as_i64()).unwrap_or(0);
        s += &format!(
            "  {} tab {id} \"{}\" {}{}\n",
            if Some(id) == selected { "*" } else { "-" },
            t.get("title").and_then(|x| x.as_str()).unwrap_or("").chars().take(80).collect::<String>().replace('"', "'"),
            t.get("url").and_then(|x| x.as_str()).unwrap_or("").chars().take(120).collect::<String>(),
            match (t.get("owned").and_then(|a| a.as_bool()).unwrap_or(false), t.get("active").and_then(|a| a.as_bool()).unwrap_or(false)) {
                (true, _) => " [agent tab]",
                (false, true) => " [shared, active]",
                (false, false) => " [shared]",
            }
        );
    }
    s
}

fn format_state(tabs: &Value, st: &Value, snapshot: &str) -> String {
    let tab = &st["tab"];
    let page = &st["page"];
    let selected = tab.get("id").and_then(|i| i.as_i64());
    let vp = &page["viewport"];
    let mut s = format!(
        "browser: {} (extension {})\n{}",
        st["browser"]["name"].as_str().unwrap_or("?"),
        st["browser"]["extension"].as_str().unwrap_or("?"),
        format_tabs(tabs, selected)
    );
    s += &format!(
        "page: tab={} \"{}\" {}\nviewport={}x{} scroll_y={}/{} elements={} rows={}{} snapshot_id={} read_ms={}\nelement_token for row [N] = {}:N\n\n",
        tab["id"],
        page["title"].as_str().unwrap_or("").chars().take(80).collect::<String>().replace('"', "'"),
        page["url"].as_str().unwrap_or(""),
        vp["w"],
        vp["h"],
        vp["scrollY"],
        vp["pageH"],
        page["count"],
        page["rows"],
        if page["truncated"].as_bool().unwrap_or(false) { " truncated=true" } else { "" },
        snapshot,
        page["ms"],
        snapshot,
    );
    s += page["text"].as_str().unwrap_or("");
    s
}

fn format_state_compact(tabs: Option<&Value>, st: &Value, snapshot: &str, body: &str) -> String {
    let tab = &st["tab"];
    let page = &st["page"];
    let selected = tab.get("id").and_then(|i| i.as_i64());
    let mut s = String::new();
    if let Some(t) = tabs {
        s += &format_tabs(t, selected);
    }
    s += &format!(
        "tab={} \"{}\" {} snapshot={} elements={}{} ({}ms)\n",
        tab["id"],
        page["title"].as_str().unwrap_or("").chars().take(60).collect::<String>().replace('"', "'"),
        page["url"].as_str().unwrap_or("").chars().take(100).collect::<String>(),
        snapshot,
        page["count"],
        if page["truncated"].as_bool().unwrap_or(false) { " truncated=true" } else { "" },
        page["ms"],
    );
    s += body;
    s
}

/// For `fastuse doctor`.
pub fn doctor() -> String {
    let hs = live_hosts();
    let mut s = String::from("Browser use:\n");
    s += &format!("  native host manifest: {}\n", install::status());
    if hs.is_empty() {
        s += "  connected browsers: none (open your browser with the fastuse extension loaded, then reload the extension)\n";
    } else {
        for h in hs {
            s += &format!("  connected: {} (host pid file {})\n", h.browser, h.path.display());
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_invocation_is_detected_by_the_extension_origin() {
        let a = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(is_host_invocation(&a(&["fastuse", "chrome-extension://abc/", "--parent-window=0"])));
        assert!(is_host_invocation(&a(&["fastuse", "--native-host"])));
        assert!(!is_host_invocation(&a(&["fastuse"])));
        assert!(!is_host_invocation(&a(&["fastuse", "doctor"])));
    }

    #[test]
    fn state_text_has_tabs_page_header_and_tree() {
        let tabs = json!({"tabs":[{"id":7,"title":"Docs \"x\"","url":"https://a.test/","active":true}],"hidden":3});
        let st = json!({"tab":{"id":7},"browser":{"name":"Brave","extension":"0.0.1"},
            "page":{"title":"Docs","url":"https://a.test/","viewport":{"w":1000,"h":700,"scrollY":0,"pageH":2000},
                    "count":1,"rows":2,"truncated":false,"ms":4.2,"text":"- Heading[1] \"Docs\"\n- [0] Button \"Go\" @(1,2,3,4)"}});
        let s = format_state(&tabs, &st, "s00000001");
        assert!(s.starts_with("browser: Brave (extension 0.0.1)\ntabs: 1 usable by the agent, 3 others hidden"));
        assert!(s.contains("* tab 7 \"Docs 'x'\" https://a.test/ [shared, active]"));
        assert!(s.contains("snapshot_id=s00000001"));
        assert!(s.contains("element_token for row [N] = s00000001:N"));
        assert!(s.ends_with("- [0] Button \"Go\" @(1,2,3,4)"));
    }
}

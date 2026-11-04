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

pub use install::{install, uninstall};

pub const NO_BROWSER: &str = "No browser is connected. Install the fastuse browser extension (docs/BROWSER.md), run `fastuse browser-install`, then reload the extension in chrome://extensions. `fastuse doctor` shows what is missing.";

pub fn data_dir() -> PathBuf {
    #[cfg(windows)]
    {
        PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".into())).join("fastuse")
    }
    #[cfg(not(windows))]
    {
        PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into())).join(".fastuse")
    }
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

/// Running hosts, newest first.
fn hosts() -> Vec<HostInfo> {
    let mut v: Vec<HostInfo> = std::fs::read_dir(hosts_dir())
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let j: Value = serde_json::from_str(&std::fs::read_to_string(e.path()).ok()?).ok()?;
            Some(HostInfo {
                endpoint: j.get("endpoint")?.as_str()?.to_string(),
                browser: j.get("browser").and_then(|b| b.as_str()).unwrap_or("browser").to_string(),
                started: j.get("started").and_then(|s| s.as_u64()).unwrap_or(0),
                path: e.path(),
            })
        })
        .collect();
    v.sort_by(|a, b| b.started.cmp(&a.started));
    v
}

/// Ask the browser extension to run `method`. Stale host files are removed on the way.
pub fn call(method: &str, params: Value, timeout: Duration) -> Result<Value, String> {
    let hs = hosts();
    if hs.is_empty() {
        return Err(NO_BROWSER.to_string());
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
                let v: Value = serde_json::from_str(&line).map_err(|e| format!("bad reply from the browser host: {e}"))?;
                match v.get("error") {
                    Some(e) => Err(e.as_str().map(String::from).unwrap_or_else(|| e.to_string())),
                    None => Ok(v.get("result").cloned().unwrap_or(Value::Null)),
                }
            }
            Ok(Err(e)) => Err(format!("browser host connection failed: {e}")),
            Err(_) => Err("the browser did not answer in time".to_string()),
        };
    }
    Err(NO_BROWSER.to_string())
}

pub struct Browser {
    snap: u64,
}

impl Browser {
    pub fn new() -> Self {
        Self { snap: 0 }
    }

    /// `browser_state`: the allowed tabs plus the compact tree of one of them.
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
        let tabs = call("tabs.list", json!({}), Duration::from_secs(10))?;
        let st = call("page.state", params, Duration::from_secs(30))?;
        self.snap += 1;
        Ok(format_state(&tabs, &st, &format!("s{:08}", self.snap)))
    }

    /// `browser_act`: run steps in the agent's own tabs, then return the new page state.
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
        let r = call("act.run", params, Duration::from_secs(120))?;
        let failed = r.get("failed").and_then(|f| f.as_bool()).unwrap_or(false);
        let mut out = format!(
            "browser_act: {} step(s), {:.0} ms{}
",
            steps.len(),
            t0.elapsed().as_secs_f64() * 1000.0,
            if failed { " (stopped at failure)" } else { "" }
        );
        for l in r.get("log").and_then(|l| l.as_array()).into_iter().flatten() {
            out += l.as_str().unwrap_or("");
            out.push('\n');
        }
        if let Some(n) = r.get("note").and_then(|n| n.as_str()) {
            out += &format!("note: {n}\n");
        }
        if let Some(st) = r.get("state") {
            self.snap += 1;
            let tabs = call("tabs.list", json!({}), Duration::from_secs(10))?;
            let st = json!({"tab": st["tab"], "page": st["page"], "browser": r["browser"]});
            out += "\n";
            out += &format_state(&tabs, &st, &format!("s{:08}", self.snap));
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

/// For `fastuse doctor`.
pub fn doctor() -> String {
    let hs = hosts();
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

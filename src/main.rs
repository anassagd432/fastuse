mod browser;
mod imaging;
#[cfg(any(windows, target_os = "macos"))]
mod overlay;
mod platform;
#[cfg(feature = "runner")]
mod runner;
mod server;
mod tools;

use std::io::{BufRead, Write};

use serde_json::{json, Value};

fn main() {
    if std::env::args().nth(1).as_deref() == Some("--version") {
        println!("fastuse {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let argv: Vec<String> = std::env::args().collect();
    // the browser starts us as its native host; no desktop backend and nothing else on stdout
    if browser::is_host_invocation(&argv) {
        std::process::exit(browser::host_main());
    }
    let setup = match argv.get(1).map(|s| s.as_str()) {
        Some("browser-install") => Some(browser::install()),
        Some("browser-uninstall") => Some(browser::uninstall()),
        _ => None,
    };
    if let Some(r) = setup {
        match r {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("fastuse: {e}");
                std::process::exit(1);
            }
        }
        return;
    }
    std::process::exit(platform::run_app(app));
}

fn app(p: Box<dyn platform::Platform>) -> i32 {
    let mut srv = server::Server::new(p);
    let instructions = tools::instructions(srv.platform_name());

    if std::env::args().nth(1).as_deref() == Some("doctor") {
        println!("{}
{}", srv.doctor(), browser::doctor());
        return 0;
    }

    if std::env::args().nth(1).as_deref() == Some("demo") {
        srv.demo();
        return 0;
    }

    if std::env::args().nth(1).as_deref() == Some("call") {
        let name = std::env::args().nth(2).unwrap_or_default();
        let raw = std::env::args().nth(3).unwrap_or_else(|| "{}".into());
        let args: Value = serde_json::from_str(&raw).unwrap_or_else(|_| json!({}));
        let res = srv.call(&name, &args);
        if let Some(items) = res.get("content").and_then(|c| c.as_array()) {
            for item in items {
                if let Some(t) = item.get("text").and_then(|t| t.as_str()) {
                    println!("{t}");
                } else {
                    println!("{item}");
                }
            }
        } else {
            println!("{res}");
        }
        return if res.get("isError").and_then(|b| b.as_bool()).unwrap_or(false) { 1 } else { 0 };
    }

    #[cfg(feature = "runner")]
    {
        let argv: Vec<String> = std::env::args().collect();
        if argv.get(1).map(|s| s.as_str()) == Some("run") {
            return runner::run(&mut srv, &argv[2..]);
        }
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
                "instructions": instructions
            }),
            "tools/list" => tools::tools(),
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
    0
}

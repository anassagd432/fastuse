mod imaging;
mod platform;
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
    let p = match platform::default() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("fastuse: {e}");
            std::process::exit(1);
        }
    };
    let mut srv = server::Server::new(p);
    let instructions = tools::instructions(srv.platform_name());

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
}

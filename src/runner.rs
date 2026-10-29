//! `fastuse run "<task>"`: a minimal Gemini function-calling loop with the fastuse tools declared
//! inline and executed in-process. No MCP, no tool-doc discovery, one HTTP connection kept alive.

use std::time::{Duration, Instant};

use serde_json::{json, Value};

const SYSTEM: &str = "You operate Windows app windows through the provided tools. Use `act` for everything: \
one call with all known steps finds the window by title, runs them, and returns the new UI state. \
Only call get_window_state or list_windows if you cannot tell what a window contains. \
When done, reply with one short line.";

fn text_of(v: &Value) -> (String, bool) {
    let t = v["content"]
        .as_array()
        .map(|a| a.iter().filter_map(|c| c["text"].as_str()).collect::<Vec<_>>().join("\n"))
        .unwrap_or_default();
    (t, v["isError"].as_bool().unwrap_or(false))
}

pub fn run(srv: &mut crate::server::Server, args: &[String]) -> i32 {
    let mut model = "gemini-3.8-flash".to_string();
    let mut think = "low".to_string();
    let mut max_turns = 12usize;
    let mut verbose = false;
    let mut task = String::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--model" => { i += 1; model = args.get(i).cloned().unwrap_or(model); }
            "--thinking" => { i += 1; think = args.get(i).cloned().unwrap_or(think); }
            "--max-turns" => { i += 1; max_turns = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(max_turns); }
            "--verbose" | "-v" => verbose = true,
            t => task = t.to_string(),
        }
        i += 1;
    }
    if task.is_empty() {
        eprintln!("usage: fastuse run [--model M] [--thinking low|medium|high] [--max-turns N] [-v] \"task\"");
        return 2;
    }
    let key = match std::env::var("GEMINI_API_KEY").or_else(|_| std::env::var("GOOGLE_API_KEY")) {
        Ok(k) if !k.is_empty() => k,
        _ => {
            eprintln!("set GEMINI_API_KEY (or GOOGLE_API_KEY) in the environment");
            return 2;
        }
    };

    let decls: Vec<Value> = crate::tools::tools()["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| {
            let mut d = json!({"name": t["name"], "description": t["description"]});
            if t["inputSchema"]["properties"].as_object().map_or(false, |p| !p.is_empty()) {
                d["parameters"] = t["inputSchema"].clone();
            }
            d
        })
        .collect();

    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(90)).build();
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent");
    let mut contents = vec![json!({"role":"user","parts":[{"text": task}]})];
    let mut use_think = true;
    let (mut model_ms, mut tool_ms) = (0.0f64, 0.0f64);
    let (mut tin, mut tout) = (0u64, 0u64);
    let mut calls: Vec<String> = Vec::new();
    let t_all = Instant::now();
    let mut answer = String::new();
    let mut turns = 0usize;

    while turns < max_turns {
        turns += 1;
        let mut body = json!({
            "systemInstruction": {"parts":[{"text": SYSTEM}]},
            "tools": [{"functionDeclarations": decls}],
            "contents": contents,
        });
        if use_think {
            body["generationConfig"] = json!({"thinkingConfig":{"thinkingLevel": think}});
        }
        let t = Instant::now();
        let resp = agent
            .post(&url)
            .set("x-goog-api-key", &key)
            .set("content-type", "application/json")
            .send_json(&body);
        let v: Value = match resp {
            Ok(r) => r.into_json().unwrap_or(Value::Null),
            Err(ureq::Error::Status(code, r)) => {
                let b = r.into_string().unwrap_or_default();
                if code == 400 && use_think && b.to_lowercase().contains("think") {
                    eprintln!("note: thinking config rejected, retrying without it");
                    use_think = false;
                    turns -= 1;
                    continue;
                }
                eprintln!("HTTP {code}: {}", b.chars().take(600).collect::<String>());
                return 1;
            }
            Err(e) => {
                eprintln!("request failed: {e}");
                return 1;
            }
        };
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        model_ms += ms;
        tin += v["usageMetadata"]["promptTokenCount"].as_u64().unwrap_or(0);
        tout += v["usageMetadata"]["candidatesTokenCount"].as_u64().unwrap_or(0) + v["usageMetadata"]["thoughtsTokenCount"].as_u64().unwrap_or(0);

        let cand = &v["candidates"][0];
        let content = cand["content"].clone();
        let parts = content["parts"].as_array().cloned().unwrap_or_default();
        let fcs: Vec<&Value> = parts.iter().filter(|p| p.get("functionCall").is_some()).collect();
        if fcs.is_empty() {
            answer = parts.iter().filter_map(|p| p["text"].as_str()).collect::<Vec<_>>().join("");
            if answer.is_empty() {
                answer = format!("(no text; finishReason={})", cand["finishReason"].as_str().unwrap_or("?"));
            }
            if verbose {
                eprintln!("turn {turns}: model {ms:.0} ms -> final answer");
            }
            break;
        }
        // echo the model turn verbatim (keeps thought signatures), then answer every call
        contents.push(content.clone());
        let mut resp_parts = Vec::new();
        for p in &fcs {
            let fc = &p["functionCall"];
            let name = fc["name"].as_str().unwrap_or("");
            let args = fc.get("args").cloned().unwrap_or(json!({}));
            let t = Instant::now();
            let out = srv.call(name, &args);
            let tms = t.elapsed().as_secs_f64() * 1000.0;
            tool_ms += tms;
            let (txt, is_err) = text_of(&out);
            calls.push(name.to_string());
            if verbose {
                eprintln!("turn {turns}: model {ms:.0} ms -> {name}({}) tool {tms:.0} ms{}", args, if is_err { " ERROR" } else { "" });
            }
            let mut fr = json!({"name": name, "response": if is_err { json!({"error": txt}) } else { json!({"result": txt}) }});
            if let Some(id) = fc.get("id") {
                fr["id"] = id.clone();
            }
            resp_parts.push(json!({"functionResponse": fr}));
        }
        contents.push(json!({"role":"user","parts": resp_parts}));
    }

    let total = t_all.elapsed().as_secs_f64() * 1000.0;
    println!("{answer}");
    eprintln!(
        "{}",
        json!({"turns": turns, "total_ms": total.round(), "model_ms": model_ms.round(), "tool_ms": tool_ms.round(),
               "input_tokens": tin, "output_tokens": tout, "calls": calls, "model": model})
    );
    0
}

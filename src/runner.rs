//! Embedded sub-agent loop supporting local Ollama, local GGUF (`llama-server`), any
//! OpenAI-compatible server (LM Studio, vLLM, OpenAI, Groq, OpenRouter), and Gemini.
//! Powers both `fastuse run` CLI and the `delegate` MCP tool (keeping frontier models fast & cheap).

use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::{json, Value};

const SYSTEM: &str = "You operate desktop windows and Chromium browser tabs via the provided tools. \
Batch multiple steps into a single `act` or `browser_act` call whenever possible. \
Pass `observe: false` on final actions and `diff: true` on repeated reads to minimize tokens. \
Only call `get_window_state`, `browser_state`, or `list_windows` when needed. \
Reply with one concise line describing the verified outcome.";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Provider {
    Ollama,
    LlamaCpp,
    OpenAiCompat,
    Gemini,
}

impl Provider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ollama => "ollama",
            Self::LlamaCpp => "llamacpp",
            Self::OpenAiCompat => "openai",
            Self::Gemini => "gemini",
        }
    }
}

pub struct RunOpts {
    pub task: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub gguf: Option<String>,
    pub endpoint: Option<String>,
    pub think: String,
    pub max_turns: usize,
    pub verbose: bool,
}

pub struct RunOutcome {
    pub answer: String,
    pub provider: String,
    pub model: String,
    pub turns: usize,
    pub total_ms: f64,
    pub model_ms: f64,
    pub tool_ms: f64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub calls: Vec<String>,
}

struct LlamaGuard(Child);

impl Drop for LlamaGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn text_of(v: &Value) -> (String, bool) {
    let t = v["content"]
        .as_array()
        .map(|a| a.iter().filter_map(|c| c["text"].as_str()).collect::<Vec<_>>().join("\n"))
        .unwrap_or_default();
    (t, v["isError"].as_bool().unwrap_or(false))
}

fn ollama_base(override_url: Option<&str>) -> String {
    let raw = override_url
        .map(String::from)
        .or_else(|| std::env::var("OLLAMA_HOST").ok())
        .unwrap_or_else(|| "http://127.0.0.1:11434".into());
    let base = raw.trim_end_matches('/').trim_end_matches("/v1");
    if base.starts_with("http://") || base.starts_with("https://") {
        base.to_string()
    } else {
        format!("http://{base}")
    }
}

fn probe_ollama_models(base: &str) -> Vec<String> {
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_millis(350)).build();
    let Ok(r) = agent.get(&format!("{base}/api/tags")).call() else { return Vec::new() };
    let Ok(v) = r.into_json::<Value>() else { return Vec::new() };
    v["models"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|m| m["name"].as_str().map(String::from)).collect())
        .unwrap_or_default()
}

fn pick_best_local_model(models: &[String]) -> Option<String> {
    const PREFER: &[&str] = &["qwen3", "qwen2.5", "qwen", "llama3.3", "llama3.2", "llama3.1", "gpt-oss", "mistral", "gemma"];
    let non_embed: Vec<&String> = models
        .iter()
        .filter(|n| !n.to_ascii_lowercase().contains("embed"))
        .collect();
    for pat in PREFER {
        if let Some(m) = non_embed.iter().find(|n| n.to_ascii_lowercase().contains(pat)) {
            return Some((*m).clone());
        }
    }
    non_embed.first().map(|s| (*s).clone()).or_else(|| models.first().cloned())
}

fn probe_openai_models(base_v1: &str) -> Vec<String> {
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_millis(350)).build();
    let url = format!("{}/models", base_v1.trim_end_matches('/'));
    let Ok(r) = agent.get(&url).call() else { return Vec::new() };
    let Ok(v) = r.into_json::<Value>() else { return Vec::new() };
    v["data"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|m| m["id"].as_str().map(String::from)).collect())
        .unwrap_or_default()
}

fn find_llama_server() -> Option<PathBuf> {
    if let Ok(explicit) = std::env::var("LLAMA_SERVER_BIN") {
        let p = PathBuf::from(explicit);
        if p.exists() {
            return Some(p);
        }
    }
    let bin = if cfg!(windows) { "llama-server.exe" } else { "llama-server" };
    if Command::new(bin).arg("--version").stdout(Stdio::null()).stderr(Stdio::null()).status().is_ok() {
        return Some(PathBuf::from(bin));
    }
    let mut candidates = Vec::new();
    #[cfg(windows)]
    {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            candidates.push(PathBuf::from(&local).join("Programs").join("llama.cpp").join(bin));
            candidates.push(PathBuf::from(&local).join("fastuse").join("bin").join(bin));
        }
        if let Ok(home) = std::env::var("USERPROFILE") {
            candidates.push(PathBuf::from(&home).join("llama.cpp").join("build").join("bin").join("Release").join(bin));
            candidates.push(PathBuf::from(&home).join("local-trainer").join("bin").join(bin));
        }
    }
    #[cfg(unix)]
    {
        if let Ok(home) = std::env::var("HOME") {
            candidates.push(PathBuf::from(&home).join(".local").join("bin").join(bin));
            candidates.push(PathBuf::from(home).join("llama.cpp").join("build").join("bin").join(bin));
        }
        candidates.push(PathBuf::from("/opt/homebrew/bin").join(bin));
        candidates.push(PathBuf::from("/usr/local/bin").join(bin));
    }
    candidates.into_iter().find(|p| p.exists())
}

fn spawn_llama_server(gguf_path: &str) -> Result<(String, LlamaGuard), String> {
    if !Path::new(gguf_path).exists() {
        return Err(format!("GGUF file not found: {gguf_path}"));
    }
    let server_bin = find_llama_server().ok_or_else(|| {
        "llama-server executable not found on PATH. Install llama.cpp (e.g. `winget install ggml.llamacpp` or `brew install llama.cpp`) or run llama-server manually on port 8080.".to_string()
    })?;
    let port = TcpListener::bind("127.0.0.1:0")
        .map_err(|e| format!("failed to allocate localhost port: {e}"))?
        .local_addr()
        .map_err(|e| format!("failed to inspect local port: {e}"))?
        .port();

    let mut child = Command::new(&server_bin)
        .args(["-m", gguf_path, "--host", "127.0.0.1", "--port", &port.to_string(), "--jinja", "-c", "8192", "-ngl", "99"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("failed to launch {}: {e}", server_bin.display()))?;

    let base = format!("http://127.0.0.1:{port}");
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_millis(500)).build();
    let deadline = Instant::now() + Duration::from_secs(45);
    while Instant::now() < deadline {
        if let Ok(Some(status)) = child.try_wait() {
            return Err(format!("llama-server exited early with status {status}"));
        }
        if let Ok(r) = agent.get(&format!("{base}/health")).call() {
            if r.status() == 200 {
                return Ok((format!("{base}/v1"), LlamaGuard(child)));
            }
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    let _ = child.kill();
    Err("timed out waiting 45s for llama-server /health to become ready".into())
}

struct ResolvedTarget {
    provider: Provider,
    endpoint: String,
    model: String,
    api_key: Option<String>,
    _guard: Option<LlamaGuard>,
}

fn resolve_target(opts: &RunOpts) -> Result<ResolvedTarget, String> {
    let req_prov = opts
        .provider
        .clone()
        .or_else(|| std::env::var("FASTUSE_PROVIDER").ok())
        .map(|s| s.to_ascii_lowercase());
    let req_model = opts.model.clone().or_else(|| std::env::var("FASTUSE_MODEL").ok());
    let req_gguf = opts.gguf.clone().or_else(|| std::env::var("FASTUSE_GGUF").ok());
    let req_ep = opts.endpoint.clone().or_else(|| std::env::var("FASTUSE_ENDPOINT").ok());

    let explicit_gguf = req_gguf.or_else(|| {
        req_model
            .as_ref()
            .filter(|m| m.to_ascii_lowercase().ends_with(".gguf"))
            .cloned()
    });

    if req_prov.as_deref() == Some("gguf") || explicit_gguf.is_some() {
        let path = explicit_gguf.ok_or("pass --gguf <path/to/model.gguf> or set FASTUSE_GGUF")?;
        let (ep, guard) = spawn_llama_server(&path)?;
        let name = Path::new(&path).file_name().and_then(|n| n.to_str()).unwrap_or("local-gguf").to_string();
        return Ok(ResolvedTarget {
            provider: Provider::LlamaCpp,
            endpoint: ep,
            model: req_model.filter(|m| !m.to_ascii_lowercase().ends_with(".gguf")).unwrap_or(name),
            api_key: None,
            _guard: Some(guard),
        });
    }

    match req_prov.as_deref() {
        Some("ollama") => {
            let base = ollama_base(req_ep.as_deref());
            let models = probe_ollama_models(&base);
            let model = req_model
                .or_else(|| pick_best_local_model(&models))
                .ok_or_else(|| format!("No Ollama models found at {base}. Run `ollama pull qwen2.5:7b` first."))?;
            return Ok(ResolvedTarget {
                provider: Provider::Ollama,
                endpoint: format!("{base}/v1"),
                model,
                api_key: None,
                _guard: None,
            });
        }
        Some("llamacpp" | "llama.cpp") => {
            let base = req_ep
                .or_else(|| std::env::var("LLAMA_SERVER_URL").ok())
                .unwrap_or_else(|| "http://127.0.0.1:8080/v1".into());
            let base = if base.trim_end_matches('/').ends_with("/v1") {
                base.trim_end_matches('/').to_string()
            } else {
                format!("{}/v1", base.trim_end_matches('/'))
            };
            let models = probe_openai_models(&base);
            let model = req_model.or_else(|| models.into_iter().next()).unwrap_or_else(|| "local-model".into());
            return Ok(ResolvedTarget {
                provider: Provider::LlamaCpp,
                endpoint: base,
                model,
                api_key: None,
                _guard: None,
            });
        }
        Some("openai" | "openai-compat" | "lmstudio" | "vllm") => {
            let base = req_ep
                .or_else(|| std::env::var("OPENAI_BASE_URL").ok())
                .unwrap_or_else(|| "https://api.openai.com/v1".into())
                .trim_end_matches('/')
                .to_string();
            let key = std::env::var("OPENAI_API_KEY").or_else(|_| std::env::var("FASTUSE_API_KEY")).ok();
            if base.contains("api.openai.com") && key.as_deref().unwrap_or("").is_empty() {
                return Err("OPENAI_API_KEY is required when calling api.openai.com".into());
            }
            return Ok(ResolvedTarget {
                provider: Provider::OpenAiCompat,
                endpoint: base,
                model: req_model.unwrap_or_else(|| "gpt-4o-mini".into()),
                api_key: key,
                _guard: None,
            });
        }
        Some("gemini") => {
            let key = std::env::var("GEMINI_API_KEY")
                .or_else(|_| std::env::var("GOOGLE_API_KEY"))
                .map_err(|_| "set GEMINI_API_KEY or GOOGLE_API_KEY for Gemini provider".to_string())?;
            return Ok(ResolvedTarget {
                provider: Provider::Gemini,
                endpoint: "https://generativelanguage.googleapis.com".into(),
                model: req_model.unwrap_or_else(|| "gemini-2.5-flash".into()),
                api_key: Some(key),
                _guard: None,
            });
        }
        Some(other) if other != "auto" => {
            return Err(format!("unknown provider '{other}' (expected auto, ollama, gguf, llamacpp, openai, gemini)"));
        }
        _ => {}
    }

    // Auto-detect: explicit endpoint -> local Ollama -> local llama-server -> OpenAI -> Gemini
    if let Some(ep) = req_ep {
        let base = if ep.trim_end_matches('/').ends_with("/v1") {
            ep.trim_end_matches('/').to_string()
        } else {
            format!("{}/v1", ep.trim_end_matches('/'))
        };
        let models = probe_openai_models(&base);
        let model = req_model.or_else(|| models.into_iter().next()).unwrap_or_else(|| "local-model".into());
        let key = std::env::var("OPENAI_API_KEY").or_else(|_| std::env::var("FASTUSE_API_KEY")).ok();
        return Ok(ResolvedTarget {
            provider: Provider::OpenAiCompat,
            endpoint: base,
            model,
            api_key: key,
            _guard: None,
        });
    }

    let obase = ollama_base(None);
    let omodels = probe_ollama_models(&obase);
    if !omodels.is_empty() {
        let model = req_model
            .or_else(|| pick_best_local_model(&omodels))
            .unwrap_or_else(|| omodels[0].clone());
        return Ok(ResolvedTarget {
            provider: Provider::Ollama,
            endpoint: format!("{obase}/v1"),
            model,
            api_key: None,
            _guard: None,
        });
    }

    let lbase = std::env::var("LLAMA_SERVER_URL").unwrap_or_else(|_| "http://127.0.0.1:8080/v1".into());
    let lmodels = probe_openai_models(&lbase);
    if !lmodels.is_empty() {
        let model = req_model.unwrap_or_else(|| lmodels[0].clone());
        return Ok(ResolvedTarget {
            provider: Provider::LlamaCpp,
            endpoint: lbase.trim_end_matches('/').to_string(),
            model,
            api_key: None,
            _guard: None,
        });
    }

    if let Ok(key) = std::env::var("OPENAI_API_KEY").or_else(|_| std::env::var("FASTUSE_API_KEY")) {
        if !key.is_empty() {
            let base = std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".into());
            return Ok(ResolvedTarget {
                provider: Provider::OpenAiCompat,
                endpoint: base.trim_end_matches('/').to_string(),
                model: req_model.unwrap_or_else(|| "gpt-4o-mini".into()),
                api_key: Some(key),
                _guard: None,
            });
        }
    }

    if let Ok(key) = std::env::var("GEMINI_API_KEY").or_else(|_| std::env::var("GOOGLE_API_KEY")) {
        if !key.is_empty() {
            return Ok(ResolvedTarget {
                provider: Provider::Gemini,
                endpoint: "https://generativelanguage.googleapis.com".into(),
                model: req_model.unwrap_or_else(|| "gemini-2.5-flash".into()),
                api_key: Some(key),
                _guard: None,
            });
        }
    }

    Err("No model available. Start Ollama (`ollama serve` & `ollama pull qwen2.5:7b`), pass `--gguf path/to/model.gguf`, or set `OPENAI_API_KEY` / `GEMINI_API_KEY`.".into())
}

fn openai_tools() -> Vec<Value> {
    crate::tools::base_tools()
        .into_iter()
        .map(|t| {
            json!({
                "type": "function",
                "function": {
                    "name": t["name"],
                    "description": t["description"],
                    "parameters": t["inputSchema"]
                }
            })
        })
        .collect()
}

fn gemini_tools() -> Vec<Value> {
    crate::tools::base_tools()
        .into_iter()
        .map(|t| {
            let mut d = json!({"name": t["name"], "description": t["description"]});
            if t["inputSchema"]["properties"].as_object().map_or(false, |p| !p.is_empty()) {
                d["parameters"] = t["inputSchema"].clone();
            }
            d
        })
        .collect()
}

/// Extracts fallback tool calls when a small local GGUF/Ollama model emits `<tool_call>{...}</tool_call>`
/// or fenced JSON inside `message.content` instead of structured `tool_calls`.
pub fn parse_fallback_tool_calls(content: &str) -> Vec<(String, String, Value)> {
    let mut out = Vec::new();
    let mut rest = content;
    while let Some(start) = rest.find("<tool_call>") {
        let after = &rest[start + 11..];
        let Some(end) = after.find("</tool_call>") else { break };
        let chunk = after[..end].trim();
        if let Ok(v) = serde_json::from_str::<Value>(chunk) {
            if let Some(name) = v.get("name").and_then(|n| n.as_str()) {
                let args = v.get("arguments").cloned().unwrap_or_else(|| json!({}));
                out.push((format!("call_{}", out.len() + 1), name.to_string(), args));
            }
        }
        rest = &after[end + 12..];
    }
    if !out.is_empty() {
        return out;
    }
    // Single fenced or raw JSON object fallback
    let stripped = content.trim();
    let candidate = stripped
        .strip_prefix("```json")
        .or_else(|| stripped.strip_prefix("```"))
        .map(|s| s.trim_end_matches("```").trim())
        .unwrap_or(stripped);
    if candidate.starts_with('{') && candidate.ends_with('}') {
        if let Ok(v) = serde_json::from_str::<Value>(candidate) {
            if let Some(name) = v.get("name").and_then(|n| n.as_str()) {
                let args = match v.get("arguments") {
                    Some(Value::String(s)) => serde_json::from_str(s).unwrap_or_else(|_| json!({})),
                    Some(other) => other.clone(),
                    None => json!({}),
                };
                out.push(("call_1".into(), name.to_string(), args));
            }
        }
    }
    out
}

fn run_openai_compat(srv: &mut crate::server::Server, target: &ResolvedTarget, opts: &RunOpts) -> Result<RunOutcome, String> {
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(120)).build();
    let url = format!("{}/chat/completions", target.endpoint.trim_end_matches('/'));
    let tools = openai_tools();
    let mut messages = vec![
        json!({"role": "system", "content": SYSTEM}),
        json!({"role": "user", "content": opts.task}),
    ];
    let t_all = Instant::now();
    let (mut model_ms, mut tool_ms) = (0.0f64, 0.0f64);
    let (mut tin, mut tout) = (0u64, 0u64);
    let mut calls = Vec::new();
    let mut answer = String::new();
    let mut turns = 0usize;

    while turns < opts.max_turns {
        turns += 1;
        let body = json!({
            "model": target.model,
            "messages": messages,
            "tools": tools,
            "tool_choice": "auto",
            "temperature": 0.1
        });
        let t = Instant::now();
        let mut req = agent.post(&url).set("content-type", "application/json");
        if let Some(ref k) = target.api_key {
            if !k.is_empty() {
                req = req.set("authorization", &format!("Bearer {k}"));
            }
        }
        let v: Value = match req.send_json(&body) {
            Ok(r) => r.into_json().map_err(|e| format!("bad JSON from {}: {e}", target.endpoint))?,
            Err(ureq::Error::Status(code, r)) => {
                let b = r.into_string().unwrap_or_default();
                return Err(format!("HTTP {code} from {}: {}", target.endpoint, b.chars().take(500).collect::<String>()));
            }
            Err(e) => return Err(format!("request to {} failed: {e}", target.endpoint)),
        };
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        model_ms += ms;
        tin += v["usage"]["prompt_tokens"].as_u64().unwrap_or(0);
        tout += v["usage"]["completion_tokens"].as_u64().unwrap_or(0);

        let msg = &v["choices"][0]["message"];
        let text_content = msg["content"].as_str().unwrap_or("").trim().to_string();
        let mut parsed_calls: Vec<(String, String, Value)> = Vec::new();

        if let Some(tcs) = msg["tool_calls"].as_array() {
            for (idx, tc) in tcs.iter().enumerate() {
                let cid = tc["id"].as_str().map(String::from).unwrap_or_else(|| format!("call_{idx}"));
                let name = tc["function"]["name"].as_str().unwrap_or("").to_string();
                let args = match &tc["function"]["arguments"] {
                    Value::String(s) => serde_json::from_str(s).unwrap_or_else(|_| json!({})),
                    obj @ Value::Object(_) => obj.clone(),
                    _ => json!({}),
                };
                if !name.is_empty() {
                    parsed_calls.push((cid, name, args));
                }
            }
        }
        if parsed_calls.is_empty() && !text_content.is_empty() {
            parsed_calls = parse_fallback_tool_calls(&text_content);
        }

        if parsed_calls.is_empty() {
            answer = if text_content.is_empty() {
                format!("(no text; finish_reason={})", v["choices"][0]["finish_reason"].as_str().unwrap_or("?"))
            } else {
                text_content
            };
            if opts.verbose {
                eprintln!("turn {turns}: {} ({ms:.0}ms) -> done", target.model);
            }
            break;
        }

        // Preserve assistant message when native tool_calls exist; otherwise wrap synthesized calls
        if msg["tool_calls"].as_array().map_or(false, |a| !a.is_empty()) {
            messages.push(msg.clone());
        } else {
            let synth: Vec<Value> = parsed_calls
                .iter()
                .map(|(cid, name, args)| {
                    json!({
                        "id": cid,
                        "type": "function",
                        "function": {"name": name, "arguments": args.to_string()}
                    })
                })
                .collect();
            messages.push(json!({"role": "assistant", "content": "", "tool_calls": synth}));
        }

        for (cid, name, args) in parsed_calls {
            let t = Instant::now();
            let out = srv.call(&name, &args);
            let tms = t.elapsed().as_secs_f64() * 1000.0;
            tool_ms += tms;
            let (txt, is_err) = text_of(&out);
            calls.push(name.clone());
            if opts.verbose {
                eprintln!("turn {turns}: {name}({args}) -> {tms:.0}ms{}", if is_err { " ERROR" } else { "" });
            }
            messages.push(json!({
                "role": "tool",
                "tool_call_id": cid,
                "content": if is_err { format!("ERROR: {txt}") } else { txt }
            }));
        }
    }

    Ok(RunOutcome {
        answer,
        provider: target.provider.as_str().into(),
        model: target.model.clone(),
        turns,
        total_ms: t_all.elapsed().as_secs_f64() * 1000.0,
        model_ms,
        tool_ms,
        input_tokens: tin,
        output_tokens: tout,
        calls,
    })
}

fn run_gemini(srv: &mut crate::server::Server, target: &ResolvedTarget, opts: &RunOpts) -> Result<RunOutcome, String> {
    let key = target.api_key.as_deref().ok_or("missing Gemini API key")?;
    let decls = gemini_tools();
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(90)).build();
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent", target.model);
    let mut contents = vec![json!({"role":"user","parts":[{"text": opts.task}]})];
    let mut use_think = true;
    let (mut model_ms, mut tool_ms) = (0.0f64, 0.0f64);
    let (mut tin, mut tout) = (0u64, 0u64);
    let mut calls = Vec::new();
    let t_all = Instant::now();
    let mut answer = String::new();
    let mut turns = 0usize;

    while turns < opts.max_turns {
        turns += 1;
        let mut body = json!({
            "systemInstruction": {"parts":[{"text": SYSTEM}]},
            "tools": [{"functionDeclarations": decls}],
            "contents": contents,
        });
        if use_think {
            body["generationConfig"] = json!({"thinkingConfig":{"thinkingLevel": opts.think}});
        }
        let t = Instant::now();
        let resp = agent
            .post(&url)
            .set("x-goog-api-key", key)
            .set("content-type", "application/json")
            .send_json(&body);
        let v: Value = match resp {
            Ok(r) => r.into_json().unwrap_or(Value::Null),
            Err(ureq::Error::Status(code, r)) => {
                let b = r.into_string().unwrap_or_default();
                if code == 400 && use_think && b.to_lowercase().contains("think") {
                    use_think = false;
                    turns -= 1;
                    continue;
                }
                return Err(format!("HTTP {code}: {}", b.chars().take(500).collect::<String>()));
            }
            Err(e) => return Err(format!("request failed: {e}")),
        };
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        model_ms += ms;
        tin += v["usageMetadata"]["promptTokenCount"].as_u64().unwrap_or(0);
        tout += v["usageMetadata"]["candidatesTokenCount"].as_u64().unwrap_or(0)
            + v["usageMetadata"]["thoughtsTokenCount"].as_u64().unwrap_or(0);

        let cand = &v["candidates"][0];
        let content = cand["content"].clone();
        let parts = content["parts"].as_array().cloned().unwrap_or_default();
        let fcs: Vec<&Value> = parts.iter().filter(|p| p.get("functionCall").is_some()).collect();
        if fcs.is_empty() {
            answer = parts.iter().filter_map(|p| p["text"].as_str()).collect::<Vec<_>>().join("");
            if answer.is_empty() {
                answer = format!("(no text; finishReason={})", cand["finishReason"].as_str().unwrap_or("?"));
            }
            break;
        }
        contents.push(content.clone());
        let mut resp_parts = Vec::new();
        for p in &fcs {
            let fc = &p["functionCall"];
            let name = fc["name"].as_str().unwrap_or("");
            let args = fc.get("args").cloned().unwrap_or_else(|| json!({}));
            let t = Instant::now();
            let out = srv.call(name, &args);
            let tms = t.elapsed().as_secs_f64() * 1000.0;
            tool_ms += tms;
            let (txt, is_err) = text_of(&out);
            calls.push(name.to_string());
            if opts.verbose {
                eprintln!("turn {turns}: {name}({args}) -> {tms:.0}ms{}", if is_err { " ERROR" } else { "" });
            }
            let mut fr = json!({"name": name, "response": if is_err { json!({"error": txt}) } else { json!({"result": txt}) }});
            if let Some(id) = fc.get("id") {
                fr["id"] = id.clone();
            }
            resp_parts.push(json!({"functionResponse": fr}));
        }
        contents.push(json!({"role":"user","parts": resp_parts}));
    }

    Ok(RunOutcome {
        answer,
        provider: target.provider.as_str().into(),
        model: target.model.clone(),
        turns,
        total_ms: t_all.elapsed().as_secs_f64() * 1000.0,
        model_ms,
        tool_ms,
        input_tokens: tin,
        output_tokens: tout,
        calls,
    })
}

pub fn execute(srv: &mut crate::server::Server, opts: &RunOpts) -> Result<RunOutcome, String> {
    let target = resolve_target(opts)?;
    match target.provider {
        Provider::Gemini => run_gemini(srv, &target, opts),
        Provider::Ollama | Provider::LlamaCpp | Provider::OpenAiCompat => run_openai_compat(srv, &target, opts),
    }
}

/// MCP `delegate` tool entry point: runs multi-step UI goal on local/cheap model and returns a concise summary.
pub fn delegate(srv: &mut crate::server::Server, a: &Value) -> Result<String, String> {
    let task = a
        .get("task")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or("delegate requires non-empty `task`")?
        .to_string();
    let opts = RunOpts {
        task,
        provider: a.get("provider").and_then(|v| v.as_str()).map(String::from),
        model: a.get("model").and_then(|v| v.as_str()).map(String::from),
        gguf: a.get("gguf").and_then(|v| v.as_str()).map(String::from),
        endpoint: a.get("endpoint").and_then(|v| v.as_str()).map(String::from),
        think: "low".into(),
        max_turns: a.get("max_turns").and_then(|v| v.as_u64()).unwrap_or(10).clamp(1, 25) as usize,
        verbose: false,
    };
    let r = execute(srv, &opts)?;
    Ok(format!(
        "{}\n[delegate: {}/{} | {} turn(s) | {:.0}ms total | {} in / {} out tokens]",
        r.answer, r.provider, r.model, r.turns, r.total_ms, r.input_tokens, r.output_tokens
    ))
}

/// CLI `fastuse models`: inspects local Ollama & llama-server endpoints.
pub fn list_models() -> String {
    let obase = ollama_base(None);
    let omodels = probe_ollama_models(&obase);
    let lbase = std::env::var("LLAMA_SERVER_URL").unwrap_or_else(|_| "http://127.0.0.1:8080/v1".into());
    let lmodels = probe_openai_models(&lbase);
    let mut s = String::from("Local LLM backends:\n");
    if omodels.is_empty() {
        s += &format!("  Ollama ({obase}): not running or no models pulled\n");
    } else {
        s += &format!("  Ollama ({obase}): {}\n", omodels.join(", "));
    }
    if lmodels.is_empty() {
        s += &format!("  llama-server ({lbase}): not running\n");
    } else {
        s += &format!("  llama-server ({lbase}): {}\n", lmodels.join(", "));
    }
    let gguf_bin = find_llama_server()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "not found on PATH".into());
    s += &format!("  llama-server binary: {gguf_bin}\n");
    s
}

/// CLI `fastuse run ...` entry point.
pub fn run(srv: &mut crate::server::Server, args: &[String]) -> i32 {
    let mut opts = RunOpts {
        task: String::new(),
        provider: None,
        model: None,
        gguf: None,
        endpoint: None,
        think: "low".into(),
        max_turns: 12,
        verbose: false,
    };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--provider" => { i += 1; opts.provider = args.get(i).cloned(); }
            "--model" => { i += 1; opts.model = args.get(i).cloned(); }
            "--gguf" => { i += 1; opts.gguf = args.get(i).cloned(); }
            "--endpoint" => { i += 1; opts.endpoint = args.get(i).cloned(); }
            "--thinking" => { i += 1; if let Some(v) = args.get(i) { opts.think = v.clone(); } }
            "--max-turns" => { i += 1; opts.max_turns = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(opts.max_turns); }
            "--verbose" | "-v" => opts.verbose = true,
            t => opts.task = t.to_string(),
        }
        i += 1;
    }
    if opts.task.is_empty() {
        eprintln!(
            "usage: fastuse run [--provider auto|ollama|gguf|llamacpp|openai|gemini] [--model M] [--gguf PATH] [--endpoint URL] [--max-turns N] [-v] \"task\""
        );
        return 2;
    }
    match execute(srv, &opts) {
        Ok(r) => {
            println!("{}", r.answer);
            eprintln!(
                "{}",
                json!({
                    "provider": r.provider, "model": r.model, "turns": r.turns,
                    "total_ms": r.total_ms.round(), "model_ms": r.model_ms.round(), "tool_ms": r.tool_ms.round(),
                    "input_tokens": r.input_tokens, "output_tokens": r.output_tokens, "calls": r.calls
                })
            );
            0
        }
        Err(e) => {
            eprintln!("fastuse run: {e}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_tool_capable_local_models() {
        let m = vec!["nomic-embed-text".into(), "qwen2.5:7b".into(), "phi3:mini".into()];
        assert_eq!(pick_best_local_model(&m).as_deref(), Some("qwen2.5:7b"));
    }

    #[test]
    fn parses_xml_and_json_tool_call_fallbacks() {
        let xml = "Sure, calling tool:\n<tool_call>{\"name\":\"act\",\"arguments\":{\"window\":\"Notepad\",\"observe\":false}}</tool_call>";
        let calls = parse_fallback_tool_calls(xml);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].1, "act");
        assert_eq!(calls[0].2["window"], "Notepad");

        let fenced = "```json\n{\"name\":\"list_windows\",\"arguments\":{}}\n```";
        let calls2 = parse_fallback_tool_calls(fenced);
        assert_eq!(calls2.len(), 1);
        assert_eq!(calls2[0].1, "list_windows");
    }
}

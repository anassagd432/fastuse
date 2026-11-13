//! MCP tool declarations and server instructions.

use serde_json::{json, Value};

pub fn tools() -> Value {
    #[allow(unused_mut)]
    let mut list = base_tools();
    #[cfg(feature = "runner")]
    list.push(json!({
        "name": "delegate",
        "description": "TOKEN-SAVER: Offloads an entire multi-step OS/browser UI workflow to a local Ollama, GGUF (llama-server), or cheap sub-agent model so the main model avoids UI tree turns. Pass a clear goal and optional provider/model.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "task": {"type": "string", "description": "Complete UI goal to perform and verify"},
                "provider": {"type": "string", "enum": ["auto", "ollama", "llamacpp", "gguf", "openai", "gemini"]},
                "model": {"type": "string", "description": "Model name (e.g. qwen2.5:7b, llama3.2, gpt-4o-mini, gemini-2.5-flash)"},
                "gguf": {"type": "string", "description": "Local .gguf model file path (spawns llama-server automatically)"},
                "endpoint": {"type": "string", "description": "OpenAI-compatible base URL (e.g. http://127.0.0.1:11434/v1)"},
                "max_turns": {"type": "integer", "description": "Max runner turns (default: 10)"}
            },
            "required": ["task"]
        }
    }));
    json!({"tools": list})
}

pub fn base_tools() -> Vec<Value> {
    let tool = |n: &str, d: &str, p: Value| json!({"name":n,"description":d,"inputSchema":{"type":"object","properties":p}});
    let win = json!({"type":"string","description":"window title substring"});
    vec![
        tool("act",
            "FAST DESKTOP BATCH: Run OS UI steps in background without moving mouse/focus. Pass steps:[{tool,target,value/text/keys}]. Step tools: click, double_click, right_click, set_value, type_text, press_key, hotkey, scroll, wait. Pass observe:false on final actions to save tokens, or diff:true for deltas.",
            json!({"window":win,"window_id":{"type":"integer"},"pid":{"type":"integer"},
                   "steps":{"type":"array","items":{"type":"object","properties":{
                       "tool":{"type":"string","enum":["click","double_click","right_click","set_value","type_text","press_key","hotkey","scroll","wait"]},
                       "target":{"type":"string"},"element_token":{"type":"string"},"value":{"type":"string"},"text":{"type":"string"},
                       "key":{"type":"string"},"keys":{"type":"string"},"x":{"type":"integer"},"y":{"type":"integer"},
                       "direction":{"type":"string","enum":["up","down","left","right"]},"amount":{"type":"integer"},"ms":{"type":"integer"},
                       "delivery_mode":{"type":"string","enum":["background","foreground"]}},"required":["tool"]}},
                   "observe":{"type":"boolean","description":"return updated UI tree (default true; false saves tokens)"},
                   "diff":{"type":"boolean","description":"return only changed elements"},
                   "filter":{"type":"string","description":"substring filter on returned elements"},
                   "details":{"type":"boolean","description":"include bounds @(x,y,w,h) and ids"},
                   "max_elements":{"type":"integer"}})),
        tool("get_window_state",
            "Read window semantic UI tree (<20ms, zero vision tokens). Use filter for large UIs or diff:true after edits.",
            json!({"window":win,"window_id":{"type":"integer"},"pid":{"type":"integer"},
                   "max_elements":{"type":"integer"},"filter":{"type":"string"},"diff":{"type":"boolean"},
                   "details":{"type":"boolean"},"screenshot":{"type":"boolean"}})),
        tool("list_windows",
            "List visible windows in <1ms.",
            json!({"exe":{"type":"boolean"}})),
        tool("screenshot",
            "FALLBACK ONLY (slow/expensive vision tokens): JPEG screenshot for canvas/games/images. Prefer act/get_window_state for UI.",
            json!({"window":win,"window_id":{"type":"integer"},"pid":{"type":"integer"},"screen":{"type":"boolean"},
                   "mode":{"type":"string","enum":["print","screen"]},"region":{"type":"array","items":{"type":"integer"}},
                   "max_dim":{"type":"integer"},"quality":{"type":"integer"},"max_age_ms":{"type":"integer"}})),
        tool("browser_act",
            "FAST BROWSER BATCH: Auto-connects or runs web steps in Brave/Chrome/Edge with logins intact. Steps: navigate, click, fill, type, select, press, scroll, wait, new_tab, switch_tab, close_tab. Pass observe:false on final actions or diff:true to save tokens. If an existing browser lacks the extension, use desktop `act` directly on the browser window or load `%LOCALAPPDATA%\\fastuse\\extension` in chrome://extensions.",
            json!({"tab":{"type":"integer"},
                   "steps":{"type":"array","items":{"type":"object","properties":{
                       "tool":{"type":"string","enum":["navigate","new_tab","back","forward","reload","switch_tab","close_tab","click","double_click","right_click","hover","fill","type","select","press","scroll","wait"]},
                       "url":{"type":"string"},"target":{"type":"string"},"element_token":{"type":"string"},"text":{"type":"string"},"value":{"type":"string"},
                       "key":{"type":"string"},"direction":{"type":"string"},"amount":{"type":"integer"},"ms":{"type":"integer"},"tab":{"type":"integer"}},"required":["tool"]}},
                   "observe":{"type":"boolean","description":"return page tree (default true; false saves tokens)"},
                   "diff":{"type":"boolean","description":"return only changed page elements"},
                   "filter":{"type":"string","description":"substring filter on returned elements"},
                   "details":{"type":"boolean","description":"include bounds, ids, and all open tabs"},
                   "max_elements":{"type":"integer"},"text":{"type":"boolean"}})),
        tool("browser_state",
            "Read active tab DOM as compact viewport-prioritized tree (<20ms, zero vision tokens).",
            json!({"tab":{"type":"integer"},"max_elements":{"type":"integer"},"text":{"type":"boolean"},
                   "filter":{"type":"string"},"diff":{"type":"boolean"},"details":{"type":"boolean"}})),
    ]
}

pub fn instructions(platform: &str) -> String {
    format!(
        "Background-first {platform} desktop (`act`) and Brave/Chrome/Edge (`browser_act`) automation. Batch multiple steps in one call; pass observe:false on final step or diff:true/filter to minimize tokens. `browser_act` auto-opens a connected browser if needed; if an already-open browser lacks the extension, use desktop `act` (with mouse/keyboard/UIA) directly on that window or load the unpacked extension folder at chrome://extensions / brave://extensions. Use `delegate` when available to offload multi-turn UI loops to local Ollama/GGUF or fast runner models."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tools_are_well_formed() {
        let t = tools();
        let names: Vec<&str> = t["tools"].as_array().unwrap().iter().map(|x| x["name"].as_str().unwrap()).collect();
        assert!(names.contains(&"act"));
        assert!(names.contains(&"browser_act"));
        for tool in t["tools"].as_array().unwrap() {
            assert!(tool["description"].as_str().map_or(false, |d| !d.is_empty()));
            assert_eq!(tool["inputSchema"]["type"], "object");
        }
    }

    #[test]
    fn base_tools_exclude_delegate() {
        let names: Vec<String> = base_tools().iter().map(|x| x["name"].as_str().unwrap().to_string()).collect();
        assert!(!names.contains(&"delegate".to_string()));
    }

    #[test]
    fn instructions_name_the_platform() {
        assert!(instructions("Linux").contains("Linux desktop"));
    }
}

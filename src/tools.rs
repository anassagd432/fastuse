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

pub fn screenshot_enabled() -> bool {
    std::env::var("FASTUSE_SCREENSHOT")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

pub fn base_tools() -> Vec<Value> {
    let tool = |n: &str, d: &str, p: Value| json!({"name":n,"description":d,"inputSchema":{"type":"object","properties":p}});
    let win = json!({"type":"string","description":"window title substring"});
    let mut list = vec![
        tool("act",
            "ZERO-THINK DESKTOP PLAYBOOK: Plan once upfront, then execute the full OS UI workflow in ONE call without pausing between steps. Auto-waits for dialogs/menus to open. Supports multi-item loops via items:[...] + {{var}} templates, saved macros (macro_save/macro_name), and optional:true steps. Pass observe:false when done.",
            json!({"window":win,"window_id":{"type":"integer"},"pid":{"type":"integer"},
                   "steps":{"type":"array","items":{"type":"object","properties":{
                       "tool":{"type":"string","enum":["click","double_click","right_click","set_value","type_text","press_key","hotkey","scroll","wait"]},
                       "target":{"type":"string"},"element_token":{"type":"string"},"value":{"type":"string"},"text":{"type":"string"},
                       "key":{"type":"string"},"keys":{"type":"string"},"x":{"type":"integer"},"y":{"type":"integer"},
                       "direction":{"type":"string","enum":["up","down","left","right"]},"amount":{"type":"integer"},"ms":{"type":"integer"},
                       "timeout_ms":{"type":"integer"},"optional":{"type":"boolean"},
                       "delivery_mode":{"type":"string","enum":["background","foreground"]}},"required":["tool"]}},
                   "items":{"type":"array","description":"Batch items (objects or strings) to loop `steps` over using {{field}} or {{item}} placeholders with zero LLM turns between items","items":{}},
                   "on_error":{"type":"string","enum":["continue","stop"],"description":"continue (default for multi-item batches) or stop on failed step/item"},
                   "macro_save":{"type":"string","description":"Save `steps` as a named reusable macro on disk"},
                   "macro_name":{"type":"string","description":"Replay a saved macro by name (no need to resend `steps`)"},
                   "observe":{"type":"boolean","description":"return updated UI tree (default true; false saves tokens)"},
                   "diff":{"type":"boolean","description":"return only changed elements"},
                   "filter":{"type":"string","description":"substring filter on returned elements"},
                   "details":{"type":"boolean","description":"include bounds @(x,y,w,h) and ids"},
                   "max_elements":{"type":"integer"}})),
        tool("get_window_state",
            "Read window semantic UI tree (<20ms, zero screenshots/vision tokens). Use filter for large UIs or diff:true after edits.",
            json!({"window":win,"window_id":{"type":"integer"},"pid":{"type":"integer"},
                   "max_elements":{"type":"integer"},"filter":{"type":"string"},"diff":{"type":"boolean"},
                   "details":{"type":"boolean"}})),
        tool("list_windows",
            "List visible windows in <1ms.",
            json!({"exe":{"type":"boolean"}})),
        tool("browser_act",
            "ZERO-THINK BROWSER PLAYBOOK: Plan the entire web task ONCE upfront, then execute all steps or multi-item loops in ONE call without thinking between steps. Auto-waits up to 3.5s per step for modals, inputs, and dynamic DOM elements to mount (matches by Role:Name, text, placeholder, or CSS selector even without reading a snapshot first). For repetitive tasks (cold outreach DMs, multi-search, lead lists), pass items:[{...}] + templated steps ({{field}}) to execute the whole batch at native speed. Pass observe:false when no final read is needed.",
            json!({"tab":{"type":"integer"},
                   "steps":{"type":"array","items":{"type":"object","properties":{
                       "tool":{"type":"string","enum":["navigate","new_tab","back","forward","reload","switch_tab","close_tab","click","double_click","right_click","hover","fill","type","select","press","scroll","wait","extract"]},
                       "url":{"type":"string"},"target":{"type":"string","description":"Role:Name, button/link/placeholder text, or css:..."},"selector":{"type":"string","description":"CSS selector on live DOM"},"element_token":{"type":"string"},
                       "text":{"type":"string"},"value":{"type":"string"},"key":{"type":"string"},"direction":{"type":"string"},"amount":{"type":"integer"},
                       "ms":{"type":"integer"},"timeout_ms":{"type":"integer"},"optional":{"type":"boolean","description":"skip step if target does not appear"},"max_chars":{"type":"integer"},"tab":{"type":"integer"}},"required":["tool"]}},
                   "items":{"type":"array","description":"Batch data (e.g. [{\"url\":\"...\",\"dm\":\"...\"}, ...]) — runs `steps` for every item with {{var}} substitution in a single turn","items":{}},
                   "on_error":{"type":"string","enum":["continue","stop"],"description":"continue (default for multi-item batches) or stop"},
                   "macro_save":{"type":"string","description":"Save `steps` as a named reusable macro on disk"},
                   "macro_name":{"type":"string","description":"Replay a saved macro by name"},
                   "observe":{"type":"boolean","description":"return page tree after execution (default true; pass false to skip)"},
                   "diff":{"type":"boolean","description":"return only changed page elements"},
                   "filter":{"type":"string","description":"substring or '|'-separated filter on returned elements"},
                   "details":{"type":"boolean","description":"include bounds, ids, and all open tabs"},
                   "max_elements":{"type":"integer"},"text":{"type":"boolean"}})),
        tool("browser_state",
            "Read active tab live DOM as a compact semantic map (<15ms, zero screenshots). Dialogs, inputs, and <main> are prioritized automatically. Pass filter:'search|message|send' to jump straight to target controls.",
            json!({"tab":{"type":"integer"},"max_elements":{"type":"integer"},"text":{"type":"boolean"},
                   "filter":{"type":"string"},"diff":{"type":"boolean"},"details":{"type":"boolean"}})),
    ];
    if screenshot_enabled() {
        list.push(tool("screenshot",
            "FALLBACK ONLY (opt-in via FASTUSE_SCREENSHOT=1): JPEG screenshot for non-DOM canvas/games.",
            json!({"window":win,"window_id":{"type":"integer"},"pid":{"type":"integer"},"screen":{"type":"boolean"},
                   "mode":{"type":"string","enum":["print","screen"]},"region":{"type":"array","items":{"type":"integer"}},
                   "max_dim":{"type":"integer"},"quality":{"type":"integer"},"max_age_ms":{"type":"integer"}})));
    }
    list
}

pub fn instructions(platform: &str) -> String {
    format!(
        "Zero-screenshot, zero-think-between-steps {platform} desktop (`act`) and browser (`browser_act`) engine. \
         WORKFLOW RULE: Think ONCE upfront, then execute the entire multi-step chain (navigate -> click -> fill -> press -> extract) in a SINGLE `browser_act` or `act` call. \
         Every step auto-polls the live DOM/UIA for modals, drawers, and inputs to appear, so you do NOT need to stop and read the page between steps. \
         For repetitive tasks (sending drafted DMs, searching multiple queries, processing a list of URLs), pass `items: [...]` with `{{{{var}}}}` placeholders in `steps` so fastuse executes all items back-to-back in one turn. \
         Save recurring workflows with `macro_save` and replay with `macro_name`. Never take screenshots."
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
        assert!(!names.contains(&"screenshot"));
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

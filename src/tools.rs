//! MCP tool declarations and server instructions.

use serde_json::{json, Value};

pub fn tools() -> Value {
    let tool = |n: &str, d: &str, p: Value| json!({"name":n,"description":d,"inputSchema":{"type":"object","properties":p}});
    let win = json!({"type":"string","description":"part of the window title"});
    json!({"tools":[
        tool("act",
            "PRIMARY FAST DESKTOP TOOL: Executes a multi-step UI task in ONE call in milliseconds in the background without stealing the mouse or focus. ALWAYS prefer this tool over screenshot. steps: array of {tool, target, value/text/keys}. Step tools: click, double_click, right_click, set_value (value), type_text (text), press_key (key), hotkey (keys), scroll, wait. target is element label or token (e.g. \"Save\", \"Button:Save\", \"[3]\", or \"sNNNNNNNN:N\"). Returns updated state in a single turn.",
            json!({"window":win,"window_id":{"type":"integer"},"pid":{"type":"integer"},
                   "steps":{"type":"array","items":{"type":"object","properties":{
                       "tool":{"type":"string","enum":["click","double_click","right_click","set_value","type_text","press_key","hotkey","scroll","wait"]},
                       "target":{"type":"string"},"element_token":{"type":"string"},"value":{"type":"string"},"text":{"type":"string"},
                       "key":{"type":"string"},"keys":{"type":"string"},"x":{"type":"integer"},"y":{"type":"integer"},
                       "direction":{"type":"string","enum":["up","down","left","right"]},"amount":{"type":"integer"},"ms":{"type":"integer"},
                       "delivery_mode":{"type":"string","enum":["background","foreground"]}},"required":["tool"]}},
                   "observe":{"type":"boolean","description":"return the UI state after the steps (default true)"}})),
        tool("get_window_state","Fast semantic read of a window's UI tree in <20ms (buttons, textboxes, links, labels) without taking screenshots. 100x faster than visual screenshots because it avoids LLM vision processing.",
            json!({"window":win,"window_id":{"type":"integer"},"pid":{"type":"integer"},"max_elements":{"type":"integer","description":"max interactive elements to return (default: 60)"},"screenshot":{"type":"boolean"}})),
        tool("list_windows","List visible desktop windows in <1ms: title, pid, window_id.", json!({"exe":{"type":"boolean"}})),
        tool("screenshot","SLOW/FALLBACK VISUAL TOOL: Takes a screenshot as JPEG. DO NOT USE FOR NORMAL UI ACTIONS — multimodal vision processing adds 3-5 seconds of LLM delay. Use act or get_window_state instead, which read and act semantically in <10ms. Only call screenshot if you specifically need to inspect non-standard visual graphics, canvas, games, or videos.",
            json!({"window":win,"window_id":{"type":"integer"},"pid":{"type":"integer"},"screen":{"type":"boolean","description":"force whole-desktop capture"},"mode":{"type":"string","enum":["print","screen"],"description":"window capture mode (print=PrintWindow, screen=BitBlt)"},"region":{"type":"array","items":{"type":"integer"},"description":"[x,y,w,h]"},
                   "max_dim":{"type":"integer","description":"max image dimension (default: 960 for fast vision)"},"quality":{"type":"integer","description":"jpeg quality (default: 55)"},"max_age_ms":{"type":"integer","description":"reuse a frame younger than this"}})),
        tool("browser_act","PRIMARY FAST BROWSER TOOL: Runs a full web flow in ONE turn in the user's active browser with their existing logins (Brave, Chrome, Edge). Never takes screenshots. Steps: navigate, click, fill, type, select, press, scroll, wait, new_tab, switch_tab, close_tab. Resolves targets by label or CSS selector.",
            json!({"tab":{"type":"integer"},
                   "steps":{"type":"array","items":{"type":"object","properties":{
                       "tool":{"type":"string","enum":["navigate","new_tab","back","forward","reload","switch_tab","close_tab","click","double_click","right_click","hover","fill","type","select","press","scroll","wait"]},
                       "url":{"type":"string"},"target":{"type":"string"},"element_token":{"type":"string"},"text":{"type":"string"},"value":{"type":"string"},
                       "key":{"type":"string"},"direction":{"type":"string"},"amount":{"type":"integer"},"ms":{"type":"integer"},"tab":{"type":"integer"}},"required":["tool"]}},
                   "observe":{"type":"boolean","description":"return the page tree after the steps (default true)"},
                   "max_elements":{"type":"integer"},"text":{"type":"boolean","description":"include paragraph text in the tree"}})),
        tool("browser_state","Read active browser tab DOM as compact clickable tree in <20ms without screenshots. Only needed to explore; browser_act already returns updated state.",
            json!({"tab":{"type":"integer","description":"tab id from the tab list; default: the active allowed tab"},
                   "max_elements":{"type":"integer"},"text":{"type":"boolean","description":"also include paragraph and list text"}}))
    ]})
}

pub fn instructions(platform: &str) -> String {
    format!(
        "Drives {platform} app windows fast and in the background. Call the `act` tool directly; no need to list files or read tool docs first. Example arguments: {{\"window\":\"<part of window title>\",\"steps\":[{{\"tool\":\"set_value\",\"target\":\"<element name>\",\"value\":\"text\"}},{{\"tool\":\"click\",\"target\":\"<element name>\"}}]}}. One call finds the window, runs all steps and returns the new UI tree, so you rarely need get_window_state or list_windows. If a target name is wrong the reply lists the elements. Step tools: click, double_click, right_click, set_value (value), type_text (text), press_key (key), hotkey (keys). For anything inside a web page use `browser_act` instead (steps like navigate, click, fill, press): it works in the user's own browser with their logins."
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
        for tool in t["tools"].as_array().unwrap() {
            assert!(tool["description"].as_str().map_or(false, |d| !d.is_empty()));
            assert_eq!(tool["inputSchema"]["type"], "object");
        }
    }

    #[test]
    fn instructions_name_the_platform() {
        assert!(instructions("Linux").starts_with("Drives Linux app windows"));
    }
}

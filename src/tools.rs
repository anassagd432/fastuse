//! MCP tool declarations and server instructions.

use serde_json::{json, Value};

pub fn tools() -> Value {
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

pub fn instructions(platform: &str) -> String {
    format!(
        "Drives {platform} app windows fast and in the background. Call the `act` tool directly; no need to list files or read tool docs first. Example arguments: {{\"window\":\"<part of window title>\",\"steps\":[{{\"tool\":\"set_value\",\"target\":\"<element name>\",\"value\":\"text\"}},{{\"tool\":\"click\",\"target\":\"<element name>\"}}]}}. One call finds the window, runs all steps and returns the new UI tree, so you rarely need get_window_state or list_windows. If a target name is wrong the reply lists the elements. Step tools: click, double_click, right_click, set_value (value), type_text (text), press_key (key), hotkey (keys)."
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

//! Zero-think playbooks: parameterized multi-item batch expansion (`items` + `{{var}}`) and saved macros.

use serde_json::{json, Value};
use std::path::PathBuf;

pub struct Playbook {
    /// One step-list per item (length 1 when `items` is omitted).
    pub batches: Vec<Vec<Value>>,
    /// Human-readable label for each batch item when `items` was passed.
    pub item_labels: Vec<String>,
    /// Whether to continue to the next item (or next step) when an error occurs.
    pub continue_on_error: bool,
    /// Optional note if a macro was saved or loaded.
    pub macro_note: Option<String>,
}

pub fn macros_dir() -> PathBuf {
    if let Some(custom) = std::env::var_os("FASTUSE_MACROS_DIR") {
        return PathBuf::from(custom);
    }
    crate::browser::data_dir().join("macros")
}

fn sanitize_macro_name(name: &str) -> Result<String, String> {
    let clean: String = name
        .trim()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect();
    if clean.is_empty() {
        return Err("macro name cannot be empty".into());
    }
    Ok(clean)
}

pub fn save_macro(name: &str, steps: &[Value]) -> Result<PathBuf, String> {
    let clean = sanitize_macro_name(name)?;
    let dir = macros_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create macros dir: {e}"))?;
    let path = dir.join(format!("{clean}.json"));
    let body = serde_json::to_string_pretty(&json!({
        "name": clean,
        "steps": steps,
    }))
    .map_err(|e| e.to_string())?;
    std::fs::write(&path, body).map_err(|e| format!("cannot write macro '{clean}': {e}"))?;
    Ok(path)
}

pub fn load_macro(name: &str) -> Result<Vec<Value>, String> {
    let clean = sanitize_macro_name(name)?;
    let path = macros_dir().join(format!("{clean}.json"));
    let raw = std::fs::read_to_string(&path).map_err(|_| {
        let avail = list_macro_names();
        if avail.is_empty() {
            format!("no saved macro named '{clean}' (no macros saved yet; pass steps + macro_save:\"{clean}\" first)")
        } else {
            format!("no saved macro named '{clean}' (available: {})", avail.join(", "))
        }
    })?;
    let v: Value = serde_json::from_str(&raw).map_err(|e| format!("corrupt macro '{clean}': {e}"))?;
    v.get("steps")
        .and_then(|s| s.as_array())
        .cloned()
        .ok_or_else(|| format!("macro '{clean}' has no steps array"))
}

pub fn list_macro_names() -> Vec<String> {
    let mut names = Vec::new();
    if let Ok(rd) = std::fs::read_dir(macros_dir()) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) == Some("json") {
                if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                    names.push(stem.to_string());
                }
            }
        }
    }
    names.sort();
    names
}

fn val_to_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn expand_str(template: &str, item: &Value, index: usize) -> String {
    if !template.contains("{{") {
        return template.to_string();
    }
    let mut out = template.replace("{{index}}", &(index + 1).to_string());
    match item {
        Value::Object(map) => {
            for (k, v) in map {
                let s = val_to_str(v);
                out = out.replace(&format!("{{{{{k}}}}}"), &s);
                out = out.replace(&format!("{{{{item.{k}}}}}"), &s);
            }
        }
        other => {
            let s = val_to_str(other);
            out = out.replace("{{item}}", &s);
            out = out.replace("{{value}}", &s);
        }
    }
    out
}

fn expand_value(v: &Value, item: &Value, index: usize) -> Value {
    match v {
        Value::String(s) => Value::String(expand_str(s, item, index)),
        Value::Array(arr) => Value::Array(arr.iter().map(|x| expand_value(x, item, index)).collect()),
        Value::Object(map) => {
            let mut next = serde_json::Map::with_capacity(map.len());
            for (k, val) in map {
                next.insert(k.clone(), expand_value(val, item, index));
            }
            Value::Object(next)
        }
        other => other.clone(),
    }
}

pub fn expand_steps(steps: &[Value], item: &Value, index: usize) -> Vec<Value> {
    steps.iter().map(|s| expand_value(s, item, index)).collect()
}

fn summarize_item(item: &Value, index: usize) -> String {
    let raw = match item {
        Value::String(s) => s.clone(),
        Value::Object(map) => map
            .iter()
            .take(2)
            .map(|(k, v)| format!("{k}={}", val_to_str(v)))
            .collect::<Vec<_>>()
            .join(", "),
        other => val_to_str(other),
    };
    let clipped: String = raw.chars().take(50).collect();
    if clipped.is_empty() {
        format!("item {}", index + 1)
    } else {
        clipped
    }
}

pub fn resolve_playbook(a: &Value) -> Result<Playbook, String> {
    let macro_load = a
        .get("macro_name")
        .or_else(|| a.get("macro"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let macro_save_name = a
        .get("macro_save")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let mut macro_note = None;
    let base_steps: Vec<Value> = match a.get("steps").and_then(|s| s.as_array()) {
        Some(arr) if !arr.is_empty() => {
            if let Some(save_as) = macro_save_name {
                save_macro(save_as, arr)?;
                macro_note = Some(format!("saved macro '{save_as}' ({} steps)", arr.len()));
            }
            arr.clone()
        }
        _ => {
            if let Some(name) = macro_load {
                let loaded = load_macro(name)?;
                macro_note = Some(format!("loaded macro '{name}' ({} steps)", loaded.len()));
                loaded
            } else {
                return Err("steps is required (or pass macro_name to replay a saved macro)".into());
            }
        }
    };

    let items = a.get("items").and_then(|x| x.as_array()).filter(|arr| !arr.is_empty());
    let explicit_on_err = a.get("on_error").and_then(|v| v.as_str());
    let continue_on_error = match explicit_on_err {
        Some("continue") => true,
        Some("stop") => false,
        _ => items.map_or(false, |arr| arr.len() > 1),
    };

    if let Some(arr) = items {
        let mut batches = Vec::with_capacity(arr.len());
        let mut labels = Vec::with_capacity(arr.len());
        for (idx, item) in arr.iter().enumerate() {
            batches.push(expand_steps(&base_steps, item, idx));
            labels.push(summarize_item(item, idx));
        }
        Ok(Playbook {
            batches,
            item_labels: labels,
            continue_on_error,
            macro_note,
        })
    } else {
        Ok(Playbook {
            batches: vec![base_steps],
            item_labels: Vec::new(),
            continue_on_error,
            macro_note,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_object_and_scalar_templates() {
        let steps = vec![
            json!({"tool": "navigate", "url": "https://x.com/{{handle}}"}),
            json!({"tool": "fill", "target": "Message", "text": "#{{index}}: {{item.dm}}"}),
        ];
        let item = json!({"handle": "anass", "dm": "Hey there!"});
        let out = expand_steps(&steps, &item, 0);
        assert_eq!(out[0]["url"], "https://x.com/anass");
        assert_eq!(out[1]["text"], "#1: Hey there!");

        let scalar_steps = vec![json!({"tool": "fill", "target": "Search", "text": "{{item}}"})];
        let out2 = expand_steps(&scalar_steps, &json!("fastuse rust"), 2);
        assert_eq!(out2[0]["text"], "fastuse rust");
    }

    #[test]
    fn resolves_multi_item_playbook_with_continue_default() {
        let args = json!({
            "items": [
                {"q": "alpha"},
                {"q": "beta"}
            ],
            "steps": [
                {"tool": "fill", "target": "Search", "text": "{{q}}"}
            ]
        });
        let pb = resolve_playbook(&args).unwrap();
        assert_eq!(pb.batches.len(), 2);
        assert!(pb.continue_on_error);
        assert_eq!(pb.batches[0][0]["text"], "alpha");
        assert_eq!(pb.batches[1][0]["text"], "beta");
    }

    #[test]
    fn saves_and_replays_macro_with_items() {
        let tmp = std::env::temp_dir().join(format!("fastuse-macros-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("FASTUSE_MACROS_DIR", &tmp);

        let save_args = json!({
            "macro_save": "cold_dm_flow",
            "steps": [
                {"tool": "navigate", "url": "{{url}}"},
                {"tool": "click", "target": "Message"},
                {"tool": "fill", "target": "Start a new message", "text": "{{dm}}"}
            ]
        });
        let pb_save = resolve_playbook(&save_args).unwrap();
        assert!(pb_save.macro_note.as_deref().unwrap_or("").contains("saved macro 'cold_dm_flow'"));

        let replay_args = json!({
            "macro_name": "cold_dm_flow",
            "items": [
                {"url": "https://x.com/u1", "dm": "msg 1"},
                {"url": "https://x.com/u2", "dm": "msg 2"}
            ]
        });
        let pb_replay = resolve_playbook(&replay_args).unwrap();
        assert_eq!(pb_replay.batches.len(), 2);
        assert_eq!(pb_replay.batches[1][0]["url"], "https://x.com/u2");
        assert_eq!(pb_replay.batches[1][2]["text"], "msg 2");

        std::env::remove_var("FASTUSE_MACROS_DIR");
        let _ = std::fs::remove_dir_all(&tmp);
    }
}

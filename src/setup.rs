//! `fastuse setup`: unpacks the browser extension, registers native hosts, and wires MCP + skills
//! across installed coding agents (Codex CLI/App, Claude Code, Claude Desktop, Cursor, Windsurf, Antigravity).

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

const SKILL_MD: &str = include_str!("../skills/fastuse/SKILL.md");

const AGY_RULE_BLOCK: &str = r#"<!-- FASTUSE_COMPUTER_USE_START -->
## Default Computer Use & Browser Use Engine (`fastuse` MCP)
`fastuse` (MCP server `fastuse`) is Antigravity's **primary, default Computer Use and Browser Use tool** on both Windows and macOS.
The user never needs to mention `fastuse` by name. Automatically route requests as follows:
1. **Browser tasks ("access my Brave browser tab", "check my Chrome tab", "click/fill/navigate on the page", "open this site in my browser"):**
   - Immediately use MCP server `fastuse` tools `browser_state` and `browser_act` (if an existing browser window lacks the extension yet, use desktop `act`/`get_window_state` directly or load `%LOCALAPPDATA%\fastuse\extension` at `chrome://extensions`).
   - Do **not** launch Playwright, Puppeteer, or headless CDP browsers when the user asks to read or interact with their browser tab.
2. **Desktop / OS tasks ("do this on my computer", "operate <app>", "click/type/scroll in <window>", "what windows are open", "screenshot <window>"):**
   - Immediately use MCP server `fastuse` tools `act`, `get_window_state`, `list_windows`, and `screenshot`.
<!-- FASTUSE_COMPUTER_USE_END -->
"#;

fn home_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE").map(PathBuf::from)
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}

fn command_exists(name: &str) -> bool {
    #[cfg(windows)]
    let probe = std::process::Command::new("where").arg(name).output();
    #[cfg(not(windows))]
    let probe = std::process::Command::new("which").arg(name).output();
    probe.map(|o| o.status.success()).unwrap_or(false)
}

pub fn upsert_mcp_json(raw: &str, exe: &str) -> Result<(String, bool), String> {
    let mut root: Value = if raw.trim().is_empty() {
        json!({})
    } else {
        serde_json::from_str(raw).map_err(|e| format!("invalid JSON: {e}"))?
    };
    let obj = root.as_object_mut().ok_or("root JSON value is not an object")?;
    let servers = obj
        .entry("mcpServers")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or("mcpServers is not an object")?;
    let desired = json!({"command": exe, "args": []});
    if servers.get("fastuse") == Some(&desired) {
        return Ok((raw.to_string(), false));
    }
    servers.insert("fastuse".to_string(), desired);
    let formatted = serde_json::to_string_pretty(&root).map_err(|e| e.to_string())?;
    Ok((format!("{formatted}\n"), true))
}

pub fn upsert_codex_toml(raw: &str, exe: &str) -> (String, bool) {
    let has_entry = raw.lines().any(|l| {
        let t = l.trim();
        t == "[mcp_servers.fastuse]" || t == "[mcp_servers.\"fastuse\"]" || t == "[mcp_servers.'fastuse']"
    });
    if has_entry {
        return (raw.to_string(), false);
    }
    let cmd_literal = if exe.contains('\'') {
        format!("\"{}\"", exe.replace('\\', "/").replace('"', "\\\""))
    } else {
        format!("'{exe}'")
    };
    let mut out = raw.to_string();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(&format!("[mcp_servers.fastuse]\ncommand = {cmd_literal}\nargs = []\n"));
    (out, true)
}

fn configure_json_client(label: &str, path: &Path, exe: &str) -> String {
    let raw = std::fs::read_to_string(path).unwrap_or_default();
    match upsert_mcp_json(&raw, exe) {
        Ok((next, changed)) => {
            if !changed {
                return format!("{label}: already registered");
            }
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match std::fs::write(path, next) {
                Ok(_) => format!("{label}: registered ({})", path.display()),
                Err(e) => format!("{label}: failed writing {}: {e}", path.display()),
            }
        }
        Err(e) => format!("{label}: skipped ({}): {e}", path.display()),
    }
}

fn install_skill(dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("SKILL.md"), SKILL_MD).map_err(|e| e.to_string())
}

pub fn run(no_agy: bool) -> Result<String, String> {
    let browser_report = crate::browser::install()?;
    let exe = std::env::current_exe()
        .map_err(|e| format!("cannot resolve executable path: {e}"))?
        .display()
        .to_string();

    let mut out = vec![browser_report, String::new(), "AI client integration:".to_string()];
    let Some(home) = home_dir() else {
        out.push("  skipped: could not locate home directory".to_string());
        return Ok(out.join("\n"));
    };

    // 1. ChatGPT Codex CLI & Desktop app (~/.codex/config.toml + skill)
    let codex_dir = home.join(".codex");
    if codex_dir.exists() || command_exists("codex") {
        let _ = std::fs::create_dir_all(&codex_dir);
        let cfg = codex_dir.join("config.toml");
        let raw = std::fs::read_to_string(&cfg).unwrap_or_default();
        let (next, changed) = upsert_codex_toml(&raw, &exe);
        if changed {
            match std::fs::write(&cfg, next) {
                Ok(_) => out.push(format!("  Codex MCP: registered ({})", cfg.display())),
                Err(e) => out.push(format!("  Codex MCP: FAILED ({e})")),
            }
        } else {
            out.push("  Codex MCP: already registered".to_string());
        }
        let _ = install_skill(&codex_dir.join("skills").join("fastuse"));
        out.push("  Codex skill: installed".to_string());
    }

    // 2. Claude Code (~/.claude.json + ~/.claude/skills/fastuse/SKILL.md)
    let claude_json = home.join(".claude.json");
    let claude_dir = home.join(".claude");
    if claude_json.exists() || claude_dir.exists() || command_exists("claude") {
        out.push(format!("  {}", configure_json_client("Claude Code MCP", &claude_json, &exe)));
        let _ = install_skill(&claude_dir.join("skills").join("fastuse"));
    }

    // 3. Claude Desktop
    #[cfg(windows)]
    let claude_desktop = std::env::var_os("APPDATA").map(|p| PathBuf::from(p).join("Claude"));
    #[cfg(target_os = "macos")]
    let claude_desktop = Some(home.join("Library/Application Support/Claude"));
    #[cfg(all(unix, not(target_os = "macos")))]
    let claude_desktop = Some(home.join(".config/Claude"));
    if let Some(cd) = claude_desktop.filter(|p| p.exists()) {
        out.push(format!(
            "  {}",
            configure_json_client("Claude Desktop MCP", &cd.join("claude_desktop_config.json"), &exe)
        ));
    }

    // 4. Cursor
    let cursor_dir = home.join(".cursor");
    if cursor_dir.exists() {
        out.push(format!("  {}", configure_json_client("Cursor MCP", &cursor_dir.join("mcp.json"), &exe)));
        let _ = install_skill(&cursor_dir.join("skills").join("fastuse"));
    }

    // 5. Windsurf
    let windsurf_dir = home.join(".codeium").join("windsurf");
    if windsurf_dir.exists() {
        out.push(format!(
            "  {}",
            configure_json_client("Windsurf MCP", &windsurf_dir.join("mcp_config.json"), &exe)
        ));
    }

    // 6. Antigravity
    if !no_agy && (home.join(".gemini").exists() || command_exists("agy")) {
        let agy_skill_1 = home.join(".gemini/antigravity/global_skills/fastuse");
        let agy_skill_2 = home.join(".gemini/config/skills/fastuse");
        let _ = install_skill(&agy_skill_1);
        let _ = install_skill(&agy_skill_2);

        let gemini_md = home.join(".gemini/config/GEMINI.md");
        if let Some(parent) = gemini_md.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let existing = std::fs::read_to_string(&gemini_md).unwrap_or_default();
        if !existing.contains("FASTUSE_COMPUTER_USE_START") {
            let sep = if existing.is_empty() || existing.ends_with('\n') { "" } else { "\n" };
            let _ = std::fs::write(&gemini_md, format!("{existing}{sep}\n{AGY_RULE_BLOCK}"));
        }
        if command_exists("agy") {
            #[cfg(windows)]
            let ok = std::process::Command::new("cmd")
                .args(["/d", "/c", "agy", "mcp", "add", "fastuse", &exe])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            #[cfg(not(windows))]
            let ok = std::process::Command::new("agy")
                .args(["mcp", "add", "fastuse", &exe])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            out.push(format!(
                "  Antigravity MCP & skills: {}",
                if ok { "registered" } else { "skills installed (run `agy mcp add fastuse` if needed)" }
            ));
        } else {
            out.push("  Antigravity skills & GEMINI.md rules: installed".to_string());
        }
    }

    Ok(out.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_mcp_json_preserves_existing_servers() {
        let orig = r#"{"mcpServers":{"other":{"command":"node"}}}"#;
        let (next, changed) = upsert_mcp_json(orig, r"C:\bin\fastuse.exe").unwrap();
        assert!(changed);
        let v: Value = serde_json::from_str(&next).unwrap();
        assert_eq!(v["mcpServers"]["other"]["command"], "node");
        assert_eq!(v["mcpServers"]["fastuse"]["command"], r"C:\bin\fastuse.exe");
        let (_, changed2) = upsert_mcp_json(&next, r"C:\bin\fastuse.exe").unwrap();
        assert!(!changed2);
    }

    #[test]
    fn upsert_codex_toml_adds_and_stays_idempotent() {
        let orig = "model = \"gpt-6.1-sol\"\n";
        let (next, changed) = upsert_codex_toml(orig, r"C:\Users\Anass\fastuse.exe");
        assert!(changed);
        assert!(next.contains("[mcp_servers.fastuse]"));
        assert!(next.contains("command = 'C:\\Users\\Anass\\fastuse.exe'"));
        let (_, changed2) = upsert_codex_toml(&next, r"C:\Users\Anass\fastuse.exe");
        assert!(!changed2);
    }
}

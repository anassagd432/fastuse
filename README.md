<div align="center">

# ⚡ fastuse

**Ultra-fast, zero-focus-steal Computer Use & Browser Use MCP server for Claude Code, Cursor, Windsurf, Claude Desktop, and Antigravity.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%2011%20%7C%20macOS-informational.svg)]()
[![Rust](https://img.shields.io/badge/Language-Rust%202021-orange.svg)]()
[![MCP](https://img.shields.io/badge/Protocol-Model%20Context%20Protocol-8A2BE2.svg)]()
[![Zero Focus Steal](https://img.shields.io/badge/Input-Background--First%20(No%20Cursor%20Steal)-success.svg)]()

<br>

![fastuse Demo](assets/demo.gif)

</div>

---

## ⚡ Why fastuse?

Traditional computer-use tools hijack your real mouse cursor, steal window focus, spin up empty headless browsers without your logins, and burn 4–6 LLM screenshot round-trips per click.

`fastuse` fixes the architecture:

* **Zero Cursor Stealing:** Dispatches clicks, typing, and UI tree reads in the background via native OS APIs (`UI Automation` on Windows, `AXUIElement` + `CoreGraphics` on macOS). Keep working undisturbed while your agent operates windows behind the scenes.
* **Your Real Browser, Logged In:** Connects directly to **Brave, Chrome, or Edge** over a Chrome Native Messaging Host (`fastuse browser-open`), keeping your logins, cookies, and active tabs intact — with an embedded extension inside the single binary.
* **Single-Turn Batches (`act` / `browser_act`):** Executes full sequences (`set_value` + `click` + `wait` + `observe`) in one MCP call (~50ms native execution vs. multi-second screenshot loops).
* **Visible Agent Overlay:** An independent glowing blue cursor and screen-edge aura track every automated action cleanly — while hidden from OS screenshots via `WDA_EXCLUDEFROMCAPTURE`.

---

## 📊 Measured Speed

Tested on Windows 11 (AMD Ryzen 9 / RTX 3070), medians over persistent MCP sessions, compared to [cua-driver](https://github.com/trycua/cua) 0.37.0 (reproducible via `bench/`):

| Operation | Standard Driver (`cua`) | `fastuse` | Speedup |
|---|---|---|---|
| **List windows** | 437 ms | **0.9 ms** | **~485x faster** |
| **Read window UI tree** | 160 ms | **29 ms** | **~5.5x faster** |
| **Click by element** | 16 ms | **1.8 ms** | **~9x faster** |
| **`set_value` (20 chars)** | 16 ms | **2.2 ms** | **~7x faster** |
| **Type 20 chars** | 153 ms | **3.6 ms** | **~42x faster** |
| **Batch flow (`set` + `click` + `observe`)** | *Multiple round-trips* | **~66 ms** | **Single turn** |

---

## 🚀 30-Second Quickstart

### Step 1 — Install `fastuse`

**Windows (PowerShell):**
```powershell
irm https://github.com/anassagd432/fastuse/releases/latest/download/install.ps1 | iex
```

**macOS / Linux (Terminal):** *(requires [Rust](https://rustup.rs) if compiling from source)*
```bash
curl -fsSL https://raw.githubusercontent.com/anassagd432/fastuse/main/install.sh | bash
```

Both installers unpack the binary, add `fastuse` to your `PATH`, extract the embedded browser extension, and register native messaging hosts for Chrome, Brave, and Edge.

---

### Step 2 — Connect Your AI Client (MCP)

<details open>
<summary><b>Claude Code (CLI)</b></summary>

*Windows:*
```powershell
claude mcp add fastuse -- "$env:LOCALAPPDATA\Programs\fastuse\fastuse.exe"
```

*macOS / Linux:*
```bash
claude mcp add fastuse -- "$HOME/.local/bin/fastuse"
```

</details>

<details>
<summary><b>Cursor / Claude Desktop / Windsurf / VS Code (<code>mcpServers</code> JSON)</b></summary>

Add `fastuse` to your client's `mcp_config.json` / `claude_desktop_config.json`:

*Windows:*
```json
{
  "mcpServers": {
    "fastuse": {
      "command": "fastuse"
    }
  }
}
```
*(Or use the full path `%LOCALAPPDATA%\Programs\fastuse\fastuse.exe` if your app hasn't picked up the updated `PATH` yet.)*

*macOS:*
```json
{
  "mcpServers": {
    "fastuse": {
      "command": "/Users/YOUR_USER/.local/bin/fastuse"
    }
  }
}
```

</details>

<details>
<summary><b>Antigravity (One-Prompt Setup)</b></summary>

`install.ps1` and `install.sh` automatically register the `fastuse` MCP server, `SKILL.md`, and routing rules with Antigravity (`agy`). To verify or configure from scratch inside chat:

```markdown
Verify or install `fastuse`:
1. Ensure `fastuse` is installed (`fastuse --version`) or run the installer for my OS.
2. Ensure `fastuse` MCP server is registered and `skills/fastuse/SKILL.md` is installed.
3. Run `fastuse doctor` and `fastuse browser-open` to verify desktop and browser readiness.
```

</details>

---

### Step 3 — Launch Your Browser Connected

No manual `chrome://extensions` Developer Mode clicks needed. Run:

```bash
fastuse browser-open
```

* **Browser closed:** Opens Brave/Chrome/Edge directly with the extension attached and all your logins/cookies preserved.
* **Browser already open:** Chromium ignores `--load-extension` on running processes, so `fastuse` launches an isolated automation window immediately — or run `fastuse browser-open --relaunch` to restart your main profile with `--restore-last-session` and keep all logins intact.

Verify everything at any time:
```bash
fastuse doctor
```

Example output:
```text
Windows: UI Automation OK, Per-Monitor v2 HiDPI aware, background input OK, overlay ready (14 visible windows).
Browser use:
  native host manifest: installed (C:\Users\you\AppData\Local\fastuse\com.fastuse.browser.json)
  connected: Brave (host pid file ...)
```

---

## 🛠️ Tool Catalog

### 1. Browser Automation (`browser_act`, `browser_state`)
Operates inside your Chromium browser via the embedded MV3 extension + native messaging bridge:

| Tool | Description |
|---|---|
| `browser_act` | Execute complete web flows in one call (`navigate`, `new_tab`, `switch_tab`, `close_tab`, `back`, `forward`, `reload`, `click`, `double_click`, `right_click`, `hover`, `fill`, `type`, `select`, `press`, `scroll`, `wait`) and receive the updated DOM tree. |
| `browser_state` | Reads visible tabs, headings, buttons, inputs, links, and selectable controls as a compact indexed tree. |

```json
{
  "steps": [
    { "tool": "navigate", "url": "https://github.com/anassagd432/fastuse" },
    { "tool": "click", "target": "Issues" },
    { "tool": "fill", "target": "Search", "value": "is:open" },
    { "tool": "press", "key": "Enter" }
  ]
}
```

### 2. Desktop Automation (`act`, `get_window_state`, `list_windows`, `screenshot`)
Operates any Win32, WPF, WinUI, Cocoa, AppKit, or Electron desktop application:

| Tool | Description |
|---|---|
| `act` | Execute an end-to-end desktop batch in one turn (`click`, `double_click`, `right_click`, `set_value`, `type_text`, `press_key`, `hotkey`, `scroll`, `wait`) against a window by title substring, `window_id`, or `pid`. |
| `get_window_state` | Reads an app's UI hierarchy as indexed controls with roles, names, and bounds. |
| `list_windows` | Lists all visible top-level windows with title, PID, process name, and bounds in `< 1 ms`. |
| `screenshot` | Background window capture (`PrintWindow` / `BitBlt`) or desktop capture, cropped and JPEG-encoded (used only as fallback for canvas/media views). |

```json
{
  "window": "Notepad",
  "steps": [
    { "tool": "type_text", "text": "Written in the background with zero focus steal.\n" },
    { "tool": "hotkey", "keys": "ctrl+s" }
  ]
}
```

---

## 🔧 Architecture

```text
   ┌─────────────────────────────────────────────────────────────┐
   │        MCP CLIENT (Claude Code / Cursor / Antigravity)      │
   └───────────────┬─────────────────────────────┬───────────────┘
                   │ stdio JSON-RPC              │ stdio JSON-RPC
                   ▼                             ▼
   ┌───────────────────────────────┐ ┌───────────────────────────┐
   │     DESKTOP ENGINE (act)      │ │   BROWSER ENGINE (MV3)    │
   │   Windows: UIA + Win32 Msg    │ │   Brave · Chrome · Edge   │
   │   macOS:   AX + CoreGraphics  │ │   Native Messaging Host   │
   └───────────────┬───────────────┘ └─────────────┬─────────────┘
                   └──────────────┬────────────────┘
                                  ▼
      ┌───────────────────────────────────────────────────────┐
      │   VISUAL OVERLAY (Excluded from screen capture)       │
      │   Independent Brand-Blue Cursor + Edge Glow Aura      │
      └───────────────────────────────────────────────────────┘
```

---

## 🔒 Safety & Privacy

* **Background-First & Focus Isolation:** Operations never steal focus unless `delivery_mode: "foreground"` is requested.
* **Sensitive Site Blocklist:** Built-in refusal prevents automated reading or input on banking, payment, password manager, and credential domains (`paypal.com`, `stripe.com`, `accounts.google.com`, `1password.com`, `bitwarden.com`, etc.).
* **Visual Accountability:** Always shows a translucent edge glow and cursor badge while an agent is interacting with your screen.
* **Capture Privacy:** The overlay uses `WDA_EXCLUDEFROMCAPTURE` so screenshots captured for vision fallbacks stay clean.

---

## 📄 License

MIT License. See [LICENSE](LICENSE) for details.

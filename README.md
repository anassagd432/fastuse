<div align="center">

# ⚡ fastuse

**Ultra-fast, zero-focus-steal Computer Use & Browser Use MCP server for [Antigravity](https://antigravity.google) and modern AI agents.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%2011%20%7C%20macOS-informational.svg)]()
[![Rust](https://img.shields.io/badge/Language-Rust%202021-orange.svg)]()
[![MCP](https://img.shields.io/badge/Protocol-Model%20Context%20Protocol-8A2BE2.svg)]()
[![Zero Focus Steal](https://img.shields.io/badge/Input-Background--First%20(No%20Cursor%20Steal)-success.svg)]()

<br>

```
   ┌─────────────────────────────────────────────────────────────┐
   │                     ANTIGRAVITY AGENT                       │
   │  "access my Brave browser tab" · "click save in Notepad"    │
   └───────────────┬─────────────────────────────┬───────────────┘
                   │ MCP (JSON-RPC)              │ MCP (JSON-RPC)
                   ▼                             ▼
   ┌───────────────────────────────┐ ┌───────────────────────────┐
   │     DESKTOP ENGINE (act)      │ │   BROWSER ENGINE (Chrome) │
   │   Windows: UIA + Win32 Msg    │ │   Brave · Chrome · Edge   │
   │   macOS:   AX + CoreGraphics  │ │   Native Messaging Bridge │
   ├───────────────────────────────┤ ├───────────────────────────┤
   │  ✓ Background clicks & typing │ │  ✓ Works with your logins │
   │  ✓ Never moves your OS cursor │ │  ✓ Real-time DOM read/act │
   │  ✓ Millisecond UI tree cache  │ │  ✓ Shadow DOM blue overlay│
   └───────────────────────────────┘ └───────────────────────────┘
                   │                             │
                   └──────────────┬──────────────┘
                                  ▼
      ┌───────────────────────────────────────────────────────┐
      │                   VISUAL OVERLAY                      │
      │   🟦 Independent Agent Cursor (Brand-Blue Glow)       │
      │   🟦 Screen-Edge Aura · "Controlled by fastuse"       │
      │   🟦 Real User Mouse & Keyboard Remain 100% Free      │
      └───────────────────────────────────────────────────────┘
```

</div>

---

## 🚀 One-Prompt Setup in Antigravity

You don't need to manually configure files, compile binaries, or write MCP configs. **Simply copy and paste the prompt below directly into your Antigravity chat:**

```markdown
Install and configure the `fastuse` MCP server on this computer:
1. Detect my OS (Windows or macOS).
2. If on Windows, run in terminal:
   powershell -ExecutionPolicy Bypass -Command "irm https://github.com/anassagd432/fastuse/releases/latest/download/install.ps1 | iex"
   If on macOS, run in terminal:
   curl -fsSL https://raw.githubusercontent.com/anassagd432/fastuse/main/install.sh | bash
3. Ensure `fastuse` is registered in my Antigravity MCP config (~/.gemini/config/mcp_config.json) pointing to the installed fastuse binary.
4. Ensure the fastuse skill is placed in ~/.gemini/config/skills/fastuse/SKILL.md.
5. In my ~/.gemini/config/GEMINI.md file, ensure this auto-routing rule is present:
<!-- FASTUSE_COMPUTER_USE_START -->
## Default Computer Use & Browser Use Engine (`fastuse` MCP)
`fastuse` (MCP server `fastuse`) is Antigravity's primary, default Computer Use and Browser Use tool on both Windows and macOS.
The user never needs to mention `fastuse` by name. Automatically route requests as follows:
1. Browser tasks ("access my Brave browser tab", "check my Chrome tab", "click/fill/navigate on the page", "open this site in my browser"):
   - Immediately use MCP server `fastuse` tools `browser_state` and `browser_act`.
   - Do not launch Playwright, Puppeteer, or headless CDP browsers when the user asks to read or interact with their browser tab.
2. Desktop / OS tasks ("do this on my computer", "operate <app>", "click/type/scroll in <window>", "what windows are open", "screenshot <window>"):
   - Immediately use MCP server `fastuse` tools `act`, `get_window_state`, `list_windows`, and `screenshot`.
   - Actions run in the background by default without stealing cursor or focus.
<!-- FASTUSE_COMPUTER_USE_END -->
6. Run `fastuse doctor` and confirm the installation is verified and ready.
```

---

## ⚡ Why fastuse?

Traditional computer-use tools hijack your real mouse cursor, steal window focus, launch heavy headless Chromium browsers that lose all your active logins, and make 4 to 6 round trips per action.

`fastuse` solves this with an architectural redesign:
* **Zero Cursor Stealing:** Uses native accessibility APIs (`UI Automation` on Windows, `AXUIElement` + `CoreGraphics` on macOS) and background message dispatching. You can continue typing in your code editor while the agent operates windows in the background.
* **Your Real Browser, Logged In:** Connects directly to your running **Brave, Chrome, or Edge** browser through a secure Chrome Native Messaging Host. All your cookies, logins, active tabs, and extensions work immediately.
* **All-In-One `act` Execution:** Instead of separate *find window → read tree → click → wait → read tree* LLM round-trips, `fastuse` runs the entire batch in **one call** and returns the resulting UI state.
* **Visible Agent Cursor & Glow:** You always know what the agent is doing. An independent, brand-blue glowing cursor glides across the target window and animates on clicks, paired with a subtle screen-edge aura and status banner.

---

## 📊 Measured Speed

Tested on Windows 11 (AMD Ryzen 9 / RTX 3070), medians over persistent MCP sessions, compared to [cua-driver](https://github.com/trycua/cua) 0.37.0. Reproducible via `bench/`:

| Operation | Standard Driver (cua) | `fastuse` | Speedup |
|---|---|---|---|
| **List windows** | 437 ms | **0.9 ms** | **~485x faster** |
| **Read window UI tree** | 160 ms | **29 ms** | **~5.5x faster** |
| **Click by element** | 16 ms | **1.8 ms** | **~9x faster** |
| **`set_value` (20 chars)** | 16 ms | **2.2 ms** | **~7x faster** |
| **Type 20 chars** | 153 ms | **3.6 ms** | **~42x faster** |
| **Batch flow (set + click + observe)** | *Multiple round-trips* | **~66 ms** | **Single turn** |

*End-to-end through Antigravity, a multi-step task drops from 30–45s down to 4–8s.*

---

## 🛠️ Tool Catalog

### 1. Browser Automation (`browser_act`, `browser_state`)
Works directly in your active Brave, Chrome, or Edge browser:

| Tool | Description |
|---|---|
| `browser_act` | Run full web workflows in one call (`navigate`, `new_tab`, `switch_tab`, `close_tab`, `back`, `forward`, `reload`, `click`, `double_click`, `right_click`, `hover`, `fill`, `type`, `select`, `press`, `scroll`, `wait`) and receive the new DOM tree. Automatically re-reads mutated DOMs on modal/dropdown opens. |
| `browser_state` | Reads active tab headings, interactive buttons, inputs, links, and selectable items as a compact structured tree. |

```json
// Example: Complete search and navigation in one call
{
  "steps": [
    { "tool": "navigate", "url": "https://github.com" },
    { "tool": "fill", "target": "Search or jump to...", "value": "fastuse" },
    { "tool": "press", "key": "Enter" }
  ]
}
```

### 2. Desktop Automation (`act`, `get_window_state`, `list_windows`, `screenshot`)
Operates any Win32, WPF, WinUI, AppKit, Cocoa, or Electron desktop app:

| Tool | Description |
|---|---|
| `act` | Execute a complete desktop task in one turn: locates the window by title or PID, executes steps in sequence (`click`, `double_click`, `right_click`, `set_value`, `type_text`, `press_key`, `hotkey`, `scroll`, `wait`), and returns the updated accessibility tree. |
| `get_window_state` | Inspects an application's UI hierarchy as an indexed tree with element roles, names, and bounding boxes. |
| `list_windows` | Lists all visible top-level windows with title, PID, process name, and bounds in < 1 ms. |
| `screenshot` | Captures high-speed JPEG frames of a specific window (background `PrintWindow` / `screencapture`) or full desktop, with optional cropping and downscaling. |

```json
// Example: Batch desktop action
{
  "window": "Notepad",
  "steps": [
    { "tool": "set_value", "target": "Text editor", "value": "Build shipped." },
    { "tool": "hotkey", "keys": "ctrl+s" }
  ]
}
```

---

## 💻 Manual Installation

### Windows (PowerShell)
```powershell
irm https://github.com/anassagd432/fastuse/releases/latest/download/install.ps1 | iex
```

### macOS (Terminal)
```bash
git clone https://github.com/anassagd432/fastuse.git
cd fastuse
./install.sh
```

### Browser Extension Setup (One-time)
1. Open `chrome://extensions` or `brave://extensions`.
2. Enable **Developer mode** (top right toggle).
3. Click **Load unpacked** and select the `fastuse/extension` directory.
4. Fastuse connects automatically to your browser via native messaging.

---

## 🩺 Diagnostics & Health Check

Run `doctor` anytime to verify accessibility permissions, native messaging manifests, and connected browsers:

```bash
fastuse doctor
```

Output:
```text
Windows: UI Automation OK, HiDPI aware, background messaging active.
Browser use:
  native host manifest: installed (com.fastuse.browser.json)
  connected: Brave (extension 0.1.0, host pid 13768)
```

---

## 🔒 Safety & Privacy

* **Background-First & Focus Isolation:** Operations never take focus unless explicitly requested via `delivery_mode: "foreground"`.
* **Sensitive Sites Blocklist:** Built-in refusal prevents opening or reading banking, payment, password-manager, or credential domains (`paypal.com`, `stripe.com`, `accounts.google.com`, `1password.com`, etc.).
* **Visual Accountability:** The agent overlay displays a noticeable brand-blue cursor and screen aura so you always know when automation is active.
* **Capture Privacy:** The overlay window uses `WDA_EXCLUDEFROMCAPTURE` so screenshots sent to AI models remain crisp and uncluttered by the agent's own indicators.

---

## 📄 License

MIT License. See [LICENSE](LICENSE) for details.

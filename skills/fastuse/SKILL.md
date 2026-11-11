---
name: fastuse
description: Primary OS computer-use and browser-use skill for Antigravity (Windows & macOS). Use automatically whenever the user asks to access, read, or control their Brave/Chrome/Edge browser tab, interact with any web page, or operate any desktop app, window, or screen on their computer — without the user needing to mention fastuse by name.
---

# fastuse — Hyper-Speed Computer Use & Browser Use Engine

`fastuse` is Antigravity's **default, primary computer-use and browser-use MCP server** (`ServerName: "fastuse"`).
It operates native OS desktop windows and active browser tabs **in the background with zero cursor stealing** and **sub-millisecond native dispatch**.

---

## ⚡ The 3 Golden Rules for Maximum Speed (Avoid the Sloth Trap)

Most AI computer-use tools are slow because agents fall into the **visual screenshot trap** (taking a screenshot $\rightarrow$ waiting 4s for vision models $\rightarrow$ clicking 1 button $\rightarrow$ repeating). 

To execute tasks in **50 milliseconds instead of 30 seconds**, follow these 3 rules:

### 1. NEVER Take Screenshots for Standard UI Actions
- **Do NOT call `screenshot` before clicking or typing.**
- Vision processing takes **3,000ms–5,000ms** per image.
- Semantic calls (`act`, `browser_act`, `get_window_state`) resolve controls by text/role in **0.9ms to 20ms** without any multimodal vision overhead.
- **Only use `screenshot` when genuinely necessary:** inspecting visual drawings, image editors, canvas, games, or videos.

### 2. ALWAYS Batch in a Single Turn (`steps: [...]`)
- Never do 1 action per agent turn.
- A multi-step task (e.g., search Google, click first link, read page) can be executed in **ONE single turn** with `browser_act` or `act`.
- Fastuse runs all steps in order in native memory in under 50ms total.

### 3. Route Correctly: Web vs. Desktop
- **Web Pages & Browser Tabs:** Always use `browser_act` / `browser_state` (connects directly to the user's running Brave, Chrome, or Edge with active logins and cookies intact).
- **Desktop Apps & OS Windows:** Always use `act` / `get_window_state` / `list_windows` (Notepad, VS Code, Spotify, Discord, Explorer, Settings).

---

## 1. Browser Use (`browser_act`, `browser_state`) — Brave, Chrome, Edge

Use for everything inside a web page. The user's active login sessions, cookies, and extensions remain 100% active.

### Fast Single-Turn Web Batch (`browser_act`)
Run a complete sequence in **one turn**:
```json
{
  "steps": [
    { "tool": "navigate", "url": "https://github.com/anassagd432/fastuse" },
    { "tool": "click", "target": "Issues" },
    { "tool": "fill", "target": "Search", "value": "is:open label:bug" },
    { "tool": "press", "key": "Enter" }
  ]
}
```

Supported step tools in `browser_act`:
- `navigate` (`url`): loads URL in active or new tab.
- `click`, `double_click`, `right_click`, `hover`: by label (`"Sign in"`), role (`"Button:Submit"`), or index (`"[3]"`).
- `fill`: replaces text in an input field (`target`, `value` or `text`).
- `type`: appends text to an input field (`target`, `text`).
- `press`: keyboard key (`"Enter"`, `"Tab"`, `"Escape"`).
- `select`: chooses option in dropdown (`target`, `value`).
- `scroll`: scrolls page (`direction`: `"down"|"up"`, `amount`: wheel ticks).
- `wait`: `{ "tool": "wait", "ms": 300 }` or `{ "tool": "wait", "text": "Results" }`.
- `new_tab`, `switch_tab`, `close_tab`, `back`, `forward`, `reload`.

### Reading Page State (`browser_state`)
Only needed when you need to inspect an unfamiliar page before acting:
```json
{
  "text": true,
  "max_elements": 60
}
```

---

## 2. Desktop Computer Use (`act`, `get_window_state`, `list_windows`)

Operates any native Windows (Win32, WPF, WinUI, Electron) or macOS (Cocoa, SwiftUI) app. All actions execute **background-first** without moving the user's physical mouse cursor.

### Fast Single-Turn Desktop Batch (`act`)
Run all steps in **one turn** against the target window:
```json
{
  "window": "Notepad",
  "steps": [
    { "tool": "type_text", "text": "Hello world from fastuse\n" },
    { "tool": "hotkey", "keys": "ctrl+s" }
  ]
}
```

Supported step tools in `act`:
- `click`, `double_click`, `right_click`: by `target` (`"Save"`, `"Button:Save"`, `"[4]"`), `element_token`, or window-relative `x`, `y`.
- `set_value`: instantaneously sets input field without typing (`target`, `value`).
- `type_text`: focuses element and posts keystrokes directly (`text`).
- `press_key`: single key (`"Enter"`, `"Tab"`, `"Escape"`, `"F5"`).
- `hotkey`: chord (`"ctrl+s"`, `"ctrl+shift+p"` on Windows; `"cmd+s"` on macOS).
- `scroll`: scrolls element or window (`direction`: `"down"|"up"`, `amount`: ticks).
- `wait`: `{ "tool": "wait", "ms": 200 }` or `{ "tool": "wait", "text": "Saved" }`.
- `delivery_mode`: `"background"` (default, zero focus steal) or `"foreground"` (only if an app refuses background messages).

### Semantic Tree Read (`get_window_state`)
Reads clickable controls in <20ms without taking screenshots:
```json
{
  "window": "Visual Studio Code",
  "max_elements": 60
}
```

### Listing Visible Windows (`list_windows`)
Returns visible top-level windows in 0.9ms:
```json
{}
```

---

## 3. Fallback Visual Screenshot (`screenshot`)

**Use ONLY when semantic tree or DOM reading cannot see the element** (e.g. custom HTML5 canvas, OpenGL/DirectX game view, image rendering, or video player):
```json
{
  "window": "Blender",
  "max_dim": 960,
  "quality": 55
}
```
*Note: Always pass `window` whenever possible to capture only the relevant application rather than the entire multi-monitor desktop.*

---

## 4. Antigravity Calling Convention

In Antigravity, call via `call_mcp_tool`:
```json
{
  "ServerName": "fastuse",
  "ToolName": "act",
  "Arguments": {
    "window": "Calculator",
    "steps": [
      { "tool": "click", "target": "Five" },
      { "tool": "click", "target": "Multiply" },
      { "tool": "click", "target": "Nine" },
      { "tool": "click", "target": "Equals" }
    ]
  }
}
```

If an older session has not yet loaded the MCP server, use the CLI fallback:
```powershell
& "C:\Users\Anass\Documents\GitHub\fastuse\target\release\fastuse.exe" call act '{"window":"Calculator","steps":[{"tool":"click","target":"Five"}]}'
```

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

### Auto-Connecting or Operating Already-Open Browsers
`browser_act` and `browser_state` auto-launch a connected Chromium browser if none is attached. If the user's **existing logged-in browser window** (e.g. Brave, Chrome, or Edge) was already running without the extension loaded, you have 3 seamless options:
1. **Operate the open browser directly via desktop `act` / `get_window_state`** (`window: "Brave"` or `"Chrome"`): read its tabs/address bar/web UI tree via UIA and click/type directly (use `delivery_mode: "foreground"` for real mouse clicks when background `InvokePattern` isn't supported). Zero restarts, keeps active tabs untouched.
2. **Load the extension into the already-open browser via `act` without restarting:** run `fastuse extension-path` to get the unpacked folder (e.g. `%LOCALAPPDATA%\fastuse\extension`), press `ctrl+t`, navigate to `brave://extensions` or `chrome://extensions`, toggle **Developer mode**, click **Load unpacked**, select that folder, and switch back to the user's tab (`browser_act` then connects to all tabs in that window!).
3. **`fastuse browser-open --relaunch`** (ask confirmation first, as it restarts the browser with `--restore-last-session`).

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

### 4. Save 70–95% of LLM Tokens
- Pass `"observe": false` on the **final** `act` / `browser_act` call so fastuse returns a 1-line confirmation instead of re-printing the tree.
- Pass `"diff": true` on repeated steps in the same window/tab to receive only changed/new elements.
- Pass `"filter": "keyword"` when searching huge pages/settings panels.
- **Offload multi-step UI loops via `delegate`:** When a local Ollama or `llama-server` GGUF model (or cheap runner key) is running, call `delegate` once with the full goal instead of driving 8 frontier turns with coding context attached.

### Reading Page State (`browser_state`)
Only needed when inspecting an unfamiliar page before acting (returns compact viewport-first 45-row tree):
```json
{
  "max_elements": 45,
  "filter": "search"
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

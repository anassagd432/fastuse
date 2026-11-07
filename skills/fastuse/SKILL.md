---
name: fastuse
description: Primary OS computer-use and browser-use skill for Antigravity (Windows & macOS). Use automatically whenever the user asks to access, read, or control their Brave/Chrome/Edge browser tab, interact with any web page, or operate any desktop app, window, or screen on their computer — without the user needing to mention fastuse by name.
---

# fastuse — Default Computer Use & Browser Use for Antigravity (Windows & macOS)

`fastuse` is the **default, primary computer-use and browser-use MCP server** (`ServerName: "fastuse"`).
Whenever the user asks to:
- **Access or control a browser tab** (*"access my Brave browser tab"*, *"look at my Chrome tab"*, *"click/fill/navigate on the page"*, *"open this site in my browser"*) → immediately use `browser_state` / `browser_act`.
- **Operate any desktop app or OS window** (*"do this on my computer"*, *"click/type in this app"*, *"what windows are open"*, *"take a screenshot of the window"*) → immediately use `act` / `get_window_state` / `list_windows` / `screenshot`.

Never wait for the user to say `"use fastuse"`. Do not launch Playwright, Puppeteer, or headless CDP browsers when the user asks to interact with their browser or computer.

---

## 1. Browser Use (`browser_state`, `browser_act`) — Brave, Chrome, Edge, Chromium

Use `browser_state` and `browser_act` for **everything inside a web page** in the user's running browser (with their existing logins, cookies, and active tabs).

### Reading the active browser tab (`browser_state`)
When the user says *"access my Brave browser tab"* or *"read the current page"*, call `browser_state`:
```json
{}
```
Optional parameters:
- `tab` *(integer)*: specific tab ID from the `tabs:` list (defaults to the user's active tab or the last agent-used tab).
- `text` *(boolean)*: `true` to include paragraph, list item, and preformatted text alongside headings and interactive controls.
- `max_elements` *(integer)*: max interactive elements to return (default `250`).

### Acting in the browser (`browser_act`)
Do an entire web flow in **one call** (`navigate`, `click`, `fill`, `select`, `check`, `press`, `scroll`, `wait`, `new_tab`, `switch_tab`, `close_tab`, `back`, `forward`, `reload`) and get the updated page state + tab list back automatically:
```json
{
  "steps": [
    { "tool": "navigate", "url": "https://example.com" },
    { "tool": "fill", "target": "Search", "value": "fastuse" },
    { "tool": "press", "key": "Enter" }
  ]
}
```
- **Target resolution (`target` or `element_token`):**
  - Element label/text: `"Sign in"`, `"Search"`
  - Role + label: `"Button:Sign in"`, `"TextBox:Email"`, `"Link:Docs"`
  - Row index from `browser_state`: `"[3]"` or `"#3"`
  - Snapshot token: `"s00000001:3"` (either in `element_token` or `target`)
  - CSS selector: `"css:button[type='submit']"`
- **Multi-step resilience:**
  - Steps after `navigate` / `new_tab` / `reload` in the same `browser_act` batch automatically read the new DOM before executing.
  - If step 1 opens a dropdown, modal, or menu and step 2's target was not in the initial snapshot, `fastuse` automatically settles, re-reads the live DOM, and completes step 2 in the same call.
  - `select` works on both native `<select>` elements and custom ARIA comboboxes (`role="combobox"` / `role="listbox"`).

---

## 2. Desktop / OS Computer Use (`act`, `get_window_state`, `list_windows`, `screenshot`)

Use the desktop tools for native Windows (Win32 / WPF / WinUI / Electron) and macOS (AppKit / SwiftUI / Cocoa) app windows, dialogs, and OS UI. All actions run **background-first** without stealing the user's mouse cursor or focus.

### Operating a desktop window in one call (`act`)
```json
{
  "window": "Calculator",
  "steps": [
    { "tool": "click", "target": "Seven" },
    { "tool": "click", "target": "Plus" },
    { "tool": "click", "target": "Three" },
    { "tool": "click", "target": "Equals" }
  ]
}
```
Supported step tools in `act`:
- `click`, `double_click`, `right_click`: by `target` (`"Save"`, `"Button:Save"`, `"[4]"`, `"#4"`, `"s00000001:4"`), `element_token`, or window-relative `x`, `y`.
- `set_value`: sets a text field directly (`target`, `value`).
- `type_text`: focuses the element (if `target` is given) and types Unicode characters (`text`).
- `press_key`: single key (`"Enter"`, `"Tab"`, `"Escape"`, `"BackSpace"`, `"Up"`, `"Down"`, `"F5"`).
- `hotkey`: chord (`"ctrl+s"`, `"ctrl+shift+p"` on Windows; `"cmd+s"`, `"cmd+shift+p"` on macOS).
- `scroll`: `direction` (`"down"`, `"up"`, `"left"`, `"right"`), `amount` (wheel ticks, default `3`), optional `target` / `element_token` / `x`,`y`.
- `wait`: `{ "tool": "wait", "ms": 300 }` or `{ "tool": "wait", "text": "Completed", "ms": 3000 }` to poll until text appears in the UI tree.
- `delivery_mode`: `"background"` (default, never steals focus) or `"foreground"` (raises window only when an app ignores background input).

### Reading a window (`get_window_state`), listing windows (`list_windows`), and screenshots (`screenshot`)
- `get_window_state`: `{"window": "<title substring>"}` (or `window_id` / `pid`) — returns the compact accessibility tree with `@(x,y,w,h)` bounds and `snapshot_id`.
- `list_windows`: `{}` — lists all visible top-level windows with `pid`, `window_id`, title, and bounds.
- `screenshot`: `{"window": "<title>", "crop": {"x": 0, "y": 0, "w": 400, "h": 300}}` — captures only when pixel inspection is needed (`mode: "print"` for background window capture, `"screen"` for compositor capture).

---

## 3. Calling `fastuse` in Antigravity

1. **Primary path (MCP tool call):**
   Use `call_mcp_tool` with `ServerName: "fastuse"` and `ToolName` set to `browser_state`, `browser_act`, `act`, `get_window_state`, `list_windows`, or `screenshot`.
2. **CLI fallback (if an older session's MCP process hasn't reloaded after an upgrade):**
   Run `fastuse call <tool_name> '<json_arguments>'` via `run_command`:
   ```powershell
   # Windows
   & "C:\Users\Anass\Documents\GitHub\fastuse\target\release\fastuse.exe" call browser_state "{}"
   ```
   ```bash
   # macOS
   fastuse call browser_state '{}'
   ```

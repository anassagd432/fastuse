# fastuse browser: plan

Goal: let an agent (Antigravity first, but also Claude, Codex or anything that speaks MCP) use **your own browser**, with your logins, quickly and with your permission. Same product family as the desktop side: one `fastuse` binary, one install, same overlay look.

Status: research and plan only. Nothing here is built yet. Claims are marked **(verified)** when I checked them against Chrome's docs or a primary source, **(reported)** when they come from forum posts or blogs, and **(to test)** when we must find out on a real browser.

## 1. What exists today (research)

| Product | How it reaches your browser | Notes |
|---|---|---|
| **Antigravity browser subagent** | A Chrome extension plus a managed Chrome window; a separate model runs the browser steps | Runs in an isolated Chrome profile, so your logins are not there (reported). Allowlist/denylist for URLs; only localhost is allowed by default (reported). Needs Chrome with remote debugging enabled (reported). |
| **Claude in Chrome** | MV3 extension + a native messaging host that Claude Code launches (`com.anthropic.claude_code_browser_extension`), then a local socket/named pipe to the CLI | Per-site permissions are the first defense against prompt injection; JavaScript execution needs per-domain approval (Anthropic's safety article). Known failure: service worker never calling `connectNative` (reported). A second host from Claude Desktop with the same extension ID can steal the connection (reported). |
| **Codex Chrome extension** | MV3 extension + native host `com.openai.codexextension` + a socket under `/tmp/codex-browser-use/` | Reports of socket mismatch and init timeouts (reported). **It is registered on this PC already** (checked the registry): `com.openai.codexextension` under Chrome. |
| **Playwright MCP Bridge** | Extension relays CDP from your tabs to the Playwright MCP server (`--extension`) | Reuses your logged-in session (official docs). |
| **Chrome DevTools MCP** | Puppeteer over CDP; `--autoConnect` attaches to a running Chrome after you enable remote debugging at `chrome://inspect/#remote-debugging`; Chrome asks permission per connection (reported) | Needs Chrome 144+ (reported). `--slim` flag exposes fewer tools. It is also in Antigravity's plugin marketplace on this PC (`chrome-devtools-plugin`). |
| **antigravity-browser-operator** (third party) | MV3 extension (`tabs`, `scripting`, `debugger`) + local Node MCP server + authenticated WebSocket on `127.0.0.1:8765` | 14 tools, token pairing, Origin check. No native messaging. Has `browser_evaluate` with no per-site gate. |

Takeaways:
1. Everyone uses the same shape: **MV3 extension in your real browser, a local bridge, an MCP server**. We copy the shape, not the code.
2. The pain points are connection reliability (MV3 service worker, two hosts, socket names) and trust (what the agent may touch). Those are where we can be better.
3. The Antigravity built-in browser agent uses a **sub-model per browser task**. Exposing browser tools directly to the main model skips that extra model round trip. This is our speed angle.
4. Nobody leads with *the user can see it*: a visible cursor, a glow around the page and a banner inside the page. We already have that look.

## 2. Architecture

```
agent (agy / Claude / Codex)
   │ MCP over stdio
   ▼
fastuse  (existing process, new browser tools)
   │ local pipe: named pipe on Windows, Unix socket on macOS/Linux
   ▼
fastuse --native-host   (started by the browser, one per browser; speaks Chrome's length-prefixed stdio)
   │ chrome.runtime.connectNative
   ▼
extension service worker (MV3)
   │ chrome.scripting / chrome.debugger / chrome.tabs
   ▼
content script in each page: reads the tree, draws the cursor + glow + banner
```

Why native messaging and not a localhost WebSocket (what antigravity-browser-operator does):
- Chrome only lets the extension IDs listed in the host manifest's `allowed_origins` start the host (verified), so a web page or another extension cannot talk to it. A WebSocket port is reachable by every local process and needs a pairing token.
- No open TCP port. Names and ACLs of the pipe are ours to restrict to the current user.
- Cost: an install step (registry key on Windows, a JSON file on macOS/Linux). `install.ps1` / `install.sh` will do it.

Facts we rely on:
- Host to browser messages are limited to **1 MB**, browser to host **64 MiB** (verified). Commands are small; screenshots flow the allowed direction.
- `connectNative()` keeps the host alive until the port closes, and Chrome keeps the service worker alive while a native port is open (reported by a W3C issue; **to test**).
- Windows host gets `--parent-window=` and the origin as first argument; stdout must be binary, only protocol on stdout (verified).
- `chrome.debugger` supports flat child sessions since Chrome 125 (verified). Chrome Web Store allows remote-logic only through the Debugger and User Scripts APIs (verified), so we must not fetch code at runtime.

### One binary, three modes
`fastuse` (MCP server, as now) · `fastuse --native-host` (what the browser launches) · `fastuse doctor` (also checks the browser side). The Rust code is shared; the browser modules live in `src/browser/`. No new language runtime for users.

### Reading a page (the speed lever)
- A content script walks the DOM once and emits the **same compact text tree** the desktop side uses (`[12] link "Pricing" @(x,y,w,h)`), only visible and interactive nodes, with snapshot-scoped refs (`s00000012:12`) that go stale on navigation. Open shadow roots and same-origin iframes are included; cross-origin iframes get their own content script and refs.
- No CDP and no yellow debugging bar for reading and for most actions.
- Password fields are redacted and refused as targets.

### Acting
1. **Default:** content-script actions: scroll into view, pointer/mouse event sequence, `click()`, native value setters plus `input`/`change` events, `insertText` for rich editors.
2. **Fallback, per tab, on demand:** attach `chrome.debugger` and send trusted input (`Input.dispatchMouseEvent`, `Input.dispatchKeyEvent`) for sites that ignore synthetic events, file choosers, drag and drop, and screenshots of background tabs. Chrome shows a "debugging this browser" bar while attached; it can only be hidden with a command-line flag, and we should not try (it is a safety signal).
3. `browser_act` takes a **batch of steps** and returns the new state, like the desktop `act`. One model round trip per screen.

### Tools (kept to four, so tool docs stay small)
- `browser_state`: tab list (id, title, url, which ones the agent may control) plus the tree of one tab.
- `browser_act`: steps `click, type, fill, press, scroll, select, hover, navigate, back, forward, reload, wait, new_tab, close_tab`; `observe` default on.
- `browser_screenshot`: tab or region, `max_dim`, quality; JPEG.
- `browser_tabs`: open/close/focus/group, mostly for setup; may merge into `browser_act`.
Later, off by default and gated per site: `browser_eval`, console and network readers.

### What the user sees
Inside the page, a shadow-DOM overlay with the same look as the desktop one: blue cursor with halo gliding to the target, glow along the page edge, a top pill "This browser tab is being controlled by fastuse". Works on every OS with no extra permission, and it does not appear in the model's screenshots when we hide it during capture. Controlled tabs go into a tab group called "fastuse".

### Permissions and safety
- Extension asks for host access **per site at runtime** (`optional_host_permissions`), not `<all_urls>` up front. The toolbar popup lists allowed sites and a master off switch.
- The agent only touches tabs in the fastuse group or tabs the user explicitly shares.
- Page text is data. The skill and the server instructions say so (same rule as desktop).
- Confirmation before: purchases, sending messages, deleting, and entering anything into password or payment fields. Default deny on password fields.
- No remote code. Local pipe restricted to the current user. `allowed_origins` pinned to our extension IDs.
- Anthropic's own guidance is that this stays imperfect and sensitive sites need care; we say the same in the README.

## 3. Baselines to beat (do this before building)

Same approach as cua-driver on the desktop: measure the others first, then set targets.
Candidates: **Claude in Chrome** (its tools are available in this session, so I can measure it directly), **chrome-devtools-mcp** (`npx`, with `--slim` too), **Playwright MCP** (and its extension mode), **antigravity-browser-operator**.

Tasks, on 4 pages (a Wikipedia article, a GitHub repo page, a search results page, a form page):
1. list tabs · 2. read page state (**latency and size in tokens**) · 3. click a link and read the new state · 4. type into a field · 5. screenshot · 6. a 5-step batch.

Targets (to confirm against the baseline): state ≤ 60 ms on a typical page, ≤ half the tokens of the Playwright snapshot; click ≤ 15 ms excluding page load; batch of 5 steps ≤ 100 ms; zero model round trips more than needed.

## 4. Milestones

| # | What | Done when |
|---|---|---|
| **B0** | Baselines (above) and a repeatable `bench/browser/` harness | Table of numbers committed |
| **B1** | Skeleton: extension (stable dev ID), `fastuse --native-host`, pipe, `browser_state` read-only | Reads the tab list and a page tree in Chrome **and** Brave on Windows; service worker survives 10 min idle |
| **B2** | `browser_act`: click, fill, type, press, scroll, navigate, wait; batch + observe; stale refs | A form on a real site completes in one batch; 100 clicks in a row without a miss |
| **B3** | In-page overlay, tab group, per-site permission popup, password-field policy | You can see the agent working; a non-allowed site is refused |
| **B4** | Screenshots; lazy `chrome.debugger` for trusted input and background tabs; file upload | Works on a site that ignores synthetic clicks |
| **B5** | Install: `install.ps1`/`install.sh` register the host for Chrome, Brave, Edge; release zip; skill update; `doctor` checks | A fresh PC goes from nothing to working in two steps |
| **B6** | macOS pass (host paths, pipe → Unix socket), benchmark table, README demo, Web Store submission | Works on your MacBook |
| **B7** | Firefox (content-script path only, no debugger), Linux | Later |

Windows comes first for this part because I can test it here; the extension is OS-independent, so the Mac work is mostly host registration paths and the socket.

## 5. Risks and unknowns

| Risk | Plan |
|---|---|
| MV3 service worker dies and the port drops | Reconnect loop on wake; test the 10 min idle case in B1. If the native port does not keep it alive, send a small keepalive every 20 s (Chrome 116+ treats traffic as activity for WebSockets (verified); for native ports **to test**). |
| Chrome branded builds removed `--load-extension` in 137 (verified) | Users load the extension via `chrome://extensions` → Load unpacked. For automated tests I use **Chrome for Testing** (a ~150 MB download) where the flag still works. |
| Brave and Edge native host registration | Different registry keys (Chrome: `HKCU\Software\Google\Chrome\NativeMessagingHosts`). Brave on this PC has none today, so we must register `BraveSoftware\Brave-Browser` ourselves and **test** whether Brave reads Chrome's key too. |
| Another host on the same extension ID (the Claude Desktop issue) | We use our own host name and our own extension ID; the pipe name includes the browser profile. |
| Chrome Web Store review of `debugger` + `nativeMessaging` + broad hosts | Ship unpacked from GitHub first. Store listing after we have the permission popup and a clear privacy page. Developer account fee applies. |
| Sites that block synthetic events or detect automation | Trusted input fallback through `chrome.debugger`; be honest in docs that some sites will refuse. |
| Prompt injection from page content | Per-site permission, treat page as data, confirm risky actions, redact secrets, no `eval` by default. |
| Conflict or overlap with Antigravity's own browser agent | Ours is additive: the skill tells the model to use `fastuse` browser tools when your logged-in browser is needed. **To test** that both can coexist. |
| Closed shadow roots, canvas apps, PDFs viewers, `chrome://` pages | Not readable by content scripts; fall back to screenshots and coordinates; say so in docs. |

## 6. What is already prepared

- `extension/manifest.json`: MV3, `minimum_chrome_version` 116, permissions `nativeMessaging, tabs, tabGroups, scripting, debugger, storage`, host access optional, and a dev public key so the unpacked extension always gets the same ID: **`andnpcpahlilpjlipaogpekbnljedfcb`**. The private key is at `~/.fastuse/dev-extension-key.pem` (not in the repo; losing it only changes the dev ID). The Web Store will assign a different ID, and the host manifest will list both.
- This plan.

## 7. Decisions for you

1. **Browsers first:** Chrome + Brave + Edge together (same extension, I recommend), or Chrome only? Your default is Brave.
2. **Debugger permission:** OK to include `chrome.debugger` (needed for trusted input and background-tab screenshots, but it triggers the Web Store's strictest review and shows Chrome's debugging bar when used)? My recommendation: yes, used lazily.
3. **Distribution:** unpacked from GitHub first, Web Store after B5. OK?
4. **Test browser:** may I download Chrome for Testing (~150 MB) for automated tests? Manual testing in your Brave/Chrome stays with you (Load unpacked).
5. **Baselines:** may I run chrome-devtools-mcp / Playwright MCP through `npx` (downloads packages) to measure them?

## Sources

- Chrome native messaging: https://developer.chrome.com/docs/extensions/develop/concepts/native-messaging
- chrome.debugger API: https://developer.chrome.com/docs/extensions/reference/api/debugger
- Web Store MV3 requirements: https://developer.chrome.com/docs/webstore/program-policies/mv3-requirements
- WebSockets in service workers (Chrome 116): https://developer.chrome.com/docs/extensions/mv3/tut_websockets
- `--load-extension` removal in Chrome 137: https://groups.google.com/a/chromium.org/g/chromium-extensions/c/1-g8EFx2BBY/m/S0ET5wPjCAAJ
- Claude in Chrome safety: https://support.claude.com/en/articles/12902428-using-claude-in-chrome-safely and https://www.anthropic.com/news/claude-for-chrome
- Claude in Chrome host/bridge issues: https://github.com/anthropics/claude-code/issues/21363, https://github.com/anthropics/claude-code/issues/48502, https://github.com/anthropics/claude-code/issues/31660
- Codex Chrome extension issues: https://github.com/openai/codex/issues/21719 (mirror https://upd.dev/openai/codex/issues/21719)
- Playwright MCP extension: https://playwright.dev/mcp/configuration/browser-extension
- Chrome DevTools MCP: https://github.com/ChromeDevTools/chrome-devtools-mcp
- Chrome DevTools MCP existing-session connection: https://lilting.ch/en/articles/chrome-devtools-mcp-active-session
- Antigravity allowlist: https://www.antigravity.google/docs/ide/allowlist-denylist
- Antigravity browser subagent notes: https://www.html.it/pag/568599/browser-subagent-strumenti-e-sicurezza-con-antigravity/ and https://discuss.ai.google.dev/t/antigravity-browser-subagent-not-working/185158
- antigravity-browser-operator: https://glama.ai/mcp/servers/hashan-lakshitha/antigravity-browser-operator/tree
- MV3 service worker and native port lifetime: https://github.com/w3c/webextensions/issues/256

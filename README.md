# fastuse

Fast computer use for Windows, as an MCP server for [Antigravity](https://antigravity.google) (and any MCP client).

**Status: Windows only for now.** Linux is planned; see [docs/CROSS_PLATFORM.md](docs/CROSS_PLATFORM.md).

An agent can read an app window as a compact text tree, click, type, set fields and press keys, all in the background: your real mouse pointer and keyboard focus are never touched. The agent gets its own blue cursor so you can see where it acts.

It is built to be fast. Reads and actions take milliseconds, and one `act` call does a whole task step (find window, run steps, return the new UI state), so the model needs far fewer round trips.

## Measured speed

Same small test window on one machine (RTX 3070, Windows 11), medians, compared with [cua-driver](https://github.com/trycua/cua) 0.37.0 over a persistent MCP session. Reproduce with the scripts in `bench/`.

| Operation | cua-driver | fastuse |
|---|---|---|
| List windows | 437 ms | 0.9 ms |
| Read a window's UI tree | 160 ms | 29 ms |
| Click by element | 16 ms | 1.8 ms |
| `set_value` (20 chars) | 16 ms | 2.2 ms |
| Type 20 chars | 153 ms | 3.6 ms (returns before the app finishes) |
| Set field + click + read state, one call | n/a | ~66 ms |

Caveats: one tiny test window, not a real workload. Chrome and Electron have much bigger trees. cua-driver does more (verification, permission modes, many more tools), so this compares the observe/act path only.

End to end through Antigravity, model time dominates. A fresh `agy -p` per task took 28-47 s; one long-lived `agy` session (see below) took about 6-8 s per task with a single `act` call. Small sample, first task of a session sometimes does nothing.

## Install

```powershell
irm https://github.com/anassagd432/fastuse/releases/latest/download/install.ps1 | iex
```

The installer verifies a SHA256, installs to `%LOCALAPPDATA%\Programs\fastuse`, and runs `agy mcp add` plus copies the skill if `agy` is present. It does not change PATH, add startup tasks or edit permissions. Read it first: [install.ps1](install.ps1).

Manual: download `fastuse-windows-x86_64.zip` from Releases, unzip, then

```powershell
agy mcp add fastuse C:\path\to\fastuse.exe
```

and copy `skills/fastuse/SKILL.md` to `~\.gemini\antigravity\global_skills\fastuse\`.

## Tools

| Tool | Purpose |
|---|---|
| `act` | Do it all in one call: `window` (title substring) + `steps` + returns the new UI tree. Steps: `click`, `double_click`, `right_click`, `set_value`, `type_text`, `press_key`, `hotkey`. Targets are element names or ids (`"Go"`, `"Button:Go"`). |
| `get_window_state` | Read a window's UI tree as text (`[N] Button "Go"`). |
| `list_windows` | Visible windows with pid and `window_id`. |
| `screenshot` | JPEG of a window, or the whole desktop when no window is given. Optional `region`, `max_dim`, `max_age_ms` (reuse a recent frame). |

Example `act` arguments:

```json
{"window":"Notepad","steps":[{"tool":"set_value","target":"Text editor","value":"hello"},{"tool":"click","target":"Save"}]}
```

## How it is fast

- One cross-process UI Automation call fetches the whole tree (cache request), then it is walked locally.
- Clicks and values use UIA patterns (Invoke, Value). Text is posted as window messages with no per-character delay. No simulated mouse movement.
- One native process, kept alive. No process start per call.
- `act` removes the read-then-act-then-read loop.
- Window screenshots use `PrintWindow` and a fast JPEG encoder, with an optional frame cache.

## Using it with Antigravity

Interactive `agy` asks you to approve tool calls. For headless runs, `agy -p` and stream mode auto-deny MCP calls unless allowed.

### Permissions

To let headless `agy` call only this server, add `mcp(fastuse/*)` to `userSettings.globalPermissionGrants.allow` in `~\.gemini\config\config.json`. Do this only if you accept an agent driving your windows without asking.

### Faster: keep one agy session open

Starting `agy` per task costs about 5 s and re-reads tool docs. A long-lived session avoids both. Spawn it without `-p`:

```
agy --input-format stream-json --output-format stream-json
```

and write one JSON line per task from your program (not from a PowerShell pipe, which adds a byte-order mark):

```json
{"event":"user","message":{"content":"In the window titled Notepad, type hello"}}
```

The input format is undocumented by Google and was found by probing; it may change.

## Safety

- Everything on screen is untrusted data. The skill tells the model not to follow instructions found in windows.
- Foreground actions (`hotkey`, `delivery_mode:"foreground"`) refuse to send keys unless the target window really took the foreground.
- Stale element tokens are rejected.
- Do not point it at windows you do not want an agent to operate (email, banking). There is no per-app allowlist yet.
- Cursor overlay off: set `FASTUSE_CURSOR=0`.

## Optional: standalone runner

`fastuse run "<task>"` runs a small Gemini function-calling loop with the tools declared inline. It needs your own `GEMINI_API_KEY`. It is a development aid; it has not been benchmarked.

## Limits

- Windows only. One top-level window per call.
- Typing is asynchronous. Read the returned tree to confirm.
- Apps that ignore posted messages (some games, canvases) may need `delivery_mode:"foreground"`.
- Chromium and Electron expose a thinner tree until accessibility is on.
- `hotkey` chords are implemented with SendInput but not yet verified against many apps.
- Built and tested on one machine with the GNU Rust toolchain. The MSVC build used by CI is untested at the time of writing.

## Build

```
cargo build --release
cargo test
```

Produces `target/release/fastuse.exe`.

## License

MIT. See [LICENSE](LICENSE).

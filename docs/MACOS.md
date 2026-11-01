# macOS plan (first port after Windows)

Status: researched, **not started**. Nothing here has been built or run. The development PC is Windows, so every runtime step needs a Mac (or macOS CI). Items marked *unverified* come from secondary sources or general knowledge and must be confirmed on a Mac before relying on them.

## Mapping from the Windows backend

| Need | Windows (done) | macOS |
|---|---|---|
| Window list | `EnumWindows` | `CGWindowListCopyWindowInfo` (titles of other apps' windows need Screen Recording permission) + AX windows per app |
| UI tree | UI Automation, one cache request | Accessibility API (`AXUIElement`). No cache request: every attribute read is an IPC call, so batch with `AXUIElementCopyMultipleAttributeValues` and set a messaging timeout. **Speed is the main unknown.** |
| Click / value | UIA Invoke / Value patterns | `AXPerformAction(kAXPressAction)`, set `AXValue` |
| Typing / keys | posted `WM_CHAR` / `WM_KEYDOWN` | `CGEventPostToPid` with unicode keyboard events (*unverified* for every app) |
| Coordinate click | posted mouse messages | `CGEventPostToPid` mouse events; cua-driver reportedly uses scoped CoreGraphics + private SkyLight events (*not public API; avoid at first*) |
| Hotkeys | `SendInput` after raising | `CGEventPostToPid` with modifier flags; foreground fallback activates the app and posts to the HID tap |
| Window screenshot | `PrintWindow` | ScreenCaptureKit `SCScreenshotManager` (macOS 14+, async, Screen Recording permission). `CGWindowListCreateImage` is deprecated and fails to compile against the macOS 15 SDK, so skip it |
| Agent cursor | layered topmost click-through window | borderless transparent `NSWindow`, `ignoresMouseEvents`, high window level, joins all Spaces; same sprite art |

Chromium/Electron expose a thin AX tree until accessibility is switched on for the app (*unverified*: setting `AXManualAccessibility` / `AXEnhancedUserInterface` on the app element).

## Constraints that change the design

1. **AppKit wants the main thread.** The cursor overlay needs an `NSApplication` run loop on the main thread, but today the MCP stdin loop runs on the main thread. Phase M0 adds a platform hook so a backend can own the main thread and the MCP loop runs on a worker thread. Windows keeps its current behavior.
2. **Permission identity (TCC).** Accessibility and Screen Recording grants attach to the *responsible process*, which for a CLI tool is usually whichever app launched it (Terminal, `agy`, the Antigravity app), not `fastuse` itself, and grants can be lost when the binary path changes. cua-driver avoids this by running a daemon inside its own app bundle (`CuaDriver.app`, launched through LaunchServices) and talking to it from a thin client. Two options, to be decided by experiment on a Mac:
   - **A. Daemon + thin shim** (cua-driver's approach): stable identity, more moving parts.
   - **B. Signed `.app` bundle executed directly** by `agy`: simpler, but whether TCC then attributes the grant to our bundle or to `agy` is *unverified*.
3. **Signing and distribution.** Ad-hoc signing is enough for development. A smooth install for other people likely needs a Developer ID certificate and notarization (Apple Developer Program). *Unverified here*: exactly how Gatekeeper treats binaries fetched by `curl` versus a browser download. Ship a universal binary (arm64 + x86_64).
4. **Grants need a restart.** A newly granted permission is not applied to an already-running process, so `fastuse doctor` must say "granted, restart me".
5. **Minimum macOS 14** keeps capture simple (ScreenCaptureKit screenshot API). Older versions are out of scope.

## Phases

- **M0 (verifiable on Windows):** main-thread/runtime hook in `Platform`; backend selection by `cfg(target_os)`; check that the Windows benchmarks do not regress. If `rustup target add aarch64-apple-darwin` works on the dev PC, `cargo check --target aarch64-apple-darwin` can type-check macOS code without a Mac (type-check only, no linking or running; *unverified* that every dependency checks cleanly).
- **M1: read-only on a Mac:** `list_windows`, `get_window_state` via AX, element centers, `fastuse doctor` (permission status). Measure tree-read latency and decide whether AX needs a cache/parallel strategy.
- **M2: actions:** AXPress, AXValue, typing, keys, coordinate clicks, hotkeys, `act`. Chromium/Electron accessibility switch.
- **M3: capture and cursor:** ScreenCaptureKit window/desktop capture; `NSWindow` cursor overlay with the same sprite.
- **M4: ship:** resolve the TCC identity question (A or B), signing and notarization, `install.sh`, CI on `macos-latest`, README, Antigravity setup (the `agy mcp add` flow is assumed to match Windows, *unverified*).

## Rust building blocks (to evaluate on a Mac)

`objc2` + `objc2-app-kit` + `objc2-application-services` or the `axuielement` crate (exposes `AXUIElementCopyMultipleAttributeValues` and a messaging timeout) for Accessibility; `core-graphics` for events and window lists; a ScreenCaptureKit binding such as `screencapturekit` (API unverified).

## Needed to proceed

1. Access to a Mac: model, macOS version, Apple Silicon or Intel, and someone to run test commands and grant permissions.
2. Optional later: an Apple Developer account for signing and notarization.
3. GitHub Actions macOS runners are blocked by the billing lock, and CI cannot easily grant Accessibility or Screen Recording, so real testing needs a physical Mac or a cloud Mac.

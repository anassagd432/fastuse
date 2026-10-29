# Cross-platform plan

Status: **Phase 0 done** (portable core + `Platform` trait, Windows backend). Linux is planned and not started. Nothing below Phase 0 has been built or tested.

## Architecture (Phase 0, done)

```
src/
  main.rs            MCP stdio loop, `run`, `--version`
  server.rs          tool logic: act/run_steps, tokens, capture requests (OS-independent)
  tools.rs           tool schemas and server instructions
  imaging.rs         crop, downscale, JPEG, frame cache (OS-independent)
  runner.rs          optional Gemini loop
  platform/
    mod.rs           `Platform` trait, shared types, `tree_row` (identical tree text everywhere)
    win/             Windows backend: UI Automation, window messages/SendInput, GDI, layered cursor
```

A platform implements `Platform` (windows, tree, element actions, typing, keys, capture, cursor) and reports `Capabilities`. Tools tell the model what is unavailable instead of failing mysteriously. Element indices are the platform's; the core owns snapshot ids and rejects stale tokens.

Phase 0 was checked against the Windows benchmarks: no regression (list ~1 ms, tree read ~30 ms, click ~2 ms, run_steps ~33 ms). Along the way it found and fixed a real bug: with the overlay cursor visible, `EnumWindows` stalled ~100 ms because it queried the cursor's own window owned by a sleeping thread; own-process windows are now skipped.

## Research findings (Linux)

- **Sessions.** Ubuntu 25.10 dropped the GNOME X11 session; KDE is moving to Wayland-only (sources disagree on when). X11-only support would age badly. Even cua-driver treats Wayland as XWayland and calls native Wayland "preview".
- **Tree and actions: AT-SPI2** over D-Bus. Works on X11 and Wayland (it does not depend on the display server). Rust: the `atspi` crate (zbus-based, async; check current version). Chromium/Electron expose their tree only after an assistive tool announces itself (set the `org.a11y.Status` flags); GNOME may need `toolkit-accessibility` on. AT-SPI is chatty over D-Bus, so speed must be measured (its per-app cache is a lead, unverified).
- **Input on Wayland** needs consent: xdg-desktop-portal RemoteDesktop (restore tokens exist but are flaky across reboot, lock and service restarts in 2026 reports) or libei via the `reis` crate (described as incomplete; compositor support not verified). X11: XTEST.
- **Screenshots on Wayland:** portal ScreenCast + PipeWire (consent). X11: standard capture. `xcap` claims X11 and Wayland; what works on Wayland is unverified.
- **Agent cursor:** X11, KDE and wlroots can draw an overlay. GNOME's Mutter has no layer-shell and Wayland has no standard always-on-top window, so the cursor would be unavailable there (a GNOME extension is an unverified lead).
- **Antigravity on Linux:** `agy` installs with a curl script to `~/.local/bin`; no official `.deb` for 2.0. Whether `agy mcp add` and the skills folder match Windows is unverified.
- **Testing:** the development PC has no WSL, Docker or virtualization. Linux testing needs a Linux machine or VM, WSL2, or CI.

## Phases

1. **Phase 1: Linux tree + actions via AT-SPI** (works on X11 and Wayland, no portals): window list, tree, click (DoAction), set value, typing; Chromium/Electron accessibility switch; a `doctor` check. Test under a virtual display with a GTK/Qt test app.
2. **Phase 2: X11 extras:** XTEST input and hotkeys, window and desktop capture, overlay cursor (click-through window).
3. **Phase 3: Wayland extras:** portal screenshots and input with saved tokens and a clear first-time consent message; overlay cursor only where layer-shell exists; capability flags in tool replies.
4. **Phase 4: Linux release:** tarball, `install.sh` mirroring `install.ps1`, CI matrix (Windows + Ubuntu with a virtual display). macOS after that (Accessibility API, `CGEvent`, ScreenCaptureKit).

## Risks

Toolkit and app coverage differ (Chromium, Qt, GTK); AT-SPI speed is unknown; Wayland compositors differ (GNOME, KDE, wlroots); no local Linux test machine; maintenance doubles.

## Needed to continue

A Linux machine or VM (distro, desktop, X11 or Wayland), and a decision on macOS timing.

#!/usr/bin/env bash
# fastuse local installer for macOS & Linux (builds or installs binary + browser host + Antigravity skill & default rule)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

echo "==> building fastuse (release)"
cargo build --release --no-default-features

BIN="$ROOT/target/release/fastuse"
"$BIN" --version
"$BIN" browser-install

for SKILL_DIR in "$HOME/.gemini/antigravity/global_skills/fastuse" "$HOME/.gemini/config/skills/fastuse"; do
  mkdir -p "$SKILL_DIR"
  cp -f "$ROOT/skills/fastuse/SKILL.md" "$SKILL_DIR/SKILL.md"
done

GEMINI_MD="$HOME/.gemini/config/GEMINI.md"
mkdir -p "$(dirname "$GEMINI_MD")"
if ! grep -q "FASTUSE_COMPUTER_USE_START" "$GEMINI_MD" 2>/dev/null; then
  cat >> "$GEMINI_MD" << 'EOF'

<!-- FASTUSE_COMPUTER_USE_START -->
## Default Computer Use & Browser Use Engine (`fastuse` MCP)
`fastuse` (MCP server `fastuse`) is Antigravity's **primary, default Computer Use and Browser Use tool** on both Windows and macOS.
The user never needs to mention `fastuse` by name. Automatically route requests as follows:
1. **Browser tasks ("access my Brave browser tab", "check my Chrome tab", "click/fill/navigate on the page", "open this site in my browser"):**
   - Immediately use MCP server `fastuse` tools `browser_state` and `browser_act`.
   - Do **not** launch Playwright, Puppeteer, or headless CDP browsers when the user asks to read or interact with their browser tab.
2. **Desktop / OS tasks ("do this on my computer", "operate <app>", "click/type/scroll in <window>", "what windows are open", "screenshot <window>"):**
   - Immediately use MCP server `fastuse` tools `act`, `get_window_state`, `list_windows`, and `screenshot`.
<!-- FASTUSE_COMPUTER_USE_END -->
EOF
fi

if command -v agy >/dev/null 2>&1; then
  agy mcp add fastuse "$BIN" || true
fi

echo "==> fastuse installed and registered as Antigravity's default computer-use & browser-use tool"
"$BIN" doctor

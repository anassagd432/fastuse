#!/usr/bin/env bash
# fastuse installer for macOS & Linux
#
#   curl -fsSL https://raw.githubusercontent.com/anassagd432/fastuse/main/install.sh | bash
#   or inside cloned repo: ./install.sh
set -euo pipefail

REPO="${FASTUSE_REPO:-anassagd432/fastuse}"
VERSION="${FASTUSE_VERSION:-latest}"
PREFIX_BIN="${FASTUSE_BIN_DIR:-$HOME/.local/bin}"
SRC_DIR="$HOME/.fastuse/src"
NO_AGY="${FASTUSE_NO_AGY:-0}"

mkdir -p "$PREFIX_BIN"

detect_root() {
  local script_path="${BASH_SOURCE[0]-}"
  if [ -n "$script_path" ] && [ -f "$script_path" ]; then
    local dir
    dir="$(cd "$(dirname "$script_path")" && pwd)"
    if [ -f "$dir/Cargo.toml" ] && grep -q 'name = "fastuse"' "$dir/Cargo.toml" 2>/dev/null; then
      echo "$dir"
      return 0
    fi
  fi
  if [ -f "$PWD/Cargo.toml" ] && grep -q 'name = "fastuse"' "$PWD/Cargo.toml" 2>/dev/null; then
    echo "$PWD"
    return 0
  fi
  return 1
}

install_prebuilt_if_available() {
  local os arch asset url tmp
  os="$(uname -s)"
  arch="$(uname -m)"
  if [ "$os" != "Darwin" ]; then
    return 1
  fi
  case "$arch" in
    arm64|aarch64) asset="fastuse-macos-arm64.tar.gz" ;;
    x86_64) asset="fastuse-macos-x86_64.tar.gz" ;;
    *) return 1 ;;
  esac
  if [ "$VERSION" = "latest" ]; then
    url="https://github.com/$REPO/releases/latest/download/$asset"
  else
    url="https://github.com/$REPO/releases/download/$VERSION/$asset"
  fi
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' RETURN
  echo "==> checking prebuilt release ($asset)"
  if curl -fsSL "$url" -o "$tmp/$asset" 2>/dev/null; then
    tar -xzf "$tmp/$asset" -C "$tmp"
    if [ -f "$tmp/fastuse/fastuse" ]; then
      cp -f "$tmp/fastuse/fastuse" "$PREFIX_BIN/fastuse"
      chmod +x "$PREFIX_BIN/fastuse"
      if [ -f "$tmp/fastuse/skills/fastuse/SKILL.md" ]; then
        install_skills "$tmp/fastuse/skills/fastuse/SKILL.md"
      fi
      return 0
    fi
  fi
  return 1
}

install_skills() {
  local skill_src="$1"
  if [ "$NO_AGY" = "1" ]; then
    return 0
  fi
  for SKILL_DIR in "$HOME/.gemini/antigravity/global_skills/fastuse" "$HOME/.gemini/config/skills/fastuse"; do
    mkdir -p "$SKILL_DIR"
    cp -f "$skill_src" "$SKILL_DIR/SKILL.md"
  done

  local GEMINI_MD="$HOME/.gemini/config/GEMINI.md"
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
}

ROOT=""
if ROOT="$(detect_root)"; then
  echo "==> using local fastuse source at $ROOT"
elif install_prebuilt_if_available; then
  echo "==> installed prebuilt binary to $PREFIX_BIN/fastuse"
else
  echo "==> no prebuilt release for $(uname -s)/$(uname -m); building from source"
  if ! command -v cargo >/dev/null 2>&1; then
    echo "Error: Rust (cargo) is required to build from source on $(uname -s)." >&2
    echo "Install Rust first:  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y" >&2
    exit 1
  fi
  if [ -d "$SRC_DIR/.git" ]; then
    git -C "$SRC_DIR" fetch --depth 1 origin main
    git -C "$SRC_DIR" reset --hard origin/main
  else
    mkdir -p "$(dirname "$SRC_DIR")"
    git clone --depth 1 "https://github.com/$REPO.git" "$SRC_DIR"
  fi
  ROOT="$SRC_DIR"
fi

if [ -n "$ROOT" ]; then
  echo "==> building fastuse (release)"
  (cd "$ROOT" && cargo build --release)
  cp -f "$ROOT/target/release/fastuse" "$PREFIX_BIN/fastuse"
  chmod +x "$PREFIX_BIN/fastuse"
  install_skills "$ROOT/skills/fastuse/SKILL.md"
fi

# Ensure ~/.local/bin is in PATH for zsh/bash if needed
case ":${PATH}:" in
  *":$PREFIX_BIN:"*) ;;
  *)
    for rc in "$HOME/.zshrc" "$HOME/.bashrc"; do
      if [ -f "$rc" ] && ! grep -q "$PREFIX_BIN" "$rc" 2>/dev/null; then
        printf '\nexport PATH="%s:$PATH"\n' "$PREFIX_BIN" >> "$rc"
      fi
    done
    export PATH="$PREFIX_BIN:$PATH"
    ;;
esac

BIN="$PREFIX_BIN/fastuse"
"$BIN" --version
if [ "$NO_AGY" = "1" ]; then
  "$BIN" setup --no-agy
else
  "$BIN" setup
fi

echo ""
echo "Installed fastuse to: $BIN"
echo "Next steps:"
echo "  1. Open a connected browser (if needed):  fastuse browser-open"
echo "  2. Verify health & local models:          fastuse doctor && fastuse models"
echo "  2. Verify health:             fastuse doctor"
echo "  3. Claude Code MCP add:       claude mcp add fastuse \"$BIN\""
"$BIN" doctor || true

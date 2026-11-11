# fastuse Futuristic Web Experience Design Spec

**Date:** 2026-11-11  
**Status:** Approved for Implementation  
**Figma Workspace:** [fastuse - Futuristic Web Experience](https://www.figma.com/design/ti7QnerquqcAPMFFXHx02d) (Key: `ti7QnerquqcAPMFFXHx02d`)  
**Repo Context:** `c:/Users/Anass/Documents/GitHub/fastuse`  

---

## 1. Intent & Why

`fastuse` is an ultra-fast, zero-focus-steal Computer Use & Browser Use MCP server built in Rust for modern AI agents (Antigravity). Most AI computer-use landing pages look like static, boilerplate marketing brochures with generic purple gradients. 

The goal of this website is to establish `fastuse` as a legendary, high-performance developer tool (matching or exceeding the visual calibre of Linear, Raycast, Vercel, and Stripe). It must **demonstrate** rather than merely assert:
1. **The Zero-Theft Dual-Cursor Advantage:** Show visitors in an interactive live canvas that an AI agent can operate a desktop window while their own mouse and keyboard remain completely unhindered.
2. **The 485x Speedup:** Visual and interactive race benchmarks comparing standard 437ms/turn drivers against fastuse's 0.9ms native execution.
3. **One-Prompt Frictionless Adoption:** Zero-friction copy-paste for Antigravity, PowerShell, and Bash.

---

## 2. Visual Identity & Design Tokens

### Color Palette
- **Canvas Base / Void:** `#07090E` (Deep Space Obsidian)
- **Panel Surface:** `#0D111A` (Card / Section Surface)
- **Elevated Border:** `#1B2234` with subtle hover glow `rgba(47, 107, 255, 0.4)`
- **Brand Electric Blue:** `#2F6BFF` (Primary Brand & Glow)
- **High-Energy Cyan:** `#00F5D4` (Telemetry & Agent Cursor Glow)
- **Terminal Green:** `#00F5A0` (Success & Millisecond Speed Indicators)
- **Warning Crimson:** `#FF3366` (Legacy Driver Bottleneck Indicators)
- **Text Primary:** `#F3F5F9`
- **Text Secondary:** `#8F9CAE`
- **Text Muted / Monospace:** `#526077`

### Typography Hierarchy
- **Display Headlines:** Space Grotesk / Inter Display (`font-display font-bold tracking-tight`)
- **Code & Telemetry:** JetBrains Mono (`font-mono`)
- **Body & Prose:** Inter / System UI (`font-sans leading-relaxed`)

### Micro-Interactions & Motion Principles
- 60fps hardware-accelerated canvas background (cyber grid with dynamic mouse aura).
- Tactile audio design (Web Audio API synthetic futuristic blips/whooshes, mutted by default with an aesthetic unmute toggle).
- Interactive drag/hover cards with glassmorphic backdrop filters.

---

## 3. Structural Information Architecture (SPA)

```
┌────────────────────────────────────────────────────────────────────────┐
│  NAVIGATION: Brand Logo (fastuse ⚡) · Interactive Demo · Benchmarks   │
│              Architecture · Playground · Docs · GitHub Stars (Live)    │
├────────────────────────────────────────────────────────────────────────┤
│  HERO SECTION                                                          │
│  - Cyberpunk HUD Status Badge: [● FASTUSE KERNEL 0.9ms DISPATCH]       │
│  - Main Headline: "Computer Use at the Speed of Thought."              │
│  - Subtitle: Zero focus theft. Your real browser. 485x faster.         │
│  - One-Prompt Antigravity CTA + OS Switcher (Win/Mac) + GitHub Star    │
│  - Live Interactive Dual-Cursor Sandbox Widget                         │
├────────────────────────────────────────────────────────────────────────┤
│  INTERACTIVE BENCHMARK LAB                                             │
│  - Side-by-side race animation (CUA 437ms vs fastuse 0.9ms)           │
│  - Interactive "Run Speed Test" trigger with live millisecond tickers   │
├────────────────────────────────────────────────────────────────────────┤
│  ARCHITECTURE & MCP PROTOCOL MATRIX                                    │
│  - Visual Node Graph: Agent ↔ MCP Server ↔ Windows UIA / macOS AX      │
│  - Real Browser Bridge: Chrome Native Messaging Host                   │
├────────────────────────────────────────────────────────────────────────┤
│  LIVE INTERACTIVE PLAYGROUND (CLI / MCP INSPECTOR)                     │
│  - Simulates fastuse doctor, act, and browser_act                      │
│  - Real-time JSON-RPC response viewer with syntax highlighting         │
├────────────────────────────────────────────────────────────────────────┤
│  ONE-PROMPT INSTALLATION & ONBOARDING HUB                              │
│  - Tabbed: Antigravity One-Prompt · PowerShell · Bash · Manual Config  │
│  - One-click copy with haptic/visual confirmation pulse                │
├────────────────────────────────────────────────────────────────────────┤
│  DOCUMENTATION & API REFERENCE (SPLIT PANE)                            │
│  - Sidebar: Quickstart, Desktop Tools (act, etc.), Browser Tools       │
│  - Content: Rich examples, parameter schemas, copyable snippets        │
├────────────────────────────────────────────────────────────────────────┤
│  FOOTER: GitHub, MIT License, System Telemetry, Social Links           │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Google Flow / Google Stash Thematic Asset Pipeline

dj can generate bespoke 3D/cybernetic visual assets via Google Flow / Google Stash in the browser:
- **Prompts & Directives:**
  1. *Hero 3D Object:* "Futuristic floating holographic computer processor core, transparent glass casing, glowing neon cobalt blue circuits, dark obsidian studio lighting, 8k render, octane render style"
  2. *Cursor Aura Texture:* "Luminous cyan-blue energy particle field, dark void background, ethereal plasma glow, abstract scientific data stream"
  3. *Browser Bridge Illustration:* "Stylized chrome browser window with glowing native bridge wires connecting to a central glowing core, cyberpunk minimalist isometric view"
- The website architecture incorporates dedicated asset slots with CSS/SVG fallback generators so the site looks stunning immediately, and any exported image or video loop from Google Flow/Stash can simply be dropped into `public/assets/` to replace the procedural graphics.

---

## 5. Technical Stack

- **Framework:** Vite + React 19 + TypeScript
- **Styling:** Tailwind CSS v4 + PostCSS
- **Icons:** Lucide React
- **Animations:** Custom CSS 3D Transforms, Canvas 2D hardware-accelerated particle/grid engine, and Web Audio API synthesizer
- **Output:** Fully self-contained static build in `site/dist` ready for Vercel, Cloudflare Pages, or GitHub Pages.

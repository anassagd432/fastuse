# fastuse Futuristic Website Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a world-class, futuristic, well-animated product website and interactive playground for `fastuse` with a live dual-cursor simulation sandbox, benchmark race lab, interactive MCP terminal, and one-prompt installer.

**Architecture:** A high-performance React + TypeScript + Tailwind CSS Single-Page Application located in `site/` featuring hardware-accelerated Canvas particle/grid background, an interactive dual-cursor sandbox simulating zero focus theft, an interactive benchmark laboratory with live tickers, an MCP JSON-RPC inspector, and a comprehensive split-pane documentation explorer.

**Tech Stack:** React 19, TypeScript, Vite, Tailwind CSS, Lucide React, HTML5 Canvas, Web Audio API.

**Spec:** [`docs/superpowers/specs/2026-11-11-fastuse-website-design.md`](file:///c:/Users/Anass/Documents/GitHub/fastuse/docs/superpowers/specs/2026-11-11-fastuse-website-design.md)  
**Figma File:** [fastuse - Futuristic Web Experience](https://www.figma.com/design/ti7QnerquqcAPMFFXHx02d) (Key: `ti7QnerquqcAPMFFXHx02d`)

## Global Constraints
- Target directory: `site/` inside the `fastuse` repository.
- Aesthetic palette: Deep Space Obsidian `#07090E`, Card Surface `#0D111A`, Border `#1B2234`, Brand Blue `#2F6BFF`, High-Energy Cyan `#00F5D4`, Terminal Green `#00F5A0`.
- Zero AI-slop: No generic purple/indigo gradients; sharp technical HUD aesthetic, crisp typography (Space Grotesk + JetBrains Mono), responsive layout.
- Fully runnable locally via `npm run dev` and static build via `npm run build`.
- Zero broken links; all tabs, copy buttons, simulators, and navigation anchors must be fully interactive.

## Review Focus
1. **Interactive Dual-Cursor Sandbox:** User's cursor and the simulated Fastuse Agent cursor must coexist smoothly without jerky physics or locking the user's cursor.
2. **Benchmark Race Simulator:** Running the benchmark must reliably animate both progress bars with real millisecond numbers and allow re-running multiple times without glitching.
3. **Clipboard Copy Actions:** One-prompt copy button must copy exact bash/powershell/antigravity commands to clipboard with clear visual confirmation.
4. **Interactive MCP Terminal / Playground:** Clicking actions (`fastuse doctor`, `act`, `browser_act`) must render instant formatted JSON outputs with realistic simulated payloads.
5. **Mobile & Viewport Responsiveness:** High-density HUD elements and split-pane documentation must gracefully collapse into touch-friendly views on smaller screens.

---

### Task 1: Project Scaffolding & Design Tokens

**Files:**
- Create: `site/package.json`
- Create: `site/tsconfig.json`
- Create: `site/tsconfig.node.json`
- Create: `site/vite.config.ts`
- Create: `site/index.html`
- Create: `site/src/index.css`
- Create: `site/src/main.tsx`

**Interfaces:**
- Produces: Runnable Vite React 19 app with Tailwind styling, Space Grotesk and JetBrains Mono fonts, and core design token classes (`bg-space-void`, `border-panel`, `text-neon-cyan`, etc.).

- [x] **Step 1: Create package.json and configuration files**
- [x] **Step 2: Install dependencies (React 19, Vite, Tailwind, Lucide React)**
- [x] **Step 3: Configure index.html with futuristic typography and meta tags**
- [x] **Step 4: Configure index.css with custom design tokens, cyber grid, and glow utilities**
- [x] **Step 5: Verify dev server builds cleanly**

---

### Task 2: Background Canvas Grid & Cyber Audio Engine

**Files:**
- Create: `site/src/components/CyberCanvas.tsx`
- Create: `site/src/utils/audio.ts`
- Create: `site/src/components/Navbar.tsx`

**Interfaces:**
- Consumes: Canvas rendering contexts, Web Audio API synthesizer.
- Produces: `CyberCanvas` component (ambient grid lines, floating glowing particles that react to mouse hover) and `audio.ts` (subtle futuristic click, hover, and shockwave audio cues with mute toggle), and `Navbar` with live GitHub stars and section navigation.

- [x] **Step 1: Implement Web Audio synthesis helper with mute toggle**
- [x] **Step 2: Implement hardware-accelerated 60fps 2D Canvas matrix / particle grid**
- [x] **Step 3: Implement futuristic sticky Navigation Bar with live links and audio toggle**
- [x] **Step 4: Verify rendering and interaction**

---

### Task 3: Hero Section & Live Dual-Cursor Sandbox

**Files:**
- Create: `site/src/components/Hero.tsx`
- Create: `site/src/components/DualCursorSandbox.tsx`

**Interfaces:**
- Consumes: Design tokens, audio helpers.
- Produces: `Hero` component with dynamic HUD badges, headline, quick install copy prompt, and the `DualCursorSandbox` component demonstrating real-time independent agent cursor automation while visitor types/clicks simultaneously.

- [x] **Step 1: Build the Hero header, status badge, and CTA buttons**
- [x] **Step 2: Build the DualCursorSandbox component simulating background OS execution**
- [x] **Step 3: Integrate autonomous gliding agent cursor with click animations and status log**
- [x] **Step 4: Verify smooth dual-interaction and responsive layout**

---

### Task 4: Interactive Benchmark Lab & Architecture Hologram

**Files:**
- Create: `site/src/components/BenchmarkLab.tsx`
- Create: `site/src/components/ArchitectureGraph.tsx`

**Interfaces:**
- Consumes: Measured benchmark data from `README.md`.
- Produces: `BenchmarkLab` with an interactive "Run Race" animation (0.9ms fastuse vs 437ms CUA), and `ArchitectureGraph` illustrating UIA / AX dispatch and Chrome Native Messaging Host.

- [x] **Step 1: Implement BenchmarkLab with live millisecond ticker and progress comparison**
- [x] **Step 2: Implement ArchitectureGraph with interactive nodes and glowing data pulse lines**
- [x] **Step 3: Verify benchmark trigger and node inspection**

---

### Task 5: Interactive MCP Playground & Terminal

**Files:**
- Create: `site/src/components/TerminalPlayground.tsx`

**Interfaces:**
- Produces: Interactive CLI / MCP Inspector where users can trigger `fastuse doctor`, `act (click/type)`, `browser_act (DOM)`, and `list_windows`, viewing syntax-highlighted JSON-RPC outputs.

- [x] **Step 1: Define simulated MCP methods and realistic JSON response trees**
- [x] **Step 2: Implement interactive terminal with command picker and execution animation**
- [x] **Step 3: Add copy response and clear terminal controls**
- [x] **Step 4: Verify terminal execution**

---

### Task 6: One-Prompt Onboarding & Split-Pane Documentation

**Files:**
- Create: `site/src/components/InstallHub.tsx`
- Create: `site/src/components/DocsViewer.tsx`
- Create: `site/src/components/Footer.tsx`
- Modify: `site/src/App.tsx`

**Interfaces:**
- Produces: `InstallHub` (Antigravity one-prompt, Windows PowerShell, macOS Bash, manual config), `DocsViewer` (collapsible split-pane API documentation), and complete assembled `App.tsx`.

- [x] **Step 1: Implement InstallHub with tabbed OS commands and copy feedback**
- [x] **Step 2: Implement DocsViewer with categorized API reference, search filter, and parameter tables**
- [x] **Step 3: Implement Footer with system status and repo links**
- [x] **Step 4: Assemble all components in `App.tsx`**
- [x] **Step 5: Run full production build (`npm run build`) and test output**

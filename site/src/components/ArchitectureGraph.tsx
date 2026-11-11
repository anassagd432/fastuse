import React, { useState } from 'react';
import { Layers, Monitor, Globe, Shield, Terminal, Zap, ArrowDown, CheckCircle2 } from 'lucide-react';
import { sound } from '../utils/audio';

interface NodeInfo {
  id: string;
  title: string;
  badge: string;
  description: string;
  details: string[];
  codeSnippet: string;
}

const NODES: Record<string, NodeInfo> = {
  agent: {
    id: 'agent',
    title: 'Antigravity & AI Agents',
    badge: 'LLM Orchestrator',
    description: 'Autonomous coding agents issue high-level user tasks without worrying about window focus or stealing user input.',
    details: [
      'Standard Model Context Protocol (MCP) JSON-RPC 2.0',
      'Single-turn "act" batch workflows',
      'Eliminates 4-6 round trips per user turn',
    ],
    codeSnippet: '{\n  "jsonrpc": "2.0",\n  "method": "tools/call",\n  "params": {\n    "name": "act",\n    "arguments": {\n      "steps": [{ "click": { "element": "#submit-btn" } }]\n    }\n  }\n}',
  },
  engine: {
    id: 'engine',
    title: 'fastuse Rust Core Engine',
    badge: 'Rust 2021 · Zero Cost',
    description: 'High-speed native daemon maintaining persistent window trees, accessibility handles, and IPC connections.',
    details: [
      'Compiled with zero-overhead native C/Win32/CoreGraphics bindings',
      'Millisecond UI tree depth caching & diffing',
      'Multi-threaded overlay rendering loop',
    ],
    codeSnippet: '// src/server.rs - Single dispatch entry point\npub fn dispatch<F, R>(&self, f: F) -> R\nwhere F: FnOnce() -> R {\n    self.p.ping_overlay(true);\n    let res = f();\n    self.p.ping_overlay(false);\n    res\n}',
  },
  desktop: {
    id: 'desktop',
    title: 'Desktop Engine (UIA & AX)',
    badge: 'Windows 11 · macOS',
    description: 'Dispatches clicks, typing, and tree inspection straight through OS accessibility interfaces without moving the physical cursor.',
    details: [
      'Windows: UI Automation COM interfaces + Win32 PostMessage/WM_CHAR',
      'macOS: AXUIElement APIs + CoreGraphics event taps',
      'User mouse and keyboard stay 100% free',
    ],
    codeSnippet: '// Windows: Send WM_CHAR directly to window handle\nPostMessageW(hwnd, WM_CHAR, ch as WPARAM, 1);\n// OS mouse position is never altered',
  },
  browser: {
    id: 'browser',
    title: 'Chrome Native Messaging Bridge',
    badge: 'Brave · Chrome · Edge',
    description: 'Connects directly to your running browser process over stdin/stdout IPC. Keeps all cookies, logins, and active tabs.',
    details: [
      'No headless Chromium sandbox that forgets your login state',
      'Direct DOM querySelector and event dispatching',
      'Shadow DOM brand-blue overlay visual cues',
    ],
    codeSnippet: '// Native Messaging Host JSON exchange\n{\n  "type": "browser_act",\n  "action": "click",\n  "selector": "button.submit"\n}',
  },
  overlay: {
    id: 'overlay',
    title: 'Visual Overlay & Aura',
    badge: 'Direct2D & CoreAnimation',
    description: 'Draws the independent brand-blue glowing cursor and screen aura so you always know what the agent is doing.',
    details: [
      'Transparent click-through WS_EX_TRANSPARENT window',
      'Independent cursor glides smoothly to target controls',
      'Screen-edge aura keeps glowing during non-mouse execution',
    ],
    codeSnippet: '// src/overlay.rs\nMsg::Active(true) => {\n    // Pin banner & breathing aura at 100% opacity\n    state.active = true;\n}',
  },
};

export const ArchitectureGraph: React.FC = () => {
  const [activeNode, setActiveNode] = useState<NodeInfo>(NODES.engine);

  const handleSelect = (key: string) => {
    setActiveNode(NODES[key]);
    sound.playHover();
  };

  return (
    <section id="architecture" className="py-24 px-4 sm:px-6 lg:px-8 max-w-7xl mx-auto scroll-mt-20">
      <div className="text-center max-w-3xl mx-auto mb-16 space-y-4">
        <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-[#0D111A] border border-[#1F283D] text-xs font-mono text-[#2F6BFF]">
          <Layers className="w-3.5 h-3.5" />
          <span>SYSTEM ARCHITECTURE</span>
        </div>
        <h2 className="font-display font-extrabold text-3xl sm:text-5xl tracking-tight text-[#F3F5F9]">
          How <span className="text-transparent bg-clip-text bg-gradient-to-r from-[#2F6BFF] to-[#00F5D4]">Zero-Theft</span> Works.
        </h2>
        <p className="text-sm sm:text-base text-[#94A3B8] font-sans">
          Click any component node below to inspect protocol telemetry and native dispatch mechanics.
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 items-center">
        {/* Left Column: Interactive Visual Node Diagram */}
        <div className="lg:col-span-6 space-y-4">
          {/* Agent Node */}
          <div
            onClick={() => handleSelect('agent')}
            className={`p-4 rounded-xl border cursor-pointer transition-all ${
              activeNode.id === 'agent'
                ? 'bg-[#131A29] border-[#00F5D4] shadow-lg shadow-[#00F5D4]/15'
                : 'bg-[#0D111A] border-[#1F283D] hover:border-[#2F6BFF]'
            }`}
          >
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="p-2 rounded-lg bg-[#2F6BFF]/20 text-[#00F5D4]">
                  <Terminal className="w-4 h-4" />
                </div>
                <div>
                  <div className="font-display font-bold text-sm text-[#F3F5F9]">
                    ANTIGRAVITY AGENT
                  </div>
                  <div className="text-[11px] font-mono text-[#94A3B8]">
                    High-level prompts · MCP client
                  </div>
                </div>
              </div>
              <span className="text-[10px] font-mono bg-[#1F283D] px-2 py-0.5 rounded text-[#94A3B8]">
                JSON-RPC
              </span>
            </div>
          </div>

          <div className="flex justify-center -my-2 text-[#2F6BFF]">
            <ArrowDown className="w-4 h-4 animate-bounce" />
          </div>

          {/* Engine Node (Center Core) */}
          <div
            onClick={() => handleSelect('engine')}
            className={`p-4 rounded-xl border cursor-pointer transition-all ${
              activeNode.id === 'engine'
                ? 'bg-[#131A29] border-[#00F5D4] shadow-lg shadow-[#00F5D4]/15'
                : 'bg-[#0D111A] border-[#1F283D] hover:border-[#2F6BFF]'
            }`}
          >
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="p-2 rounded-lg bg-[#00F5D4]/20 text-[#00F5D4]">
                  <Zap className="w-4 h-4" />
                </div>
                <div>
                  <div className="font-display font-bold text-sm text-[#F3F5F9]">
                    fastuse RUST ENGINE
                  </div>
                  <div className="text-[11px] font-mono text-[#00F5D4]">
                    Sub-millisecond dispatch · Zero focus steal
                  </div>
                </div>
              </div>
              <span className="text-[10px] font-mono bg-[#00F5A0]/20 text-[#00F5A0] px-2 py-0.5 rounded font-bold">
                0.9ms Core
              </span>
            </div>
          </div>

          <div className="flex justify-center -my-2 text-[#2F6BFF]">
            <ArrowDown className="w-4 h-4" />
          </div>

          {/* Dual Dispatches: Desktop vs Browser */}
          <div className="grid grid-cols-2 gap-4">
            <div
              onClick={() => handleSelect('desktop')}
              className={`p-3.5 rounded-xl border cursor-pointer transition-all ${
                activeNode.id === 'desktop'
                  ? 'bg-[#131A29] border-[#00F5D4] shadow-lg shadow-[#00F5D4]/15'
                  : 'bg-[#0D111A] border-[#1F283D] hover:border-[#2F6BFF]'
              }`}
            >
              <div className="flex items-center gap-2 mb-1.5">
                <Monitor className="w-4 h-4 text-[#2F6BFF]" />
                <span className="font-display font-bold text-xs text-[#F3F5F9]">
                  Desktop Engine
                </span>
              </div>
              <p className="text-[10px] font-mono text-[#94A3B8]">
                Win32 UIA + macOS AX (background input)
              </p>
            </div>

            <div
              onClick={() => handleSelect('browser')}
              className={`p-3.5 rounded-xl border cursor-pointer transition-all ${
                activeNode.id === 'browser'
                  ? 'bg-[#131A29] border-[#00F5D4] shadow-lg shadow-[#00F5D4]/15'
                  : 'bg-[#0D111A] border-[#1F283D] hover:border-[#2F6BFF]'
              }`}
            >
              <div className="flex items-center gap-2 mb-1.5">
                <Globe className="w-4 h-4 text-[#00F5D4]" />
                <span className="font-display font-bold text-xs text-[#F3F5F9]">
                  Browser Bridge
                </span>
              </div>
              <p className="text-[10px] font-mono text-[#94A3B8]">
                Brave/Chrome/Edge native messaging
              </p>
            </div>
          </div>

          {/* Overlay Node */}
          <div
            onClick={() => handleSelect('overlay')}
            className={`p-3.5 rounded-xl border cursor-pointer transition-all ${
              activeNode.id === 'overlay'
                ? 'bg-[#131A29] border-[#00F5D4] shadow-lg shadow-[#00F5D4]/15'
                : 'bg-[#0D111A] border-[#1F283D] hover:border-[#2F6BFF]'
            }`}
          >
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Shield className="w-4 h-4 text-[#00F5D4]" />
                <span className="font-display font-bold text-xs text-[#F3F5F9]">
                  Visual Overlay & Screen-Edge Aura
                </span>
              </div>
              <span className="text-[10px] font-mono text-[#548AFF]">Independent Cursor</span>
            </div>
          </div>
        </div>

        {/* Right Column: Node Detailed Inspector */}
        <div className="lg:col-span-6 cyber-panel rounded-2xl p-6 sm:p-8 space-y-5">
          <div className="flex items-center justify-between border-b border-[#1F283D] pb-3">
            <div>
              <span className="text-[10px] font-mono uppercase tracking-wider text-[#00F5D4]">
                {activeNode.badge}
              </span>
              <h3 className="font-display font-bold text-xl text-[#F3F5F9] mt-0.5">
                {activeNode.title}
              </h3>
            </div>
          </div>

          <p className="text-xs font-sans text-[#94A3B8] leading-relaxed">
            {activeNode.description}
          </p>

          <div className="space-y-2">
            <span className="text-xs font-mono font-semibold text-[#F3F5F9]">
              Key Architectural Guarantees:
            </span>
            <ul className="space-y-1.5">
              {activeNode.details.map((detail, idx) => (
                <li key={idx} className="flex items-start gap-2 text-xs font-mono text-[#94A3B8]">
                  <CheckCircle2 className="w-3.5 h-3.5 text-[#00F5A0] shrink-0 mt-0.5" />
                  <span>{detail}</span>
                </li>
              ))}
            </ul>
          </div>

          {/* Code Snippet Box */}
          <div className="space-y-1.5">
            <span className="text-[10px] font-mono text-[#548AFF]">Implementation Detail:</span>
            <div className="p-3 rounded-xl bg-[#07090E] border border-[#161D2B] font-mono text-[11px] text-[#00F5D4] overflow-x-auto leading-relaxed shadow-inner">
              <pre>{activeNode.codeSnippet}</pre>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};

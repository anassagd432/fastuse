import React, { useState } from 'react';
import { Zap, Copy, Check, Terminal, ArrowRight, Cpu } from 'lucide-react';
import { DualCursorSandbox } from './DualCursorSandbox';
import { sound } from '../utils/audio';

export const Hero: React.FC = () => {
  const [copied, setCopied] = useState(false);
  const promptText = `Install and configure fastuse MCP:
1. Detect my OS (Windows or macOS).
2. Run installation script and register in ~/.gemini/config/mcp_config.json.
3. Add fastuse auto-routing rules to GEMINI.md.
4. Run fastuse doctor.`;

  const handleCopy = () => {
    navigator.clipboard.writeText(promptText);
    setCopied(true);
    sound.playSuccess();
    setTimeout(() => setCopied(false), 2500);
  };

  return (
    <section className="relative pt-32 pb-20 px-4 sm:px-6 lg:px-8 max-w-7xl mx-auto overflow-hidden">
      {/* Background radial gradient spotlight */}
      <div className="absolute top-1/4 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[300px] bg-[#2F6BFF]/15 blur-[120px] rounded-full pointer-events-none -z-10" />

      {/* Hero Header */}
      <div className="text-center max-w-4xl mx-auto space-y-6">
        {/* Status Badge */}
        <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-[#0D111A] border border-[#1F283D] shadow-lg shadow-[#2F6BFF]/10 text-xs font-mono">
          <span className="w-2 h-2 rounded-full bg-[#00F5D4] animate-pulse" />
          <span className="text-[#94A3B8]">MCP PROTOCOL ACTIVE:</span>
          <span className="text-[#00F5D4] font-semibold">0.9ms Native Dispatch</span>
          <span className="hidden sm:inline text-[#1F283D]">|</span>
          <span className="hidden sm:inline text-[#548AFF]">Zero Focus Theft</span>
        </div>

        {/* Main Headline */}
        <h1 className="font-display font-extrabold text-4xl sm:text-6xl lg:text-7xl tracking-tight text-[#F3F5F9] leading-[1.08]">
          Computer Use at the{' '}
          <span className="text-transparent bg-clip-text bg-gradient-to-r from-[#2F6BFF] via-[#00F5D4] to-[#548AFF] drop-shadow-[0_0_35px_rgba(47,107,255,0.4)]">
            Speed of Thought.
          </span>
        </h1>

        {/* Subtitle */}
        <p className="text-base sm:text-xl text-[#94A3B8] max-w-2xl mx-auto font-sans leading-relaxed">
          The background-first Computer & Browser Use engine for{' '}
          <span className="text-[#F3F5F9] font-medium">Antigravity</span>. Never steals your mouse cursor.
          Connects to your real, logged-in browser. Runs up to <span className="text-[#00F5D4] font-semibold">485x faster</span>.
        </p>

        {/* Action Buttons & One-Prompt Copy Box */}
        <div className="pt-2 flex flex-col sm:flex-row items-center justify-center gap-4">
          <a
            href="#install"
            className="w-full sm:w-auto flex items-center justify-center gap-2 px-6 py-3.5 rounded-xl bg-gradient-to-r from-[#2F6BFF] to-[#1A4BCC] hover:from-[#487DFF] hover:to-[#2F6BFF] text-white font-display font-bold text-sm tracking-wide shadow-xl shadow-[#2F6BFF]/30 hover:shadow-[#2F6BFF]/55 transition-all active:scale-95"
            onClick={() => sound.playClick()}
          >
            <Zap className="w-4 h-4 text-[#00F5D4]" />
            <span>Install in 1 Prompt</span>
            <ArrowRight className="w-4 h-4 ml-1" />
          </a>

          <a
            href="#benchmarks"
            className="w-full sm:w-auto flex items-center justify-center gap-2 px-6 py-3.5 rounded-xl bg-[#0D111A] hover:bg-[#161E30] border border-[#1F283D] hover:border-[#2F6BFF] text-[#F3F5F9] font-mono text-xs transition-all"
            onClick={() => sound.playClick()}
          >
            <Cpu className="w-4 h-4 text-[#94A3B8]" />
            <span>View 0.9ms Benchmarks</span>
          </a>
        </div>

        {/* One-Prompt Quick Copy Bar */}
        <div className="pt-4 max-w-2xl mx-auto">
          <div className="flex items-center justify-between p-2.5 rounded-xl bg-[#0B0F17]/90 border border-[#1F283D] hover:border-[#2F6BFF]/50 transition-all shadow-inner">
            <div className="flex items-center gap-2.5 overflow-hidden text-left pl-2">
              <Terminal className="w-4 h-4 text-[#00F5D4] shrink-0" />
              <span className="font-mono text-xs text-[#94A3B8] truncate">
                Paste prompt in Antigravity: <span className="text-[#E2E8F0]">Install and configure fastuse MCP...</span>
              </span>
            </div>
            <button
              onClick={handleCopy}
              className={`shrink-0 flex items-center gap-1.5 px-3 py-1.5 rounded-lg font-mono text-xs font-semibold transition-all ${
                copied
                  ? 'bg-[#00F5A0]/20 text-[#00F5A0] border border-[#00F5A0]/40'
                  : 'bg-[#161E30] text-[#00F5D4] hover:bg-[#2F6BFF] hover:text-white border border-[#1F283D]'
              }`}
            >
              {copied ? (
                <>
                  <Check className="w-3.5 h-3.5" /> Copied
                </>
              ) : (
                <>
                  <Copy className="w-3.5 h-3.5" /> Copy Prompt
                </>
              )}
            </button>
          </div>
        </div>

        {/* Telemetry Highlights Strip */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-3 pt-6 max-w-4xl mx-auto">
          {[
            { metric: '0.9 ms', label: 'Window Listing', sub: 'vs 437ms CUA driver' },
            { metric: '0%', label: 'Cursor Theft', sub: 'Background UIA / AX dispatch' },
            { metric: '100%', label: 'Logged-In Tabs', sub: 'Chrome Native Messaging' },
            { metric: 'Single', label: 'Turn Batching', sub: 'All-in-one act execution' },
          ].map((item, i) => (
            <div
              key={i}
              className="p-3 rounded-xl bg-[#0D111A]/60 border border-[#1F283D] hover:border-[#2F6BFF]/40 transition-all text-left"
              onMouseEnter={() => sound.playHover()}
            >
              <div className="font-mono font-bold text-xl sm:text-2xl text-[#00F5D4]">
                {item.metric}
              </div>
              <div className="font-display font-semibold text-xs text-[#F3F5F9] mt-0.5">
                {item.label}
              </div>
              <div className="text-[10px] font-mono text-[#94A3B8] mt-0.5 truncate">
                {item.sub}
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Live Dual-Cursor Sandbox Widget */}
      <div id="sandbox" className="mt-14 scroll-mt-24">
        <DualCursorSandbox />
      </div>
    </section>
  );
};

import React from 'react';
import { Zap, Github, Terminal, ArrowUp } from 'lucide-react';
import { sound } from '../utils/audio';

export const Footer: React.FC = () => {
  const scrollToTop = () => {
    window.scrollTo({ top: 0, behavior: 'smooth' });
    sound.playClick();
  };

  return (
    <footer className="border-t border-[#1F283D] bg-[#05070B] py-14 px-4 sm:px-6 lg:px-8 relative z-10">
      <div className="max-w-7xl mx-auto flex flex-col md:flex-row items-center justify-between gap-8">
        {/* Brand Column */}
        <div className="space-y-3 text-center md:text-left">
          <div className="flex items-center justify-center md:justify-start gap-2.5">
            <div className="w-7 h-7 rounded-lg bg-gradient-to-br from-[#2F6BFF] to-[#00F5D4] p-[1px]">
              <div className="w-full h-full bg-[#0D111A] rounded-[7px] flex items-center justify-center">
                <Zap className="w-4 h-4 text-[#00F5D4]" />
              </div>
            </div>
            <span className="font-display font-extrabold text-base tracking-tight text-[#F3F5F9]">
              fastuse
            </span>
          </div>
          <p className="text-xs font-mono text-[#94A3B8] max-w-sm">
            Ultra-fast, zero-focus-steal Computer Use & Browser Use MCP server for Antigravity and modern AI agents.
          </p>
          <div className="flex items-center justify-center md:justify-start gap-2 text-[11px] font-mono text-[#00F5A0]">
            <span className="w-2 h-2 rounded-full bg-[#00F5A0] animate-pulse" />
            <span>Open Source under MIT License</span>
          </div>
        </div>

        {/* Links Column */}
        <div className="flex flex-wrap items-center justify-center gap-6 text-xs font-mono text-[#94A3B8]">
          <a
            href="https://github.com/anassagd432/fastuse"
            target="_blank"
            rel="noopener noreferrer"
            className="hover:text-[#00F5D4] transition-all flex items-center gap-1.5"
            onMouseEnter={() => sound.playHover()}
          >
            <Github className="w-4 h-4" />
            <span>GitHub Repository</span>
          </a>
          <a
            href="https://github.com/anassagd432/fastuse/releases"
            target="_blank"
            rel="noopener noreferrer"
            className="hover:text-[#00F5D4] transition-all flex items-center gap-1.5"
            onMouseEnter={() => sound.playHover()}
          >
            <Terminal className="w-4 h-4" />
            <span>Releases</span>
          </a>
          <a
            href="https://antigravity.google"
            target="_blank"
            rel="noopener noreferrer"
            className="hover:text-[#00F5D4] transition-all"
            onMouseEnter={() => sound.playHover()}
          >
            Antigravity Integration
          </a>
        </div>

        {/* Back to top */}
        <button
          onClick={scrollToTop}
          className="p-3 rounded-xl bg-[#0D111A] border border-[#1F283D] hover:border-[#2F6BFF] text-[#94A3B8] hover:text-[#00F5D4] transition-all flex items-center gap-2 text-xs font-mono group"
        >
          <span>Top</span>
          <ArrowUp className="w-3.5 h-3.5 group-hover:-translate-y-0.5 transition-transform" />
        </button>
      </div>

      <div className="max-w-7xl mx-auto mt-10 pt-6 border-t border-[#161D2B] text-center text-[11px] font-mono text-[#4B5565] flex flex-col sm:flex-row items-center justify-between gap-2">
        <span>© 2026 fastuse engine · Built with precision for autonomous agents.</span>
        <span className="flex items-center gap-1 text-[#94A3B8]">
          0.9ms dispatch <span className="text-[#00F5D4]">⚡</span> zero cursor steal
        </span>
      </div>
    </footer>
  );
};

import React, { useState } from 'react';
import { Copy, Check, Download, ShieldCheck, CheckCircle2 } from 'lucide-react';
import { sound } from '../utils/audio';

interface InstallTab {
  id: string;
  label: string;
  icon?: string;
  command: string;
  description: string;
}

const TABS: InstallTab[] = [
  {
    id: 'antigravity',
    label: '⚡ Antigravity (1-Prompt)',
    command: `Install and configure the \`fastuse\` MCP server on this computer:
1. Detect my OS (Windows or macOS).
2. If on Windows, run in terminal:
   powershell -ExecutionPolicy Bypass -Command "irm https://github.com/anassagd432/fastuse/releases/latest/download/install.ps1 | iex"
   If on macOS, run in terminal:
   curl -fsSL https://raw.githubusercontent.com/anassagd432/fastuse/main/install.sh | bash
3. Ensure \`fastuse\` is registered in my Antigravity MCP config (~/.gemini/config/mcp_config.json) pointing to the installed fastuse binary.
4. Ensure the fastuse skill is placed in ~/.gemini/config/skills/fastuse/SKILL.md.
5. In my ~/.gemini/config/GEMINI.md file, ensure fastuse default routing rule is present.
6. Run \`fastuse doctor\` and confirm the installation is verified and ready.`,
    description: 'Simply paste this prompt into your Antigravity chat. The agent handles binary download, path setup, and auto-routing rules automatically.',
  },
  {
    id: 'windows',
    label: 'Windows (PowerShell)',
    command: 'powershell -ExecutionPolicy Bypass -Command "irm https://github.com/anassagd432/fastuse/releases/latest/download/install.ps1 | iex"',
    description: 'Installs the native Windows binary, registers UI Automation hooks, Direct2D overlay engine, and Chrome Native Messaging Host.',
  },
  {
    id: 'macos',
    label: 'macOS (Terminal)',
    command: 'curl -fsSL https://raw.githubusercontent.com/anassagd432/fastuse/main/install.sh | bash',
    description: 'Installs the universal binary, sets up AXUIElement permissions, and creates the Chrome Native Messaging host manifest.',
  },
  {
    id: 'cargo',
    label: 'Cargo / Source',
    command: 'cargo install --git https://github.com/anassagd432/fastuse',
    description: 'Compiles the binary directly from source with maximum Rust compiler optimizations.',
  },
];

export const InstallHub: React.FC = () => {
  const [activeTab, setActiveTab] = useState<InstallTab>(TABS[0]);
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(activeTab.command);
    setCopied(true);
    sound.playSuccess();
    setTimeout(() => setCopied(false), 2200);
  };

  const handleSelectTab = (tab: InstallTab) => {
    setActiveTab(tab);
    sound.playHover();
  };

  return (
    <section id="install" className="py-24 px-4 sm:px-6 lg:px-8 max-w-7xl mx-auto scroll-mt-20">
      <div className="text-center max-w-3xl mx-auto mb-14 space-y-4">
        <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-[#0D111A] border border-[#1F283D] text-xs font-mono text-[#2F6BFF]">
          <Download className="w-3.5 h-3.5 text-[#00F5D4]" />
          <span>EFFORTLESS ONBOARDING</span>
        </div>
        <h2 className="font-display font-extrabold text-3xl sm:text-5xl tracking-tight text-[#F3F5F9]">
          Get Started in <span className="text-[#00F5D4]">One Prompt</span>.
        </h2>
        <p className="text-sm sm:text-base text-[#94A3B8] font-sans">
          No manual JSON editing, no PATH troubleshooting. Choose your setup route below.
        </p>
      </div>

      <div className="max-w-4xl mx-auto cyber-panel rounded-2xl overflow-hidden border border-[#1F283D] shadow-2xl">
        {/* OS Tabs */}
        <div className="bg-[#090C13] border-b border-[#1F283D] px-3 sm:px-6 flex flex-wrap items-center justify-between gap-2">
          <div className="flex flex-wrap items-center gap-1">
            {TABS.map((tab) => (
              <button
                key={tab.id}
                onClick={() => handleSelectTab(tab)}
                className={`py-3.5 px-3 sm:px-4 text-xs font-mono font-medium transition-all border-b-2 -mb-[1px] ${
                  activeTab.id === tab.id
                    ? 'border-[#00F5D4] text-[#00F5D4] font-bold'
                    : 'border-transparent text-[#94A3B8] hover:text-[#F3F5F9]'
                }`}
              >
                {tab.label}
              </button>
            ))}
          </div>

          <button
            onClick={handleCopy}
            className={`my-2 flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg text-xs font-mono font-bold transition-all shadow-md ${
              copied
                ? 'bg-[#00F5A0] text-[#07090E]'
                : 'bg-[#2F6BFF] hover:bg-[#487DFF] text-white shadow-[#2F6BFF]/30 active:scale-95'
            }`}
          >
            {copied ? <Check className="w-3.5 h-3.5" /> : <Copy className="w-3.5 h-3.5" />}
            <span>{copied ? 'Copied to Clipboard!' : 'Copy Snippet'}</span>
          </button>
        </div>

        {/* Tab Content Display */}
        <div className="p-4 sm:p-6 bg-[#07090E] space-y-4">
          <p className="text-xs font-mono text-[#94A3B8] leading-relaxed">
            {activeTab.description}
          </p>

          <div className="relative group">
            <div className="p-4 rounded-xl bg-[#0A0E17] border border-[#1F283D] font-mono text-xs text-[#00F5D4] overflow-x-auto leading-relaxed shadow-inner">
              <pre className="whitespace-pre-wrap select-all">{activeTab.command}</pre>
            </div>
          </div>

          {/* Verification Checklist Card */}
          <div className="p-4 rounded-xl bg-[#0D111A] border border-[#161D2B] space-y-3">
            <div className="flex items-center gap-2 text-xs font-mono font-semibold text-[#F3F5F9]">
              <ShieldCheck className="w-4 h-4 text-[#00F5A0]" />
              <span>Automatic Verification Steps (`fastuse doctor`):</span>
            </div>
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-2 text-[11px] font-mono text-[#94A3B8]">
              <div className="flex items-center gap-1.5 bg-[#090C13] p-2 rounded border border-[#1F283D]">
                <CheckCircle2 className="w-3.5 h-3.5 text-[#00F5A0] shrink-0" />
                <span>UIA / AX permissions</span>
              </div>
              <div className="flex items-center gap-1.5 bg-[#090C13] p-2 rounded border border-[#1F283D]">
                <CheckCircle2 className="w-3.5 h-3.5 text-[#00F5A0] shrink-0" />
                <span>Chrome Host registry</span>
              </div>
              <div className="flex items-center gap-1.5 bg-[#090C13] p-2 rounded border border-[#1F283D]">
                <CheckCircle2 className="w-3.5 h-3.5 text-[#00F5A0] shrink-0" />
                <span>Antigravity auto-route</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};

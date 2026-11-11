import React, { useState } from 'react';
import { Terminal, Play, Copy, Check, Trash2, Sparkles, ChevronRight } from 'lucide-react';
import { sound } from '../utils/audio';

interface CommandPreset {
  id: string;
  label: string;
  cmd: string;
  runtimeMs: string;
  response: object;
}

const PRESETS: CommandPreset[] = [
  {
    id: 'doctor',
    label: 'fastuse doctor',
    cmd: 'fastuse doctor',
    runtimeMs: '1.2ms',
    response: {
      status: 'healthy',
      platform: 'Windows 11 (build 26100)',
      architecture: 'x86_64',
      accessibility: {
        ui_automation: 'connected',
        core_messaging: 'active',
        focus_stealing_protection: 'enforced',
      },
      browser_bridge: {
        status: 'ready',
        host_registered: true,
        targets: ['Brave Browser (pid 14208)', 'Google Chrome (pid 9940)'],
      },
      visual_overlay: {
        renderer: 'Direct2D',
        independent_cursor: 'online',
        frame_idle_decay: '30s',
      },
    },
  },
  {
    id: 'list_windows',
    label: 'list_windows',
    cmd: 'fastuse call list_windows',
    runtimeMs: '0.9ms',
    response: {
      windows: [
        { hwnd: 132402, title: 'Visual Studio Code - fastuse', process: 'Code.exe', bounds: { x: 0, y: 0, w: 1440, h: 1080 } },
        { hwnd: 67120, title: 'Brave - GitHub Pull Requests', process: 'brave.exe', bounds: { x: 1440, y: 0, w: 1120, h: 1080 } },
        { hwnd: 49812, title: 'Windows Terminal (PowerShell)', process: 'WindowsTerminal.exe', bounds: { x: 400, y: 300, w: 800, h: 600 } },
      ],
      total_found: 3,
      discovery_latency: '0.92ms',
    },
  },
  {
    id: 'act_click',
    label: 'act (Background Click)',
    cmd: 'fastuse call act \'{"steps":[{"click":{"element":"#approve-pr"}}]}\'',
    runtimeMs: '1.8ms',
    response: {
      status: 'success',
      action_type: 'background_invoke',
      target_element: '#approve-pr',
      native_interface: 'IUIAutomationInvokePattern',
      user_cursor_moved: false,
      user_focus_stolen: false,
      overlay: {
        agent_cursor_glided_to: { x: 842, y: 310 },
        visual_glow_duration: '300ms',
      },
      latency: '1.84ms',
    },
  },
  {
    id: 'browser_act',
    label: 'browser_act (Real Browser)',
    cmd: 'fastuse call browser_act \'{"action":"click","selector":"button.btn-primary"}\'',
    runtimeMs: '2.4ms',
    response: {
      status: 'ok',
      bridge: 'Chrome Native Messaging IPC',
      active_tab: 'https://github.com/anassagd432/fastuse/pull/42',
      cookies_preserved: true,
      dom_event: 'MouseEvent (trusted)',
      shadow_overlay_shown: true,
      execution_ms: '2.41ms',
    },
  },
];

export const TerminalPlayground: React.FC = () => {
  const [selectedPreset, setSelectedPreset] = useState<CommandPreset>(PRESETS[0]);
  const [history, setHistory] = useState<Array<{ cmd: string; ms: string; res: object }>>([
    { cmd: PRESETS[0].cmd, ms: PRESETS[0].runtimeMs, res: PRESETS[0].response },
  ]);
  const [copied, setCopied] = useState(false);

  const handleRun = (preset: CommandPreset) => {
    setSelectedPreset(preset);
    sound.playClick();
    setTimeout(() => {
      sound.playSuccess();
      setHistory((prev) => [{ cmd: preset.cmd, ms: preset.runtimeMs, res: preset.response }, ...prev]);
    }, 180);
  };

  const handleCopy = () => {
    navigator.clipboard.writeText(JSON.stringify(history[0]?.res || {}, null, 2));
    setCopied(true);
    sound.playSuccess();
    setTimeout(() => setCopied(false), 2000);
  };

  const handleClear = () => {
    setHistory([]);
    sound.playClick();
  };

  return (
    <section id="playground" className="py-24 px-4 sm:px-6 lg:px-8 max-w-7xl mx-auto scroll-mt-20">
      <div className="text-center max-w-3xl mx-auto mb-14 space-y-4">
        <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-[#0D111A] border border-[#1F283D] text-xs font-mono text-[#00F5A0]">
          <Terminal className="w-3.5 h-3.5" />
          <span>INTERACTIVE MCP PLAYGROUND</span>
        </div>
        <h2 className="font-display font-extrabold text-3xl sm:text-5xl tracking-tight text-[#F3F5F9]">
          Test the Protocol in <span className="text-[#00F5D4]">Your Browser</span>.
        </h2>
        <p className="text-sm sm:text-base text-[#94A3B8] font-sans">
          Select sample commands below to simulate real-time JSON-RPC dispatch through the fastuse engine.
        </p>
      </div>

      <div className="max-w-5xl mx-auto cyber-panel rounded-2xl overflow-hidden border border-[#1F283D] shadow-2xl">
        {/* Terminal Header */}
        <div className="bg-[#090C13] border-b border-[#1F283D] px-4 py-3 flex flex-wrap items-center justify-between gap-3">
          <div className="flex items-center gap-2">
            <div className="w-3 h-3 rounded-full bg-[#FF3366]" />
            <div className="w-3 h-3 rounded-full bg-[#FFB800]" />
            <div className="w-3 h-3 rounded-full bg-[#00F5A0]" />
            <span className="ml-2 font-mono text-xs text-[#94A3B8]">
              fastuse-mcp-inspector // session: active
            </span>
          </div>

          <div className="flex items-center gap-2">
            <button
              onClick={handleCopy}
              className="flex items-center gap-1.5 px-2.5 py-1 rounded bg-[#131926] hover:bg-[#1C263A] border border-[#1F283D] text-xs font-mono text-[#94A3B8] hover:text-[#00F5D4] transition-all"
            >
              {copied ? <Check className="w-3.5 h-3.5 text-[#00F5A0]" /> : <Copy className="w-3.5 h-3.5" />}
              <span>{copied ? 'Copied' : 'Copy JSON'}</span>
            </button>
            <button
              onClick={handleClear}
              className="p-1.5 rounded bg-[#131926] hover:bg-[#1C263A] border border-[#1F283D] text-[#94A3B8] hover:text-[#FF3366] transition-all"
              title="Clear terminal"
            >
              <Trash2 className="w-3.5 h-3.5" />
            </button>
          </div>
        </div>

        {/* Preset Selector Toolbar */}
        <div className="bg-[#0B0F17] px-4 py-2.5 border-b border-[#161D2B] flex flex-wrap items-center gap-2">
          <span className="text-[11px] font-mono text-[#94A3B8] mr-2 flex items-center gap-1">
            <Sparkles className="w-3 h-3 text-[#2F6BFF]" />
            Command Presets:
          </span>
          {PRESETS.map((p) => (
            <button
              key={p.id}
              onClick={() => handleRun(p)}
              className={`px-3 py-1 rounded-lg text-xs font-mono transition-all flex items-center gap-1.5 ${
                selectedPreset.id === p.id
                  ? 'bg-[#2F6BFF] text-white shadow-md shadow-[#2F6BFF]/30 font-semibold'
                  : 'bg-[#131926] text-[#94A3B8] hover:text-[#F3F5F9] hover:bg-[#1A2336] border border-[#1F283D]'
              }`}
            >
              <Play className="w-2.5 h-2.5 fill-current opacity-70" />
              <span>{p.label}</span>
              <span className="text-[9px] opacity-75 font-mono">({p.runtimeMs})</span>
            </button>
          ))}
        </div>

        {/* Terminal Screen Body */}
        <div className="p-4 sm:p-6 bg-[#07090E] min-h-[360px] max-h-[500px] overflow-y-auto font-mono text-xs space-y-5 scanlines">
          {history.length === 0 ? (
            <div className="text-center py-12 text-[#94A3B8]">
              Terminal cleared. Click a preset above to run an MCP action.
            </div>
          ) : (
            history.map((item, idx) => (
              <div key={idx} className="space-y-2 border-b border-[#161D2B] pb-4 last:border-b-0">
                {/* Prompt Line */}
                <div className="flex items-center gap-2 text-[#00F5D4]">
                  <ChevronRight className="w-4 h-4 text-[#2F6BFF]" />
                  <span className="text-[#F3F5F9] font-bold">{item.cmd}</span>
                  <span className="ml-auto text-[10px] text-[#00F5A0] bg-[#00F5A0]/10 border border-[#00F5A0]/20 px-2 py-0.5 rounded">
                    ⚡ {item.ms}
                  </span>
                </div>

                {/* Formatted JSON Output */}
                <div className="p-3.5 rounded-xl bg-[#0A0E17] border border-[#1F283D] text-[#86E1FC] overflow-x-auto leading-relaxed shadow-inner">
                  <pre>{JSON.stringify(item.res, null, 2)}</pre>
                </div>
              </div>
            ))
          )}
        </div>

        {/* Terminal Status Bar */}
        <div className="bg-[#090C13] border-t border-[#1F283D] px-4 py-2 flex items-center justify-between text-[11px] font-mono text-[#94A3B8]">
          <span className="flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-full bg-[#00F5A0]" />
            Daemon: Running (0 warnings)
          </span>
          <span className="text-[#548AFF]">MCP Schema: 2024-11-05</span>
        </div>
      </div>
    </section>
  );
};

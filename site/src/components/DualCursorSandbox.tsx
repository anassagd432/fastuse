import React, { useState, useEffect, useRef } from 'react';
import { MousePointer, Terminal, Play, Pause, CheckCircle2, ShieldCheck, Sparkles } from 'lucide-react';
import { sound } from '../utils/audio';

export const DualCursorSandbox: React.FC = () => {
  const [isRunning, setIsRunning] = useState(true);
  const [userText, setUserText] = useState(
    '// You can click and type here right now!\n// fastuse never steals your cursor or keyboard.\nfunction testProductivity() {\n  return "My focus is 100% uninterrupted";\n}'
  );
  const [agentPos, setAgentPos] = useState({ x: 340, y: 140 });
  const [agentAction, setAgentAction] = useState('Idle');
  const [agentClicking, setAgentClicking] = useState(false);
  const [browserTabs, setBrowserTabs] = useState([
    { id: 1, title: 'GitHub PR #42', active: true, content: 'PR #42: Add background accessibility click pipeline' },
    { id: 2, title: 'Brave Dashboard', active: false, content: 'Logged in session: 14 active tasks' },
  ]);
  const [logs, setLogs] = useState<string[]>([
    '[fastuse] Daemon initialized. Zero focus steal active.',
    '[fastuse] Hooked Windows UI Automation & Chrome Native Messaging.',
  ]);

  const containerRef = useRef<HTMLDivElement | null>(null);

  // Simulated Agent autonomous loop
  useEffect(() => {
    if (!isRunning) return;

    const waypoints = [
      { x: 320, y: 90, action: 'Hovering PR Approve Button', click: true, log: '[fastuse] act::click on element #approve-pr (1.2ms)' },
      { x: 420, y: 150, action: 'Reading DOM UI tree', click: false, log: '[fastuse] browser_state: read 48 elements (18ms)' },
      { x: 280, y: 220, action: 'Typing comment in background', click: true, log: '[fastuse] act::set_value "LGTM, verified in background" (2.1ms)' },
      { x: 380, y: 280, action: 'Confirming merge', click: true, log: '[fastuse] act::click on #confirm-merge (0.9ms)' },
    ];

    let index = 0;
    const interval = setInterval(() => {
      const target = waypoints[index];
      setAgentPos({ x: target.x, y: target.y });
      setAgentAction(target.action);

      if (target.click) {
        setTimeout(() => {
          setAgentClicking(true);
          sound.playClick();
          setTimeout(() => setAgentClicking(false), 250);
        }, 500);
      }

      setLogs((prev) => [target.log, ...prev.slice(0, 4)]);
      index = (index + 1) % waypoints.length;
    }, 2400);

    return () => clearInterval(interval);
  }, [isRunning]);

  return (
    <div className="w-full max-w-5xl mx-auto rounded-2xl cyber-panel cyber-panel-glow border border-[#1F283D] overflow-hidden shadow-2xl relative">
      {/* Outer Glow Edge Accent */}
      <div className="absolute inset-x-0 top-0 h-[2px] bg-gradient-to-r from-transparent via-[#00F5D4] to-transparent animate-pulse-glow" />

      {/* Simulated OS Window Top Bar */}
      <div className="bg-[#090C13] border-b border-[#1F283D] px-4 py-2.5 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 rounded-full bg-[#FF3366]/80" />
          <div className="w-3 h-3 rounded-full bg-[#FFB800]/80" />
          <div className="w-3 h-3 rounded-full bg-[#00F5A0]/80" />
          <span className="ml-2 text-xs font-mono text-[#94A3B8] flex items-center gap-1.5">
            <ShieldCheck className="w-3.5 h-3.5 text-[#00F5D4]" />
            Live Dual-Cursor Sandbox · Try typing while fastuse acts
          </span>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={() => {
              setIsRunning(!isRunning);
              sound.playClick();
            }}
            className="flex items-center gap-1.5 px-2.5 py-1 rounded bg-[#131926] hover:bg-[#1C263A] border border-[#1F283D] text-[11px] font-mono text-[#F3F5F9] transition-all"
          >
            {isRunning ? (
              <>
                <Pause className="w-3 h-3 text-[#FFB800]" /> Pause Agent
              </>
            ) : (
              <>
                <Play className="w-3 h-3 text-[#00F5A0]" /> Resume Agent
              </>
            )}
          </button>
          <div className="px-2 py-0.5 rounded bg-[#00F5A0]/10 border border-[#00F5A0]/30 text-[#00F5A0] text-[10px] font-mono font-bold animate-pulse">
            0.9ms DISPATCH
          </div>
        </div>
      </div>

      {/* Sandbox Workspace Body */}
      <div
        ref={containerRef}
        className="p-4 sm:p-6 grid grid-cols-1 md:grid-cols-2 gap-4 relative min-h-[380px] bg-[#07090E]/90 scanlines"
      >
        {/* LEFT PANE: User Editor (Physical Focus) */}
        <div className="flex flex-col rounded-xl bg-[#0B0F17] border border-[#1F283D] p-3 shadow-inner">
          <div className="flex items-center justify-between pb-2 mb-2 border-b border-[#161D2B]">
            <span className="text-xs font-mono font-semibold text-[#00F5D4] flex items-center gap-1.5">
              <span className="w-2 h-2 rounded-full bg-[#00F5D4] animate-ping" />
              YOUR WINDOW (Active User Focus)
            </span>
            <span className="text-[10px] font-mono text-[#94A3B8]">Click inside & type</span>
          </div>
          <textarea
            value={userText}
            onChange={(e) => setUserText(e.target.value)}
            className="w-full flex-1 bg-transparent font-mono text-xs text-[#E2E8F0] resize-none outline-none leading-relaxed selection:bg-[#2F6BFF]/40"
            rows={10}
            placeholder="Type anything here..."
          />
          <div className="pt-2 mt-auto border-t border-[#161D2B] flex items-center justify-between text-[11px] font-mono text-[#94A3B8]">
            <span className="text-[#00F5A0] flex items-center gap-1">
              <CheckCircle2 className="w-3 h-3" /> Focus never stolen
            </span>
            <span>OS Cursor: Uninterrupted</span>
          </div>
        </div>

        {/* RIGHT PANE: Background Browser (fastuse Agent Operating) */}
        <div className="flex flex-col rounded-xl bg-[#0D121D] border border-[#2F6BFF]/30 p-3 relative shadow-inner overflow-hidden">
          {/* Agent Screen Edge Aura Glow */}
          <div className="absolute inset-0 border border-[#2F6BFF] rounded-xl pointer-events-none opacity-40 shadow-[inset_0_0_20px_rgba(47,107,255,0.3)] animate-pulse-glow" />

          {/* Browser Mock Tab Bar */}
          <div className="flex items-center gap-2 pb-2 mb-2 border-b border-[#1F283D]">
            {browserTabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => {
                  setBrowserTabs((tabs) =>
                    tabs.map((t) => ({ ...t, active: t.id === tab.id }))
                  );
                  sound.playClick();
                }}
                className={`text-[10px] font-mono px-2 py-1 rounded transition-all ${
                  tab.active
                    ? 'bg-[#1E293B] text-[#00F5D4] border border-[#2F6BFF]/40'
                    : 'text-[#94A3B8] hover:text-[#F3F5F9]'
                }`}
              >
                {tab.title}
              </button>
            ))}
            <span className="ml-auto text-[10px] font-mono text-[#548AFF]">Chrome Native Bridge</span>
          </div>

          {/* Mock Browser Content */}
          <div className="p-3 space-y-3 font-mono text-xs">
            <div className="p-2.5 rounded bg-[#131A29] border border-[#1F283D]">
              <div className="text-[11px] text-[#94A3B8] mb-1">Target Application:</div>
              <div className="text-[#F3F5F9] font-semibold text-xs">
                Pull Request: Background accessibility click pipeline
              </div>
            </div>

            <div className="grid grid-cols-2 gap-2">
              <button
                id="approve-btn"
                className="py-1.5 px-3 rounded bg-[#00F5A0]/15 border border-[#00F5A0]/40 text-[#00F5A0] text-xs font-semibold text-center hover:bg-[#00F5A0]/25 transition-all"
              >
                Approve PR
              </button>
              <button
                id="merge-btn"
                className="py-1.5 px-3 rounded bg-[#2F6BFF]/20 border border-[#2F6BFF]/50 text-[#548AFF] text-xs font-semibold text-center hover:bg-[#2F6BFF]/30 transition-all"
              >
                Fast-Forward Merge
              </button>
            </div>

            <div className="p-2 rounded bg-[#0B0F17] border border-[#161D2B] text-[11px] text-[#94A3B8]">
              Automated comment: <span className="text-[#00F5D4]">"LGTM, verified in background"</span>
            </div>
          </div>

          {/* fastuse Agent Status Bar inside window */}
          <div className="mt-auto pt-2 border-t border-[#1F283D] flex items-center justify-between text-[11px] font-mono">
            <span className="text-[#548AFF] flex items-center gap-1.5">
              <Sparkles className="w-3.5 h-3.5 text-[#00F5D4]" />
              Agent Action: <span className="text-[#F3F5F9]">{agentAction}</span>
            </span>
            <span className="text-[10px] text-[#00F5A0]">0.9ms</span>
          </div>

          {/* AUTONOMOUS AGENT CURSOR (GLOWING BLUE/CYAN) */}
          <div
            className="absolute pointer-events-none transition-all duration-700 ease-out z-30 flex flex-col items-start"
            style={{
              left: `${agentPos.x}px`,
              top: `${agentPos.y}px`,
            }}
          >
            <div className="relative">
              {/* Outer halo */}
              <div
                className={`absolute -inset-3 rounded-full bg-[#00F5D4]/40 blur-md transition-transform duration-200 ${
                  agentClicking ? 'scale-150 bg-[#2F6BFF]/80' : 'scale-100'
                }`}
              />
              {/* Cursor SVG */}
              <MousePointer className="w-5 h-5 text-[#00F5D4] drop-shadow-[0_0_8px_#00F5D4] fill-[#00F5D4]/20 relative z-10 -rotate-12" />
            </div>
            {/* Tooltip badge */}
            <div className="mt-1 px-1.5 py-0.5 rounded bg-[#07090E]/90 border border-[#00F5D4]/50 text-[9px] font-mono text-[#00F5D4] shadow-lg whitespace-nowrap">
              fastuse Agent
            </div>
          </div>
        </div>
      </div>

      {/* Terminal Live Telemetry Log Footer */}
      <div className="bg-[#05070B] border-t border-[#1F283D] px-4 py-2 flex flex-col sm:flex-row sm:items-center justify-between gap-2 text-[11px] font-mono text-[#94A3B8]">
        <div className="flex items-center gap-2 overflow-hidden text-ellipsis whitespace-nowrap">
          <Terminal className="w-3.5 h-3.5 text-[#2F6BFF] shrink-0" />
          <span className="text-[#00F5D4] shrink-0">Live Telemetry:</span>
          <span className="text-[#E2E8F0] truncate">{logs[0]}</span>
        </div>
        <div className="flex items-center gap-3 shrink-0 text-[10px]">
          <span className="flex items-center gap-1 text-[#00F5A0]">
            <span className="w-1.5 h-1.5 rounded-full bg-[#00F5A0]" />
            Background Dispatch: OK
          </span>
          <span className="text-[#94A3B8]">User Collision: 0%</span>
        </div>
      </div>
    </div>
  );
};

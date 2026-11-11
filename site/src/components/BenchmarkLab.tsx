import React, { useState } from 'react';
import { Play, RotateCcw, Cpu, Zap, ArrowRight, Gauge } from 'lucide-react';
import { sound } from '../utils/audio';

interface BenchmarkItem {
  id: string;
  name: string;
  standardMs: number;
  fastuseMs: number;
  speedup: string;
  description: string;
}

const BENCHMARKS: BenchmarkItem[] = [
  {
    id: 'list_windows',
    name: 'List System Windows',
    standardMs: 437,
    fastuseMs: 0.9,
    speedup: '485x',
    description: 'Direct Win32 EnumWindows + UIA window handle cache vs recursive OS probing.',
  },
  {
    id: 'read_tree',
    name: 'Read UI Accessibility Tree',
    standardMs: 160,
    fastuseMs: 29,
    speedup: '5.5x',
    description: 'Selective subtree depth pruning with millisecond UIA cache invalidation.',
  },
  {
    id: 'type_chars',
    name: 'Type 20 Characters',
    standardMs: 153,
    fastuseMs: 3.6,
    speedup: '42x',
    description: 'WM_CHAR message queue injection without hijacking the physical keyboard.',
  },
  {
    id: 'click_element',
    name: 'Click by Element Token',
    standardMs: 16,
    fastuseMs: 1.8,
    speedup: '9x',
    description: 'Direct UI Automation InvokeProvider call without moving host mouse.',
  },
  {
    id: 'set_value',
    name: 'Set Text Field Value (20 chars)',
    standardMs: 16,
    fastuseMs: 2.2,
    speedup: '7x',
    description: 'Native ValuePattern interface write, instantaneous zero-keystroke input.',
  },
  {
    id: 'batch_flow',
    name: 'Batch Action (Type + Click + Observe)',
    standardMs: 650,
    fastuseMs: 66,
    speedup: '10x',
    description: 'Consolidates 4-6 agent network round trips into an all-in-one act call.',
  },
];

export const BenchmarkLab: React.FC = () => {
  const [activeItem, setActiveItem] = useState<BenchmarkItem>(BENCHMARKS[0]);
  const [racing, setRacing] = useState(false);
  const [fastuseProgress, setFastuseProgress] = useState(100);
  const [standardProgress, setStandardProgress] = useState(100);
  const [liveFastuseMs, setLiveFastuseMs] = useState<number>(BENCHMARKS[0].fastuseMs);
  const [liveStandardMs, setLiveStandardMs] = useState<number>(BENCHMARKS[0].standardMs);

  const runRace = (item = activeItem) => {
    setRacing(true);
    setFastuseProgress(0);
    setStandardProgress(0);
    setLiveFastuseMs(0);
    setLiveStandardMs(0);
    sound.playClick();

    // Fastuse finishes nearly instantly
    const fastDuration = Math.max(item.fastuseMs * 15, 120);
    const fastStart = Date.now();
    const fastTimer = setInterval(() => {
      const elapsed = Date.now() - fastStart;
      const pct = Math.min((elapsed / fastDuration) * 100, 100);
      setFastuseProgress(pct);
      setLiveFastuseMs(Number(((pct / 100) * item.fastuseMs).toFixed(1)));

      if (pct >= 100) {
        clearInterval(fastTimer);
        sound.playSuccess();
      }
    }, 16);

    // Standard driver takes much longer
    const standardDuration = Math.min(item.standardMs * 3.5, 2200);
    const standardStart = Date.now();
    const standardTimer = setInterval(() => {
      const elapsed = Date.now() - standardStart;
      const pct = Math.min((elapsed / standardDuration) * 100, 100);
      setStandardProgress(pct);
      setLiveStandardMs(Math.round((pct / 100) * item.standardMs));

      if (pct >= 100) {
        clearInterval(standardTimer);
        setRacing(false);
      }
    }, 24);
  };

  const handleSelect = (item: BenchmarkItem) => {
    setActiveItem(item);
    setLiveFastuseMs(item.fastuseMs);
    setLiveStandardMs(item.standardMs);
    setFastuseProgress(100);
    setStandardProgress(100);
    sound.playHover();
  };

  return (
    <section id="benchmarks" className="py-24 px-4 sm:px-6 lg:px-8 max-w-7xl mx-auto scroll-mt-20">
      <div className="text-center max-w-3xl mx-auto mb-14 space-y-4">
        <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-[#0D111A] border border-[#1F283D] text-xs font-mono text-[#00F5D4]">
          <Gauge className="w-3.5 h-3.5" />
          <span>MEASURED TELEMETRY</span>
        </div>
        <h2 className="font-display font-extrabold text-3xl sm:text-5xl tracking-tight text-[#F3F5F9]">
          Engineered for <span className="text-[#00F5D4]">Sub-Millisecond</span> Latency.
        </h2>
        <p className="text-sm sm:text-base text-[#94A3B8] font-sans">
          Tested on Windows 11 (Ryzen 9 / RTX 3070), medians over persistent MCP sessions compared to standard driver architecture.
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 items-start">
        {/* Left Column: Operation Selector */}
        <div className="lg:col-span-5 space-y-2.5">
          {BENCHMARKS.map((item) => (
            <button
              key={item.id}
              onClick={() => handleSelect(item)}
              className={`w-full text-left p-3.5 rounded-xl border transition-all flex items-center justify-between ${
                activeItem.id === item.id
                  ? 'bg-[#111724] border-[#00F5D4]/60 shadow-lg shadow-[#00F5D4]/10'
                  : 'bg-[#0D111A]/70 border-[#1F283D] hover:border-[#2F6BFF]/50 hover:bg-[#131926]'
              }`}
            >
              <div className="space-y-1 pr-3">
                <div className="font-display font-semibold text-sm text-[#F3F5F9]">
                  {item.name}
                </div>
                <div className="text-[11px] font-mono text-[#94A3B8] line-clamp-1">
                  {item.description}
                </div>
              </div>
              <div className="text-right shrink-0">
                <div className="font-mono font-bold text-sm text-[#00F5D4]">
                  {item.fastuseMs} ms
                </div>
                <div className="text-[10px] font-mono font-bold text-[#00F5A0] bg-[#00F5A0]/10 px-1.5 py-0.5 rounded border border-[#00F5A0]/30 inline-block mt-0.5">
                  {item.speedup}
                </div>
              </div>
            </button>
          ))}
        </div>

        {/* Right Column: Interactive Race Simulator Card */}
        <div className="lg:col-span-7 cyber-panel rounded-2xl p-6 sm:p-8 space-y-6">
          <div className="flex items-center justify-between border-b border-[#1F283D] pb-4">
            <div>
              <span className="text-xs font-mono text-[#94A3B8]">Active Benchmark:</span>
              <h3 className="font-display font-bold text-xl text-[#F3F5F9]">
                {activeItem.name}
              </h3>
            </div>
            <button
              onClick={() => runRace(activeItem)}
              disabled={racing}
              className={`flex items-center gap-2 px-4 py-2 rounded-xl font-display font-semibold text-xs tracking-wider uppercase transition-all shadow-lg ${
                racing
                  ? 'bg-[#1F283D] text-[#94A3B8] cursor-not-allowed'
                  : 'bg-[#00F5D4] hover:bg-[#00D1B5] text-[#07090E] shadow-[#00F5D4]/30 active:scale-95'
              }`}
            >
              {racing ? (
                <>
                  <RotateCcw className="w-3.5 h-3.5 animate-spin" /> Racing...
                </>
              ) : (
                <>
                  <Play className="w-3.5 h-3.5 fill-current" /> Run Speed Test
                </>
              )}
            </button>
          </div>

          <p className="text-xs font-mono text-[#94A3B8] leading-relaxed bg-[#090C13] p-3 rounded-lg border border-[#161D2B]">
            {activeItem.description}
          </p>

          {/* Visual Race Progress Comparison */}
          <div className="space-y-6 pt-2">
            {/* fastuse Track */}
            <div className="space-y-2">
              <div className="flex items-center justify-between text-xs font-mono">
                <span className="text-[#00F5D4] font-bold flex items-center gap-1.5">
                  <Zap className="w-3.5 h-3.5 text-[#00F5D4]" />
                  fastuse Engine
                </span>
                <span className="text-sm font-bold text-[#00F5D4] font-mono">
                  {liveFastuseMs} ms
                </span>
              </div>
              <div className="h-4 bg-[#090C13] rounded-full overflow-hidden p-0.5 border border-[#1F283D]">
                <div
                  className="h-full bg-gradient-to-r from-[#2F6BFF] via-[#00F5D4] to-[#548AFF] rounded-full transition-all duration-150 shadow-[0_0_12px_#00F5D4]"
                  style={{ width: `${fastuseProgress}%` }}
                />
              </div>
            </div>

            {/* Standard Driver Track */}
            <div className="space-y-2">
              <div className="flex items-center justify-between text-xs font-mono">
                <span className="text-[#94A3B8] flex items-center gap-1.5">
                  <Cpu className="w-3.5 h-3.5 text-[#94A3B8]" />
                  Standard Driver (cua 0.37)
                </span>
                <span className="text-sm font-bold text-[#FF3366] font-mono">
                  {liveStandardMs} ms
                </span>
              </div>
              <div className="h-4 bg-[#090C13] rounded-full overflow-hidden p-0.5 border border-[#1F283D]">
                <div
                  className="h-full bg-gradient-to-r from-[#FF3366] to-[#CC1E4A] rounded-full transition-all duration-150"
                  style={{ width: `${standardProgress}%` }}
                />
              </div>
            </div>
          </div>

          {/* Speedup Badge Box */}
          <div className="p-4 rounded-xl bg-gradient-to-br from-[#111724] to-[#0D111A] border border-[#2F6BFF]/40 flex flex-col sm:flex-row items-center justify-between gap-3">
            <div className="flex items-center gap-3">
              <div className="w-12 h-12 rounded-xl bg-[#00F5A0]/15 border border-[#00F5A0]/30 flex items-center justify-center font-mono font-extrabold text-lg text-[#00F5A0]">
                {activeItem.speedup}
              </div>
              <div>
                <div className="font-display font-bold text-sm text-[#F3F5F9]">
                  {activeItem.speedup} Faster Execution
                </div>
                <div className="text-[11px] font-mono text-[#94A3B8]">
                  Zero OS focus interruption throughout action lifecycle.
                </div>
              </div>
            </div>
            <a
              href="https://github.com/anassagd432/fastuse/tree/main/bench"
              target="_blank"
              rel="noopener noreferrer"
              className="text-xs font-mono text-[#00F5D4] hover:underline flex items-center gap-1 shrink-0"
            >
              <span>View bench/ suite</span>
              <ArrowRight className="w-3 h-3" />
            </a>
          </div>
        </div>
      </div>
    </section>
  );
};

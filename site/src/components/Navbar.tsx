import React, { useState, useEffect } from 'react';
import { Zap, Volume2, VolumeX, Github, Terminal, ArrowUpRight } from 'lucide-react';
import { sound } from '../utils/audio';

export const Navbar: React.FC = () => {
  const [muted, setMuted] = useState(sound.isMuted());
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const unsub = sound.subscribe((m) => setMuted(m));
    const onScroll = () => setScrolled(window.scrollY > 20);
    window.addEventListener('scroll', onScroll);
    return () => {
      unsub();
      window.removeEventListener('scroll', onScroll);
    };
  }, []);

  const handleAudioToggle = () => {
    sound.toggleMute();
  };

  return (
    <header
      className={`fixed top-0 left-0 right-0 z-50 transition-all duration-300 ${
        scrolled
          ? 'bg-[#07090E]/85 backdrop-blur-md border-b border-[#1F283D] py-3.5 shadow-2xl shadow-[#000000]/80'
          : 'bg-transparent py-5'
      }`}
    >
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 flex items-center justify-between">
        {/* Brand */}
        <a
          href="#"
          className="flex items-center gap-3 group"
          onMouseEnter={() => sound.playHover()}
        >
          <div className="w-9 h-9 rounded-lg bg-gradient-to-br from-[#2F6BFF] to-[#00F5D4] p-[1px] shadow-lg shadow-[#2F6BFF]/30 group-hover:shadow-[#00F5D4]/50 transition-all duration-300">
            <div className="w-full h-full bg-[#0D111A] rounded-[7px] flex items-center justify-center">
              <Zap className="w-5 h-5 text-[#00F5D4] group-hover:scale-110 transition-transform" />
            </div>
          </div>
          <div className="flex flex-col">
            <span className="font-display font-extrabold text-lg tracking-tight text-[#F3F5F9] flex items-center gap-1.5">
              fastuse
              <span className="text-[10px] font-mono font-bold uppercase tracking-wider px-1.5 py-0.5 rounded bg-[#2F6BFF]/15 text-[#00F5D4] border border-[#2F6BFF]/30">
                v1.0
              </span>
            </span>
            <span className="text-[11px] font-mono text-[#94A3B8] -mt-1 hidden sm:inline">
              zero-focus-steal
            </span>
          </div>
        </a>

        {/* Navigation links */}
        <nav className="hidden md:flex items-center gap-1 bg-[#0D111A]/80 border border-[#1F283D] rounded-full px-3 py-1.5 shadow-inner">
          {[
            { label: 'Sandbox', href: '#sandbox' },
            { label: 'Benchmarks', href: '#benchmarks' },
            { label: 'Architecture', href: '#architecture' },
            { label: 'Playground', href: '#playground' },
            { label: 'Docs', href: '#docs' },
          ].map((item) => (
            <a
              key={item.href}
              href={item.href}
              className="text-xs font-mono font-medium text-[#94A3B8] hover:text-[#00F5D4] hover:bg-[#161E30] px-3 py-1.5 rounded-full transition-all"
              onMouseEnter={() => sound.playHover()}
            >
              {item.label}
            </a>
          ))}
        </nav>

        {/* Right controls */}
        <div className="flex items-center gap-3">
          {/* Audio toggle */}
          <button
            onClick={handleAudioToggle}
            className="p-2 rounded-lg bg-[#0D111A] border border-[#1F283D] text-[#94A3B8] hover:text-[#00F5D4] hover:border-[#2F6BFF] transition-all"
            title={muted ? 'Unmute cybernetic SFX' : 'Mute SFX'}
          >
            {muted ? <VolumeX className="w-4 h-4" /> : <Volume2 className="w-4 h-4 text-[#00F5D4]" />}
          </button>

          {/* GitHub button */}
          <a
            href="https://github.com/anassagd432/fastuse"
            target="_blank"
            rel="noopener noreferrer"
            className="hidden sm:flex items-center gap-2 px-3 py-1.5 text-xs font-mono font-medium bg-[#0D111A] border border-[#1F283D] rounded-lg text-[#F3F5F9] hover:border-[#2F6BFF] hover:bg-[#161E30] transition-all"
            onMouseEnter={() => sound.playHover()}
          >
            <Github className="w-4 h-4 text-[#94A3B8]" />
            <span>Star</span>
            <span className="text-[10px] bg-[#1F283D] px-1.5 py-0.5 rounded text-[#00F5D4]">
              MIT
            </span>
          </a>

          {/* Install CTA */}
          <a
            href="#install"
            className="flex items-center gap-1.5 px-4 py-2 rounded-lg bg-gradient-to-r from-[#2F6BFF] to-[#1E4ECC] hover:from-[#487DFF] hover:to-[#2F6BFF] text-white font-display font-semibold text-xs tracking-wide shadow-lg shadow-[#2F6BFF]/25 hover:shadow-[#2F6BFF]/50 transition-all active:scale-95"
            onClick={() => sound.playClick()}
          >
            <Terminal className="w-3.5 h-3.5" />
            <span>Install</span>
            <ArrowUpRight className="w-3 h-3 opacity-70" />
          </a>
        </div>
      </div>
    </header>
  );
};

import React from 'react';
import { CyberCanvas } from './components/CyberCanvas';
import { Navbar } from './components/Navbar';
import { Hero } from './components/Hero';
import { BenchmarkLab } from './components/BenchmarkLab';
import { ArchitectureGraph } from './components/ArchitectureGraph';
import { TerminalPlayground } from './components/TerminalPlayground';
import { InstallHub } from './components/InstallHub';
import { DocsViewer } from './components/DocsViewer';
import { Footer } from './components/Footer';

export const App: React.FC = () => {
  return (
    <div className="relative min-h-screen bg-[#07090E] text-[#F3F5F9] selection:bg-[#2F6BFF]/30 selection:text-[#00F5D4] overflow-x-hidden">
      {/* 60fps Interactive Cyber Grid & Particle Canvas */}
      <CyberCanvas />

      {/* Navigation Header */}
      <Navbar />

      {/* Main Content Sections */}
      <main className="relative z-10 space-y-6 sm:space-y-12">
        <Hero />
        <BenchmarkLab />
        <ArchitectureGraph />
        <TerminalPlayground />
        <InstallHub />
        <DocsViewer />
      </main>

      {/* Global Footer */}
      <Footer />
    </div>
  );
};

export default App;

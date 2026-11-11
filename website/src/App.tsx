import React, { useState, useEffect } from 'react';
import { Shield, Volume2, VolumeX, Github, ExternalLink, FileText, Lock, Terminal, Cpu } from 'lucide-react';
import { CyberCanvas } from './components/CyberCanvas';
import { HeroSection } from './components/HeroSection';
import { AboutSection } from './components/AboutSection';
import { ExplodedArchitecture } from './components/ExplodedArchitecture';
import { AntiCaptureDemo } from './components/AntiCaptureDemo';
import { MakerCheckerDemo } from './components/MakerCheckerDemo';
import { BenchmarkTelemetry } from './components/BenchmarkTelemetry';
import { DownloadsSection } from './components/DownloadsSection';
import { soundEffects } from './components/AudioEffects';

export const App: React.FC = () => {
  const [isMuted, setIsMuted] = useState<boolean>(false);
  const [lang, setLang] = useState<'en' | 'si'>(() => {
    try {
      const saved = localStorage.getItem('aegis_lang');
      if (saved === 'si' || saved === 'en') return saved;
    } catch {}
    return 'en';
  });

  useEffect(() => {
    try {
      localStorage.setItem('aegis_lang', lang);
    } catch {}
  }, [lang]);

  const toggleSound = () => {
    const muted = soundEffects.toggleMute();
    setIsMuted(muted);
    if (!muted) soundEffects.click();
  };

  const scrollToSection = (id: string) => {
    soundEffects.click();
    const element = document.getElementById(id);
    if (element) {
      element.scrollIntoView({ behavior: 'smooth' });
    }
  };

  const isSi = lang === 'si';

  return (
    <div className="min-h-screen bg-[#050608] text-slate-100 relative selection:bg-amber-500/30 selection:text-amber-200">
      {/* 60FPS Interactive Cyber Particle & Frequency Wave Canvas */}
      <CyberCanvas />

      {/* Persistent Scanlines Overlay */}
      <div className="fixed inset-0 scanlines z-30 pointer-events-none opacity-40" />

      {/* Top Navigation Bar */}
      <header className="fixed top-0 left-0 right-0 z-40 bg-[#050608]/85 backdrop-blur-md border-b border-white/10">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-16 flex items-center justify-between">
          {/* Logo & Brand */}
          <div
            onClick={() => window.scrollTo({ top: 0, behavior: 'smooth' })}
            onMouseEnter={() => soundEffects.hover()}
            className="flex items-center gap-3 cursor-pointer group"
          >
            <div className="p-2 rounded-lg bg-amber-500/10 border border-amber-500/30 group-hover:border-amber-400 group-hover:bg-amber-500/20 transition-all">
              <Shield className="w-5 h-5 text-amber-400 group-hover:rotate-12 transition-transform" />
            </div>
            <div>
              <div className="font-display font-extrabold text-base tracking-wider text-white flex items-center gap-1.5">
                <span>AEGIS</span>
                <span className="text-amber-400 font-mono text-xs font-normal">[CORE]</span>
              </div>
              <div className="font-mono text-[9px] text-slate-400 tracking-widest uppercase">
                {isSi ? 'ශුන්‍ය විශ්වාස ඩෙස්ක්ටොප්' : 'ZERO-TRUST DESKTOP'}
              </div>
            </div>
          </div>

          {/* Desktop Navigation Links */}
          <nav className="hidden md:flex items-center gap-8 font-mono text-xs">
            <button
              onClick={() => scrollToSection('about')}
              onMouseEnter={() => soundEffects.hover()}
              className="text-slate-400 hover:text-amber-300 transition-colors tracking-wide"
            >
              // {isSi ? 'හැඳින්වීම' : 'ABOUT'}
            </button>
            <button
              onClick={() => scrollToSection('architecture')}
              onMouseEnter={() => soundEffects.hover()}
              className="text-slate-400 hover:text-amber-300 transition-colors tracking-wide"
            >
              // {isSi ? 'ගෘහ නිර්මාණය' : 'ARCHITECTURE'}
            </button>
            <button
              onClick={() => scrollToSection('anti-capture')}
              onMouseEnter={() => soundEffects.hover()}
              className="text-slate-400 hover:text-amber-300 transition-colors tracking-wide"
            >
              // {isSi ? 'තිර ආරක්ෂාව' : 'ANTI-CAPTURE'}
            </button>
            <button
              onClick={() => scrollToSection('maker-checker')}
              onMouseEnter={() => soundEffects.hover()}
              className="text-slate-400 hover:text-amber-300 transition-colors tracking-wide"
            >
              // {isSi ? 'FOUR-EYES' : 'FOUR-EYES'}
            </button>
            <button
              onClick={() => scrollToSection('benchmarks')}
              onMouseEnter={() => soundEffects.hover()}
              className="text-slate-400 hover:text-amber-300 transition-colors tracking-wide"
            >
              // {isSi ? 'වේග මිනුම්' : 'BENCHMARKS'}
            </button>
            <button
              onClick={() => scrollToSection('downloads')}
              onMouseEnter={() => soundEffects.hover()}
              className="text-slate-400 hover:text-amber-300 transition-colors tracking-wide"
            >
              // {isSi ? 'බාගැනීම්' : 'DOWNLOADS'}
            </button>
          </nav>

          {/* Right Utility Buttons */}
          <div className="flex items-center gap-3">
            {/* Language Switcher (EN / SI) */}
            <div className="flex items-center bg-white/5 border border-white/10 rounded-lg p-0.5 font-mono text-xs">
              <button
                onClick={() => {
                  soundEffects.click();
                  setLang('en');
                }}
                className={`px-2.5 py-1 rounded transition-colors ${
                  lang === 'en' ? 'bg-amber-500 text-black font-bold' : 'text-slate-400 hover:text-white'
                }`}
              >
                EN
              </button>
              <button
                onClick={() => {
                  soundEffects.click();
                  setLang('si');
                }}
                className={`px-2.5 py-1 rounded transition-colors ${
                  lang === 'si' ? 'bg-amber-500 text-black font-bold' : 'text-slate-400 hover:text-white'
                }`}
              >
                සිංහල
              </button>
            </div>

            {/* Audio Toggle Button */}
            <button
              onClick={toggleSound}
              title={isMuted ? 'Unmute Cyber Synthesizer FX' : 'Mute Sound FX'}
              className="p-2 rounded-lg bg-white/5 hover:bg-white/10 text-slate-400 hover:text-amber-300 border border-white/10 transition-colors"
            >
              {isMuted ? <VolumeX className="w-4 h-4 text-slate-500" /> : <Volume2 className="w-4 h-4 text-amber-400 animate-pulse" />}
            </button>

            {/* GitHub Repository Link */}
            <a
              href="https://github.com/charitha-only/aegis-desktop"
              target="_blank"
              rel="noopener noreferrer"
              onMouseEnter={() => soundEffects.hover()}
              className="hidden sm:flex items-center gap-2 px-3.5 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-slate-300 hover:text-white border border-white/10 font-mono text-xs transition-colors"
            >
              <Github className="w-4 h-4 text-slate-300" />
              <span>GITHUB</span>
              <span className="text-[10px] text-amber-400 bg-amber-500/10 px-1.5 py-0.5 rounded border border-amber-500/20">
                v1.2.0
              </span>
            </a>

            {/* Download Button */}
            <button
              onClick={() => scrollToSection('downloads')}
              onMouseEnter={() => soundEffects.hover()}
              className="px-4 py-1.5 rounded-lg bg-gradient-to-r from-amber-500 to-amber-600 hover:from-amber-400 hover:to-amber-500 text-black font-mono text-xs font-bold tracking-wider shadow-md shadow-amber-500/20 transition-all"
            >
              {isSi ? 'බාගන්න' : 'DOWNLOAD'}
            </button>
          </div>
        </div>
      </header>

      {/* Main Page Content */}
      <main className="relative z-10">
        <HeroSection
          lang={lang}
          onExploreArchitecture={() => scrollToSection('architecture')}
          onExploreDemo={() => scrollToSection('anti-capture')}
          onExploreDownloads={() => scrollToSection('downloads')}
        />

        <AboutSection />

        <ExplodedArchitecture lang={lang} />

        <AntiCaptureDemo />

        <MakerCheckerDemo />

        <BenchmarkTelemetry />

        <DownloadsSection />
      </main>

      {/* Footer */}
      <footer className="relative z-10 bg-[#030406] border-t border-white/10 py-16 px-4 sm:px-6 lg:px-8 font-mono text-xs">
        <div className="max-w-7xl mx-auto flex flex-col md:flex-row items-center justify-between gap-6">
          <div className="flex items-center gap-3">
            <div className="p-1.5 rounded bg-amber-500/10 border border-amber-500/30">
              <Shield className="w-4 h-4 text-amber-400" />
            </div>
            <div>
              <div className="text-white font-bold tracking-wide">AEGIS DESKTOP APPLICATION FRAMEWORK</div>
              <div className="text-[10px] text-slate-500">
                Certified for Enterprise Banking, Defense & Mission-Critical Operations
              </div>
            </div>
          </div>

          <div className="flex flex-wrap items-center gap-6 text-slate-400">
            <a
              href="https://github.com/charitha-only/aegis-desktop/blob/main/docs/ENTERPRISE_SECURITY_WHITEPAPER.md"
              target="_blank"
              rel="noopener noreferrer"
              className="hover:text-amber-300 transition-colors flex items-center gap-1"
            >
              <FileText className="w-3.5 h-3.5 text-amber-400" />
              <span>Security Whitepaper</span>
            </a>
            <a
              href="https://github.com/charitha-only/aegis-desktop/blob/main/docs/SBOM.json"
              target="_blank"
              rel="noopener noreferrer"
              className="hover:text-amber-300 transition-colors flex items-center gap-1"
            >
              <Cpu className="w-3.5 h-3.5 text-cyan-400" />
              <span>SPDX 2.3 SBOM</span>
            </a>
            <a
              href="https://github.com/charitha-only/aegis-desktop"
              target="_blank"
              rel="noopener noreferrer"
              className="hover:text-amber-300 transition-colors flex items-center gap-1"
            >
              <Github className="w-3.5 h-3.5 text-slate-300" />
              <span>Repository</span>
            </a>
          </div>

          <div className="text-slate-500 text-[11px] text-center md:text-right">
            <div>© 2026 Aegis Project. MIT Open Source License.</div>
            <div className="text-slate-600 mt-0.5">Dual-Engine Authenticode Signed // Zero Vulnerabilities</div>
          </div>
        </div>
      </footer>
    </div>
  );
};

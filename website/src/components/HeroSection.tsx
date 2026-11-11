import React from 'react';
import { Shield, Terminal, ArrowRight, Download, Cpu, Lock, CheckCircle2 } from 'lucide-react';
import { soundEffects } from './AudioEffects';

interface HeroSectionProps {
  onExploreArchitecture: () => void;
  onExploreDemo: () => void;
  onExploreDownloads: () => void;
}

export const HeroSection: React.FC<HeroSectionProps> = ({
  onExploreArchitecture,
  onExploreDemo,
  onExploreDownloads,
}) => {
  return (
    <section className="relative min-h-screen flex items-center justify-center pt-24 pb-20 px-4 sm:px-6 lg:px-8 overflow-hidden">
      {/* Background Statue Graphic with Cyber Overlay */}
      <div className="absolute inset-0 z-0 pointer-events-none select-none flex items-center justify-end overflow-hidden">
        <div className="relative w-full lg:w-3/5 h-full opacity-40 lg:opacity-75 transition-opacity duration-1000">
          <img
            src="./assets/aegis-hero-statue.jpg"
            alt="Aegis Classical Cybernetic Bust"
            className="w-full h-full object-cover object-center lg:object-right filter contrast-125 brightness-95"
          />
          {/* Gradients to fade smoothly into deep black */}
          <div className="absolute inset-0 bg-gradient-to-r from-[#050608] via-[#050608]/70 to-transparent" />
          <div className="absolute inset-0 bg-gradient-to-t from-[#050608] via-transparent to-[#050608]/80" />
          <div className="absolute inset-0 bg-gradient-to-b from-[#050608] via-transparent to-[#050608]" />

          {/* Interactive Scanning HUD Ring & Reticle Overlay */}
          <div className="absolute top-1/3 right-1/4 w-72 h-72 border border-amber-500/20 rounded-full animate-pulse pointer-events-none hidden sm:block">
            <div className="absolute inset-4 border border-dashed border-amber-500/30 rounded-full animate-spin" style={{ animationDuration: '30s' }} />
            <div className="absolute inset-12 border border-amber-500/10 rounded-full" />
            <div className="absolute top-1/2 left-0 right-0 h-px bg-amber-500/30" />
            <div className="absolute left-1/2 top-0 bottom-0 w-px bg-amber-500/30" />
            <div className="absolute -top-6 right-0 font-mono text-[10px] text-amber-400/70 tracking-widest bg-[#050608]/90 px-2 py-0.5 border border-amber-500/20">
              TARGET // CLASSICAL_SECURITY_MODEL
            </div>
          </div>
        </div>
      </div>

      {/* Main Content Container */}
      <div className="relative z-10 max-w-7xl mx-auto w-full grid grid-cols-1 lg:grid-cols-12 gap-12 items-center">
        <div className="lg:col-span-8 space-y-8">
          {/* Top Status Capsule */}
          <div className="inline-flex items-center gap-3 px-3.5 py-1.5 rounded-full border border-amber-500/30 bg-amber-500/5 backdrop-blur-md">
            <span className="relative flex h-2 w-2">
              <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75" />
              <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500" />
            </span>
            <span className="font-mono text-xs tracking-wider uppercase text-amber-300 font-semibold">
              AEGIS CORE v1.1.0 // BANKING & DEFENSE TIER
            </span>
            <span className="text-white/20">|</span>
            <span className="font-mono text-[11px] text-slate-400 hidden sm:inline">
              OWASP ASVS 5.0 L2 CERTIFIED
            </span>
          </div>

          {/* Hero Main Headline (Neo-Brutalist Technical Typography) */}
          <div className="space-y-3">
            <h1 className="text-4xl sm:text-6xl xl:text-7xl font-extrabold tracking-tight font-display text-white leading-none">
              VULNERABILITIES HIDE.
              <br />
              <span className="text-transparent bg-clip-text bg-gradient-to-r from-amber-400 via-amber-200 to-amber-500 glow-amber">
                [WE DON'T.]
              </span>
            </h1>
            <p className="max-w-2xl text-base sm:text-lg text-slate-300 font-light leading-relaxed">
              The zero-trust cross-platform desktop application framework. Built in memory-safe <strong className="text-amber-400 font-medium">Rust</strong> and sandboxed <strong className="text-cyan-400 font-medium">React</strong> with hardware-enforced anti-screen capture, Four-Eyes cryptographic consensus, and Wasm instruction fuel isolation.
            </p>
          </div>

          {/* Key Feature Badge Grid */}
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 pt-2">
            {[
              { title: 'Win32 DWM Mask', desc: 'OBS Anti-Capture', icon: Shield },
              { title: 'Four-Eyes Consensus', desc: 'Maker-Checker SOD', icon: Lock },
              { title: 'Wasm Fuel Trap', desc: 'Zero-Trust Sandbox', icon: Cpu },
              { title: 'Authenticode Signed', desc: 'DigiCert Timestamp', icon: CheckCircle2 },
            ].map((item, idx) => (
              <div
                key={idx}
                onMouseEnter={() => soundEffects.hover()}
                className="p-3 bg-[#0d1017]/80 border border-white/10 hover:border-amber-500/40 rounded-lg transition-all backdrop-blur-sm group crosshair"
              >
                <div className="flex items-center gap-2 mb-1">
                  <item.icon className="w-4 h-4 text-amber-400 group-hover:scale-110 transition-transform" />
                  <span className="font-mono text-xs font-semibold text-slate-200">{item.title}</span>
                </div>
                <p className="font-mono text-[11px] text-slate-400">{item.desc}</p>
              </div>
            ))}
          </div>

          {/* CTA Action Buttons */}
          <div className="flex flex-wrap items-center gap-4 pt-4">
            <button
              onClick={() => {
                soundEffects.click();
                onExploreDownloads();
              }}
              className="px-6 py-3.5 bg-gradient-to-r from-amber-500 to-amber-600 hover:from-amber-400 hover:to-amber-500 text-black font-semibold rounded-lg font-mono text-sm tracking-wide shadow-lg shadow-amber-500/20 flex items-center gap-2 transition-all transform hover:-translate-y-0.5 active:translate-y-0"
            >
              <Download className="w-4 h-4" />
              <span>DOWNLOAD v1.1.0</span>
            </button>

            <button
              onClick={() => {
                soundEffects.click();
                onExploreDemo();
              }}
              className="px-6 py-3.5 bg-[#121622]/90 hover:bg-[#1a2030] text-amber-300 border border-amber-500/30 hover:border-amber-400 font-mono text-sm tracking-wide rounded-lg flex items-center gap-2 transition-all"
            >
              <Terminal className="w-4 h-4 text-amber-400" />
              <span>LIVE INTERACTIVE SIMULATOR</span>
            </button>

            <button
              onClick={() => {
                soundEffects.click();
                onExploreArchitecture();
              }}
              className="px-5 py-3.5 text-slate-400 hover:text-white font-mono text-sm tracking-wide flex items-center gap-1.5 transition-colors group"
            >
              <span>3D Exploded Architecture</span>
              <ArrowRight className="w-4 h-4 group-hover:translate-x-1 transition-transform text-amber-400" />
            </button>
          </div>

          {/* Quick Terminal Command Snippet */}
          <div className="pt-2">
            <div className="inline-flex items-center gap-3 px-4 py-2 rounded-lg bg-[#0a0d14] border border-white/10 font-mono text-xs text-slate-300">
              <span className="text-amber-500 select-none">$</span>
              <span className="select-all">git clone https://github.com/charitha-only/aegis-desktop.git</span>
              <span className="text-[10px] text-slate-500 ml-2 hidden sm:inline">(100% Rust & React)</span>
            </div>
          </div>
        </div>

        {/* Right Column: Live Telemetry Widget HUD */}
        <div className="lg:col-span-4 hidden lg:block space-y-4">
          <div className="p-5 rounded-xl bg-[#0b0e17]/90 border border-amber-500/20 backdrop-blur-md shadow-2xl relative crosshair">
            <div className="flex items-center justify-between pb-3 border-b border-white/10">
              <div className="flex items-center gap-2">
                <div className="w-2 h-2 rounded-full bg-emerald-400 animate-ping" />
                <span className="font-mono text-xs font-semibold text-slate-300 tracking-wider">HARDWARE TELEMETRY</span>
              </div>
              <span className="font-mono text-[10px] text-amber-400 bg-amber-500/10 px-2 py-0.5 rounded border border-amber-500/20">
                HOST: PHYSICAL X64
              </span>
            </div>

            <div className="space-y-4 pt-4">
              <div>
                <div className="flex justify-between text-xs font-mono mb-1">
                  <span className="text-slate-400">Cold Startup Latency</span>
                  <span className="text-emerald-400 font-bold">25.8 ms <span className="text-slate-500 font-normal">(&lt; 100ms)</span></span>
                </div>
                <div className="w-full bg-white/5 h-1.5 rounded-full overflow-hidden">
                  <div className="bg-emerald-500 h-full rounded-full" style={{ width: '25.8%' }} />
                </div>
              </div>

              <div>
                <div className="flex justify-between text-xs font-mono mb-1">
                  <span className="text-slate-400">IPC Roundtrip Latency</span>
                  <span className="text-emerald-400 font-bold">1.6 ms <span className="text-slate-500 font-normal">(p50)</span></span>
                </div>
                <div className="w-full bg-white/5 h-1.5 rounded-full overflow-hidden">
                  <div className="bg-emerald-400 h-full rounded-full" style={{ width: '16%' }} />
                </div>
              </div>

              <div>
                <div className="flex justify-between text-xs font-mono mb-1">
                  <span className="text-slate-400">Resident Memory (RSS)</span>
                  <span className="text-amber-400 font-bold">8.80 MiB <span className="text-slate-500 font-normal">(&lt; 50MB)</span></span>
                </div>
                <div className="w-full bg-white/5 h-1.5 rounded-full overflow-hidden">
                  <div className="bg-amber-500 h-full rounded-full" style={{ width: '17.6%' }} />
                </div>
              </div>

              <div>
                <div className="flex justify-between text-xs font-mono mb-1">
                  <span className="text-slate-400">OWASP ASVS 5.0 Tests</span>
                  <span className="text-cyan-400 font-bold">138 / 138 Passing</span>
                </div>
                <div className="w-full bg-white/5 h-1.5 rounded-full overflow-hidden">
                  <div className="bg-cyan-500 h-full rounded-full" style={{ width: '100%' }} />
                </div>
              </div>
            </div>

            <div className="mt-5 pt-4 border-t border-white/10 flex items-center justify-between font-mono text-[11px] text-slate-400">
              <span>ZERO ALLOCATION RING</span>
              <span className="text-emerald-400">STABLE O(1)</span>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};

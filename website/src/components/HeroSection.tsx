import React, { useState, useEffect } from 'react';
import { Shield, Terminal, ArrowRight, Download, Cpu, Lock, CheckCircle2, AlertTriangle, Activity, Wifi } from 'lucide-react';
import { soundEffects } from './AudioEffects';

interface HeroSectionProps {
  lang?: 'en' | 'si';
  onExploreArchitecture: () => void;
  onExploreDemo: () => void;
  onExploreDownloads: () => void;
}

export const HeroSection: React.FC<HeroSectionProps> = ({
  lang = 'en',
  onExploreArchitecture,
  onExploreDemo,
  onExploreDownloads,
}) => {
  const [tickerOffset, setTickerOffset] = useState(0);

  useEffect(() => {
    const timer = setInterval(() => {
      setTickerOffset((prev) => (prev + 1) % 100);
    }, 50);
    return () => clearInterval(timer);
  }, []);

  const isSi = lang === 'si';

  return (
    <section className="relative min-h-screen flex flex-col justify-center pt-24 pb-16 px-4 sm:px-6 lg:px-8 overflow-hidden">
      {/* Background Statue Graphic with Cybernetic Sliced Rings & Waveform Mask */}
      <div className="absolute inset-0 z-0 pointer-events-none select-none flex items-center justify-end overflow-hidden">
        <div className="relative w-full lg:w-3/5 h-full opacity-40 lg:opacity-85 transition-opacity duration-1000">
          <img
            src="./assets/aegis-hero-statue.jpg"
            alt="Aegis Classical Cybernetic Bust"
            className="w-full h-full object-cover object-center lg:object-right filter contrast-125 brightness-95"
          />
          {/* Gradients to fade smoothly into deep black */}
          <div className="absolute inset-0 bg-gradient-to-r from-[#050608] via-[#050608]/75 to-transparent" />
          <div className="absolute inset-0 bg-gradient-to-t from-[#050608] via-transparent to-[#050608]/80" />
          <div className="absolute inset-0 bg-gradient-to-b from-[#050608] via-transparent to-[#050608]" />

          {/* Sliced Concentric Rings Overlay (Directly matching Image 1 & 2) */}
          <div className="absolute top-[28%] right-[22%] w-80 h-80 pointer-events-none hidden lg:block">
            {/* Outer Ring */}
            <div className="absolute inset-0 border border-amber-500/25 rounded-full animate-spin" style={{ animationDuration: '40s' }}>
              <span className="absolute -top-3 left-1/2 -translate-x-1/2 font-mono text-[9px] text-amber-400 bg-black/90 px-2 py-0.5 border border-amber-500/30">
                [LEDGER RECOVERY]
              </span>
            </div>

            {/* Mid Ring */}
            <div className="absolute inset-8 border border-dashed border-cyan-500/30 rounded-full animate-spin" style={{ animationDuration: '25s', animationDirection: 'reverse' }}>
              <span className="absolute -bottom-3 left-1/2 -translate-x-1/2 font-mono text-[9px] text-cyan-400 bg-black/90 px-2 py-0.5 border border-cyan-500/30">
                [CROSS-MARGIN AGGREGATOR]
              </span>
            </div>

            {/* Inner Concentric Target Disks */}
            <div className="absolute inset-16 border border-amber-400/20 rounded-full">
              <span className="absolute top-1/2 -right-4 -translate-y-1/2 font-mono text-[9px] text-amber-300 bg-black/90 px-2 py-0.5 border border-amber-500/30">
                [WALLET INTELLIGENCE]
              </span>
            </div>
            <div className="absolute inset-24 border border-dashed border-emerald-400/30 rounded-full animate-pulse" />

            {/* Crosshairs */}
            <div className="absolute top-1/2 left-0 right-0 h-px bg-amber-500/20" />
            <div className="absolute left-1/2 top-0 bottom-0 w-px bg-amber-500/20" />
          </div>

          {/* Holographic Waveform Glitch Mask over Eyes (Directly matching Image 2 & 3) */}
          <div className="absolute top-[32%] right-[26%] w-56 h-20 pointer-events-none hidden lg:block overflow-hidden opacity-85">
            <svg className="w-full h-full" viewBox="0 0 200 60" preserveAspectRatio="none">
              <path
                d="M0 30 Q 25 10, 50 30 T 100 30 T 150 30 T 200 30"
                fill="none"
                stroke="#f59e0b"
                strokeWidth="1.5"
                className="opacity-70 animate-pulse"
              />
              <path
                d="M0 30 Q 35 5, 70 30 T 140 30 T 200 30"
                fill="none"
                stroke="#38bdf8"
                strokeWidth="1"
                className="opacity-60"
              />
              <path
                d="M0 30 Q 15 45, 60 30 T 120 30 T 200 30"
                fill="none"
                stroke="#10b981"
                strokeWidth="1"
                className="opacity-50"
              />
            </svg>
            <div className="absolute top-0 right-0 font-mono text-[8px] text-amber-400 bg-black/80 px-1 border border-amber-500/20">
              SCANLINE // HUD.0x8F
            </div>
          </div>
        </div>
      </div>

      {/* Main Content Container */}
      <div className="relative z-10 max-w-7xl mx-auto w-full grid grid-cols-1 lg:grid-cols-12 gap-12 items-center flex-1">
        <div className="lg:col-span-8 space-y-7">
          {/* Top Status Capsule */}
          <div className="inline-flex items-center gap-3 px-3.5 py-1.5 rounded-full border border-amber-500/30 bg-amber-500/5 backdrop-blur-md">
            <span className="relative flex h-2 w-2">
              <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75" />
              <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500" />
            </span>
            <span className="font-mono text-xs tracking-wider uppercase text-amber-300 font-semibold">
              AEGIS CORE v1.2.0 // {isSi ? 'ආරක්ෂක හා බැංකු මට්ටම' : 'BANKING & FINANCIAL TIER'}
            </span>
            <span className="text-white/20">|</span>
            <span className="font-mono text-[11px] text-slate-400 hidden sm:inline">
              OWASP ASVS 5.0 L2 CERTIFIED
            </span>
          </div>

          {/* Hero Main Headline (Neo-Brutalist Technical Typography matching Image 1 & 2) */}
          <div className="space-y-3">
            <h1 className="text-4xl sm:text-6xl xl:text-7xl font-extrabold tracking-tight font-display text-white leading-none">
              {isSi ? 'වෙළඳපොළ රැවටිය හැක.' : 'THE MARKET LIES.'}
              <br />
              <span className="text-transparent bg-clip-text bg-gradient-to-r from-amber-400 via-amber-200 to-amber-500 glow-amber">
                {isSi ? '[අප එසේ නොවේ.]' : '[WE DON\'T.]'}
              </span>
            </h1>
            <p className="max-w-2xl text-base sm:text-lg text-slate-300 font-light leading-relaxed">
              {isSi
                ? 'ශුන්‍ය විශ්වාස (Zero-Trust) ඩෙස්ක්ටොප් රාමුව. මතක ආරක්ෂිත Rust සහ වැටකෙයියා කළ React මඟින් දෘඩාංග මට්ටමේ තිර ග්‍රහණ වැළැක්වීම (Win32 DWM), RASP ස්වයං-ආරක්ෂණය, Four-Eyes මූල්‍ය අනුමැතිය සහ Wasm fuel වැලිපිල්ල සපයයි.'
                : 'The zero-trust cross-platform desktop application framework. Built in memory-safe Rust and sandboxed React with hardware anti-screen capture, RASP anti-tampering, Four-Eyes cryptographic consensus, and Wasm instruction fuel isolation.'}
            </p>
          </div>

          {/* Key Feature Badge Grid */}
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 pt-2">
            {[
              { title: isSi ? 'DWM තිර ආවරණය' : 'Win32 DWM Mask', desc: isSi ? 'OBS වැළැක්වීම' : 'OBS Anti-Capture', icon: Shield },
              { title: isSi ? 'Four-Eyes අනුමැතිය' : 'Four-Eyes Consensus', desc: isSi ? 'Maker-Checker SOD' : 'Maker-Checker SOD', icon: Lock },
              { title: isSi ? 'RASP ආරක්ෂාව' : 'Ring-0 RASP Guard', desc: isSi ? 'මතක කොකු වැළැක්වීම' : 'Anti-Tampering', icon: Cpu },
              { title: isSi ? 'FIDO2 / YubiKey' : 'FIDO2 Hardware Key', desc: isSi ? 'ස්පර්ශක අත්සන්' : 'Touch Authenticator', icon: CheckCircle2 },
            ].map((item, idx) => (
              <div
                key={idx}
                onMouseEnter={() => soundEffects.hover()}
                className="p-3 bg-[#0d1017]/85 border border-white/10 hover:border-amber-500/40 rounded-lg transition-all backdrop-blur-sm group crosshair"
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
          <div className="flex flex-wrap items-center gap-4 pt-3">
            <button
              onClick={() => {
                soundEffects.click();
                onExploreDownloads();
              }}
              className="px-6 py-3.5 bg-gradient-to-r from-amber-500 to-amber-600 hover:from-amber-400 hover:to-amber-500 text-black font-semibold rounded-lg font-mono text-sm tracking-wide shadow-lg shadow-amber-500/20 flex items-center gap-2 transition-all transform hover:-translate-y-0.5 active:translate-y-0"
            >
              <Download className="w-4 h-4" />
              <span>{isSi ? 'බාගත කරන්න (v1.2.0)' : 'DOWNLOAD v1.2.0'}</span>
            </button>

            <button
              onClick={() => {
                soundEffects.click();
                onExploreDemo();
              }}
              className="px-6 py-3.5 bg-[#121622]/90 hover:bg-[#1a2030] text-amber-300 border border-amber-500/30 hover:border-amber-400 font-mono text-sm tracking-wide rounded-lg flex items-center gap-2 transition-all"
            >
              <Terminal className="w-4 h-4 text-amber-400" />
              <span>{isSi ? 'සජීවී සිමියුලේටරය' : 'LIVE SIMULATOR'}</span>
            </button>

            <button
              onClick={() => {
                soundEffects.click();
                onExploreArchitecture();
              }}
              className="px-5 py-3.5 text-slate-400 hover:text-white font-mono text-sm tracking-wide flex items-center gap-1.5 transition-colors group"
            >
              <span>{isSi ? '3D Blueprint සැලැස්ම' : '3D Exploded Blueprint'}</span>
              <ArrowRight className="w-4 h-4 group-hover:translate-x-1 transition-transform text-amber-400" />
            </button>
          </div>

          {/* Quick Terminal Command Snippet */}
          <div className="pt-1">
            <div className="inline-flex items-center gap-3 px-4 py-2 rounded-lg bg-[#0a0d14] border border-white/10 font-mono text-xs text-slate-300">
              <span className="text-amber-500 select-none">$</span>
              <span className="select-all">git clone https://github.com/charitha-only/aegis-desktop.git</span>
              <span className="text-[10px] text-slate-500 ml-2 hidden sm:inline">(100% Rust & React)</span>
            </div>
          </div>
        </div>

        {/* Right Column: Floating HUD Network Topology Card (Directly matching Image 1) */}
        <div className="lg:col-span-4 hidden lg:block space-y-4">
          {/* Main Floating Alert Topology Card */}
          <div className="p-5 rounded-xl bg-[#0b0e17]/95 border border-amber-500/30 backdrop-blur-md shadow-2xl relative crosshair group">
            {/* Header: [SYS ALERT 01] */}
            <div className="flex items-center justify-between pb-3 border-b border-white/10">
              <div className="flex items-center gap-2">
                <AlertTriangle className="w-4 h-4 text-amber-400 animate-pulse" />
                <span className="font-mono text-xs font-bold text-amber-400 tracking-wider">
                  [SYS ALERT 01]
                </span>
              </div>
              <span className="font-mono text-[9px] text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded border border-emerald-500/20">
                ACTIVE DEFENSE
              </span>
            </div>

            <div className="text-[11px] font-mono text-slate-300 font-semibold pt-2 pb-1">
              COMBINED ATTACK VECTOR BLOCKED
            </div>

            {/* Simulated Dynamic Node Topology Graph */}
            <div className="relative h-40 bg-black/60 rounded-lg border border-white/5 my-3 overflow-hidden flex items-center justify-center">
              {/* Background Network Grid Lines */}
              <svg className="absolute inset-0 w-full h-full" xmlns="http://www.w3.org/2000/svg">
                {/* Node Interconnection Lines */}
                <line x1="30" y1="40" x2="90" y2="30" stroke="#f59e0b" strokeWidth="1" strokeDasharray="3,3" opacity="0.6" />
                <line x1="90" y1="30" x2="150" y2="50" stroke="#f59e0b" strokeWidth="1.5" opacity="0.8" />
                <line x1="150" y1="50" x2="210" y2="35" stroke="#f59e0b" strokeWidth="1" opacity="0.5" />
                <line x1="90" y1="30" x2="110" y2="110" stroke="#38bdf8" strokeWidth="1" opacity="0.6" />
                <line x1="150" y1="50" x2="170" y2="120" stroke="#f59e0b" strokeWidth="1.2" opacity="0.7" />
                <line x1="110" y1="110" x2="170" y2="120" stroke="#10b981" strokeWidth="1.5" opacity="0.8" />
                <line x1="170" y1="120" x2="230" y2="100" stroke="#38bdf8" strokeWidth="1" strokeDasharray="2,2" opacity="0.5" />

                {/* Nodes (Orange & White Vertices from Image 1) */}
                <circle cx="30" cy="40" r="3.5" fill="#ffffff" />
                <circle cx="90" cy="30" r="5" fill="#f59e0b" className="animate-pulse" />
                <circle cx="150" cy="50" r="6" fill="#f59e0b" />
                <circle cx="210" cy="35" r="4" fill="#ffffff" />
                <circle cx="110" cy="110" r="5.5" fill="#f59e0b" />
                <circle cx="170" cy="120" r="6.5" fill="#f59e0b" className="animate-pulse" />
                <circle cx="230" cy="100" r="4" fill="#ffffff" />
              </svg>

              {/* Status Badge in Graph */}
              <div className="absolute bottom-2 left-2 font-mono text-[9px] text-slate-400 bg-black/80 px-2 py-0.5 rounded border border-white/10">
                TOPOLOGY: 7 NODES SECURED
              </div>
              <div className="absolute top-2 right-2 font-mono text-[9px] text-amber-400 bg-amber-500/10 px-2 py-0.5 rounded border border-amber-500/20">
                ZERO MITM
              </div>
            </div>

            {/* Telemetry Micro-Data */}
            <div className="space-y-1.5 font-mono text-[10px] text-slate-400 border-t border-white/10 pt-3">
              <div className="flex justify-between">
                <span>MEM HOOK DETECTION:</span>
                <span className="text-emerald-400 font-bold">FAIL-CLOSED [LOCKED]</span>
              </div>
              <div className="flex justify-between">
                <span>CERT SPKI PINNING:</span>
                <span className="text-cyan-400 font-bold">SHA256 MATCH</span>
              </div>
              <div className="flex justify-between">
                <span>FIDO2 HARDWARE KEY:</span>
                <span className="text-amber-400 font-bold">PHYSICAL TOUCH CONFIRMED</span>
              </div>
              <div className="flex justify-between">
                <span>COORD TELEMETRY:</span>
                <span className="text-slate-300">0x8F94 // ALL SYSTEMS NOMINAL</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Real-Time Crypto & Defense Ticker Bar (Directly matching Image 2) */}
      <div className="relative z-10 w-full mt-10 border-y border-white/10 bg-[#080b12]/90 backdrop-blur-md py-2.5 overflow-hidden font-mono text-xs">
        <div
          className="flex items-center gap-10 whitespace-nowrap transition-transform duration-100 ease-linear"
          style={{ transform: `translateX(-${tickerOffset}px)` }}
        >
          <div className="flex items-center gap-2">
            <span className="text-amber-400 font-bold">AEGIS-CORE:</span>
            <span className="text-slate-300">v1.2.0</span>
            <span className="text-emerald-400 text-[11px]">▲ 143/143 PASS</span>
          </div>

          <div className="flex items-center gap-2">
            <span className="text-slate-400">BTC/USD:</span>
            <span className="text-slate-200 font-semibold">$98,240</span>
            <span className="text-emerald-400 text-[11px]">▲ +2.18%</span>
          </div>

          <div className="flex items-center gap-2">
            <span className="text-slate-400">ETH/USD:</span>
            <span className="text-slate-200 font-semibold">$3,412</span>
            <span className="text-emerald-400 text-[11px]">▲ +1.73%</span>
          </div>

          <div className="flex items-center gap-2">
            <span className="text-slate-400">IPC LATENCY:</span>
            <span className="text-emerald-400 font-bold">1.6 ms (p50)</span>
            <span className="text-slate-500 text-[11px]">(&lt; 5ms)</span>
          </div>

          <div className="flex items-center gap-2">
            <span className="text-slate-400">COLD STARTUP:</span>
            <span className="text-emerald-400 font-bold">26.4 ms</span>
          </div>

          <div className="flex items-center gap-2">
            <span className="text-slate-400">MEMORY RSS:</span>
            <span className="text-amber-400 font-bold">8.80 MiB</span>
            <span className="text-slate-500 text-[11px]">(ZERO-ALLOC RING)</span>
          </div>

          <div className="flex items-center gap-2">
            <span className="text-cyan-400 font-bold">DWM AFFINITY:</span>
            <span className="text-slate-300">WDA_EXCLUDEFROMCAPTURE ACTIVE</span>
          </div>

          {/* Repeat items for continuous ticker stream */}
          <div className="flex items-center gap-2">
            <span className="text-amber-400 font-bold">AEGIS-CORE:</span>
            <span className="text-slate-300">v1.2.0</span>
            <span className="text-emerald-400 text-[11px]">▲ 143/143 PASS</span>
          </div>

          <div className="flex items-center gap-2">
            <span className="text-slate-400">SOL/USD:</span>
            <span className="text-slate-200 font-semibold">$187.4</span>
            <span className="text-emerald-400 text-[11px]">▲ +4.21%</span>
          </div>
        </div>
      </div>
    </section>
  );
};

import React from 'react';
import { Shield, Sparkles, Cpu, Award, Zap, Terminal, Compass, CheckCircle2 } from 'lucide-react';
import { soundEffects } from './AudioEffects';

export const AboutSection: React.FC = () => {
  const pillars = [
    {
      number: '01',
      title: 'THE SHIELD OF ATHENA',
      subtitle: 'Mythological Origin & Philosophy',
      desc: 'In classical mythology, the Aegis (αἰγίς) was the divine, impenetrable shield forged for Athena and Zeus, symbolizing invulnerability, justice, and absolute protection. We brought that ethos to the modern desktop operating system.',
      tag: 'ETHOS // ANCIENT DEFENSE',
      icon: Shield,
    },
    {
      number: '02',
      title: 'RADICAL ZERO-TRUST',
      subtitle: 'Eliminating Ambient Authority',
      desc: 'Traditional frameworks (Electron, CEF) grant ambient Node.js powers to web scripts, enabling devastating remote code execution. In Aegis, no code has default authority. Every operation requires verified capability tokens and deterministic instruction fuel bounds.',
      tag: 'ARCHITECTURE // WASM & RUST',
      icon: Cpu,
    },
    {
      number: '03',
      title: 'HARDWARE CONVERGENCE',
      subtitle: 'OS-Level Compositor Masking',
      desc: 'True security cannot remain in software userland. Aegis interfaces directly with the OS Desktop Window Manager (DWM) and biometric hardware enclaves (TPM 2.0 / Windows Hello) to black out video capture layers and require physical presence verification.',
      tag: 'DEFENSE // WIN32 DWM & FIDO2',
      icon: Zap,
    },
    {
      number: '04',
      title: 'FINANCIAL GOVERNANCE',
      subtitle: 'Basel Four-Eyes & PCI-DSS 10',
      desc: 'Designed for corporate treasury, sovereign institutions, and commercial banking. Built-in Segregation of Duties (SOD) guarantees that no single rogue operator can self-approve critical financial transactions or tamper with the immutable SHA-256 chained audit ledger.',
      tag: 'COMPLIANCE // FOUR-EYES SOD',
      icon: Award,
    },
  ];

  return (
    <section id="about" className="relative py-28 px-4 sm:px-6 lg:px-8 bg-[#050608] border-t border-white/10 overflow-hidden">
      {/* Subtle Background Blueprint Grid */}
      <div className="absolute inset-0 pointer-events-none opacity-20 bg-[linear-gradient(to_right,#80808012_1px,transparent_1px),linear-gradient(to_bottom,#80808012_1px,transparent_1px)] bg-[size:48px_48px]" />

      <div className="relative z-10 max-w-7xl mx-auto">
        {/* Section Header */}
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-12 items-center mb-20">
          <div className="lg:col-span-7 space-y-4">
            <div className="inline-flex items-center gap-2 font-mono text-xs text-amber-400 uppercase tracking-widest">
              <Compass className="w-4 h-4" />
              <span>[ABOUT AEGIS FRAMEWORK] // THE MISSION</span>
            </div>
            <h2 className="text-3xl sm:text-5xl font-extrabold font-display text-white tracking-tight leading-tight">
              REBUILDING THE DESKTOP
              <br />
              <span className="text-amber-400 glow-amber">FROM FIRST PRINCIPLES.</span>
            </h2>
            <p className="text-slate-300 text-sm sm:text-base font-light leading-relaxed">
              Desktop applications hold humanity's most sensitive data: private cryptographic keys, multi-million dollar corporate banking ledgers, classified source code, and confidential legal archives. Yet for a decade, the industry settled for bundling bloated 150MB web browsers with full OS administrative privileges.
            </p>
            <p className="text-slate-400 text-sm sm:text-base font-light leading-relaxed">
              <strong>Aegis Desktop</strong> was conceived as the antidote: an ultra-lean, memory-safe, zero-trust framework built in <strong>Rust 1.92</strong> and sandboxed <strong>React</strong> that consumes under <strong>9MB of RAM</strong>, launches in <strong>25 milliseconds</strong>, and enforces hardware-level protections against screen scraping, malware interception, and multi-tenant tampering.
            </p>
          </div>

          {/* Right Visual Card: Classical Hacker Aesthetic (sample-glitch.jpg) */}
          <div className="lg:col-span-5">
            <div className="rounded-2xl bg-[#0a0d16] border border-amber-500/30 overflow-hidden shadow-2xl relative crosshair group">
              <div className="relative aspect-[4/3] overflow-hidden bg-black">
                <img
                  src="./assets/sample-glitch.jpg"
                  alt="Aegis Classical Codex with Cybernetic Glitch"
                  className="w-full h-full object-cover filter contrast-125 brightness-90 group-hover:scale-105 transition-transform duration-700"
                />
                <div className="absolute inset-0 bg-gradient-to-t from-[#0a0d16] via-transparent to-transparent" />
                <div className="absolute top-3 left-3 px-2.5 py-1 rounded bg-black/80 border border-amber-500/30 font-mono text-[10px] text-amber-300 backdrop-blur-md">
                  AEGIS PROTOCOL // CODEX ARCHIVE
                </div>
                <div className="absolute bottom-3 right-3 px-2.5 py-1 rounded bg-black/80 border border-white/10 font-mono text-[10px] text-slate-400 backdrop-blur-md">
                  OWASP ASVS 5.0 L2
                </div>
              </div>

              <div className="p-5 font-mono text-xs space-y-2 border-t border-white/10">
                <div className="flex items-center justify-between text-slate-300 font-semibold">
                  <span>FOUNDATION CODENAME</span>
                  <span className="text-amber-400">PROJECT AEGIS</span>
                </div>
                <div className="flex items-center justify-between text-slate-400 text-[11px]">
                  <span>LICENSE</span>
                  <span className="text-emerald-400">MIT OPEN SOURCE</span>
                </div>
                <div className="flex items-center justify-between text-slate-400 text-[11px]">
                  <span>AUDITED SPEC</span>
                  <span className="text-slate-200">NIST SP 800-218 & ISO 27001</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* 4 Core Pillars Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          {pillars.map((pillar, idx) => {
            const Icon = pillar.icon;
            return (
              <div
                key={idx}
                onMouseEnter={() => soundEffects.hover()}
                className="p-6 rounded-xl bg-[#090c14] border border-white/10 hover:border-amber-500/40 transition-all flex flex-col justify-between group crosshair relative hover:bg-[#0c101a]"
              >
                <div>
                  <div className="flex items-center justify-between mb-4">
                    <span className="font-mono text-xs font-bold text-amber-500/80 bg-amber-500/10 px-2 py-0.5 rounded border border-amber-500/20">
                      {pillar.number}
                    </span>
                    <div className="p-2 rounded-lg bg-white/5 group-hover:bg-amber-500/10 text-slate-300 group-hover:text-amber-400 transition-colors">
                      <Icon className="w-4 h-4" />
                    </div>
                  </div>

                  <div className="font-mono text-[10px] text-slate-500 uppercase tracking-widest mb-1">
                    {pillar.tag}
                  </div>
                  <h3 className="font-display font-bold text-lg text-white mb-1 group-hover:text-amber-300 transition-colors">
                    {pillar.title}
                  </h3>
                  <div className="font-mono text-xs text-amber-400/90 mb-3 font-medium">
                    {pillar.subtitle}
                  </div>
                  <p className="text-xs text-slate-400 font-light leading-relaxed">
                    {pillar.desc}
                  </p>
                </div>

                <div className="mt-6 pt-4 border-t border-white/5 flex items-center justify-between font-mono text-[11px] text-slate-500">
                  <span>STANDARD</span>
                  <span className="text-emerald-400 flex items-center gap-1">
                    <CheckCircle2 className="w-3 h-3" /> VERIFIED
                  </span>
                </div>
              </div>
            );
          })}
        </div>

        {/* High-Level Framework Specs Bar */}
        <div className="mt-12 p-6 rounded-xl bg-[#080b12] border border-amber-500/20 flex flex-wrap items-center justify-between gap-6 font-mono text-xs crosshair">
          <div className="flex items-center gap-3">
            <Terminal className="w-5 h-5 text-amber-400" />
            <div>
              <div className="text-slate-200 font-bold">100% INDEPENDENT OF BROWSER DAEMONS</div>
              <div className="text-[11px] text-slate-500">No Chromium background bloatware • No unsandboxed Node.js child processes</div>
            </div>
          </div>

          <div className="flex items-center gap-4 text-[11px]">
            <div className="px-3 py-1.5 rounded bg-white/5 border border-white/10 text-slate-300">
              CORE: <span className="text-amber-400 font-bold">RUST</span>
            </div>
            <div className="px-3 py-1.5 rounded bg-white/5 border border-white/10 text-slate-300">
              SANDBOX: <span className="text-cyan-400 font-bold">WASM</span>
            </div>
            <div className="px-3 py-1.5 rounded bg-white/5 border border-white/10 text-slate-300">
              PRESENTATION: <span className="text-purple-400 font-bold">REACT / TS</span>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};

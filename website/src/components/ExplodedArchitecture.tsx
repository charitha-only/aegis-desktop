import React, { useState } from 'react';
import { Layers, ShieldCheck, Cpu, Database, EyeOff, Key, Code2, Sparkles } from 'lucide-react';
import { soundEffects } from './AudioEffects';

interface ArchitectureLayer {
  id: string;
  number: string;
  title: string;
  badge: string;
  category: string;
  description: string;
  specs: string[];
  code: string;
  icon: React.ComponentType<{ className?: string }>;
  accentColor: string;
}

export const ExplodedArchitecture: React.FC = () => {
  const [selectedLayer, setSelectedLayer] = useState<number>(0);
  const [isExploded, setIsExploded] = useState<boolean>(true);

  const layers: ArchitectureLayer[] = [
    {
      id: 'layer-hardware',
      number: '01',
      title: 'HARDWARE SECURITY & OS DWM AFFINITY',
      badge: 'WIN32 DWM / TPM 2.0',
      category: 'PHYSICAL LAYER',
      description: 'Interfaces directly with the Windows Desktop Window Manager and platform hardware cryptographic enclaves to prevent physical or malware-based display interception.',
      specs: [
        'SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)',
        'Direct DWM frame black-out against OBS & TeamViewer',
        'Windows Hello & Touch ID Biometric FIDO2 Challenge-Response',
        'TPM 2.0 Hardware Key Cryptographic Binding',
      ],
      code: `// Hardware-level DWM display masking
let affinity = WDA_EXCLUDEFROMCAPTURE; // 0x00000011
let result = SetWindowDisplayAffinity(hwnd, affinity);
if result == 0 {
    return Err(SecurityError::DisplayAffinityFailed);
}`,
      icon: EyeOff,
      accentColor: 'from-amber-500 to-yellow-600',
    },
    {
      id: 'layer-kernel',
      number: '02',
      title: 'AEGIS ZERO-TRUST RUST MICRO-KERNEL',
      badge: 'RUST 1.92 / ZERO-ALLOCATION',
      category: 'CORE RUNTIME',
      description: 'High-concurrency, memory-safe IPC server with scoped thread execution, O(1) rate limiting, and constant-time cryptographic zeroization.',
      specs: [
        'Multi-threaded scoped IPC workers (std::thread::scope)',
        'Constant-time crypto comparisons (timing-attack immune)',
        'Memory-guarded zeroize on plaintext buffers',
        'Ring-buffer O(1) sliding window anti-DoS rate limiter',
      ],
      code: `// Constant-time authenticated token verification
pub fn constant_time_eq_ascii(a: &str, b: &str) -> bool {
    if a.len() != b.len() { return false; }
    let mut acc = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        acc |= x ^ y;
    }
    acc == 0
}`,
      icon: ShieldCheck,
      accentColor: 'from-cyan-500 to-blue-600',
    },
    {
      id: 'layer-wasm',
      number: '03',
      title: 'WASM INSTRUCTION-METERED SANDBOX',
      badge: 'WASMTIME / FUEL TRAPS',
      category: 'UNTRUSTED EXTENSION ENGINE',
      description: 'Strictly bounded execution environment for third-party plugins with deterministic fuel counters, instruction limits, and fail-closed trap containment.',
      specs: [
        'Deterministic instruction fuel metering (anti-infinite loops)',
        'Memory allocation ceiling (max 64 MB heap cap)',
        'Declared capability manifests (crypto:sha256, storage:read)',
        'Host trap containment preventing core crashes',
      ],
      code: `// Instruction fuel metering & trap boundary
store.set_fuel(5_000_000)?;
let result = instance.call(&mut store, "execute", &[]);
match result {
    Err(trap) if trap.is_out_of_fuel() => Err(PluginError::QuotaExceeded),
    Ok(data) => Ok(data),
}`,
      icon: Cpu,
      accentColor: 'from-emerald-500 to-teal-600',
    },
    {
      id: 'layer-consensus',
      number: '04',
      title: 'MAKER-CHECKER FOUR-EYES CONSENSUS',
      badge: 'PCI-DSS REQ 10 / BASEL',
      category: 'FINANCIAL GOVERNANCE',
      description: 'Enforces corporate banking Segregation of Duties (SOD) and maintains an immutable cryptographic chained ledger with SHA-256 block hash continuity.',
      specs: [
        'Strict Segregation of Duties (checker_id != maker_id)',
        'Mandatory biometric step-up for amounts >= $10,000 USD',
        'Ed25519 threshold digital signature verification',
        'Chained SHA-256 block ledger with genesis continuity checks',
      ],
      code: `// Basel Four-Eyes Principle enforcement
if tx.maker_id.eq_ignore_ascii_case(checker_id) {
    return Err(BankingError::SelfApprovalForbidden);
}
let block_hash = compute_sha256_chain(&prev_hash, &tx);
ledger.append(AuditBlock::new(block_hash, tx));`,
      icon: Key,
      accentColor: 'from-amber-400 to-orange-600',
    },
    {
      id: 'layer-ui',
      number: '05',
      title: 'SANDBOXED FRONTEND & WEBVIEW2 HOST',
      badge: 'REACT 18 / WCAG 2.1 AA',
      category: 'PRESENTATION TIER',
      description: 'Isolated client runtime communicating solely via authenticated HTTP/IPC loopback with strict Content Security Policy and zero unverified origins.',
      specs: [
        'Strict Content-Security-Policy: frame-ancestors none',
        'Normalized RFC 6454 origin pre-validation',
        'Accessible WCAG 2.1 AA focus visible & screen-reader ARIA',
        'Zero-flash native desktop WebView2 embedding',
      ],
      code: `// Loopback IPC bridge contract validation
const response = await bridge.call('banking_transaction_propose', {
  makerId, fromAccount, toAccount, amount, currency
});
assertValidContract(response, TransactionResponseSchema);`,
      icon: Database,
      accentColor: 'from-purple-500 to-indigo-600',
    },
  ];

  return (
    <section id="architecture" className="relative py-28 px-4 sm:px-6 lg:px-8 border-t border-white/10 bg-[#050608]">
      {/* Tactical Background Grid Lines */}
      <div className="absolute inset-0 pointer-events-none opacity-20 bg-[linear-gradient(to_right,#80808012_1px,transparent_1px),linear-gradient(to_bottom,#80808012_1px,transparent_1px)] bg-[size:32px_32px]" />

      <div className="relative z-10 max-w-7xl mx-auto">
        {/* Section Header */}
        <div className="flex flex-col md:flex-row md:items-end justify-between mb-16 gap-6">
          <div className="space-y-3">
            <div className="inline-flex items-center gap-2 font-mono text-xs text-amber-400 uppercase tracking-widest">
              <Layers className="w-4 h-4" />
              <span>[SCHEMATIC SPECIFICATION] // 5-LAYER STACK</span>
            </div>
            <h2 className="text-3xl sm:text-5xl font-extrabold font-display text-white tracking-tight">
              DESIGNED AT THE <span className="text-amber-400 glow-amber">[TRANSISTOR]</span>
            </h2>
            <p className="text-slate-400 max-w-2xl text-sm sm:text-base font-light">
              Every layer isolated. Every permission capability-checked. Explore the 5 zero-trust security tiers separating untrusted frontend code from physical cryptographic hardware.
            </p>
          </div>

          <div className="flex items-center gap-3">
            <button
              onClick={() => {
                soundEffects.click();
                setIsExploded(!isExploded);
              }}
              className="px-4 py-2 rounded-lg bg-[#0d111a] border border-amber-500/30 hover:border-amber-400 text-amber-300 font-mono text-xs tracking-wider flex items-center gap-2 transition-all"
            >
              <Sparkles className="w-3.5 h-3.5" />
              <span>{isExploded ? 'COLLAPSE 3D STACK' : 'EXPLODE 3D STACK'}</span>
            </button>
          </div>
        </div>

        {/* 3D Interactive Stack & Details Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-10 items-start">
          {/* Left Column: 3D Exploded Layer Stack Cards */}
          <div className="lg:col-span-6 space-y-4 perspective-container">
            {layers.map((layer, index) => {
              const isSelected = selectedLayer === index;
              const LayerIcon = layer.icon;

              return (
                <div
                  key={layer.id}
                  onClick={() => {
                    soundEffects.click();
                    setSelectedLayer(index);
                  }}
                  onMouseEnter={() => soundEffects.hover()}
                  style={{
                    transform: isExploded
                      ? `translateY(${index * 4}px) translateZ(${isSelected ? 30 : 0}px)`
                      : 'none',
                  }}
                  className={`cursor-pointer isometric-layer rounded-xl border p-5 transition-all duration-300 relative crosshair ${
                    isSelected
                      ? 'bg-[#0f1420] border-amber-400 shadow-xl shadow-amber-500/10 border-glow-amber'
                      : 'bg-[#0a0d15]/90 border-white/10 hover:border-white/30 hover:bg-[#0c101a]'
                  }`}
                >
                  <div className="flex items-start justify-between">
                    <div className="flex items-center gap-3">
                      <div className={`p-2.5 rounded-lg bg-gradient-to-br ${layer.accentColor} text-black font-bold shadow-md`}>
                        <LayerIcon className="w-5 h-5 text-black" />
                      </div>
                      <div>
                        <div className="flex items-center gap-2">
                          <span className="font-mono text-xs text-amber-400 font-bold">L{layer.number}</span>
                          <span className="text-white/20">•</span>
                          <span className="font-mono text-[10px] text-slate-400 tracking-wider uppercase">{layer.category}</span>
                        </div>
                        <h3 className="font-display font-bold text-base text-white mt-0.5 tracking-tight">
                          {layer.title}
                        </h3>
                      </div>
                    </div>

                    <span className="font-mono text-[11px] text-slate-400 bg-white/5 px-2.5 py-1 rounded border border-white/10">
                      {layer.badge}
                    </span>
                  </div>

                  <p className="text-xs text-slate-400 mt-3 font-light leading-relaxed">
                    {layer.description}
                  </p>
                </div>
              );
            })}
          </div>

          {/* Right Column: Layer Micro-Inspection & Code Terminal */}
          <div className="lg:col-span-6 sticky top-28">
            <div className="rounded-xl bg-[#080b12] border border-amber-500/30 overflow-hidden shadow-2xl relative crosshair">
              {/* Terminal Title Bar */}
              <div className="flex items-center justify-between px-5 py-3.5 bg-[#0d121c] border-b border-white/10">
                <div className="flex items-center gap-2">
                  <div className="flex gap-1.5">
                    <div className="w-2.5 h-2.5 rounded-full bg-red-500/70" />
                    <div className="w-2.5 h-2.5 rounded-full bg-amber-500/70" />
                    <div className="w-2.5 h-2.5 rounded-full bg-emerald-500/70" />
                  </div>
                  <span className="font-mono text-xs text-slate-300 ml-2 font-semibold">
                    INSPECT // LAYER_{layers[selectedLayer].number}
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <Code2 className="w-3.5 h-3.5 text-amber-400" />
                  <span className="font-mono text-[11px] text-amber-400">RUST / IPC ENGINE</span>
                </div>
              </div>

              {/* Technical Specifications Checklist */}
              <div className="p-6 space-y-6">
                <div>
                  <h4 className="font-mono text-xs uppercase text-slate-400 tracking-wider mb-3">
                    Active Security Guardrails:
                  </h4>
                  <div className="grid grid-cols-1 sm:grid-cols-2 gap-2.5">
                    {layers[selectedLayer].specs.map((spec, sidx) => (
                      <div
                        key={sidx}
                        className="flex items-center gap-2 p-2.5 rounded bg-white/[0.02] border border-white/5 font-mono text-xs text-slate-300"
                      >
                        <div className="w-1.5 h-1.5 rounded-full bg-amber-400" />
                        <span className="truncate">{spec}</span>
                      </div>
                    ))}
                  </div>
                </div>

                {/* Live Code Implementation Preview */}
                <div>
                  <h4 className="font-mono text-xs uppercase text-slate-400 tracking-wider mb-2 flex items-center justify-between">
                    <span>Implementation Code Snapshot:</span>
                    <span className="text-[10px] text-emerald-400">VERIFIED PASS</span>
                  </h4>
                  <div className="p-4 rounded-lg bg-[#040507] border border-white/10 font-mono text-xs text-amber-200/90 overflow-x-auto leading-relaxed shadow-inner">
                    <pre>{layers[selectedLayer].code}</pre>
                  </div>
                </div>
              </div>

              {/* Bottom Telemetry Bar */}
              <div className="px-5 py-3 bg-[#0a0d16] border-t border-white/10 flex items-center justify-between text-[11px] font-mono text-slate-400">
                <span>STATUS: HARDWARE ACTIVE</span>
                <span className="text-emerald-400">FAIL-CLOSED ENFORCED</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};


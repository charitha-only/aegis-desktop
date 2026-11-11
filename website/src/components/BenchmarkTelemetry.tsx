import React from 'react';
import { Gauge, Zap, Database, ShieldCheck, CheckCircle2 } from 'lucide-react';

export const BenchmarkTelemetry: React.FC = () => {
  return (
    <section id="benchmarks" className="relative py-28 px-4 sm:px-6 lg:px-8 bg-[#07090e] border-t border-white/10">
      <div className="max-w-7xl mx-auto">
        {/* Section Header */}
        <div className="max-w-3xl mb-16 space-y-3">
          <div className="inline-flex items-center gap-2 font-mono text-xs text-amber-400 uppercase tracking-widest">
            <Gauge className="w-4 h-4" />
            <span>[LOW-RESOURCE PHYSICAL PROFILING]</span>
          </div>
          <h2 className="text-3xl sm:text-5xl font-extrabold font-display text-white tracking-tight">
            LIGHTNING TELEMETRY <span className="text-amber-400 glow-amber">[BENCHMARKS]</span>
          </h2>
          <p className="text-slate-400 text-sm sm:text-base font-light">
            Desktop security must not penalize battery life or user responsiveness. Pinned on physical AMD Ryzen hardware, Aegis delivers native sub-30ms startup and less than 10MB resident memory footprint.
          </p>
        </div>

        {/* 3 Large Telemetry Dials */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
          <div className="p-6 rounded-xl bg-[#0b0e17] border border-amber-500/30 relative crosshair">
            <div className="flex items-center justify-between text-slate-400 font-mono text-xs mb-3">
              <span>COLD STARTUP TIME</span>
              <Zap className="w-4 h-4 text-emerald-400" />
            </div>
            <div className="text-4xl font-extrabold font-display text-emerald-400 mb-1">25.8 ms</div>
            <div className="font-mono text-xs text-slate-400">Target Budget: &lt; 100.0 ms (74% faster)</div>
          </div>

          <div className="p-6 rounded-xl bg-[#0b0e17] border border-amber-500/30 relative crosshair">
            <div className="flex items-center justify-between text-slate-400 font-mono text-xs mb-3">
              <span>RESIDENT MEMORY (RSS)</span>
              <Database className="w-4 h-4 text-amber-400" />
            </div>
            <div className="text-4xl font-extrabold font-display text-amber-400 mb-1">8.80 MiB</div>
            <div className="font-mono text-xs text-slate-400">Total Tree: 20.4 MiB (Budget &lt; 50 MB)</div>
          </div>

          <div className="p-6 rounded-xl bg-[#0b0e17] border border-amber-500/30 relative crosshair">
            <div className="flex items-center justify-between text-slate-400 font-mono text-xs mb-3">
              <span>IPC ROUNDTRIP TIME</span>
              <ShieldCheck className="w-4 h-4 text-cyan-400" />
            </div>
            <div className="text-4xl font-extrabold font-display text-cyan-400 mb-1">1.6 ms</div>
            <div className="font-mono text-xs text-slate-400">Amortized O(1) Scoped IPC Workers</div>
          </div>
        </div>

        {/* Comparison Matrix Table */}
        <div className="rounded-xl bg-[#0a0d16] border border-white/10 overflow-hidden shadow-2xl relative crosshair">
          <div className="px-6 py-4 bg-[#0e1320] border-b border-white/10 flex items-center justify-between font-mono text-xs text-slate-200">
            <span className="font-bold">HEAD-TO-HEAD ARCHITECTURAL COMPARISON</span>
            <span className="text-amber-400">PHYSICAL TEST EVIDENCE</span>
          </div>

          <div className="overflow-x-auto">
            <table className="w-full text-left font-mono text-xs">
              <thead>
                <tr className="border-b border-white/10 text-slate-400 bg-white/[0.01]">
                  <th className="py-3 px-6">METRIC / SPEC</th>
                  <th className="py-3 px-6 text-amber-400 font-bold">AEGIS DESKTOP v1.1.0</th>
                  <th className="py-3 px-6 text-slate-400">TAURI 2.x</th>
                  <th className="py-3 px-6 text-slate-400">ELECTRON 32</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-white/5 text-slate-300">
                <tr>
                  <td className="py-3.5 px-6 font-semibold">Cold Startup Latency</td>
                  <td className="py-3.5 px-6 text-emerald-400 font-bold">25.8 ms</td>
                  <td className="py-3.5 px-6 text-slate-400">~110 ms</td>
                  <td className="py-3.5 px-6 text-red-400">~480 ms</td>
                </tr>
                <tr>
                  <td className="py-3.5 px-6 font-semibold">Idle Process Memory (RSS)</td>
                  <td className="py-3.5 px-6 text-emerald-400 font-bold">8.80 MiB</td>
                  <td className="py-3.5 px-6 text-slate-400">~35.0 MiB</td>
                  <td className="py-3.5 px-6 text-red-400">~145.0 MiB</td>
                </tr>
                <tr>
                  <td className="py-3.5 px-6 font-semibold">Standalone Bundle Size</td>
                  <td className="py-3.5 px-6 text-emerald-400 font-bold">2.70 MB</td>
                  <td className="py-3.5 px-6 text-slate-400">~12.0 MB</td>
                  <td className="py-3.5 px-6 text-red-400">~85.0 MB</td>
                </tr>
                <tr>
                  <td className="py-3.5 px-6 font-semibold">Screen Capture Masking</td>
                  <td className="py-3.5 px-6 text-emerald-400 font-bold">Hardware Win32 DWM Built-in</td>
                  <td className="py-3.5 px-6 text-slate-500">Unimplemented Native Plugin</td>
                  <td className="py-3.5 px-6 text-red-400">No (Leaks to Screen Share)</td>
                </tr>
                <tr>
                  <td className="py-3.5 px-6 font-semibold">Four-Eyes Dual Control</td>
                  <td className="py-3.5 px-6 text-emerald-400 font-bold">Ed25519 + SHA-256 Chained</td>
                  <td className="py-3.5 px-6 text-slate-500">Requires Custom Backend</td>
                  <td className="py-3.5 px-6 text-slate-500">Requires Custom Backend</td>
                </tr>
                <tr>
                  <td className="py-3.5 px-6 font-semibold">Plugin Sandbox Safety</td>
                  <td className="py-3.5 px-6 text-emerald-400 font-bold">Wasm Instruction Fuel Limits</td>
                  <td className="py-3.5 px-6 text-slate-400">Native Rust Grants</td>
                  <td className="py-3.5 px-6 text-red-400">Vulnerable to Node Privileges</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </section>
  );
};

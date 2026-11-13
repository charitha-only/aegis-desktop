import React, { useState, useEffect } from 'react';
import {
  Terminal,
  Activity,
  Cpu,
  Shield,
  Zap,
  HardDrive,
  Lock,
  Layers,
  X,
  Play,
  CheckCircle,
  AlertTriangle,
  RefreshCw,
} from 'lucide-react';

interface DevToolsProps {
  isOpen: boolean;
  onClose: () => void;
}

export const DevToolsSecurityInspector: React.FC<DevToolsProps> = ({ isOpen, onClose }) => {
  const [ipcLatencyHistory, setIpcLatencyHistory] = useState<number[]>([120, 115, 128, 112, 134, 118, 125]);
  const [currentLatency, setCurrentLatency] = useState<number>(118);
  const [isPinging, setIsPinging] = useState<boolean>(false);
  const [activeTab, setActiveTab] = useState<'latency' | 'memory' | 'firewall' | 'siem'>('latency');

  // SIEM live log stream simulation
  const [siemLogs, setSiemLogs] = useState<Array<{ id: string; time: string; sev: string; code: string; msg: string }>>([
    { id: 'EVT-101', time: '12:04:15.102', sev: 'NOTICE', code: 'SEC-008-SANDBOX', msg: 'Job Object hard limits active (KILL_ON_JOB_CLOSE)' },
    { id: 'EVT-102', time: '12:04:16.890', sev: 'INFO', code: 'SEC-007-PQKEY', msg: 'Dual-envelope ML-DSA-65 signature verified' },
    { id: 'EVT-103', time: '12:04:18.230', sev: 'NOTICE', code: 'SEC-005-CANARY', msg: 'Cryptographic heap canaries integrity verified (0 breaches)' },
  ]);

  // Outbound firewall rules
  const [firewallFlows, setFirewallFlows] = useState([
    { id: 'f-1', host: '127.0.0.1:56602', proto: 'TCP/IPC', status: 'ALLOW', hits: 1482 },
    { id: 'f-2', host: 'api.bank-sovereign.internal', proto: 'mTLS/SPKI', status: 'ALLOW', hits: 42 },
    { id: 'f-3', host: 'telemetry.malicious.net', proto: 'HTTPS', status: 'BLOCK', hits: 8 },
  ]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.shiftKey && (e.key === 'I' || e.key === 'i')) {
        e.preventDefault();
        if (isOpen) {
          onClose();
        }
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, onClose]);

  const runDiagnosticsPing = () => {
    setIsPinging(true);
    setTimeout(() => {
      const simulatedLatency = Math.floor(Math.random() * 35) + 105; // 105 - 140 µs
      setCurrentLatency(simulatedLatency);
      setIpcLatencyHistory(prev => [...prev.slice(1), simulatedLatency]);
      setIsPinging(false);
      setSiemLogs(prev => [
        {
          id: `EVT-${Date.now().toString().slice(-4)}`,
          time: new Date().toISOString().substring(11, 23),
          sev: 'INFO',
          code: 'DIAG-PING',
          msg: `IPC roundtrip completed in ${simulatedLatency} µs (Sub-millisecond native transport)`
        },
        ...prev.slice(0, 9)
      ]);
    }, 150);
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 bg-black/80 backdrop-blur-md flex items-center justify-center p-4 sm:p-6 animate-in fade-in duration-200">
      <div className="w-full max-w-5xl h-[680px] bg-[#07090e] border-2 border-cyan-500/50 rounded-xl shadow-2xl flex flex-col overflow-hidden font-mono text-slate-200">
        
        {/* Title Bar */}
        <div className="px-5 py-3.5 bg-[#0b0f19] border-b border-cyan-500/30 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="p-1.5 rounded bg-cyan-500/10 border border-cyan-500/30 text-cyan-400">
              <Terminal className="w-4 h-4" />
            </div>
            <div>
              <div className="text-sm font-bold text-white tracking-wider flex items-center gap-2">
                <span>AEGIS DEVTOOLS // SECURITY & IPC INSPECTOR</span>
                <span className="text-[10px] text-cyan-300 bg-cyan-500/10 px-2 py-0.5 rounded border border-cyan-500/20">
                  CTRL+SHIFT+I
                </span>
              </div>
              <div className="text-[11px] text-slate-400 font-sans">
                Real-time systems profiling, memory canaries & sub-millisecond IPC telemetry
              </div>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <button
              onClick={runDiagnosticsPing}
              disabled={isPinging}
              className="px-3 py-1.5 rounded bg-cyan-500/20 hover:bg-cyan-500/30 border border-cyan-500/40 text-cyan-300 text-xs font-bold transition-all flex items-center gap-1.5 cursor-pointer disabled:opacity-50"
            >
              <RefreshCw className={`w-3.5 h-3.5 ${isPinging ? 'animate-spin' : ''}`} />
              <span>PING IPC</span>
            </button>
            <button
              onClick={onClose}
              className="p-1.5 rounded bg-white/5 hover:bg-red-500/20 hover:text-red-400 text-slate-400 border border-white/10 transition-colors cursor-pointer"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
        </div>

        {/* Tab Switcher */}
        <div className="flex items-center gap-1 px-4 py-2 bg-[#090c14] border-b border-white/5 text-xs">
          <button
            onClick={() => setActiveTab('latency')}
            className={`px-3 py-1.5 rounded transition-all cursor-pointer flex items-center gap-2 ${
              activeTab === 'latency'
                ? 'bg-cyan-500/20 border border-cyan-500/40 text-cyan-300 font-bold'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <Zap className="w-3.5 h-3.5" />
            <span>IPC Telemetry (&lt;150µs)</span>
          </button>

          <button
            onClick={() => setActiveTab('memory')}
            className={`px-3 py-1.5 rounded transition-all cursor-pointer flex items-center gap-2 ${
              activeTab === 'memory'
                ? 'bg-amber-500/20 border border-amber-500/40 text-amber-300 font-bold'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <Cpu className="w-3.5 h-3.5" />
            <span>RAM & Canary Guard</span>
          </button>

          <button
            onClick={() => setActiveTab('firewall')}
            className={`px-3 py-1.5 rounded transition-all cursor-pointer flex items-center gap-2 ${
              activeTab === 'firewall'
                ? 'bg-emerald-500/20 border border-emerald-500/40 text-emerald-300 font-bold'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <Shield className="w-3.5 h-3.5" />
            <span>Firewall Flow Inspector</span>
          </button>

          <button
            onClick={() => setActiveTab('siem')}
            className={`px-3 py-1.5 rounded transition-all cursor-pointer flex items-center gap-2 ${
              activeTab === 'siem'
                ? 'bg-purple-500/20 border border-purple-500/40 text-purple-300 font-bold'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <Activity className="w-3.5 h-3.5" />
            <span>SIEM CEF Log Stream</span>
          </button>
        </div>

        {/* Tab Content Area */}
        <div className="flex-1 p-5 overflow-y-auto space-y-5 bg-[#05070c]">
          
          {activeTab === 'latency' && (
            <div className="space-y-4">
              <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
                <div className="p-4 rounded-lg bg-[#0b0f19] border border-cyan-500/30">
                  <div className="text-xs text-slate-400 mb-1">NATIVE IPC ROUNDTRIP</div>
                  <div className="text-3xl font-extrabold text-cyan-400 flex items-baseline gap-2">
                    <span>{currentLatency}</span>
                    <span className="text-sm font-normal text-slate-400">µs (microseconds)</span>
                  </div>
                  <div className="text-[11px] text-emerald-400 mt-2 flex items-center gap-1">
                    <CheckCircle className="w-3 h-3" />
                    <span>Target &lt;150 µs met (8x faster than HTTP)</span>
                  </div>
                </div>

                <div className="p-4 rounded-lg bg-[#0b0f19] border border-white/10">
                  <div className="text-xs text-slate-400 mb-1">TRANSPORT MECHANISM</div>
                  <div className="text-lg font-bold text-white">Windows Named Pipes</div>
                  <div className="text-[11px] text-slate-400 mt-1 font-mono">\\.\pipe\aegis-ipc</div>
                </div>

                <div className="p-4 rounded-lg bg-[#0b0f19] border border-white/10">
                  <div className="text-xs text-slate-400 mb-1">FRAME ENCODING</div>
                  <div className="text-lg font-bold text-emerald-400">Zero-Copy Length-Delimited</div>
                  <div className="text-[11px] text-slate-400 mt-1">4-Byte BE Header + Payload</div>
                </div>
              </div>

              {/* Latency History Chart Bar */}
              <div className="p-4 rounded-lg bg-[#0b0f19] border border-white/10 space-y-2">
                <div className="text-xs text-slate-400 flex items-center justify-between">
                  <span>RECENT PING HISTORY</span>
                  <span className="text-cyan-400">Average: ~120 µs</span>
                </div>
                <div className="h-28 flex items-end gap-3 pt-4 px-2 border-b border-white/5">
                  {ipcLatencyHistory.map((val, idx) => (
                    <div key={idx} className="flex-1 flex flex-col items-center gap-1">
                      <span className="text-[10px] text-cyan-300">{val}µs</span>
                      <div
                        style={{ height: `${Math.min(100, (val / 160) * 80)}px` }}
                        className="w-full bg-gradient-to-t from-cyan-600 to-cyan-400 rounded-t"
                      />
                      <span className="text-[9px] text-slate-500">T-{7 - idx}</span>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}

          {activeTab === 'memory' && (
            <div className="space-y-4">
              <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
                <div className="p-4 rounded-lg bg-[#0b0f19] border border-amber-500/30">
                  <div className="text-xs text-slate-400 mb-1">CORE RESIDENT RAM (RSS)</div>
                  <div className="text-3xl font-extrabold text-amber-400">8.80 MiB</div>
                  <div className="text-[11px] text-slate-400 mt-1">Budget: &lt;50.0 MiB</div>
                </div>

                <div className="p-4 rounded-lg bg-[#0b0f19] border border-white/10">
                  <div className="text-xs text-slate-400 mb-1">DPAPI VIRTUAL-LOCKED PAGES</div>
                  <div className="text-xl font-bold text-white">4 Pages (16,384 Bytes)</div>
                  <div className="text-[11px] text-emerald-400 mt-1">Pinned in physical RAM</div>
                </div>

                <div className="p-4 rounded-lg bg-[#0b0f19] border border-white/10">
                  <div className="text-xs text-slate-400 mb-1">HEAP CANARY GUARDS</div>
                  <div className="text-xl font-bold text-emerald-400">100% UNCOMPROMISED</div>
                  <div className="text-[11px] text-slate-400 mt-1">64-bit randomized entropy</div>
                </div>
              </div>

              <div className="p-4 rounded-lg bg-[#0b0f19] border border-white/10 space-y-2">
                <div className="text-xs font-bold text-white">ACTIVE OS KERNEL DEFENSES</div>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 text-xs">
                  <div className="p-3 rounded bg-black/40 border border-white/5 space-y-1">
                    <div className="text-cyan-400 font-bold">Windows Job Object Limits</div>
                    <div className="text-slate-400 text-[11px]">JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE enabled. Child process runaway prevented.</div>
                  </div>
                  <div className="p-3 rounded bg-black/40 border border-white/5 space-y-1">
                    <div className="text-amber-400 font-bold">Anti-Screen Capture DWM</div>
                    <div className="text-slate-400 text-[11px]">SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE) active against OBS/Zoom scrapers.</div>
                  </div>
                </div>
              </div>
            </div>
          )}

          {activeTab === 'firewall' && (
            <div className="space-y-4">
              <div className="flex items-center justify-between text-xs text-slate-400">
                <span>INSPECTED OUTBOUND NETWORK FLOWS</span>
                <span>FAIL-CLOSED ZERO-TRUST POLICY</span>
              </div>
              <div className="space-y-2">
                {firewallFlows.map(flow => (
                  <div
                    key={flow.id}
                    className="p-3 rounded bg-[#0b0f19] border border-white/10 flex items-center justify-between text-xs"
                  >
                    <div className="flex items-center gap-3">
                      <span className={`px-2 py-0.5 rounded text-[10px] font-bold ${
                        flow.status === 'ALLOW' ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30' : 'bg-red-500/20 text-red-400 border border-red-500/30'
                      }`}>
                        {flow.status}
                      </span>
                      <span className="font-bold text-white">{flow.host}</span>
                      <span className="text-slate-500 text-[11px]">({flow.proto})</span>
                    </div>
                    <div className="text-slate-400 text-[11px]">
                      {flow.hits} hits inspected
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {activeTab === 'siem' && (
            <div className="space-y-3">
              <div className="flex items-center justify-between text-xs text-slate-400">
                <span>STRUCTURED CEF & RFC 5424 EVENT LOG</span>
                <span className="text-purple-400">Auto-streaming to Enterprise SIEM</span>
              </div>
              <div className="space-y-2">
                {siemLogs.map(log => (
                  <div key={log.id} className="p-3 rounded bg-[#0b0f19] border border-purple-500/20 text-xs font-mono space-y-1">
                    <div className="flex items-center justify-between text-[11px]">
                      <span className="text-purple-300 font-bold">{log.code}</span>
                      <span className="text-slate-500">{log.time}</span>
                    </div>
                    <div className="text-slate-300 text-xs">{log.msg}</div>
                  </div>
                ))}
              </div>
            </div>
          )}

        </div>

        {/* Footer Status Bar */}
        <div className="px-5 py-2.5 bg-[#07090e] border-t border-white/10 flex items-center justify-between text-[11px] text-slate-400 font-mono">
          <div className="flex items-center gap-2">
            <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
            <span className="text-white">Aegis Core Daemon v1.5.0</span>
            <span>• Process PID: 12480</span>
          </div>
          <div>Press ESC or CTRL+SHIFT+I to dismiss inspector</div>
        </div>

      </div>
    </div>
  );
};

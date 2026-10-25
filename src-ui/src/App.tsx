import React, { useState, useEffect } from 'react';
import {
  Shield,
  ShieldCheck,
  ShieldAlert,
  Lock,
  Cpu,
  FolderLock,
  FileCode,
  Terminal,
  Activity,
  CheckCircle2,
  AlertTriangle,
  RefreshCw,
  FileText,
  Key,
  Database,
  ExternalLink,
  Zap,
} from 'lucide-react';
import {
  aegis,
  SecurityStatus,
  SystemInfo,
  SandboxEntry,
  IpcResponse,
} from './aegis-bridge';

interface AuditLog {
  id: string;
  timestamp: string;
  type: 'SUCCESS' | 'BLOCKED' | 'INFO';
  command: string;
  details: string;
}

export default function App() {
  const [activeTab, setActiveTab] = useState<'security' | 'filesystem' | 'diagnostics' | 'logs'>('security');
  const [connected, setConnected] = useState(false);
  const [nonce, setNonce] = useState<string | null>(null);
  const [securityStatus, setSecurityStatus] = useState<SecurityStatus | null>(null);
  const [sysInfo, setSysInfo] = useState<SystemInfo | null>(null);
  const [auditLogs, setAuditLogs] = useState<AuditLog[]>([]);

  // File System State
  const [sandboxFiles, setSandboxFiles] = useState<SandboxEntry[]>([]);
  const [fileName, setFileName] = useState('secure_document.txt');
  const [fileContent, setFileContent] = useState('Encrypted record verified by Aegis Core Sandbox Engine.');
  const [readResult, setReadResult] = useState<string | null>(null);

  // Crypto State
  const [hashInput, setHashInput] = useState('Aegis Zero Trust Security Payload 2026');
  const [hashResult, setHashResult] = useState<string | null>(null);

  const addLog = (type: 'SUCCESS' | 'BLOCKED' | 'INFO', command: string, details: string) => {
    const newLog: AuditLog = {
      id: Math.random().toString(36).substring(2, 9),
      timestamp: new Date().toLocaleTimeString(),
      type,
      command,
      details,
    };
    setAuditLogs((prev) => [newLog, ...prev.slice(0, 49)]);
  };

  const connectToCore = async () => {
    const handshake = await aegis.init();
    if (handshake.success) {
      setConnected(true);
      setNonce(handshake.token || null);
      addLog('SUCCESS', 'handshake', 'Cryptographic Nonce negotiated with Aegis Core');

      const sec = await aegis.getSecurityStatus();
      if (sec.success && sec.data) {
        setSecurityStatus(sec.data);
      }

      const sys = await aegis.getSysInfo();
      if (sys.success && sys.data) {
        setSysInfo(sys.data);
      }

      refreshFiles();
    } else {
      setConnected(false);
      addLog('BLOCKED', 'handshake_failed', handshake.error || 'Connection refused');
    }
  };

  const refreshFiles = async () => {
    const res = await aegis.listSandbox();
    if (res.success && res.data) {
      setSandboxFiles(res.data.entries);
    }
  };

  useEffect(() => {
    connectToCore();
  }, []);

  // Attack Simulators
  const handleSimulateTraversal = async () => {
    addLog('INFO', 'attack_sim:traversal', 'Injecting path traversal payload: "../../../../Windows/win.ini"');
    const res = await aegis.simulatePathTraversalAttack();
    if (!res.success) {
      addLog('BLOCKED', 'path_traversal_blocked', res.error || 'Blocked by Rust canonicalization engine');
    } else {
      addLog('SUCCESS', 'traversal_allowed', 'Warning: Action permitted');
    }
  };

  const handleSimulateShell = async () => {
    addLog('INFO', 'attack_sim:shell', 'Attempting arbitrary shell execution: "cmd.exe /c whoami"');
    const res = await aegis.simulateShellExecutionAttack();
    if (!res.success) {
      addLog('BLOCKED', 'shell_exec_blocked', res.error || 'Capability revoked');
    } else {
      addLog('SUCCESS', 'shell_exec_allowed', 'Action permitted');
    }
  };

  const handleSimulateSpoofedNonce = async () => {
    addLog('INFO', 'attack_sim:nonce', 'Simulating forged nonce: "FORGED_INVALID_TOKEN_99999"');
    const res = await aegis.simulateSpoofedNonceAttack();
    if (!res.success) {
      addLog('BLOCKED', 'nonce_breach_blocked', res.error || 'Cryptographic nonce mismatch');
    } else {
      addLog('SUCCESS', 'nonce_accepted', 'Forged token accepted');
    }
  };

  const handleWriteFile = async () => {
    if (!fileName) return;
    const res = await aegis.writeSandboxFile(fileName, fileContent);
    if (res.success) {
      addLog('SUCCESS', 'fs_write_file', `Safely written to ./sandbox/${fileName}`);
      refreshFiles();
    } else {
      addLog('BLOCKED', 'fs_write_file', res.error || 'Write error');
    }
  };

  const handleReadFile = async (path: string) => {
    const res = await aegis.readSandboxFile(path);
    if (res.success && res.data) {
      setReadResult(res.data.content);
      addLog('SUCCESS', 'fs_read_file', `Read from ./sandbox/${path}`);
    } else {
      addLog('BLOCKED', 'fs_read_file', res.error || 'Read error');
    }
  };

  const handleComputeHash = async () => {
    const res = await aegis.hashInput(hashInput);
    if (res.success && res.data) {
      setHashResult(res.data.hash);
      addLog('SUCCESS', 'crypto_hash', `Generated SHA-256: ${res.data.hash.slice(0, 16)}...`);
    } else {
      addLog('BLOCKED', 'crypto_hash', res.error || 'Hash failed');
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh', width: '100vw', background: 'radial-gradient(circle at 50% 10%, #1e1e38 0%, #0a0b14 100%)' }}>
      {/* Custom Window Titlebar */}
      <header style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '10px 20px', borderBottom: '1px solid rgba(255,255,255,0.08)', background: 'rgba(10, 12, 22, 0.85)', backdropFilter: 'blur(10px)' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
          <div style={{ background: 'linear-gradient(135deg, #6366f1, #a855f7)', padding: '6px', borderRadius: '8px', display: 'flex' }}>
            <Shield style={{ width: '20px', height: '20px', color: '#fff' }} />
          </div>
          <div>
            <div style={{ fontSize: '14px', fontWeight: 600, letterSpacing: '0.5px', color: '#f1f5f9' }}>
              AEGIS DESKTOP <span style={{ fontSize: '11px', color: '#818cf8', background: 'rgba(99, 102, 241, 0.15)', padding: '2px 6px', borderRadius: '4px', marginLeft: '6px' }}>v1.0 ENTERPRISE</span>
            </div>
            <div style={{ fontSize: '11px', color: '#64748b' }}>Custom Rust Core + React High-Security Framework</div>
          </div>
        </div>

        {/* Connection & Nonce Badge */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', background: connected ? 'rgba(16, 185, 129, 0.1)' : 'rgba(239, 68, 68, 0.1)', border: `1px solid ${connected ? 'rgba(16, 185, 129, 0.3)' : 'rgba(239, 68, 68, 0.3)'}`, padding: '4px 12px', borderRadius: '20px' }}>
            <div style={{ width: '8px', height: '8px', borderRadius: '50%', background: connected ? '#10b981' : '#ef4444', boxShadow: connected ? '0 0 8px #10b981' : 'none' }}></div>
            <span style={{ fontSize: '12px', fontWeight: 500, color: connected ? '#34d399' : '#f87171' }}>
              {connected ? 'Rust Core Connected' : 'Core Offline'}
            </span>
          </div>

          <button
            onClick={connectToCore}
            title="Reconnect"
            style={{ background: 'rgba(255,255,255,0.05)', border: '1px solid rgba(255,255,255,0.1)', padding: '6px', borderRadius: '6px', cursor: 'pointer', color: '#94a3b8' }}
          >
            <RefreshCw style={{ width: '14px', height: '14px' }} />
          </button>
        </div>
      </header>

      {/* Main Layout */}
      <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
        {/* Navigation Sidebar */}
        <nav style={{ width: '240px', background: 'rgba(12, 15, 29, 0.95)', borderRight: '1px solid rgba(255,255,255,0.06)', padding: '20px 12px', display: 'flex', flexDirection: 'column', gap: '8px' }}>
          <div style={{ fontSize: '11px', fontWeight: 600, color: '#475569', textTransform: 'uppercase', letterSpacing: '1px', padding: '0 12px 8px' }}>
            Framework Controls
          </div>

          <button
            onClick={() => setActiveTab('security')}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '10px',
              padding: '10px 14px',
              borderRadius: '8px',
              border: 'none',
              background: activeTab === 'security' ? 'linear-gradient(90deg, rgba(99, 102, 241, 0.2), rgba(99, 102, 241, 0.05))' : 'transparent',
              color: activeTab === 'security' ? '#818cf8' : '#94a3b8',
              cursor: 'pointer',
              fontWeight: 500,
              fontSize: '13px',
              textAlign: 'left',
              transition: 'all 0.15s ease',
            }}
          >
            <ShieldCheck style={{ width: '18px', height: '18px' }} />
            Security & Defense
          </button>

          <button
            onClick={() => setActiveTab('filesystem')}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '10px',
              padding: '10px 14px',
              borderRadius: '8px',
              border: 'none',
              background: activeTab === 'filesystem' ? 'linear-gradient(90deg, rgba(99, 102, 241, 0.2), rgba(99, 102, 241, 0.05))' : 'transparent',
              color: activeTab === 'filesystem' ? '#818cf8' : '#94a3b8',
              cursor: 'pointer',
              fontWeight: 500,
              fontSize: '13px',
              textAlign: 'left',
              transition: 'all 0.15s ease',
            }}
          >
            <FolderLock style={{ width: '18px', height: '18px' }} />
            Sandboxed FS
          </button>

          <button
            onClick={() => setActiveTab('diagnostics')}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '10px',
              padding: '10px 14px',
              borderRadius: '8px',
              border: 'none',
              background: activeTab === 'diagnostics' ? 'linear-gradient(90deg, rgba(99, 102, 241, 0.2), rgba(99, 102, 241, 0.05))' : 'transparent',
              color: activeTab === 'diagnostics' ? '#818cf8' : '#94a3b8',
              cursor: 'pointer',
              fontWeight: 500,
              fontSize: '13px',
              textAlign: 'left',
              transition: 'all 0.15s ease',
            }}
          >
            <Cpu style={{ width: '18px', height: '18px' }} />
            Diagnostics & Crypto
          </button>

          <button
            onClick={() => setActiveTab('logs')}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '10px',
              padding: '10px 14px',
              borderRadius: '8px',
              border: 'none',
              background: activeTab === 'logs' ? 'linear-gradient(90deg, rgba(99, 102, 241, 0.2), rgba(99, 102, 241, 0.05))' : 'transparent',
              color: activeTab === 'logs' ? '#818cf8' : '#94a3b8',
              cursor: 'pointer',
              fontWeight: 500,
              fontSize: '13px',
              textAlign: 'left',
              transition: 'all 0.15s ease',
            }}
          >
            <Activity style={{ width: '18px', height: '18px' }} />
            Live Audit Stream
            {auditLogs.length > 0 && (
              <span style={{ marginLeft: 'auto', background: 'rgba(255,255,255,0.1)', fontSize: '10px', padding: '2px 6px', borderRadius: '10px', color: '#cbd5e1' }}>
                {auditLogs.length}
              </span>
            )}
          </button>

          {/* Security Pill Info Box */}
          <div style={{ marginTop: 'auto', background: 'rgba(15, 23, 42, 0.6)', border: '1px solid rgba(255,255,255,0.05)', padding: '12px', borderRadius: '8px' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '6px', fontSize: '11px', color: '#38bdf8', marginBottom: '6px' }}>
              <Lock style={{ width: '12px', height: '12px' }} />
              Zero-Trust Architecture
            </div>
            <div style={{ fontSize: '10px', color: '#64748b', lineHeight: '1.4' }}>
              Every IPC invocation passes through strict Rust capability verification & cryptographic nonces.
            </div>
          </div>
        </nav>

        {/* Content Area */}
        <main style={{ flex: 1, padding: '24px 32px', overflowY: 'auto' }}>
          {/* TAB 1: SECURITY & DEFENSE */}
          {activeTab === 'security' && (
            <div>
              <div style={{ marginBottom: '24px' }}>
                <h1 style={{ fontSize: '22px', fontWeight: 600, color: '#f8fafc', marginBottom: '6px' }}>
                  Security Architecture & Threat Defense
                </h1>
                <p style={{ fontSize: '13px', color: '#94a3b8' }}>
                  Enterprise-grade capability access control, CSP isolation, and real-time defense simulations.
                </p>
              </div>

              {/* Security Metrics Cards */}
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '16px', marginBottom: '24px' }}>
                <div style={{ background: 'rgba(15, 23, 42, 0.65)', border: '1px solid rgba(99, 102, 241, 0.2)', padding: '18px', borderRadius: '12px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '10px' }}>
                    <span style={{ fontSize: '12px', color: '#94a3b8' }}>Session Nonce</span>
                    <Key style={{ width: '16px', height: '16px', color: '#818cf8' }} />
                  </div>
                  <div style={{ fontSize: '12px', fontFamily: 'monospace', color: '#a5b4fc', background: 'rgba(0,0,0,0.3)', padding: '6px 8px', borderRadius: '6px', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                    {nonce ? nonce : 'Awaiting Connection...'}
                  </div>
                  <div style={{ fontSize: '11px', color: '#34d399', marginTop: '8px', display: 'flex', alignItems: 'center', gap: '4px' }}>
                    <CheckCircle2 style={{ width: '12px', height: '12px' }} /> Replay Protection Active
                  </div>
                </div>

                <div style={{ background: 'rgba(15, 23, 42, 0.65)', border: '1px solid rgba(16, 185, 129, 0.2)', padding: '18px', borderRadius: '12px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '10px' }}>
                    <span style={{ fontSize: '12px', color: '#94a3b8' }}>Sandbox Boundary</span>
                    <FolderLock style={{ width: '16px', height: '16px', color: '#34d399' }} />
                  </div>
                  <div style={{ fontSize: '18px', fontWeight: 600, color: '#34d399' }}>STRICT ISOLATION</div>
                  <div style={{ fontSize: '11px', color: '#64748b', marginTop: '6px' }}>
                    Filesystem scoped strictly to <code style={{ color: '#cbd5e1' }}>./sandbox/</code>
                  </div>
                </div>

                <div style={{ background: 'rgba(15, 23, 42, 0.65)', border: '1px solid rgba(239, 68, 68, 0.2)', padding: '18px', borderRadius: '12px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '10px' }}>
                    <span style={{ fontSize: '12px', color: '#94a3b8' }}>Shell Execution Policy</span>
                    <Terminal style={{ width: '16px', height: '16px', color: '#f87171' }} />
                  </div>
                  <div style={{ fontSize: '18px', fontWeight: 600, color: '#f87171' }}>REVOKED / BLOCKED</div>
                  <div style={{ fontSize: '11px', color: '#64748b', marginTop: '6px' }}>
                    Arbitrary shell invocation disabled by default policy
                  </div>
                </div>
              </div>

              {/* Interactive Threat Defense Simulator */}
              <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '24px' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '16px' }}>
                  <Zap style={{ width: '20px', height: '20px', color: '#fbbf24' }} />
                  <h3 style={{ fontSize: '16px', fontWeight: 600, color: '#f8fafc' }}>
                    Interactive Attack Simulator (Live Threat Testing)
                  </h3>
                </div>
                <p style={{ fontSize: '13px', color: '#94a3b8', marginBottom: '20px' }}>
                  Click below to trigger malicious attacks against the Aegis Rust engine. Observe how the engine safely traps and denies each violation.
                </p>

                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '14px' }}>
                  <button
                    onClick={handleSimulateTraversal}
                    style={{
                      background: 'rgba(239, 68, 68, 0.1)',
                      border: '1px solid rgba(239, 68, 68, 0.3)',
                      color: '#fca5a5',
                      padding: '14px',
                      borderRadius: '8px',
                      cursor: 'pointer',
                      textAlign: 'left',
                      transition: 'all 0.15s',
                    }}
                  >
                    <div style={{ fontWeight: 600, fontSize: '13px', marginBottom: '4px' }}>1. Path Traversal Attack</div>
                    <div style={{ fontSize: '11px', color: '#f87171' }}>Injects <code style={{ color: '#fff' }}>../../Windows/win.ini</code></div>
                  </button>

                  <button
                    onClick={handleSimulateShell}
                    style={{
                      background: 'rgba(245, 158, 11, 0.1)',
                      border: '1px solid rgba(245, 158, 11, 0.3)',
                      color: '#fcd34d',
                      padding: '14px',
                      borderRadius: '8px',
                      cursor: 'pointer',
                      textAlign: 'left',
                      transition: 'all 0.15s',
                    }}
                  >
                    <div style={{ fontWeight: 600, fontSize: '13px', marginBottom: '4px' }}>2. Arbitrary Shell Attack</div>
                    <div style={{ fontSize: '11px', color: '#fbbf24' }}>Invokes <code style={{ color: '#fff' }}>cmd.exe /c whoami</code></div>
                  </button>

                  <button
                    onClick={handleSimulateSpoofedNonce}
                    style={{
                      background: 'rgba(168, 85, 247, 0.1)',
                      border: '1px solid rgba(168, 85, 247, 0.3)',
                      color: '#d8b4fe',
                      padding: '14px',
                      borderRadius: '8px',
                      cursor: 'pointer',
                      textAlign: 'left',
                      transition: 'all 0.15s',
                    }}
                  >
                    <div style={{ fontWeight: 600, fontSize: '13px', marginBottom: '4px' }}>3. Forged Nonce Attack</div>
                    <div style={{ fontSize: '11px', color: '#c084fc' }}>Spoofs cryptographic session token</div>
                  </button>
                </div>
              </div>
            </div>
          )}

          {/* TAB 2: SANDBOXED FILE SYSTEM */}
          {activeTab === 'filesystem' && (
            <div>
              <div style={{ marginBottom: '24px' }}>
                <h1 style={{ fontSize: '22px', fontWeight: 600, color: '#f8fafc', marginBottom: '6px' }}>
                  Sandboxed Storage & Isolation Manager
                </h1>
                <p style={{ fontSize: '13px', color: '#94a3b8' }}>
                  Read and write files strictly within the protected sandbox directory without exposing the host OS.
                </p>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px' }}>
                {/* File Editor Card */}
                <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
                  <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc', marginBottom: '14px', display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <FileCode style={{ width: '16px', height: '16px', color: '#6366f1' }} />
                    Write Sandboxed File
                  </h3>

                  <div style={{ marginBottom: '12px' }}>
                    <label style={{ fontSize: '12px', color: '#94a3b8', display: 'block', marginBottom: '6px' }}>Filename (Inside ./sandbox/)</label>
                    <input
                      type="text"
                      value={fileName}
                      onChange={(e) => setFileName(e.target.value)}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', padding: '8px 12px', borderRadius: '6px', color: '#f1f5f9', fontSize: '13px', outline: 'none' }}
                    />
                  </div>

                  <div style={{ marginBottom: '16px' }}>
                    <label style={{ fontSize: '12px', color: '#94a3b8', display: 'block', marginBottom: '6px' }}>Content Payload</label>
                    <textarea
                      rows={5}
                      value={fileContent}
                      onChange={(e) => setFileContent(e.target.value)}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', padding: '8px 12px', borderRadius: '6px', color: '#f1f5f9', fontSize: '13px', outline: 'none', fontFamily: 'monospace' }}
                    />
                  </div>

                  <button
                    onClick={handleWriteFile}
                    style={{ background: '#6366f1', border: 'none', color: '#fff', padding: '10px 18px', borderRadius: '6px', fontWeight: 500, fontSize: '13px', cursor: 'pointer' }}
                  >
                    Commit File to Sandbox
                  </button>
                </div>

                {/* Sandbox Browser Card */}
                <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '14px' }}>
                    <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc', display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <Database style={{ width: '16px', height: '16px', color: '#10b981' }} />
                      Sandbox Files ({sandboxFiles.length})
                    </h3>
                    <button onClick={refreshFiles} style={{ background: 'transparent', border: 'none', color: '#94a3b8', cursor: 'pointer' }}>
                      <RefreshCw style={{ width: '14px', height: '14px' }} />
                    </button>
                  </div>

                  <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', maxHeight: '180px', overflowY: 'auto' }}>
                    {sandboxFiles.length === 0 ? (
                      <div style={{ fontSize: '12px', color: '#64748b', textAlign: 'center', padding: '20px' }}>No files currently in sandbox</div>
                    ) : (
                      sandboxFiles.map((f) => (
                        <div
                          key={f.name}
                          onClick={() => handleReadFile(f.name)}
                          style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '8px 12px', background: 'rgba(255,255,255,0.03)', borderRadius: '6px', cursor: 'pointer', border: '1px solid rgba(255,255,255,0.05)' }}
                        >
                          <span style={{ fontSize: '13px', color: '#e2e8f0', fontFamily: 'monospace' }}>{f.name}</span>
                          <span style={{ fontSize: '11px', color: '#64748b' }}>{f.sizeBytes} bytes</span>
                        </div>
                      ))
                    )}
                  </div>

                  {readResult && (
                    <div style={{ marginTop: '16px', padding: '12px', background: 'rgba(0,0,0,0.4)', borderRadius: '6px', border: '1px solid rgba(255,255,255,0.08)' }}>
                      <div style={{ fontSize: '11px', color: '#38bdf8', marginBottom: '4px' }}>File Content Viewer:</div>
                      <pre style={{ fontSize: '12px', color: '#cbd5e1', whiteSpace: 'pre-wrap', maxHeight: '100px', overflowY: 'auto' }}>
                        {readResult}
                      </pre>
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}

          {/* TAB 3: DIAGNOSTICS & CRYPTO */}
          {activeTab === 'diagnostics' && (
            <div>
              <div style={{ marginBottom: '24px' }}>
                <h1 style={{ fontSize: '22px', fontWeight: 600, color: '#f8fafc', marginBottom: '6px' }}>
                  Diagnostics & Cryptographic Subsystem
                </h1>
                <p style={{ fontSize: '13px', color: '#94a3b8' }}>
                  Hardware telemetry, Rust runtime specs, and high-performance SHA-256 acceleration.
                </p>
              </div>

              {/* System Specs */}
              <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px', marginBottom: '20px' }}>
                <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc', marginBottom: '14px' }}>Rust Runtime Telemetry</h3>
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '12px' }}>
                  <div style={{ background: 'rgba(0,0,0,0.3)', padding: '12px', borderRadius: '8px' }}>
                    <div style={{ fontSize: '11px', color: '#64748b' }}>Target OS</div>
                    <div style={{ fontSize: '14px', fontWeight: 600, color: '#e2e8f0', marginTop: '4px' }}>{sysInfo?.os || 'Windows'}</div>
                  </div>
                  <div style={{ background: 'rgba(0,0,0,0.3)', padding: '12px', borderRadius: '8px' }}>
                    <div style={{ fontSize: '11px', color: '#64748b' }}>CPU Architecture</div>
                    <div style={{ fontSize: '14px', fontWeight: 600, color: '#e2e8f0', marginTop: '4px' }}>{sysInfo?.arch || 'x86_64'}</div>
                  </div>
                  <div style={{ background: 'rgba(0,0,0,0.3)', padding: '12px', borderRadius: '8px' }}>
                    <div style={{ fontSize: '11px', color: '#64748b' }}>Compiler Engine</div>
                    <div style={{ fontSize: '14px', fontWeight: 600, color: '#818cf8', marginTop: '4px' }}>{sysInfo?.rustVersion || 'Rust 1.99.1'}</div>
                  </div>
                  <div style={{ background: 'rgba(0,0,0,0.3)', padding: '12px', borderRadius: '8px' }}>
                    <div style={{ fontSize: '11px', color: '#64748b' }}>Active Process ID</div>
                    <div style={{ fontSize: '14px', fontWeight: 600, color: '#34d399', marginTop: '4px' }}>{sysInfo?.processId || 'PID-48291'}</div>
                  </div>
                </div>
              </div>

              {/* SHA-256 Engine */}
              <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
                <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc', marginBottom: '14px' }}>
                  Rust Native Cryptographic Hashing (SHA-256)
                </h3>
                <div style={{ display: 'flex', gap: '10px', marginBottom: '12px' }}>
                  <input
                    type="text"
                    value={hashInput}
                    onChange={(e) => setHashInput(e.target.value)}
                    style={{ flex: 1, background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', padding: '8px 12px', borderRadius: '6px', color: '#f1f5f9', fontSize: '13px' }}
                  />
                  <button
                    onClick={handleComputeHash}
                    style={{ background: '#6366f1', border: 'none', color: '#fff', padding: '8px 18px', borderRadius: '6px', cursor: 'pointer', fontWeight: 500 }}
                  >
                    Compute Hash
                  </button>
                </div>

                {hashResult && (
                  <div style={{ padding: '12px', background: 'rgba(0,0,0,0.4)', borderRadius: '6px', border: '1px solid rgba(99, 102, 241, 0.2)' }}>
                    <div style={{ fontSize: '11px', color: '#818cf8', marginBottom: '4px' }}>SHA-256 Checksum:</div>
                    <code style={{ fontSize: '13px', color: '#34d399', wordBreak: 'break-all' }}>{hashResult}</code>
                  </div>
                )}
              </div>
            </div>
          )}

          {/* TAB 4: LIVE AUDIT STREAM */}
          {activeTab === 'logs' && (
            <div>
              <div style={{ marginBottom: '20px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <div>
                  <h1 style={{ fontSize: '22px', fontWeight: 600, color: '#f8fafc', marginBottom: '4px' }}>
                    Real-time Security Audit Stream
                  </h1>
                  <p style={{ fontSize: '13px', color: '#94a3b8' }}>
                    Chronological audit ledger of all IPC calls, handshakes, and blocked threat vectors.
                  </p>
                </div>
                <button
                  onClick={() => setAuditLogs([])}
                  style={{ background: 'rgba(255,255,255,0.05)', border: '1px solid rgba(255,255,255,0.1)', padding: '6px 14px', borderRadius: '6px', color: '#94a3b8', fontSize: '12px', cursor: 'pointer' }}
                >
                  Clear Logs
                </button>
              </div>

              <div style={{ background: 'rgba(10, 12, 22, 0.9)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '16px', maxHeight: '520px', overflowY: 'auto' }}>
                {auditLogs.length === 0 ? (
                  <div style={{ textAlign: 'center', padding: '40px', color: '#64748b', fontSize: '13px' }}>
                    No events logged yet. Perform actions to see the live stream.
                  </div>
                ) : (
                  auditLogs.map((log) => (
                    <div
                      key={log.id}
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: '12px',
                        padding: '10px 12px',
                        borderBottom: '1px solid rgba(255,255,255,0.04)',
                        fontSize: '12px',
                      }}
                    >
                      <span style={{ color: '#64748b', fontFamily: 'monospace' }}>[{log.timestamp}]</span>
                      <span
                        style={{
                          fontSize: '10px',
                          fontWeight: 600,
                          padding: '2px 8px',
                          borderRadius: '4px',
                          background:
                            log.type === 'SUCCESS'
                              ? 'rgba(16, 185, 129, 0.15)'
                              : log.type === 'BLOCKED'
                              ? 'rgba(239, 68, 68, 0.2)'
                              : 'rgba(99, 102, 241, 0.15)',
                          color:
                            log.type === 'SUCCESS'
                              ? '#34d399'
                              : log.type === 'BLOCKED'
                              ? '#fca5a5'
                              : '#a5b4fc',
                        }}
                      >
                        {log.type}
                      </span>
                      <span style={{ color: '#e2e8f0', fontFamily: 'monospace', fontWeight: 500 }}>{log.command}</span>
                      <span style={{ color: '#94a3b8', marginLeft: 'auto' }}>{log.details}</span>
                    </div>
                  ))
                )}
              </div>
            </div>
          )}
        </main>
      </div>
    </div>
  );
}

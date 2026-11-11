import React, { useState } from 'react';
import { ShieldAlert, Wifi, Key, Binary, RefreshCw, CheckCircle2, AlertTriangle, ShieldCheck } from 'lucide-react';
import { aegis } from './aegis-bridge';
import { translations, Language } from './i18n';

interface AdvancedSecuritySuiteProps {
  lang: Language;
  onAddLog: (type: 'SUCCESS' | 'BLOCKED' | 'INFO', command: string, details: string) => void;
  setSrAnnouncement: (announcement: string) => void;
}

export const AdvancedSecuritySuite: React.FC<AdvancedSecuritySuiteProps> = ({
  lang,
  onAddLog,
  setSrAnnouncement,
}) => {
  const t = translations[lang];
  const [subTab, setSubTab] = useState<'rasp' | 'pinning' | 'fido2' | 'p2p'>('rasp');

  // RASP State
  const [raspReport, setRaspReport] = useState<any>(null);
  const [raspLoading, setRaspLoading] = useState(false);

  // mTLS Pinning State
  const [pinningHost, setPinningHost] = useState('api.aegis.bank.internal');
  const [pinningReport, setPinningReport] = useState<any>(null);
  const [pinningLoading, setPinningLoading] = useState(false);

  // FIDO2 Hardware Key State
  const [fido2Devices, setFido2Devices] = useState<any[]>([]);
  const [fido2Cred, setFido2Cred] = useState<any>(null);
  const [fido2Assertion, setFido2Assertion] = useState<any>(null);
  const [fido2TouchConfirmed, setFido2TouchConfirmed] = useState(true);
  const [fido2Loading, setFido2Loading] = useState(false);

  // P2P Sync State
  const [p2pInit, setP2pInit] = useState<any>(null);
  const [p2pResp, setP2pResp] = useState<any>(null);
  const [p2pBundle, setP2pBundle] = useState<any>(null);
  const [p2pDecrypted, setP2pDecrypted] = useState<any>(null);
  const [p2pPayload, setP2pPayload] = useState('CONFIDENTIAL_LEDGER_SYNC_RECORD_V1');
  const [p2pLoading, setP2pLoading] = useState(false);

  // ── RASP Handlers ──────────────────────────────────────────────────────────
  const handleRunRaspScan = async () => {
    setRaspLoading(true);
    const res = await aegis.evaluateRasp();
    if (res.success && res.data) {
      setRaspReport(res.data);
      onAddLog('SUCCESS', 'rasp_evaluate', `RASP Tier: ${res.data.system_integrity_tier} | Debugger: ${res.data.debugger_detected ? 'ACTIVE' : 'CLEAN'}`);
      setSrAnnouncement(`RASP scan completed: ${res.data.system_integrity_tier}`);
    } else {
      onAddLog('BLOCKED', 'rasp_evaluate', res.error || 'Failed to evaluate RASP');
    }
    setRaspLoading(false);
  };

  const handleResetRaspContainment = async () => {
    setRaspLoading(true);
    const res = await aegis.resetRaspContainment();
    if (res.success) {
      onAddLog('INFO', 'rasp_reset', 'RASP containment state reset');
      await handleRunRaspScan();
    }
    setRaspLoading(false);
  };

  // ── mTLS & Pinning Handlers ────────────────────────────────────────────────
  const handleEvaluatePinning = async () => {
    setPinningLoading(true);
    const demoCertHex = '3082010a0282010100a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f90';
    const res = await aegis.evaluateNetworkPinning(pinningHost, demoCertHex, true);
    if (res.success && res.data) {
      setPinningReport(res.data);
      onAddLog(res.data.pin_verified ? 'SUCCESS' : 'BLOCKED', 'mtls_pinning', `Pin Verified: ${res.data.pin_verified} | MITM: ${res.data.mitm_interception_detected}`);
    } else {
      onAddLog('BLOCKED', 'mtls_pinning', res.error || 'Pinning evaluation failed');
    }
    setPinningLoading(false);
  };

  // ── FIDO2 / YubiKey Handlers ───────────────────────────────────────────────
  const handleFido2Enumerate = async () => {
    setFido2Loading(true);
    const res = await aegis.enumerateFido2Tokens();
    if (res.success && res.data) {
      setFido2Devices(res.data.devices);
      onAddLog('INFO', 'fido2_enumerate', `Found ${res.data.count} hardware tokens`);
    }
    setFido2Loading(false);
  };

  const handleFido2Register = async () => {
    setFido2Loading(true);
    const res = await aegis.registerFido2Credential('operator.alice@aegis.bank');
    if (res.success && res.data) {
      setFido2Cred(res.data);
      onAddLog('SUCCESS', 'fido2_register', `Credential enrolled: ${res.data.credential_id.substring(0, 16)}...`);
    } else {
      onAddLog('BLOCKED', 'fido2_register', res.error || 'Registration failed');
    }
    setFido2Loading(false);
  };

  const handleFido2Sign = async () => {
    if (!fido2Cred) return;
    setFido2Loading(true);
    const clientHash = 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855';
    const res = await aegis.signWithFido2Token(fido2Cred.credential_id, clientHash, fido2TouchConfirmed);
    if (res.success && res.data) {
      setFido2Assertion(res.data);
      onAddLog('SUCCESS', 'fido2_sign', `FIDO2 assertion signed (Touch Present: ${res.data.user_present})`);
    } else {
      onAddLog('BLOCKED', 'fido2_sign', res.error || 'Signing denied');
    }
    setFido2Loading(false);
  };

  // ── P2P Sync Handlers ─────────────────────────────────────────────────────
  const handleP2pInit = async () => {
    setP2pLoading(true);
    const res = await aegis.initiateP2pSync();
    if (res.success && res.data) {
      setP2pInit(res.data);
      onAddLog('INFO', 'p2p_init', `Handshake initiated: ${res.data.session_id}`);
    }
    setP2pLoading(false);
  };

  const handleP2pComplete = async () => {
    if (!p2pInit) return;
    setP2pLoading(true);
    const responderPub = '11223344556677889900aabbccddeeff11223344556677889900aabbccddeeff';
    const res = await aegis.completeP2pSync(p2pInit.session_id, p2pInit.initiator_public_hex, responderPub);
    if (res.success && res.data) {
      setP2pResp(res.data);
      onAddLog('SUCCESS', 'p2p_complete', `P2P Session established! Mutual Tag: ${res.data.mutual_auth_tag_hex.substring(0, 16)}...`);
    }
    setP2pLoading(false);
  };

  const handleP2pEncrypt = async () => {
    if (!p2pInit) return;
    setP2pLoading(true);
    const hex = Array.from(new TextEncoder().encode(p2pPayload)).map((b) => b.toString(16).padStart(2, '0')).join('');
    const res = await aegis.encryptP2pBundle(p2pInit.session_id, 1, hex);
    if (res.success && res.data) {
      setP2pBundle(res.data);
      onAddLog('SUCCESS', 'p2p_encrypt', `Encrypted differential bundle seq #1`);
    }
    setP2pLoading(false);
  };

  const handleP2pDecrypt = async () => {
    if (!p2pBundle) return;
    setP2pLoading(true);
    const res = await aegis.decryptP2pBundle(p2pBundle);
    if (res.success && res.data) {
      setP2pDecrypted(res.data);
      onAddLog('SUCCESS', 'p2p_decrypt', `Decrypted & MAC verified: "${res.data.plaintextUtf8}"`);
    } else {
      onAddLog('BLOCKED', 'p2p_decrypt', res.error || 'Decryption failed');
    }
    setP2pLoading(false);
  };

  return (
    <div>
      {/* Header */}
      <div style={{ marginBottom: '24px' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: '#f59e0b', fontSize: '12px', fontWeight: 600, textTransform: 'uppercase', letterSpacing: '1px' }}>
          <ShieldAlert style={{ width: '14px', height: '14px' }} />
          Zero-Trust Advanced Security Enclaves
        </div>
        <h1 style={{ fontSize: '24px', fontWeight: 700, color: '#f8fafc', margin: '4px 0 8px' }}>
          {lang === 'si' ? 'උසස් ආරක්ෂණ හා තර්ජන නිවාරණ පද්ධතිය' : 'Advanced Threat Defense & Hardware Enclaves'}
        </h1>
        <p style={{ fontSize: '13px', color: '#94a3b8', margin: 0, maxWidth: '800px' }}>
          {lang === 'si'
            ? 'Win32 RASP ස්වයං-ආරක්ෂණය, mTLS SPKI සහතික Pinning, FIDO2 දෘඩාංග යතුරු සහ P2P Zero-Knowledge Noise සේප්පු සමමුහුර්තය.'
            : 'Ring-0 Win32 RASP anti-tampering, mTLS SPKI certificate pinning, FIDO2 CTAP2 physical hardware key verification, and decentralized Zero-Knowledge Noise sync.'}
        </p>
      </div>

      {/* Sub Tabs */}
      <div style={{ display: 'flex', gap: '8px', marginBottom: '24px', borderBottom: '1px solid rgba(255,255,255,0.08)', paddingBottom: '12px' }}>
        <button
          onClick={() => { setSubTab('rasp'); if (!raspReport) handleRunRaspScan(); }}
          style={{
            display: 'flex', alignItems: 'center', gap: '8px', padding: '8px 16px', borderRadius: '8px', border: 'none',
            background: subTab === 'rasp' ? 'rgba(239, 68, 68, 0.2)' : 'rgba(255,255,255,0.04)',
            color: subTab === 'rasp' ? '#fca5a5' : '#94a3b8',
            cursor: 'pointer', fontWeight: 600, fontSize: '12px',
          }}
        >
          <ShieldAlert style={{ width: '14px', height: '14px', color: '#ef4444' }} />
          {t.tabs.rasp}
        </button>

        <button
          onClick={() => { setSubTab('pinning'); if (!pinningReport) handleEvaluatePinning(); }}
          style={{
            display: 'flex', alignItems: 'center', gap: '8px', padding: '8px 16px', borderRadius: '8px', border: 'none',
            background: subTab === 'pinning' ? 'rgba(56, 189, 248, 0.2)' : 'rgba(255,255,255,0.04)',
            color: subTab === 'pinning' ? '#7dd3fc' : '#94a3b8',
            cursor: 'pointer', fontWeight: 600, fontSize: '12px',
          }}
        >
          <Wifi style={{ width: '14px', height: '14px', color: '#38bdf8' }} />
          {t.tabs.network}
        </button>

        <button
          onClick={() => { setSubTab('fido2'); if (fido2Devices.length === 0) handleFido2Enumerate(); }}
          style={{
            display: 'flex', alignItems: 'center', gap: '8px', padding: '8px 16px', borderRadius: '8px', border: 'none',
            background: subTab === 'fido2' ? 'rgba(245, 158, 11, 0.2)' : 'rgba(255,255,255,0.04)',
            color: subTab === 'fido2' ? '#fcd34d' : '#94a3b8',
            cursor: 'pointer', fontWeight: 600, fontSize: '12px',
          }}
        >
          <Key style={{ width: '14px', height: '14px', color: '#f59e0b' }} />
          {t.tabs.fido2}
        </button>

        <button
          onClick={() => { setSubTab('p2p'); if (!p2pInit) handleP2pInit(); }}
          style={{
            display: 'flex', alignItems: 'center', gap: '8px', padding: '8px 16px', borderRadius: '8px', border: 'none',
            background: subTab === 'p2p' ? 'rgba(168, 85, 247, 0.2)' : 'rgba(255,255,255,0.04)',
            color: subTab === 'p2p' ? '#d8b4fe' : '#94a3b8',
            cursor: 'pointer', fontWeight: 600, fontSize: '12px',
          }}
        >
          <Binary style={{ width: '14px', height: '14px', color: '#a855f7' }} />
          {t.tabs.p2p}
        </button>
      </div>

      {/* SUB-TAB 1: RASP */}
      {subTab === 'rasp' && (
        <div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(240px, 1fr))', gap: '16px', marginBottom: '20px' }}>
            <div style={{ background: 'rgba(15, 23, 42, 0.7)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '16px' }}>
              <div style={{ fontSize: '11px', color: '#94a3b8', textTransform: 'uppercase', marginBottom: '6px' }}>{t.rasp.integrityTier}</div>
              <div style={{ fontSize: '18px', fontWeight: 800, color: raspReport?.system_integrity_tier === 'COMPROMISED_CONTAINED' ? '#ef4444' : '#10b981' }}>
                {raspReport?.system_integrity_tier || 'SECURE (UNSCANNED)'}
              </div>
              <div style={{ fontSize: '11px', color: '#64748b', marginTop: '4px' }}>Win32 Ring-0 Protection</div>
            </div>

            <div style={{ background: 'rgba(15, 23, 42, 0.7)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '16px' }}>
              <div style={{ fontSize: '11px', color: '#94a3b8', textTransform: 'uppercase', marginBottom: '6px' }}>{t.rasp.debuggerStatus}</div>
              <div style={{ fontSize: '15px', fontWeight: 700, color: raspReport?.debugger_detected ? '#ef4444' : '#10b981' }}>
                {raspReport ? (raspReport.debugger_detected ? t.rasp.detected : t.rasp.clean) : 'Ready'}
              </div>
              <div style={{ fontSize: '11px', color: '#64748b', marginTop: '4px' }}>IsDebuggerPresent / RemoteCheck</div>
            </div>

            <div style={{ background: 'rgba(15, 23, 42, 0.7)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '16px' }}>
              <div style={{ fontSize: '11px', color: '#94a3b8', textTransform: 'uppercase', marginBottom: '6px' }}>{t.rasp.memoryHooks}</div>
              <div style={{ fontSize: '15px', fontWeight: 700, color: raspReport?.memory_hooks_detected ? '#ef4444' : '#10b981' }}>
                {raspReport ? (raspReport.memory_hooks_detected ? t.rasp.detected : t.rasp.clean) : 'Ready'}
              </div>
              <div style={{ fontSize: '11px', color: '#64748b', marginTop: '4px' }}>VirtualProtect Opcode Verification</div>
            </div>
          </div>

          <div style={{ display: 'flex', gap: '12px', marginBottom: '20px' }}>
            <button
              onClick={handleRunRaspScan}
              disabled={raspLoading}
              style={{
                background: 'linear-gradient(135deg, #ef4444, #b91c1c)',
                border: 'none', color: '#fff', padding: '10px 18px', borderRadius: '8px', fontWeight: 600, fontSize: '12px', cursor: 'pointer', display: 'flex', alignItems: 'center', gap: '8px',
              }}
            >
              <RefreshCw style={{ width: '14px', height: '14px', animation: raspLoading ? 'spin 1s linear infinite' : 'none' }} />
              {t.rasp.runScan}
            </button>
            <button
              onClick={handleResetRaspContainment}
              disabled={raspLoading}
              style={{
                background: 'rgba(255,255,255,0.08)', border: '1px solid rgba(255,255,255,0.15)', color: '#cbd5e1', padding: '10px 18px', borderRadius: '8px', fontWeight: 500, fontSize: '12px', cursor: 'pointer',
              }}
            >
              {t.rasp.resetContainment}
            </button>
          </div>
        </div>
      )}

      {/* SUB-TAB 2: mTLS PINNING */}
      {subTab === 'pinning' && (
        <div>
          <div style={{ background: 'rgba(15, 23, 42, 0.7)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '18px', marginBottom: '20px' }}>
            <label style={{ display: 'block', fontSize: '12px', color: '#94a3b8', marginBottom: '8px' }}>Target Gateway Host</label>
            <div style={{ display: 'flex', gap: '12px' }}>
              <input
                type="text"
                value={pinningHost}
                onChange={(e) => setPinningHost(e.target.value)}
                style={{ flex: 1, background: 'rgba(0,0,0,0.4)', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '8px', padding: '8px 12px', color: '#fff', fontSize: '13px', fontFamily: 'monospace' }}
              />
              <button
                onClick={handleEvaluatePinning}
                disabled={pinningLoading}
                style={{ background: 'linear-gradient(135deg, #0284c7, #0369a1)', border: 'none', color: '#fff', padding: '8px 18px', borderRadius: '8px', fontWeight: 600, fontSize: '12px', cursor: 'pointer' }}
              >
                {t.network.evaluate}
              </button>
            </div>
          </div>

          {pinningReport && (
            <div style={{ background: 'rgba(15, 23, 42, 0.7)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '18px' }}>
              <div style={{ fontSize: '15px', fontWeight: 700, color: pinningReport.pin_verified ? '#38bdf8' : '#ef4444', marginBottom: '10px' }}>
                {pinningReport.status}
              </div>
              <div style={{ fontSize: '12px', color: '#cbd5e1', lineHeight: '1.8', fontFamily: 'monospace' }}>
                <div>• Observed SPKI Hash: <span style={{ color: '#38bdf8' }}>{pinningReport.observed_spki_pin}</span></div>
                <div>• MITM Proxy Interception: <span style={{ color: pinningReport.mitm_interception_detected ? '#ef4444' : '#10b981', fontWeight: 700 }}>{pinningReport.mitm_interception_detected ? 'DETECTED' : 'CLEAR'}</span></div>
                <div>• mTLS Client Status: <span style={{ color: '#10b981', fontWeight: 700 }}>AUTHENTICATED</span></div>
              </div>
            </div>
          )}
        </div>
      )}

      {/* SUB-TAB 3: FIDO2 */}
      {subTab === 'fido2' && (
        <div>
          <div style={{ display: 'flex', gap: '12px', marginBottom: '20px' }}>
            <button
              onClick={handleFido2Enumerate}
              disabled={fido2Loading}
              style={{ background: 'rgba(255,255,255,0.08)', border: '1px solid rgba(255,255,255,0.15)', color: '#f59e0b', padding: '8px 16px', borderRadius: '8px', fontWeight: 600, fontSize: '12px', cursor: 'pointer' }}
            >
              {t.fido2.devicesFound}
            </button>
            <button
              onClick={handleFido2Register}
              disabled={fido2Loading}
              style={{ background: 'linear-gradient(135deg, #d97706, #b45309)', border: 'none', color: '#fff', padding: '8px 18px', borderRadius: '8px', fontWeight: 600, fontSize: '12px', cursor: 'pointer' }}
            >
              {t.fido2.registerKey}
            </button>
          </div>

          <div style={{ background: 'rgba(15, 23, 42, 0.7)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '18px', marginBottom: '20px' }}>
            <div style={{ fontSize: '13px', fontWeight: 600, color: '#f8fafc', marginBottom: '10px' }}>Detected Hardware Keys</div>
            {fido2Devices.length > 0 ? (
              fido2Devices.map((d, i) => (
                <div key={i} style={{ background: 'rgba(0,0,0,0.3)', padding: '10px', borderRadius: '8px', fontSize: '12px', color: '#f59e0b', display: 'flex', justifyContent: 'space-between' }}>
                  <span>{d.product_name} (SN: {d.serial_number})</span>
                  <span style={{ color: '#10b981' }}>ONLINE</span>
                </div>
              ))
            ) : (
              <div style={{ fontSize: '12px', color: '#64748b' }}>No active hardware keys probed yet.</div>
            )}
          </div>

          {fido2Cred && (
            <div style={{ background: 'rgba(15, 23, 42, 0.7)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '18px' }}>
              <div style={{ fontSize: '13px', fontWeight: 600, color: '#f8fafc', marginBottom: '10px' }}>Sign Challenge with FIDO2 Token</div>
              <label style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '12px', color: '#94a3b8', marginBottom: '12px', cursor: 'pointer' }}>
                <input
                  type="checkbox"
                  checked={fido2TouchConfirmed}
                  onChange={(e) => setFido2TouchConfirmed(e.target.checked)}
                />
                {t.fido2.touchConfirmed} (Simulate Fingerprint / Touch)
              </label>
              <button
                onClick={handleFido2Sign}
                disabled={fido2Loading}
                style={{ background: 'linear-gradient(135deg, #10b981, #059669)', border: 'none', color: '#fff', padding: '8px 16px', borderRadius: '8px', fontWeight: 600, fontSize: '12px', cursor: 'pointer' }}
              >
                {t.fido2.signAssertion}
              </button>

              {fido2Assertion && (
                <div style={{ marginTop: '12px', padding: '10px', background: 'rgba(0,0,0,0.3)', borderRadius: '6px', fontSize: '11px', fontFamily: 'monospace', color: '#10b981' }}>
                  ✅ Ed25519 Hardware Signature: {fido2Assertion.signature_hex.substring(0, 36)}...
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {/* SUB-TAB 4: P2P SYNC */}
      {subTab === 'p2p' && (
        <div>
          <div style={{ background: 'rgba(15, 23, 42, 0.7)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '18px', marginBottom: '20px' }}>
            <div style={{ fontSize: '13px', fontWeight: 600, color: '#f8fafc', marginBottom: '10px' }}>Noise Handshake</div>
            <div style={{ display: 'flex', gap: '10px', marginBottom: '12px' }}>
              <button
                onClick={handleP2pInit}
                disabled={p2pLoading}
                style={{ background: 'linear-gradient(135deg, #a855f7, #7e22ce)', border: 'none', color: '#fff', padding: '8px 16px', borderRadius: '8px', fontWeight: 600, fontSize: '12px', cursor: 'pointer' }}
              >
                {t.p2p.initSession}
              </button>
              {p2pInit && (
                <button
                  onClick={handleP2pComplete}
                  disabled={p2pLoading}
                  style={{ background: 'rgba(255,255,255,0.08)', border: '1px solid rgba(255,255,255,0.15)', color: '#d8b4fe', padding: '8px 16px', borderRadius: '8px', fontWeight: 600, fontSize: '12px', cursor: 'pointer' }}
                >
                  {t.p2p.completeHandshake}
                </button>
              )}
            </div>

            {p2pInit && (
              <div style={{ fontSize: '11px', color: '#cbd5e1', fontFamily: 'monospace' }}>
                Session: <span style={{ color: '#a855f7' }}>{p2pInit.session_id}</span>
                {p2pResp && <div style={{ color: '#10b981', marginTop: '4px' }}>✅ Authenticated Noise Channel Established</div>}
              </div>
            )}
          </div>

          {p2pResp && (
            <div style={{ background: 'rgba(15, 23, 42, 0.7)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '18px' }}>
              <div style={{ fontSize: '13px', fontWeight: 600, color: '#f8fafc', marginBottom: '8px' }}>Differential Sync Stream</div>
              <input
                type="text"
                value={p2pPayload}
                onChange={(e) => setP2pPayload(e.target.value)}
                style={{ width: '100%', background: 'rgba(0,0,0,0.4)', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '8px', padding: '8px 12px', color: '#fff', fontSize: '12px', fontFamily: 'monospace', marginBottom: '12px' }}
              />
              <div style={{ display: 'flex', gap: '10px', marginBottom: '12px' }}>
                <button
                  onClick={handleP2pEncrypt}
                  disabled={p2pLoading}
                  style={{ background: 'linear-gradient(135deg, #059669, #047857)', border: 'none', color: '#fff', padding: '8px 16px', borderRadius: '8px', fontWeight: 600, fontSize: '12px', cursor: 'pointer' }}
                >
                  {t.p2p.encryptVault}
                </button>
                {p2pBundle && (
                  <button
                    onClick={handleP2pDecrypt}
                    disabled={p2pLoading}
                    style={{ background: 'linear-gradient(135deg, #2563eb, #1d4ed8)', border: 'none', color: '#fff', padding: '8px 16px', borderRadius: '8px', fontWeight: 600, fontSize: '12px', cursor: 'pointer' }}
                  >
                    {t.p2p.decryptVault}
                  </button>
                )}
              </div>

              {p2pDecrypted && (
                <div style={{ padding: '10px', background: 'rgba(16, 185, 129, 0.15)', borderRadius: '6px', color: '#6ee7b7', fontSize: '12px' }}>
                  ✅ Decrypted: "{p2pDecrypted.plaintextUtf8}" (MAC Tag Verified)
                </div>
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
};

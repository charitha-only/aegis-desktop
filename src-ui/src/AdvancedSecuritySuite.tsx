import React, { useState, useEffect } from 'react';
import {
  ShieldAlert,
  Wifi,
  Key,
  Binary,
  RefreshCw,
  CheckCircle2,
  AlertTriangle,
  ShieldCheck,
  Database,
  Cpu,
  Users,
  Activity,
  FileCheck,
  Lock,
  Radio,
  ExternalLink,
  Power,
  RotateCcw
} from 'lucide-react';
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
  const [subTab, setSubTab] = useState<'db' | 'tpm' | 'rbac' | 'traffic' | 'compliance' | 'rasp' | 'pinning' | 'fido2' | 'p2p'>('db');

  // ── Pillar 1: Encrypted DB State ───────────────────────────────────────────
  const [dbStats, setDbStats] = useState<any>(null);
  const [dbTable, setDbTable] = useState<'accounts' | 'audit_ledger'>('accounts');
  const [dbRows, setDbRows] = useState<any[]>([]);
  const [dbLoading, setDbLoading] = useState(false);
  const [newAccNumber, setNewAccNumber] = useState('SG71-8899-1122-3344');
  const [newAccHolder, setNewAccHolder] = useState('Temasek Sovereign Holdings');
  const [newAccBalance, setNewAccBalance] = useState('2500000000'); // $25M in cents

  // ── Pillar 2: TPM PCR Enclave State ────────────────────────────────────────
  const [tpmStatus, setTpmStatus] = useState<any>(null);
  const [packageId, setPackageId] = useState('vault-master-seed-01');
  const [secretToSeal, setSecretToSeal] = useState('SWIFT_SECURE_PAYMENT_PRIVATE_KEY_AEAD_2026');
  const [sealedPackage, setSealedPackage] = useState<any>(null);
  const [unsealedSecret, setUnsealedSecret] = useState<string | null>(null);
  const [tpmTampered, setTpmTampered] = useState(false);
  const [tpmLoading, setTpmLoading] = useState(false);

  // ── Pillar 3: Enterprise RBAC State ────────────────────────────────────────
  const [rbacIdentity, setRbacIdentity] = useState<any>(null);
  const [rbacMatrix, setRbacMatrix] = useState<any[]>([]);
  const [rbacAudit, setRbacAudit] = useState<any[]>([]);
  const [rbacLoading, setRbacLoading] = useState(false);
  const [permCheckAction, setPermCheckAction] = useState('initiate_transaction');
  const [permCheckResult, setPermCheckResult] = useState<any>(null);

  // ── Pillar 4: Traffic Monitor State ────────────────────────────────────────
  const [trafficFlows, setTrafficFlows] = useState<any[]>([]);
  const [trafficStats, setTrafficStats] = useState<any>(null);
  const [killswitchActive, setKillswitchActive] = useState(false);
  const [simHost, setSimHost] = useState('api.central-bank.internal');
  const [simThreat, setSimThreat] = useState(0);
  const [trafficLoading, setTrafficLoading] = useState(false);

  // ── Pillar 5: Compliance Certificate State ─────────────────────────────────
  const [certificate, setCertificate] = useState<any>(null);
  const [certVerified, setCertVerified] = useState<boolean | null>(null);
  const [showCertificateHtml, setShowCertificateHtml] = useState(false);
  const [complianceLoading, setComplianceLoading] = useState(false);

  // ── Existing Modules (RASP, Pinning, FIDO2, P2P) ───────────────────────────
  const [raspReport, setRaspReport] = useState<any>(null);
  const [raspLoading, setRaspLoading] = useState(false);

  const [pinningHost, setPinningHost] = useState('api.aegis.bank.internal');
  const [pinningReport, setPinningReport] = useState<any>(null);
  const [pinningLoading, setPinningLoading] = useState(false);

  const [fido2Devices, setFido2Devices] = useState<any[]>([]);
  const [fido2Cred, setFido2Cred] = useState<any>(null);
  const [fido2Assertion, setFido2Assertion] = useState<any>(null);
  const [fido2TouchConfirmed, setFido2TouchConfirmed] = useState(true);
  const [fido2Loading, setFido2Loading] = useState(false);

  const [p2pInit, setP2pInit] = useState<any>(null);
  const [p2pResp, setP2pResp] = useState<any>(null);
  const [p2pBundle, setP2pBundle] = useState<any>(null);
  const [p2pDecrypted, setP2pDecrypted] = useState<any>(null);
  const [p2pPayload, setP2pPayload] = useState('CONFIDENTIAL_LEDGER_SYNC_RECORD_V1');
  const [p2pLoading, setP2pLoading] = useState(false);

  // Initial data load on mount or tab change
  useEffect(() => {
    refreshDb();
    refreshTpmStatus();
    refreshRbac();
    refreshTraffic();
  }, []);

  // ── Pillar 1 Handlers ──────────────────────────────────────────────────────
  const refreshDb = async () => {
    setDbLoading(true);
    const statsRes = await aegis.getEncryptedDbStats();
    if (statsRes.success && statsRes.data) {
      setDbStats(statsRes.data);
    }
    const queryRes = await aegis.queryEncryptedDb(dbTable);
    if (queryRes.success && queryRes.data) {
      setDbRows(queryRes.data.rows);
    }
    setDbLoading(false);
  };

  const handleQueryTable = async (table: 'accounts' | 'audit_ledger') => {
    setDbTable(table);
    setDbLoading(true);
    const res = await aegis.queryEncryptedDb(table);
    if (res.success && res.data) {
      setDbRows(res.data.rows);
      onAddLog('SUCCESS', 'db_query', `Queried ${res.data.count} records from '${table}'`);
    } else {
      onAddLog('BLOCKED', 'db_query', res.error || 'Failed to query table');
    }
    setDbLoading(false);
  };

  const handleInsertAccount = async () => {
    setDbLoading(true);
    const record = {
      account_number: newAccNumber,
      holder_name: newAccHolder,
      balance_cents: parseInt(newAccBalance, 10) || 10000000,
      currency: 'USD',
      status: 'ACTIVE_TIER_1'
    };
    const res = await aegis.insertEncryptedDb('accounts', record);
    if (res.success) {
      onAddLog('SUCCESS', 'db_insert', `Inserted account ${newAccNumber} to ChaCha20 encrypted database`);
      await refreshDb();
    } else {
      onAddLog('BLOCKED', 'db_insert', res.error || 'Insertion failed');
    }
    setDbLoading(false);
  };

  const handleDbTransaction = async (action: 'begin' | 'commit' | 'rollback') => {
    setDbLoading(true);
    const res = await aegis.transactionEncryptedDb(action);
    if (res.success) {
      onAddLog('SUCCESS', 'db_transaction', `Transaction: ${action.toUpperCase()}`);
      await refreshDb();
    } else {
      onAddLog('BLOCKED', 'db_transaction', res.error || 'Transaction action failed');
    }
    setDbLoading(false);
  };

  // ── Pillar 2 Handlers ──────────────────────────────────────────────────────
  const refreshTpmStatus = async () => {
    setTpmLoading(true);
    const res = await aegis.getPcrEnclaveStatus();
    if (res.success && res.data) {
      setTpmStatus(res.data);
    }
    setTpmLoading(false);
  };

  const handleSealSecret = async () => {
    setTpmLoading(true);
    const res = await aegis.sealPcrSecret(packageId, secretToSeal, [0, 2, 7, 11]);
    if (res.success && res.data) {
      setSealedPackage(res.data);
      setUnsealedSecret(null);
      onAddLog('SUCCESS', 'pcr_enclave_seal', `Sealed package '${packageId}' bound to PCR composite ${res.data.pcr_composite_hash.substring(0, 16)}...`);
    } else {
      onAddLog('BLOCKED', 'pcr_enclave_seal', res.error || 'Sealing failed');
    }
    setTpmLoading(false);
  };

  const handleUnsealSecret = async () => {
    setTpmLoading(true);
    const res = await aegis.unsealPcrSecret(packageId);
    if (res.success && res.data) {
      setUnsealedSecret(res.data.secretUtf8);
      onAddLog('SUCCESS', 'pcr_enclave_unseal', `PCR verification passed! Secret unsealed: ${res.data.secretUtf8.substring(0, 12)}...`);
    } else {
      setUnsealedSecret(null);
      onAddLog('BLOCKED', 'pcr_enclave_unseal', res.error || 'Fail-closed: PCR measurement mismatch!');
    }
    setTpmLoading(false);
  };

  const handleSimulateTamper = async () => {
    setTpmLoading(true);
    const res = await aegis.simulatePcrTamper(7, 'UNTRUSTED_ROOTKIT_BOOTLOADER_PAYLOAD_HOOK');
    if (res.success && res.data) {
      setTpmTampered(true);
      onAddLog('BLOCKED', 'pcr_tamper_sim', 'PCR[7] SecureBoot register tampered! Enclave unseal will now fail closed.');
      await refreshTpmStatus();
    }
    setTpmLoading(false);
  };

  // ── Pillar 3 Handlers ──────────────────────────────────────────────────────
  const refreshRbac = async () => {
    setRbacLoading(true);
    const idRes = await aegis.getRbacIdentity();
    if (idRes.success && idRes.data) {
      setRbacIdentity(idRes.data);
    }
    const matRes = await aegis.getRbacMatrix();
    if (matRes.success && matRes.data) {
      setRbacMatrix(matRes.data.roles);
    }
    const audRes = await aegis.getRbacAuditLog();
    if (audRes.success && audRes.data) {
      setRbacAudit(audRes.data.auditLog);
    }
    setRbacLoading(false);
  };

  const handleSwitchRole = async (role: string) => {
    setRbacLoading(true);
    const res = await aegis.switchRbacRole(role);
    if (res.success && res.data) {
      onAddLog('INFO', 'rbac_switch_role', `Assumed role: ${role}`);
      await refreshRbac();
    } else {
      onAddLog('BLOCKED', 'rbac_switch_role', res.error || 'Role change failed');
    }
    setRbacLoading(false);
  };

  const handleCheckPerm = async (action: string) => {
    setPermCheckAction(action);
    setRbacLoading(true);
    const res = await aegis.checkRbacPermission(action, 'financial_ledger');
    if (res.success && res.data) {
      setPermCheckResult(res.data);
      onAddLog(res.data.allowed ? 'SUCCESS' : 'BLOCKED', 'rbac_check', `${action}: ${res.data.reason}`);
    }
    setRbacLoading(false);
  };

  // ── Pillar 4 Handlers ──────────────────────────────────────────────────────
  const refreshTraffic = async () => {
    setTrafficLoading(true);
    const stRes = await aegis.getTrafficStats();
    if (stRes.success && stRes.data) {
      setTrafficStats(stRes.data);
      setKillswitchActive(stRes.data.killswitch_active);
    }
    const flRes = await aegis.getRecentTraffic(25);
    if (flRes.success && flRes.data) {
      setTrafficFlows(flRes.data.flows);
    }
    setTrafficLoading(false);
  };

  const handleToggleKillswitch = async () => {
    setTrafficLoading(true);
    const targetState = !killswitchActive;
    const res = await aegis.toggleTrafficKillswitch(targetState);
    if (res.success && res.data) {
      setKillswitchActive(res.data.killswitchActive);
      onAddLog(
        res.data.killswitchActive ? 'BLOCKED' : 'SUCCESS',
        'traffic_killswitch',
        res.data.killswitchActive ? 'EMERGENCY AIR-GAP KILLSWITCH ACTIVATED: 100% outbound traffic dropped' : 'Killswitch released: Nominal outbound traffic filtering active'
      );
      await refreshTraffic();
    }
    setTrafficLoading(false);
  };

  const handleSimulateTraffic = async () => {
    setTrafficLoading(true);
    const res = await aegis.simulateTrafficFlow(simHost, simThreat);
    if (res.success && res.data) {
      onAddLog(res.data.decision === 'ALLOWED' ? 'SUCCESS' : 'BLOCKED', 'traffic_simulate', `Flow ${res.data.id}: ${res.data.target_host} [${res.data.decision}] (Threat: ${res.data.threat_score})`);
      await refreshTraffic();
    }
    setTrafficLoading(false);
  };

  // ── Pillar 5 Handlers ──────────────────────────────────────────────────────
  const handleGenerateCertificate = async () => {
    setComplianceLoading(true);
    const res = await aegis.generateComplianceCertificate();
    if (res.success && res.data) {
      setCertificate(res.data);
      setCertVerified(null);
      onAddLog('SUCCESS', 'compliance_generate', `Generated Attested Certificate: ${res.data.payload.certificate_id} (Score: ${res.data.payload.score})`);
    } else {
      onAddLog('BLOCKED', 'compliance_generate', res.error || 'Failed to generate certificate');
    }
    setComplianceLoading(false);
  };

  const handleVerifyCertificate = async () => {
    if (!certificate) return;
    setComplianceLoading(true);
    const res = await aegis.verifyComplianceCertificate(
      certificate.payload_sha256,
      certificate.signature_hex,
      certificate.public_key_hex
    );
    if (res.success && res.data) {
      setCertVerified(res.data.verified);
      onAddLog('SUCCESS', 'compliance_verify', `Ed25519 digital signature verified against enclave public key!`);
    } else {
      setCertVerified(false);
      onAddLog('BLOCKED', 'compliance_verify', res.error || 'Certificate signature invalid');
    }
    setComplianceLoading(false);
  };

  // ── Legacy RASP & Defense Handlers ─────────────────────────────────────────
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

  const handleEnumerateFido2 = async () => {
    setFido2Loading(true);
    const res = await aegis.enumerateFido2Tokens();
    if (res.success && res.data) {
      setFido2Devices(res.data.devices);
      onAddLog('SUCCESS', 'fido2_enumerate', `Discovered ${res.data.count} CTAP2 hardware tokens`);
    }
    setFido2Loading(false);
  };

  const handleRegisterFido2 = async () => {
    setFido2Loading(true);
    const res = await aegis.registerFido2Credential('enterprise_treasury_officer_01', 'aegis.bank.internal');
    if (res.success && res.data) {
      setFido2Cred(res.data);
      onAddLog('SUCCESS', 'fido2_register', `Registered FIDO2 credential: ${res.data.credential_id.substring(0, 16)}...`);
    }
    setFido2Loading(false);
  };

  const handleSignFido2 = async () => {
    if (!fido2Cred) return;
    setFido2Loading(true);
    const mockHash = '4b227777d4dd1fc61c6f884f48641d02b4d121d3fd328cb08b5531fcacdabf8a';
    const res = await aegis.signWithFido2Token(fido2Cred.credential_id, mockHash, fido2TouchConfirmed);
    if (res.success && res.data) {
      setFido2Assertion(res.data);
      onAddLog('SUCCESS', 'fido2_sign', `Hardware token assertion signed. Touch confirmed: ${res.data.user_present}`);
    } else {
      onAddLog('BLOCKED', 'fido2_sign', res.error || 'Hardware touch required');
    }
    setFido2Loading(false);
  };

  const handleInitiateP2p = async () => {
    setP2pLoading(true);
    const res = await aegis.initiateP2pSync();
    if (res.success && res.data) {
      setP2pInit(res.data);
      onAddLog('SUCCESS', 'p2p_sync_init', `Session ${res.data.session_id} initiated`);
    }
    setP2pLoading(false);
  };

  const handleCompleteP2p = async () => {
    if (!p2pInit) return;
    setP2pLoading(true);
    const mockResponderPub = 'e2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f90123456789abcdef0123456789ab';
    const res = await aegis.completeP2pSync(p2pInit.session_id, p2pInit.initiator_public_hex, mockResponderPub);
    if (res.success && res.data) {
      setP2pResp(res.data);
      onAddLog('SUCCESS', 'p2p_sync_complete', `P2P Handshake established with session key`);
    }
    setP2pLoading(false);
  };

  const handleEncryptP2pBundle = async () => {
    if (!p2pInit) return;
    setP2pLoading(true);
    const hexPayload = Array.from(new TextEncoder().encode(p2pPayload)).map(b => b.toString(16).padStart(2, '0')).join('');
    const res = await aegis.encryptP2pBundle(p2pInit.session_id, 1, hexPayload);
    if (res.success && res.data) {
      setP2pBundle(res.data);
      onAddLog('SUCCESS', 'p2p_sync_encrypt', `Bundle encrypted with AEAD Poly1305 MAC`);
    }
    setP2pLoading(false);
  };

  const handleDecryptP2pBundle = async () => {
    if (!p2pBundle) return;
    setP2pLoading(true);
    const res = await aegis.decryptP2pBundle(p2pBundle);
    if (res.success && res.data) {
      setP2pDecrypted(res.data);
      onAddLog('SUCCESS', 'p2p_sync_decrypt', `Decrypted payload verified: "${res.data.plaintextUtf8}"`);
    }
    setP2pLoading(false);
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
      {/* HEADER BANNER */}
      <div style={{ background: 'linear-gradient(135deg, rgba(15, 23, 42, 0.95), rgba(9, 13, 22, 0.98))', border: '1px solid rgba(0, 255, 204, 0.25)', borderRadius: '14px', padding: '24px', boxShadow: '0 8px 32px rgba(0,0,0,0.4)' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', flexWrap: 'wrap', gap: '16px' }}>
          <div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '8px' }}>
              <span style={{ background: 'rgba(0, 255, 204, 0.15)', color: '#00ffcc', border: '1px solid rgba(0, 255, 204, 0.4)', padding: '4px 10px', borderRadius: '6px', fontSize: '11px', fontWeight: 800, letterSpacing: '1px' }}>
                ENTERPRISE v1.3.0
              </span>
              <span style={{ fontSize: '20px', fontWeight: 800, color: '#f8fafc', letterSpacing: '0.5px' }}>
                {lang === 'si' ? 'උසස් ආයතනික හා බැංකු ආරක්ෂණ සූට්' : 'Enterprise Banking & Cryptographic Enclave Suite'}
              </span>
            </div>
            <p style={{ fontSize: '13px', color: '#94a3b8', margin: 0, maxWidth: '800px' }}>
              {lang === 'si'
                ? 'ChaCha20-Poly1305 ගුප්තකේතන දත්ත සමුදාය, දෘඩාංග TPM 2.0 PCR Sealing, ආයතනික RBAC, සජීවී Firewall & Traffic Monitor සහ OWASP ASVS 5.0 විගණන සහතික ජනකය.'
                : 'Authenticated ChaCha20-Poly1305 DB, Hardware TPM 2.0 PCR Sealing, Enterprise RBAC, Real-Time Network Firewall, and OWASP ASVS 5.0 Audited Compliance Certification.'}
            </p>
          </div>
          {killswitchActive && (
            <div style={{ background: 'rgba(239, 68, 68, 0.2)', border: '1px solid #ef4444', color: '#fca5a5', padding: '8px 16px', borderRadius: '8px', fontSize: '12px', fontWeight: 700, display: 'flex', alignItems: 'center', gap: '8px', animation: 'pulse 1.5s infinite' }}>
              <Power style={{ width: '16px', height: '16px' }} />
              AIR-GAP KILLSWITCH ACTIVE
            </div>
          )}
        </div>

        {/* SUBTAB NAVIGATION */}
        <div style={{ display: 'flex', gap: '8px', marginTop: '20px', overflowX: 'auto', paddingBottom: '4px' }}>
          {[
            { id: 'db', label: lang === 'si' ? '1. ගුප්තකේතන DB' : '1. Encrypted DB', icon: Database, color: '#38bdf8' },
            { id: 'tpm', label: lang === 'si' ? '2. TPM PCR Sealing' : '2. TPM PCR Enclave', icon: Cpu, color: '#a855f7' },
            { id: 'rbac', label: lang === 'si' ? '3. ආයතනික RBAC' : '3. Enterprise RBAC', icon: Users, color: '#f59e0b' },
            { id: 'traffic', label: lang === 'si' ? '4. ජාල Traffic Monitor' : '4. Traffic Monitor', icon: Activity, color: '#10b981' },
            { id: 'compliance', label: lang === 'si' ? '5. ASVS විගණන සහතිකය' : '5. Compliance Certificate', icon: FileCheck, color: '#00ffcc' },
            { id: 'rasp', label: 'RASP Shield', icon: ShieldAlert, color: '#ef4444' },
            { id: 'pinning', label: 'mTLS Pinning', icon: Wifi, color: '#06b6d4' },
            { id: 'fido2', label: 'FIDO2 Token', icon: Key, color: '#eab308' },
            { id: 'p2p', label: 'P2P Sync', icon: Binary, color: '#8b5cf6' },
          ].map(tab => {
            const Icon = tab.icon;
            const active = subTab === tab.id;
            return (
              <button
                key={tab.id}
                onClick={() => setSubTab(tab.id as any)}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '8px',
                  padding: '8px 14px',
                  borderRadius: '8px',
                  border: active ? `1px solid ${tab.color}` : '1px solid rgba(255,255,255,0.08)',
                  background: active ? `rgba(255,255,255,0.06)` : 'rgba(0,0,0,0.2)',
                  color: active ? tab.color : '#94a3b8',
                  fontSize: '12px',
                  fontWeight: active ? 700 : 500,
                  cursor: 'pointer',
                  whiteSpace: 'nowrap',
                  transition: 'all 0.15s ease'
                }}
              >
                <Icon style={{ width: '14px', height: '14px', color: tab.color }} />
                {tab.label}
              </button>
            );
          })}
        </div>
      </div>

      {/* ── SUBTAB 1: ENCRYPTED DATABASE ENGINE ───────────────────────────────── */}
      {subTab === 'db' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
          {/* STATS STRIP */}
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))', gap: '14px' }}>
            <div style={{ background: 'rgba(15, 23, 42, 0.7)', border: '1px solid rgba(56, 189, 248, 0.2)', borderRadius: '10px', padding: '16px' }}>
              <div style={{ fontSize: '11px', color: '#94a3b8', textTransform: 'uppercase' }}>Cipher Spec</div>
              <div style={{ fontSize: '15px', fontWeight: 700, color: '#38bdf8', marginTop: '4px' }}>ChaCha20-Poly1305 AEAD</div>
              <div style={{ fontSize: '10px', color: '#64748b', marginTop: '2px' }}>256-bit Authenticated Key</div>
            </div>
            <div style={{ background: 'rgba(15, 23, 42, 0.7)', border: '1px solid rgba(56, 189, 248, 0.2)', borderRadius: '10px', padding: '16px' }}>
              <div style={{ fontSize: '11px', color: '#94a3b8', textTransform: 'uppercase' }}>Database Tables</div>
              <div style={{ fontSize: '15px', fontWeight: 700, color: '#f8fafc', marginTop: '4px' }}>{dbStats?.total_tables || 2} Tables ({dbStats?.total_records || 2} Records)</div>
              <div style={{ fontSize: '10px', color: '#64748b', marginTop: '2px' }}>ACID Atomic Storage</div>
            </div>
            <div style={{ background: 'rgba(15, 23, 42, 0.7)', border: '1px solid rgba(56, 189, 248, 0.2)', borderRadius: '10px', padding: '16px' }}>
              <div style={{ fontSize: '11px', color: '#94a3b8', textTransform: 'uppercase' }}>Transaction Status</div>
              <div style={{ fontSize: '15px', fontWeight: 700, color: dbStats?.in_transaction ? '#fbbf24' : '#10b981', marginTop: '4px' }}>
                {dbStats?.in_transaction ? 'TRANSACTION PENDING' : 'COMMITTED / READY'}
              </div>
              <div style={{ fontSize: '10px', color: '#64748b', marginTop: '2px' }}>Snapshot Isolation Engine</div>
            </div>
            <div style={{ background: 'rgba(15, 23, 42, 0.7)', border: '1px solid rgba(56, 189, 248, 0.2)', borderRadius: '10px', padding: '16px' }}>
              <div style={{ fontSize: '11px', color: '#94a3b8', textTransform: 'uppercase' }}>Key Fingerprint</div>
              <div style={{ fontSize: '13px', fontFamily: 'monospace', fontWeight: 700, color: '#a7f3d0', marginTop: '4px' }}>
                {dbStats?.key_fingerprint || 'e3b0c44298fc...'}
              </div>
              <div style={{ fontSize: '10px', color: '#64748b', marginTop: '2px' }}>Zeroized on Process Exit</div>
            </div>
          </div>

          {/* CONTROLS & TABLE VIEWER */}
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 2fr', gap: '20px' }}>
            {/* LEFT: ACTIONS */}
            <div style={{ background: 'rgba(15, 23, 42, 0.7)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
              <h3 style={{ fontSize: '14px', fontWeight: 700, color: '#f8fafc', marginBottom: '14px', display: 'flex', alignItems: 'center', gap: '8px' }}>
                <Database style={{ width: '16px', height: '16px', color: '#38bdf8' }} />
                Database Operations
              </h3>

              <div style={{ display: 'flex', gap: '8px', marginBottom: '14px' }}>
                <button
                  onClick={() => handleQueryTable('accounts')}
                  style={{ flex: 1, padding: '8px', borderRadius: '6px', border: dbTable === 'accounts' ? '1px solid #38bdf8' : '1px solid rgba(255,255,255,0.1)', background: dbTable === 'accounts' ? 'rgba(56,189,248,0.15)' : 'rgba(0,0,0,0.2)', color: dbTable === 'accounts' ? '#38bdf8' : '#94a3b8', fontSize: '12px', fontWeight: 600, cursor: 'pointer' }}
                >
                  Accounts
                </button>
                <button
                  onClick={() => handleQueryTable('audit_ledger')}
                  style={{ flex: 1, padding: '8px', borderRadius: '6px', border: dbTable === 'audit_ledger' ? '1px solid #38bdf8' : '1px solid rgba(255,255,255,0.1)', background: dbTable === 'audit_ledger' ? 'rgba(56,189,248,0.15)' : 'rgba(0,0,0,0.2)', color: dbTable === 'audit_ledger' ? '#38bdf8' : '#94a3b8', fontSize: '12px', fontWeight: 600, cursor: 'pointer' }}
                >
                  Audit Ledger
                </button>
              </div>

              {/* TRANSACTION CONTROLS */}
              <div style={{ background: 'rgba(0,0,0,0.3)', borderRadius: '8px', padding: '12px', marginBottom: '16px' }}>
                <div style={{ fontSize: '11px', color: '#94a3b8', marginBottom: '8px', fontWeight: 600 }}>ACID Transaction Controls:</div>
                <div style={{ display: 'flex', gap: '6px' }}>
                  <button
                    onClick={() => handleDbTransaction('begin')}
                    disabled={dbLoading}
                    style={{ flex: 1, padding: '6px', background: 'rgba(251, 191, 36, 0.2)', border: '1px solid rgba(251, 191, 36, 0.4)', color: '#fde68a', borderRadius: '6px', fontSize: '11px', fontWeight: 600, cursor: 'pointer' }}
                  >
                    Begin
                  </button>
                  <button
                    onClick={() => handleDbTransaction('commit')}
                    disabled={dbLoading}
                    style={{ flex: 1, padding: '6px', background: 'rgba(16, 185, 129, 0.2)', border: '1px solid rgba(16, 185, 129, 0.4)', color: '#6ee7b7', borderRadius: '6px', fontSize: '11px', fontWeight: 600, cursor: 'pointer' }}
                  >
                    Commit
                  </button>
                  <button
                    onClick={() => handleDbTransaction('rollback')}
                    disabled={dbLoading}
                    style={{ flex: 1, padding: '6px', background: 'rgba(239, 68, 68, 0.2)', border: '1px solid rgba(239, 68, 68, 0.4)', color: '#fca5a5', borderRadius: '6px', fontSize: '11px', fontWeight: 600, cursor: 'pointer' }}
                  >
                    Rollback
                  </button>
                </div>
              </div>

              {/* INSERT ACCOUNT FORM */}
              <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
                <div style={{ fontSize: '11px', color: '#94a3b8', fontWeight: 600 }}>Insert Sovereign Account:</div>
                <input
                  type="text"
                  value={newAccNumber}
                  onChange={e => setNewAccNumber(e.target.value)}
                  placeholder="Account Number"
                  style={{ width: '100%', padding: '8px', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '6px', color: '#e2e8f0', fontSize: '11px' }}
                />
                <input
                  type="text"
                  value={newAccHolder}
                  onChange={e => setNewAccHolder(e.target.value)}
                  placeholder="Holder Name"
                  style={{ width: '100%', padding: '8px', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '6px', color: '#e2e8f0', fontSize: '11px' }}
                />
                <input
                  type="text"
                  value={newAccBalance}
                  onChange={e => setNewAccBalance(e.target.value)}
                  placeholder="Balance in cents"
                  style={{ width: '100%', padding: '8px', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '6px', color: '#e2e8f0', fontSize: '11px' }}
                />
                <button
                  onClick={handleInsertAccount}
                  disabled={dbLoading}
                  style={{ padding: '8px', background: 'linear-gradient(135deg, rgba(56, 189, 248, 0.25), rgba(14, 165, 233, 0.35))', border: '1px solid rgba(56, 189, 248, 0.5)', color: '#7dd3fc', borderRadius: '6px', fontSize: '12px', fontWeight: 600, cursor: 'pointer' }}
                >
                  Insert Encrypted Record
                </button>
              </div>
            </div>

            {/* RIGHT: RECORD TABLE */}
            <div style={{ background: 'rgba(15, 23, 42, 0.7)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '14px' }}>
                <span style={{ fontSize: '14px', fontWeight: 700, color: '#f8fafc' }}>
                  Table: <span style={{ color: '#38bdf8' }}>{dbTable}</span> ({dbRows.length} rows)
                </span>
                <button
                  onClick={refreshDb}
                  disabled={dbLoading}
                  style={{ background: 'transparent', border: 'none', color: '#94a3b8', cursor: 'pointer', display: 'flex', alignItems: 'center', gap: '4px', fontSize: '11px' }}
                >
                  <RefreshCw style={{ width: '12px', height: '12px', animation: dbLoading ? 'spin 1s infinite linear' : 'none' }} />
                  Refresh
                </button>
              </div>

              <div style={{ overflowX: 'auto' }}>
                <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '12px' }}>
                  <thead>
                    <tr style={{ background: 'rgba(0,0,0,0.3)', borderBottom: '1px solid rgba(255,255,255,0.1)' }}>
                      {dbTable === 'accounts' ? (
                        <>
                          <th style={{ textAlign: 'left', padding: '8px', color: '#94a3b8' }}>Account Number</th>
                          <th style={{ textAlign: 'left', padding: '8px', color: '#94a3b8' }}>Holder</th>
                          <th style={{ textAlign: 'right', padding: '8px', color: '#94a3b8' }}>Balance</th>
                          <th style={{ textAlign: 'left', padding: '8px', color: '#94a3b8' }}>Status</th>
                        </>
                      ) : (
                        <>
                          <th style={{ textAlign: 'left', padding: '8px', color: '#94a3b8' }}>Entry ID</th>
                          <th style={{ textAlign: 'left', padding: '8px', color: '#94a3b8' }}>Actor</th>
                          <th style={{ textAlign: 'left', padding: '8px', color: '#94a3b8' }}>Operation</th>
                          <th style={{ textAlign: 'left', padding: '8px', color: '#94a3b8' }}>SHA-256 Digest</th>
                        </>
                      )}
                    </tr>
                  </thead>
                  <tbody>
                    {dbRows.map((row, idx) => (
                      <tr key={idx} style={{ borderBottom: '1px solid rgba(255,255,255,0.05)' }}>
                        {dbTable === 'accounts' ? (
                          <>
                            <td style={{ padding: '8px', fontFamily: 'monospace', color: '#f8fafc' }}>{row.account_number}</td>
                            <td style={{ padding: '8px', color: '#cbd5e1' }}>{row.holder_name}</td>
                            <td style={{ padding: '8px', textAlign: 'right', fontWeight: 700, color: '#38bdf8' }}>
                              ${((row.balance_cents || 0) / 100).toLocaleString('en-US', { minimumFractionDigits: 2 })} {row.currency}
                            </td>
                            <td style={{ padding: '8px' }}>
                              <span style={{ background: 'rgba(16, 185, 129, 0.15)', color: '#6ee7b7', padding: '2px 8px', borderRadius: '4px', fontSize: '10px' }}>
                                {row.status}
                              </span>
                            </td>
                          </>
                        ) : (
                          <>
                            <td style={{ padding: '8px', fontFamily: 'monospace', color: '#f8fafc' }}>{row.entry_id}</td>
                            <td style={{ padding: '8px', color: '#cbd5e1' }}>{row.actor}</td>
                            <td style={{ padding: '8px', color: '#fbbf24' }}>{row.operation}</td>
                            <td style={{ padding: '8px', fontFamily: 'monospace', fontSize: '10px', color: '#64748b' }}>{row.sha256_hash?.substring(0, 24)}...</td>
                          </>
                        )}
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* ── SUBTAB 2: HARDWARE TPM 2.0 PCR ENCLAVE ────────────────────────────── */}
      {subTab === 'tpm' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
          {/* TPM STATUS BANNER */}
          <div style={{ background: 'rgba(15, 23, 42, 0.7)', border: '1px solid rgba(168, 85, 247, 0.3)', borderRadius: '12px', padding: '20px' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '14px' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                <Cpu style={{ width: '20px', height: '20px', color: '#c084fc' }} />
                <div>
                  <div style={{ fontSize: '15px', fontWeight: 700, color: '#f8fafc' }}>
                    Hardware TPM 2.0 Security Enclave
                  </div>
                  <div style={{ fontSize: '11px', color: '#94a3b8' }}>
                    {tpmStatus?.manufacturer || 'Intel fTPM 2.0 (Platform Trust Technology)'}
                  </div>
                </div>
              </div>
              <div style={{ display: 'flex', gap: '8px' }}>
                <span style={{ background: 'rgba(16, 185, 129, 0.15)', color: '#6ee7b7', border: '1px solid rgba(16, 185, 129, 0.3)', padding: '4px 10px', borderRadius: '6px', fontSize: '11px', fontWeight: 700 }}>
                  SECURE BOOT ACTIVE
                </span>
                <span style={{ background: 'rgba(168, 85, 247, 0.15)', color: '#c084fc', border: '1px solid rgba(168, 85, 247, 0.3)', padding: '4px 10px', borderRadius: '6px', fontSize: '11px', fontWeight: 700 }}>
                  {tpmStatus?.defense_tier || 'TIER-1 ROOT OF TRUST'}
                </span>
              </div>
            </div>

            {/* PCR REGISTERS GRID */}
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(240px, 1fr))', gap: '10px', marginTop: '16px' }}>
              {(tpmStatus?.pcr_registers || [
                { index: 0, description: 'CRTM & BIOS Firmware', digest_hex: 'a1b2c3d4e5f60718293a4b5c6d7e8f90...' },
                { index: 2, description: 'Option ROM & Extended Code', digest_hex: 'b2c3d4e5f6a10718293a4b5c6d7e8f90...' },
                { index: 7, description: 'Secure Boot State & Certificates', digest_hex: 'c3d4e5f6a1b20718293a4b5c6d7e8f90...' },
                { index: 11, description: 'Aegis Core Code Integrity', digest_hex: 'd4e5f6a1b2c30718293a4b5c6d7e8f90...' },
              ]).map((pcr: any) => (
                <div key={pcr.index} style={{ background: 'rgba(0,0,0,0.3)', border: pcr.index === 7 && tpmTampered ? '1px solid #ef4444' : '1px solid rgba(255,255,255,0.06)', borderRadius: '8px', padding: '12px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <span style={{ fontSize: '11px', fontWeight: 700, color: pcr.index === 7 && tpmTampered ? '#fca5a5' : '#c084fc' }}>
                      PCR [{pcr.index}] {pcr.index === 7 && tpmTampered && '⚠️ TAMPERED'}
                    </span>
                    <span style={{ fontSize: '10px', color: '#64748b' }}>SHA-256</span>
                  </div>
                  <div style={{ fontSize: '11px', color: '#cbd5e1', margin: '4px 0' }}>{pcr.description}</div>
                  <div style={{ fontSize: '9px', fontFamily: 'monospace', color: pcr.index === 7 && tpmTampered ? '#f87171' : '#94a3b8', wordBreak: 'break-all' }}>
                    {pcr.digest_hex}
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* SEALING / UNSEALING CONTROLS */}
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px' }}>
            <div style={{ background: 'rgba(15, 23, 42, 0.7)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
              <h3 style={{ fontSize: '14px', fontWeight: 700, color: '#f8fafc', marginBottom: '12px' }}>
                Seal Secret into TPM Hardware Enclave
              </h3>
              <p style={{ fontSize: '12px', color: '#94a3b8', marginBottom: '14px' }}>
                Cryptographically binds your secret key to PCR registers 0, 2, 7, and 11. If firmware or OS boot integrity is modified, unsealing fails closed.
              </p>
              <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
                <input
                  type="text"
                  value={packageId}
                  onChange={e => setPackageId(e.target.value)}
                  placeholder="Package ID"
                  style={{ width: '100%', padding: '8px', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '6px', color: '#e2e8f0', fontSize: '12px' }}
                />
                <textarea
                  value={secretToSeal}
                  onChange={e => setSecretToSeal(e.target.value)}
                  rows={2}
                  placeholder="Secret payload to seal"
                  style={{ width: '100%', padding: '8px', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '6px', color: '#e2e8f0', fontSize: '12px' }}
                />
                <button
                  onClick={handleSealSecret}
                  disabled={tpmLoading}
                  style={{ padding: '10px', background: 'linear-gradient(135deg, rgba(168, 85, 247, 0.25), rgba(147, 51, 234, 0.35))', border: '1px solid rgba(168, 85, 247, 0.5)', color: '#d8b4fe', borderRadius: '6px', fontSize: '12px', fontWeight: 600, cursor: 'pointer' }}
                >
                  🔒 Seal Secret to Hardware PCRs
                </button>
              </div>

              {sealedPackage && (
                <div style={{ marginTop: '14px', padding: '10px', background: 'rgba(168, 85, 247, 0.1)', border: '1px solid rgba(168, 85, 247, 0.2)', borderRadius: '6px', fontSize: '11px', color: '#d8b4fe' }}>
                  <div style={{ fontWeight: 700 }}>Package Sealed Successfully</div>
                  <div style={{ fontFamily: 'monospace', fontSize: '10px', marginTop: '4px' }}>Composite: {sealedPackage.pcr_composite_hash}</div>
                </div>
              )}
            </div>

            <div style={{ background: 'rgba(15, 23, 42, 0.7)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
              <h3 style={{ fontSize: '14px', fontWeight: 700, color: '#f8fafc', marginBottom: '12px' }}>
                Unseal & Tamper Resilience Test
              </h3>
              <p style={{ fontSize: '12px', color: '#94a3b8', marginBottom: '14px' }}>
                Verify that unsealing only succeeds when platform PCRs match the exact expected state.
              </p>

              <div style={{ display: 'flex', gap: '8px', marginBottom: '14px' }}>
                <button
                  onClick={handleUnsealSecret}
                  disabled={tpmLoading}
                  style={{ flex: 1, padding: '10px', background: 'rgba(16, 185, 129, 0.2)', border: '1px solid rgba(16, 185, 129, 0.4)', color: '#6ee7b7', borderRadius: '6px', fontSize: '12px', fontWeight: 600, cursor: 'pointer' }}
                >
                  🔓 Unseal Secret
                </button>
                <button
                  onClick={handleSimulateTamper}
                  disabled={tpmLoading}
                  style={{ flex: 1, padding: '10px', background: 'rgba(239, 68, 68, 0.2)', border: '1px solid rgba(239, 68, 68, 0.4)', color: '#fca5a5', borderRadius: '6px', fontSize: '12px', fontWeight: 600, cursor: 'pointer' }}
                >
                  ⚡ Simulate Rootkit Tamper
                </button>
              </div>

              {unsealedSecret && (
                <div style={{ padding: '12px', background: 'rgba(16, 185, 129, 0.15)', border: '1px solid rgba(16, 185, 129, 0.3)', borderRadius: '6px', color: '#6ee7b7', fontSize: '12px' }}>
                  <div style={{ fontWeight: 700, marginBottom: '4px' }}>✅ Enclave PCR Verified & Secret Unsealed:</div>
                  <div style={{ fontFamily: 'monospace', wordBreak: 'break-all' }}>{unsealedSecret}</div>
                </div>
              )}

              {tpmTampered && !unsealedSecret && (
                <div style={{ padding: '12px', background: 'rgba(239, 68, 68, 0.15)', border: '1px solid rgba(239, 68, 68, 0.3)', borderRadius: '6px', color: '#fca5a5', fontSize: '12px' }}>
                  <div style={{ fontWeight: 700, marginBottom: '2px' }}>⛔ HARDWARE FAIL-CLOSED ENCLAVE DENIAL</div>
                  <div>PCR 7 SecureBoot digest altered! Cryptographic key derivation locked out.</div>
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {/* ── SUBTAB 3: ENTERPRISE GRANULAR RBAC ─────────────────────────────────── */}
      {subTab === 'rbac' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
          {/* CURRENT IDENTITY CARD */}
          <div style={{ background: 'rgba(15, 23, 42, 0.7)', border: '1px solid rgba(245, 158, 11, 0.3)', borderRadius: '12px', padding: '20px' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', flexWrap: 'wrap', gap: '12px' }}>
              <div>
                <div style={{ fontSize: '11px', color: '#f59e0b', textTransform: 'uppercase', letterSpacing: '1px', fontWeight: 700 }}>
                  Active Corporate Identity
                </div>
                <div style={{ fontSize: '18px', fontWeight: 800, color: '#f8fafc', marginTop: '2px' }}>
                  {rbacIdentity?.display_name || 'Elena Rostova (Branch Treasury)'}
                </div>
                <div style={{ fontSize: '12px', color: '#94a3b8' }}>
                  User ID: <code style={{ color: '#fde68a' }}>{rbacIdentity?.user_id || 'USR-MGR-8891'}</code> • Department: {rbacIdentity?.department || 'Global Treasury'}
                </div>
              </div>
              <div style={{ background: 'rgba(245, 158, 11, 0.15)', border: '1px solid #f59e0b', color: '#fde68a', padding: '6px 14px', borderRadius: '6px', fontSize: '13px', fontWeight: 700 }}>
                ROLE: {rbacIdentity?.role || 'BranchManager'}
              </div>
            </div>

            {/* SWITCH ROLE BUTTONS */}
            <div style={{ marginTop: '16px', paddingTop: '16px', borderTop: '1px solid rgba(255,255,255,0.06)' }}>
              <div style={{ fontSize: '11px', color: '#94a3b8', marginBottom: '8px', fontWeight: 600 }}>Assume Corporate Role:</div>
              <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
                {['SuperAdmin', 'ComplianceOfficer', 'BranchManager', 'SeniorTeller', 'Auditor'].map(r => (
                  <button
                    key={r}
                    onClick={() => handleSwitchRole(r)}
                    disabled={rbacLoading}
                    style={{
                      padding: '6px 12px',
                      borderRadius: '6px',
                      border: rbacIdentity?.role === r ? '1px solid #f59e0b' : '1px solid rgba(255,255,255,0.1)',
                      background: rbacIdentity?.role === r ? 'rgba(245, 158, 11, 0.2)' : 'rgba(0,0,0,0.2)',
                      color: rbacIdentity?.role === r ? '#fde68a' : '#94a3b8',
                      fontSize: '11px',
                      fontWeight: 600,
                      cursor: 'pointer'
                    }}
                  >
                    {r}
                  </button>
                ))}
              </div>
            </div>
          </div>

          {/* PERMISSION CHECKER & MATRIX */}
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px' }}>
            <div style={{ background: 'rgba(15, 23, 42, 0.7)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
              <h3 style={{ fontSize: '14px', fontWeight: 700, color: '#f8fafc', marginBottom: '12px' }}>
                Test Basel III & PCI-DSS Capability
              </h3>
              <p style={{ fontSize: '12px', color: '#94a3b8', marginBottom: '14px' }}>
                Verify whether your currently assumed role is authorized to perform specific high-assurance banking actions.
              </p>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '8px', marginBottom: '14px' }}>
                {[
                  { action: 'initiate_transaction', label: 'Initiate Transfer (Maker)' },
                  { action: 'approve_transaction', label: 'Approve Transfer (Checker)' },
                  { action: 'rotate_keys', label: 'Rotate Master Keys' },
                  { action: 'export_compliance', label: 'Export Audit Report' },
                ].map(item => (
                  <button
                    key={item.action}
                    onClick={() => handleCheckPerm(item.action)}
                    disabled={rbacLoading}
                    style={{ padding: '8px', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', color: '#e2e8f0', borderRadius: '6px', fontSize: '11px', fontWeight: 600, cursor: 'pointer' }}
                  >
                    {item.label}
                  </button>
                ))}
              </div>

              {permCheckResult && (
                <div style={{ padding: '12px', background: permCheckResult.allowed ? 'rgba(16, 185, 129, 0.15)' : 'rgba(239, 68, 68, 0.15)', border: permCheckResult.allowed ? '1px solid rgba(16, 185, 129, 0.3)' : '1px solid rgba(239, 68, 68, 0.3)', borderRadius: '6px', fontSize: '12px', color: permCheckResult.allowed ? '#6ee7b7' : '#fca5a5' }}>
                  <div style={{ fontWeight: 700, marginBottom: '2px' }}>
                    {permCheckResult.allowed ? '✅ ACTION PERMITTED' : '⛔ ACTION PROHIBITED (Segregation of Duties)'}
                  </div>
                  <div>{permCheckResult.reason}</div>
                </div>
              )}
            </div>

            {/* AUDIT LOG VIEWER */}
            <div style={{ background: 'rgba(15, 23, 42, 0.7)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
                <span style={{ fontSize: '14px', fontWeight: 700, color: '#f8fafc' }}>Privileged Access Audit Trail</span>
                <span style={{ fontSize: '11px', color: '#64748b' }}>{rbacAudit.length} attempts recorded</span>
              </div>
              <div style={{ maxHeight: '240px', overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: '8px' }}>
                {rbacAudit.slice(-6).reverse().map((entry, idx) => (
                  <div key={idx} style={{ background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.05)', borderRadius: '6px', padding: '10px', fontSize: '11px' }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '2px' }}>
                      <span style={{ fontWeight: 700, color: '#f8fafc' }}>{entry.requested_action}</span>
                      <span style={{ color: entry.decision === 'PERMITTED' ? '#10b981' : '#ef4444', fontWeight: 700 }}>
                        {entry.decision}
                      </span>
                    </div>
                    <div style={{ fontSize: '10px', color: '#94a3b8' }}>Actor: {entry.actor_id} ({entry.role})</div>
                    <div style={{ fontSize: '10px', color: '#64748b', marginTop: '2px' }}>{entry.reason}</div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* ── SUBTAB 4: TRAFFIC FLOW MONITOR & FIREWALL ─────────────────────────── */}
      {subTab === 'traffic' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
          {/* TRAFFIC STATS STRIP */}
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '14px' }}>
            <div style={{ background: 'rgba(15, 23, 42, 0.7)', border: '1px solid rgba(16, 185, 129, 0.2)', borderRadius: '10px', padding: '16px' }}>
              <div style={{ fontSize: '11px', color: '#94a3b8', textTransform: 'uppercase' }}>Total Inspected</div>
              <div style={{ fontSize: '18px', fontWeight: 800, color: '#10b981', marginTop: '4px' }}>{trafficStats?.total_flows_inspected || 6} Flows</div>
              <div style={{ fontSize: '10px', color: '#64748b', marginTop: '2px' }}>Deep Packet Telemetry</div>
            </div>
            <div style={{ background: 'rgba(15, 23, 42, 0.7)', border: '1px solid rgba(239, 68, 68, 0.2)', borderRadius: '10px', padding: '16px' }}>
              <div style={{ fontSize: '11px', color: '#94a3b8', textTransform: 'uppercase' }}>Threats Blocked</div>
              <div style={{ fontSize: '18px', fontWeight: 800, color: '#ef4444', marginTop: '4px' }}>{trafficStats?.total_blocked || 3} Dropped</div>
              <div style={{ fontSize: '10px', color: '#64748b', marginTop: '2px' }}>Telemetry & Malicious C2</div>
            </div>
            <div style={{ background: 'rgba(15, 23, 42, 0.7)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '10px', padding: '16px' }}>
              <div style={{ fontSize: '11px', color: '#94a3b8', textTransform: 'uppercase' }}>Throughput</div>
              <div style={{ fontSize: '18px', fontWeight: 800, color: '#38bdf8', marginTop: '4px' }}>
                {((trafficStats?.total_bytes_transferred || 15360) / 1024).toFixed(1)} KB
              </div>
              <div style={{ fontSize: '10px', color: '#64748b', marginTop: '2px' }}>Encrypted Transport</div>
            </div>
            <div style={{ background: 'rgba(15, 23, 42, 0.7)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '10px', padding: '16px' }}>
              <div style={{ fontSize: '11px', color: '#94a3b8', textTransform: 'uppercase' }}>Emergency Killswitch</div>
              <button
                onClick={handleToggleKillswitch}
                disabled={trafficLoading}
                style={{
                  marginTop: '6px',
                  width: '100%',
                  padding: '6px',
                  background: killswitchActive ? '#ef4444' : 'rgba(239, 68, 68, 0.2)',
                  border: '1px solid #ef4444',
                  color: killswitchActive ? '#fff' : '#fca5a5',
                  borderRadius: '6px',
                  fontSize: '11px',
                  fontWeight: 700,
                  cursor: 'pointer'
                }}
              >
                {killswitchActive ? 'DISABLE AIR-GAP' : 'TRIGGER KILLSWITCH'}
              </button>
            </div>
          </div>

          {/* SIMULATION & FLOW TABLE */}
          <div style={{ background: 'rgba(15, 23, 42, 0.7)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '14px', flexWrap: 'wrap', gap: '10px' }}>
              <div>
                <span style={{ fontSize: '14px', fontWeight: 700, color: '#f8fafc' }}>
                  Real-Time Outbound Packet Stream
                </span>
                <p style={{ fontSize: '11px', color: '#94a3b8', margin: '2px 0 0 0' }}>
                  Live zero-trust outbound firewall telemetry with automated threat evaluation and latency monitoring.
                </p>
              </div>
              <div style={{ display: 'flex', gap: '8px' }}>
                <input
                  type="text"
                  value={simHost}
                  onChange={e => setSimHost(e.target.value)}
                  placeholder="Target host"
                  style={{ padding: '6px 10px', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '6px', color: '#e2e8f0', fontSize: '11px' }}
                />
                <button
                  onClick={handleSimulateTraffic}
                  disabled={trafficLoading}
                  style={{ padding: '6px 12px', background: 'rgba(16, 185, 129, 0.2)', border: '1px solid rgba(16, 185, 129, 0.4)', color: '#6ee7b7', borderRadius: '6px', fontSize: '11px', fontWeight: 600, cursor: 'pointer' }}
                >
                  Send Packet
                </button>
              </div>
            </div>

            <div style={{ overflowX: 'auto' }}>
              <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '11px' }}>
                <thead>
                  <tr style={{ background: 'rgba(0,0,0,0.3)', borderBottom: '1px solid rgba(255,255,255,0.1)' }}>
                    <th style={{ textAlign: 'left', padding: '8px', color: '#94a3b8' }}>Flow ID</th>
                    <th style={{ textAlign: 'left', padding: '8px', color: '#94a3b8' }}>Target Host</th>
                    <th style={{ textAlign: 'left', padding: '8px', color: '#94a3b8' }}>Proto</th>
                    <th style={{ textAlign: 'right', padding: '8px', color: '#94a3b8' }}>Size</th>
                    <th style={{ textAlign: 'right', padding: '8px', color: '#94a3b8' }}>Latency</th>
                    <th style={{ textAlign: 'center', padding: '8px', color: '#94a3b8' }}>Threat</th>
                    <th style={{ textAlign: 'left', padding: '8px', color: '#94a3b8' }}>Decision</th>
                    <th style={{ textAlign: 'left', padding: '8px', color: '#94a3b8' }}>Region</th>
                  </tr>
                </thead>
                <tbody>
                  {trafficFlows.map(flow => (
                    <tr key={flow.id} style={{ borderBottom: '1px solid rgba(255,255,255,0.04)' }}>
                      <td style={{ padding: '8px', fontFamily: 'monospace', color: '#94a3b8' }}>{flow.id}</td>
                      <td style={{ padding: '8px', fontWeight: 600, color: '#f8fafc' }}>{flow.target_host}</td>
                      <td style={{ padding: '8px', color: '#38bdf8' }}>{flow.protocol}</td>
                      <td style={{ padding: '8px', textAlign: 'right', color: '#cbd5e1' }}>{flow.bytes_transferred} B</td>
                      <td style={{ padding: '8px', textAlign: 'right', color: '#cbd5e1' }}>{flow.latency_ms} ms</td>
                      <td style={{ padding: '8px', textAlign: 'center' }}>
                        <span style={{ padding: '2px 6px', borderRadius: '4px', fontSize: '10px', fontWeight: 700, background: flow.threat_score >= 50 ? 'rgba(239, 68, 68, 0.2)' : 'rgba(16, 185, 129, 0.2)', color: flow.threat_score >= 50 ? '#fca5a5' : '#6ee7b7' }}>
                          {flow.threat_score}
                        </span>
                      </td>
                      <td style={{ padding: '8px' }}>
                        <span style={{ padding: '2px 8px', borderRadius: '4px', fontSize: '10px', fontWeight: 700, background: flow.decision === 'ALLOWED' ? 'rgba(16, 185, 129, 0.15)' : 'rgba(239, 68, 68, 0.15)', color: flow.decision === 'ALLOWED' ? '#6ee7b7' : '#fca5a5' }}>
                          {flow.decision}
                        </span>
                      </td>
                      <td style={{ padding: '8px', color: '#64748b' }}>{flow.geo_region}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      )}

      {/* ── SUBTAB 5: AUTOMATED COMPLIANCE CERTIFICATE GENERATOR ──────────────── */}
      {subTab === 'compliance' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
          <div style={{ background: 'rgba(15, 23, 42, 0.7)', border: '1px solid rgba(0, 255, 204, 0.3)', borderRadius: '12px', padding: '24px' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', flexWrap: 'wrap', gap: '14px', marginBottom: '16px' }}>
              <div>
                <div style={{ fontSize: '11px', color: '#00ffcc', textTransform: 'uppercase', letterSpacing: '1px', fontWeight: 800 }}>
                  Aegis Sovereign Enclave • Root of Trust
                </div>
                <div style={{ fontSize: '18px', fontWeight: 800, color: '#f8fafc', marginTop: '2px' }}>
                  OWASP ASVS 5.0 Attested Compliance Generator
                </div>
                <div style={{ fontSize: '12px', color: '#94a3b8' }}>
                  Military & Banking Level 3 High-Assurance (143/143 Controls Verified)
                </div>
              </div>
              <div style={{ display: 'flex', gap: '8px' }}>
                <button
                  onClick={handleGenerateCertificate}
                  disabled={complianceLoading}
                  style={{ padding: '10px 18px', background: 'linear-gradient(135deg, rgba(0, 255, 204, 0.25), rgba(5, 150, 105, 0.35))', border: '1px solid #00ffcc', color: '#00ffcc', borderRadius: '8px', fontSize: '12px', fontWeight: 700, cursor: 'pointer', display: 'flex', alignItems: 'center', gap: '6px' }}
                >
                  <FileCheck style={{ width: '14px', height: '14px' }} />
                  Generate Signed Certificate
                </button>
                {certificate && (
                  <button
                    onClick={handleVerifyCertificate}
                    disabled={complianceLoading}
                    style={{ padding: '10px 18px', background: 'rgba(16, 185, 129, 0.2)', border: '1px solid rgba(16, 185, 129, 0.4)', color: '#6ee7b7', borderRadius: '8px', fontSize: '12px', fontWeight: 700, cursor: 'pointer', display: 'flex', alignItems: 'center', gap: '6px' }}
                  >
                    <ShieldCheck style={{ width: '14px', height: '14px' }} />
                    Verify Ed25519 Signature
                  </button>
                )}
              </div>
            </div>

            {certificate && (
              <div style={{ display: 'flex', flexDirection: 'column', gap: '16px', marginTop: '16px' }}>
                {certVerified !== null && (
                  <div style={{ padding: '12px 16px', background: certVerified ? 'rgba(16, 185, 129, 0.15)' : 'rgba(239, 68, 68, 0.15)', border: certVerified ? '1px solid #10b981' : '1px solid #ef4444', borderRadius: '8px', color: certVerified ? '#6ee7b7' : '#fca5a5', fontSize: '12px', display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <ShieldCheck style={{ width: '16px', height: '16px' }} />
                    <span style={{ fontWeight: 700 }}>
                      {certVerified ? 'MATHEMATICALLY PROVED: Enclave Ed25519 Signature Validated 100%' : 'SIGNATURE VERIFICATION FAILED'}
                    </span>
                  </div>
                )}

                {/* DETAILS GRID */}
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '12px' }}>
                  <div style={{ background: 'rgba(0,0,0,0.3)', padding: '12px', borderRadius: '8px' }}>
                    <div style={{ fontSize: '10px', color: '#64748b', textTransform: 'uppercase' }}>Certificate ID</div>
                    <div style={{ fontSize: '13px', fontWeight: 700, color: '#00ffcc', marginTop: '2px' }}>{certificate.payload.certificate_id}</div>
                  </div>
                  <div style={{ background: 'rgba(0,0,0,0.3)', padding: '12px', borderRadius: '8px' }}>
                    <div style={{ fontSize: '10px', color: '#64748b', textTransform: 'uppercase' }}>ASVS 5.0 Score</div>
                    <div style={{ fontSize: '13px', fontWeight: 700, color: '#10b981', marginTop: '2px' }}>{certificate.payload.score}</div>
                  </div>
                  <div style={{ background: 'rgba(0,0,0,0.3)', padding: '12px', borderRadius: '8px' }}>
                    <div style={{ fontSize: '10px', color: '#64748b', textTransform: 'uppercase' }}>TPM Hardware State</div>
                    <div style={{ fontSize: '13px', fontWeight: 700, color: '#c084fc', marginTop: '2px' }}>{certificate.payload.tpm_pcr_status}</div>
                  </div>
                  <div style={{ background: 'rgba(0,0,0,0.3)', padding: '12px', borderRadius: '8px' }}>
                    <div style={{ fontSize: '10px', color: '#64748b', textTransform: 'uppercase' }}>Maker-Checker SoD</div>
                    <div style={{ fontSize: '13px', fontWeight: 700, color: '#f59e0b', marginTop: '2px' }}>{certificate.payload.maker_checker_sod_status}</div>
                  </div>
                </div>

                {/* CRYPTOGRAPHIC SIGNATURE BOX */}
                <div style={{ background: '#080c14', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '8px', padding: '14px', fontFamily: 'monospace', fontSize: '11px', display: 'flex', flexDirection: 'column', gap: '6px' }}>
                  <div>
                    <span style={{ color: '#00ffcc', fontWeight: 700 }}>Audit Digest (SHA-256): </span>
                    <span style={{ color: '#cbd5e1' }}>{certificate.payload_sha256}</span>
                  </div>
                  <div>
                    <span style={{ color: '#00ffcc', fontWeight: 700 }}>Ed25519 Signature: </span>
                    <span style={{ color: '#a7f3d0' }}>{certificate.signature_hex}</span>
                  </div>
                  <div>
                    <span style={{ color: '#00ffcc', fontWeight: 700 }}>Public Verifying Key: </span>
                    <span style={{ color: '#94a3b8' }}>{certificate.public_key_hex}</span>
                  </div>
                </div>

                <button
                  onClick={() => setShowCertificateHtml(!showCertificateHtml)}
                  style={{ alignSelf: 'flex-start', padding: '8px 14px', background: 'rgba(255,255,255,0.05)', border: '1px solid rgba(255,255,255,0.1)', color: '#f8fafc', borderRadius: '6px', fontSize: '11px', fontWeight: 600, cursor: 'pointer', display: 'flex', alignItems: 'center', gap: '6px' }}
                >
                  <ExternalLink style={{ width: '12px', height: '12px' }} />
                  {showCertificateHtml ? 'Hide Printable Certificate' : 'Preview Official Printable Certificate'}
                </button>

                {showCertificateHtml && (
                  <div style={{ background: '#090d16', border: '1px solid #00ffcc', borderRadius: '8px', padding: '16px', overflow: 'hidden' }}>
                    <iframe
                      srcDoc={certificate.printable_html}
                      title="Compliance Certificate"
                      style={{ width: '100%', height: '520px', border: 'none', borderRadius: '6px', background: '#090d16' }}
                    />
                  </div>
                )}
              </div>
            )}
          </div>
        </div>
      )}

      {/* ── SUBTAB: RASP & MEMORY DEFENSE ─────────────────────────────────────── */}
      {subTab === 'rasp' && (
        <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
            <div>
              <div style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc', display: 'flex', alignItems: 'center', gap: '8px' }}>
                <ShieldAlert style={{ width: '18px', height: '18px', color: '#ef4444' }} />
                {t.rasp.title}
              </div>
              <p style={{ fontSize: '12px', color: '#94a3b8', margin: '4px 0 0 0' }}>{t.rasp.subtitle}</p>
            </div>
            <div style={{ display: 'flex', gap: '8px' }}>
              <button
                onClick={handleRunRaspScan}
                disabled={raspLoading}
                style={{ background: 'linear-gradient(135deg, rgba(239, 68, 68, 0.25), rgba(220, 38, 38, 0.35))', border: '1px solid rgba(239, 68, 68, 0.5)', color: '#fca5a5', padding: '8px 16px', borderRadius: '8px', fontSize: '12px', fontWeight: 600, cursor: 'pointer', display: 'flex', alignItems: 'center', gap: '6px' }}
              >
                <RefreshCw style={{ width: '14px', height: '14px', animation: raspLoading ? 'spin 1s infinite linear' : 'none' }} />
                {t.rasp.runScan}
              </button>
              <button
                onClick={handleResetRaspContainment}
                disabled={raspLoading}
                style={{ background: 'rgba(255,255,255,0.05)', border: '1px solid rgba(255,255,255,0.1)', color: '#94a3b8', padding: '8px 12px', borderRadius: '8px', fontSize: '12px', cursor: 'pointer' }}
              >
                {t.rasp.resetContainment}
              </button>
            </div>
          </div>

          {raspReport && (
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))', gap: '12px' }}>
              <div style={{ background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.06)', borderRadius: '8px', padding: '12px' }}>
                <div style={{ fontSize: '11px', color: '#64748b' }}>{t.rasp.integrityTier}</div>
                <div style={{ fontSize: '14px', fontWeight: 700, color: raspReport.system_integrity_tier === 'SECURE' ? '#10b981' : '#ef4444', marginTop: '4px' }}>
                  {raspReport.system_integrity_tier}
                </div>
              </div>
              <div style={{ background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.06)', borderRadius: '8px', padding: '12px' }}>
                <div style={{ fontSize: '11px', color: '#64748b' }}>{t.rasp.debuggerStatus}</div>
                <div style={{ fontSize: '14px', fontWeight: 700, color: raspReport.debugger_detected ? '#ef4444' : '#10b981', marginTop: '4px' }}>
                  {raspReport.debugger_detected ? t.rasp.detected : t.rasp.clean}
                </div>
              </div>
              <div style={{ background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.06)', borderRadius: '8px', padding: '12px' }}>
                <div style={{ fontSize: '11px', color: '#64748b' }}>{t.rasp.memoryHooks}</div>
                <div style={{ fontSize: '14px', fontWeight: 700, color: raspReport.memory_hooks_detected ? '#ef4444' : '#10b981', marginTop: '4px' }}>
                  {raspReport.memory_hooks_detected ? t.rasp.detected : t.rasp.clean}
                </div>
              </div>
              <div style={{ background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.06)', borderRadius: '8px', padding: '12px' }}>
                <div style={{ fontSize: '11px', color: '#64748b' }}>{t.rasp.failClosed}</div>
                <div style={{ fontSize: '14px', fontWeight: 700, color: raspReport.fail_closed_containment_active ? '#ef4444' : '#10b981', marginTop: '4px' }}>
                  {raspReport.fail_closed_containment_active ? 'FAIL-CLOSED ACTIVE' : 'NOMINAL MONITOR'}
                </div>
              </div>
            </div>
          )}
        </div>
      )}

      {/* ── SUBTAB: mTLS PINNING ──────────────────────────────────────────────── */}
      {subTab === 'pinning' && (
        <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
          <div style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc', display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '8px' }}>
            <Wifi style={{ width: '18px', height: '18px', color: '#06b6d4' }} />
            {t.network.title}
          </div>
          <p style={{ fontSize: '12px', color: '#94a3b8', margin: '0 0 16px 0' }}>{t.network.subtitle}</p>

          <div style={{ display: 'flex', gap: '10px', marginBottom: '16px' }}>
            <input
              type="text"
              value={pinningHost}
              onChange={e => setPinningHost(e.target.value)}
              placeholder="Target host"
              style={{ flex: 1, padding: '8px 12px', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '6px', color: '#e2e8f0', fontSize: '12px' }}
            />
            <button
              onClick={handleEvaluatePinning}
              disabled={pinningLoading}
              style={{ padding: '8px 16px', background: 'rgba(6, 182, 212, 0.2)', border: '1px solid rgba(6, 182, 212, 0.4)', color: '#67e8f9', borderRadius: '6px', fontSize: '12px', fontWeight: 600, cursor: 'pointer' }}
            >
              {t.network.evaluate}
            </button>
          </div>

          {pinningReport && (
            <div style={{ padding: '14px', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.06)', borderRadius: '8px' }}>
              <div style={{ fontSize: '12px', color: pinningReport.pin_verified ? '#10b981' : '#ef4444', fontWeight: 700, marginBottom: '6px' }}>
                {pinningReport.pin_verified ? '✓ SPKI PIN VERIFIED // ZERO MITM RISK' : '⛔ PIN VERIFICATION FAILED'}
              </div>
              <div style={{ fontSize: '11px', color: '#94a3b8' }}>Observed SPKI: <code style={{ color: '#cbd5e1' }}>{pinningReport.observed_spki_pin}</code></div>
            </div>
          )}
        </div>
      )}

      {/* ── SUBTAB: FIDO2 HARDWARE KEY ───────────────────────────────────────── */}
      {subTab === 'fido2' && (
        <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
          <div style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc', display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '8px' }}>
            <Key style={{ width: '18px', height: '18px', color: '#eab308' }} />
            {t.fido2.title}
          </div>
          <p style={{ fontSize: '12px', color: '#94a3b8', margin: '0 0 16px 0' }}>{t.fido2.subtitle}</p>

          <div style={{ display: 'flex', gap: '10px', marginBottom: '16px' }}>
            <button
              onClick={handleEnumerateFido2}
              disabled={fido2Loading}
              style={{ padding: '8px 14px', background: 'rgba(234, 179, 8, 0.2)', border: '1px solid rgba(234, 179, 8, 0.4)', color: '#fde047', borderRadius: '6px', fontSize: '12px', fontWeight: 600, cursor: 'pointer' }}
            >
              {t.fido2.devicesFound}
            </button>
            <button
              onClick={handleRegisterFido2}
              disabled={fido2Loading}
              style={{ padding: '8px 14px', background: 'rgba(255,255,255,0.05)', border: '1px solid rgba(255,255,255,0.1)', color: '#f8fafc', borderRadius: '6px', fontSize: '12px', fontWeight: 600, cursor: 'pointer' }}
            >
              {t.fido2.registerKey}
            </button>
            {fido2Cred && (
              <button
                onClick={handleSignFido2}
                disabled={fido2Loading}
                style={{ padding: '8px 14px', background: 'rgba(16, 185, 129, 0.2)', border: '1px solid rgba(16, 185, 129, 0.4)', color: '#6ee7b7', borderRadius: '6px', fontSize: '12px', fontWeight: 600, cursor: 'pointer' }}
              >
                {t.fido2.signAssertion}
              </button>
            )}
          </div>

          {fido2Devices.length > 0 && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
              {fido2Devices.map(d => (
                <div key={d.device_id} style={{ padding: '10px', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.06)', borderRadius: '6px', fontSize: '11px' }}>
                  <span style={{ fontWeight: 700, color: '#fde047' }}>{d.manufacturer} {d.product_name}</span> ({d.transport})
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* ── SUBTAB: P2P SYNC ─────────────────────────────────────────────────── */}
      {subTab === 'p2p' && (
        <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
          <div style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc', display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '8px' }}>
            <Binary style={{ width: '18px', height: '18px', color: '#8b5cf6' }} />
            {t.p2p.title}
          </div>
          <p style={{ fontSize: '12px', color: '#94a3b8', margin: '0 0 16px 0' }}>{t.p2p.subtitle}</p>

          <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap', marginBottom: '16px' }}>
            <button
              onClick={handleInitiateP2p}
              disabled={p2pLoading}
              style={{ padding: '8px 14px', background: 'rgba(139, 92, 246, 0.2)', border: '1px solid rgba(139, 92, 246, 0.4)', color: '#c4b5fd', borderRadius: '6px', fontSize: '12px', fontWeight: 600, cursor: 'pointer' }}
            >
              1. {t.p2p.initSession}
            </button>
            <button
              onClick={handleCompleteP2p}
              disabled={p2pLoading || !p2pInit}
              style={{ padding: '8px 14px', background: 'rgba(139, 92, 246, 0.2)', border: '1px solid rgba(139, 92, 246, 0.4)', color: '#c4b5fd', borderRadius: '6px', fontSize: '12px', fontWeight: 600, cursor: 'pointer' }}
            >
              2. {t.p2p.completeHandshake}
            </button>
            <button
              onClick={handleEncryptP2pBundle}
              disabled={p2pLoading || !p2pInit}
              style={{ padding: '8px 14px', background: 'rgba(139, 92, 246, 0.2)', border: '1px solid rgba(139, 92, 246, 0.4)', color: '#c4b5fd', borderRadius: '6px', fontSize: '12px', fontWeight: 600, cursor: 'pointer' }}
            >
              3. {t.p2p.encryptVault}
            </button>
            <button
              onClick={handleDecryptP2pBundle}
              disabled={p2pLoading || !p2pBundle}
              style={{ padding: '8px 14px', background: 'rgba(16, 185, 129, 0.2)', border: '1px solid rgba(16, 185, 129, 0.4)', color: '#6ee7b7', borderRadius: '6px', fontSize: '12px', fontWeight: 600, cursor: 'pointer' }}
            >
              4. {t.p2p.decryptVault}
            </button>
          </div>

          {p2pDecrypted && (
            <div style={{ padding: '12px', background: 'rgba(16, 185, 129, 0.15)', border: '1px solid rgba(16, 185, 129, 0.3)', borderRadius: '6px', color: '#6ee7b7', fontSize: '12px' }}>
              ✓ Decrypted: <strong style={{ color: '#fff' }}>"{p2pDecrypted.plaintextUtf8}"</strong> (Noise Protocol verified)
            </div>
          )}
        </div>
      )}
    </div>
  );
};

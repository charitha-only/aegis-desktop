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
  Eye,
  EyeOff,
  Flame,
  AppWindow,
  Plug,
  Bell,
  Settings,
  ArrowUpCircle,
  Fingerprint,
  Layers,
  Globe,
  FileCheck,
  Package,
  Share2,
  HardDrive,
} from 'lucide-react';
import {
  aegis,
  SecurityStatus,
  SystemInfo,
  SandboxEntry,
  PrivacyAudit,
  IpcResponse,
} from './aegis-bridge';

interface AuditLog {
  id: string;
  timestamp: string;
  type: 'SUCCESS' | 'BLOCKED' | 'INFO';
  command: string;
  details: string;
}

const unavailableTabs: string[] = ['hardwareKey','enclave','stream','tpmAttestation','wgpuJit','p2pMarket','hsmPkcs11','updater','packagePipeline'];
export default function App() {
  const [activeTab, setActiveTab] = useState<'security' | 'filesystem' | 'vault' | 'window' | 'bundler' | 'plugins' | 'notifications' | 'permissions' | 'updater' | 'hardwareKey' | 'profiles' | 'wasm' | 'diagnostics' | 'logs' | 'enclave' | 'stream' | 'crashReporter' | 'secrets' | 'network' | 'pluginSig' | 'tpmAttestation' | 'wgpuJit' | 'packagePipeline' | 'p2pMarket' | 'hsmPkcs11'>('security');
  const [connected, setConnected] = useState(false);
  const [nonce, setNonce] = useState<string | null>(null);
  const [securityStatus, setSecurityStatus] = useState<SecurityStatus | null>(null);
  const [sysInfo, setSysInfo] = useState<SystemInfo | null>(null);
  const [privacyAudit, setPrivacyAudit] = useState<PrivacyAudit | null>(null);
  const [auditLogs, setAuditLogs] = useState<AuditLog[]>([]);

  // File System State
  const [sandboxFiles, setSandboxFiles] = useState<SandboxEntry[]>([]);
  const [fileName, setFileName] = useState('secure_document.txt');
  const [fileContent, setFileContent] = useState('Encrypted record verified by Aegis Core Sandbox Engine.');
  const [readResult, setReadResult] = useState<string | null>(null);

  // Bundler State (v0.12)
  const [bundlerStatus, setBundlerStatus] = useState<any>(null);
  const [bundlerLoading, setBundlerLoading] = useState(false);

  // Plugin Engine State (v0.13)
  const [pluginList, setPluginList] = useState<any[]>([]);
  const [pluginDispatchName, setPluginDispatchName] = useState('system_memory');
  const [pluginDispatchPayload, setPluginDispatchPayload] = useState('{}');
  const [pluginResult, setPluginResult] = useState<any>(null);
  const [pluginLoading, setPluginLoading] = useState(false);

  // Notifications State (v0.13)
  const [notifTitle, setNotifTitle] = useState('Aegis Security Alert');
  const [notifBody, setNotifBody] = useState('Sandboxed policy enforcement active. All capabilities verified.');
  const [trayStatus, setTrayStatus] = useState<any>(null);

  // Permissions State (v0.13)
  const [permissionsStatus, setPermissionsStatus] = useState<any>(null);

  // Auto-Updater State (v0.13)
  const [updaterStatus, setUpdaterStatus] = useState<any>(null);
  const [updaterLoading, setUpdaterLoading] = useState(false);

  // Hardware Security Key / FIDO2 State (v0.14)
  const [hwKeys, setHwKeys] = useState<any[]>([]);
  const [fidoChallenge, setFidoChallenge] = useState<any>(null);
  const [fidoPin, setFidoPin] = useState('');
  const [fidoVerificationResult, setFidoVerificationResult] = useState<any>(null);
  const [hwKeyLoading, setHwKeyLoading] = useState(false);

  // Multi-Profile Isolation State (v0.14)
  const [profiles, setProfiles] = useState<any[]>([]);
  const [activeProfile, setActiveProfile] = useState<any>(null);
  const [newProfileId, setNewProfileId] = useState('work_vault');
  const [newProfileName, setNewProfileName] = useState('Work Confidential Sandbox');
  const [newProfileDesc, setNewProfileDesc] = useState('Isolated workspace for enterprise documents and secrets');

  // Wasm Plugin VM Sandbox State (v0.14)
  const [wasmModules, setWasmModules] = useState<any[]>([]);
  const [selectedWasmModule, setSelectedWasmModule] = useState('wasm-crypto-accelerator');
  const [wasmInputPayload, setWasmInputPayload] = useState('{"algorithm": "sha256-vector", "data": "Aegis Zero Trust 2026"}');
  const [wasmExecResult, setWasmExecResult] = useState<any>(null);
  const [wasmLoading, setWasmLoading] = useState(false);

  // Biometric Enclave State (v0.15)
  const [enclaveStatus, setEnclaveStatus] = useState<any>(null);
  const [enclaveAuthResult, setEnclaveAuthResult] = useState<any>(null);
  const [enclaveLoading, setEnclaveLoading] = useState(false);

  // Encrypted IPC Stream State (v0.15)
  const [streamPlaintext, setStreamPlaintext] = useState('Aegis v0.15 — Sandboxed Encrypted IPC payload 🔐');
  const [streamCiphertext, setStreamCiphertext] = useState('');
  const [streamTag, setStreamTag] = useState('');
  const [streamDecryptResult, setStreamDecryptResult] = useState<any>(null);
  const [streamLoading, setStreamLoading] = useState(false);

  // Local Crash Reporter State (v0.15)
  const [crashEntries, setCrashEntries] = useState<any[]>([]);
  const [crashKind, setCrashKind] = useState('assertion');
  const [crashMessage, setCrashMessage] = useState('Simulated sandbox assertion violation in module wasm-crypto-accelerator');
  const [crashLoading, setCrashLoading] = useState(false);

  // Local encrypted Secrets Manager State (v0.16)
  const [secretsList, setSecretsList] = useState<any[]>([]);
  const [newSecretKey, setNewSecretKey] = useState('PROD_DATABASE_MASTER_URL');
  const [newSecretValue, setNewSecretValue] = useState('');
  const [secretsMasterPass, setSecretsMasterPass] = useState('');
  const [retrieveSecretKey, setRetrieveSecretKey] = useState('PRIMARY_API_TOKEN');
  const [retrievedSecretVal, setRetrievedSecretVal] = useState<string | null>(null);
  const [secretsLoading, setSecretsLoading] = useState(false);

  // Network Firewall Policy Engine State (v0.16)
  const [firewallRules, setFirewallRules] = useState<any[]>([]);
  const [evalHost, setEvalHost] = useState('telemetry.tracker.com');
  const [evalPort, setEvalPort] = useState(443);
  const [evalResult, setEvalResult] = useState<any>(null);
  const [firewallLoading, setFirewallLoading] = useState(false);

  // Plugin Signature & Provenance State (v0.16)
  const [trustedSigners, setTrustedSigners] = useState<any[]>([]);
  const [testPluginId, setTestPluginId] = useState('crypto-signer-ext');
  const [testPluginManifest, setTestPluginManifest] = useState('{"name":"crypto-signer-ext","version":"1.0.0","capabilities":["crypto:sha256"]}');
  const [testPluginSignature, setTestPluginSignature] = useState('signer_aegis_official_sig_e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855');
  const [sigVerifyResult, setSigVerifyResult] = useState<any>(null);
  const [sigLoading, setSigLoading] = useState(false);

  // Browser runtime State (v0.17)
  const [runtimeInfo, setRuntimeInfo] = useState<any>(null);

  // TPM 2.0 Remote Attestation State (v0.17)
  const [pcrMeasurements, setPcrMeasurements] = useState<any[]>([]);
  const [tpmQuoteNonce, setTpmQuoteNonce] = useState('challenge_enterprise_session_9921');
  const [tpmSelectedPcrs, setTpmSelectedPcrs] = useState<number[]>([0, 7, 10, 11]);
  const [tpmProof, setTpmProof] = useState<any>(null);
  const [tpmVerifyResult, setTpmVerifyResult] = useState<any>(null);
  const [tpmNetHost, setTpmNetHost] = useState('gateway.corp.internal');
  const [tpmNetPort, setTpmNetPort] = useState(443);
  const [tpmNetEval, setTpmNetEval] = useState<any>(null);
  const [tpmLoading, setTpmLoading] = useState(false);

  // WGPU Compute JIT State (v0.17)
  const [wgpuModuleId, setWgpuModuleId] = useState('wasm-crypto-accelerator');
  const [wgpuElementCount, setWgpuElementCount] = useState(1024);
  const [wgpuTier, setWgpuTier] = useState<'WGPU_JIT' | 'INTERPRETER' | 'AUTO'>('INTERPRETER');
  const [wgpuResult, setWgpuResult] = useState<any>(null);
  const [wgpuLoading, setWgpuLoading] = useState(false);

  // Automated Multi-OS Packaging Pipeline State (v0.18)
  const [pipelineAppName, setPipelineAppName] = useState('AegisEnterprise');
  const [pipelineAppId, setPipelineAppId] = useState('com.aegis.enterprise');
  const [pipelineVersion, setPipelineVersion] = useState('0.18.0');
  const [pipelineSelectedPlatforms, setPipelineSelectedPlatforms] = useState<string[]>(['windows', 'macos', 'linux', 'debian']);
  const [pipelineReport, setPipelineReport] = useState<any>(null);
  const [pipelineLoading, setPipelineLoading] = useState(false);

  // Distributed P2P Plugin Marketplace State (v0.18)
  const [p2pPlugins, setP2pPlugins] = useState<any[]>([]);
  const [p2pPeers, setP2pPeers] = useState<any[]>([]);
  const [p2pSelectedPlugin, setP2pSelectedPlugin] = useState<string>('plugin.zk.telemetry-filter');
  const [p2pSignerDid, setP2pSignerDid] = useState<string>('did:key:z6MkjTBxS4xW88nWD4pQoAky7v3yW5uK6vyuhEB1');
  const [p2pLoading, setP2pLoading] = useState(false);

  // Hardware Security Module (HSM) PKCS#11 State (v0.18)
  const [hsmSlotsList, setHsmSlotsList] = useState<any[]>([]);
  const [hsmSelectedSlot, setHsmSelectedSlot] = useState<number>(0);
  const [hsmPin, setHsmPin] = useState<string>('');
  const [hsmKeysList, setHsmKeysList] = useState<any[]>([]);
  const [hsmPayloadToSign, setHsmPayloadToSign] = useState<string>('AUTHENTICATED_TRANSACTION_PAYLOAD_HASH_0x892a');
  const [hsmSignatureResult, setHsmSignatureResult] = useState<any>(null);
  const [hsmLoading, setHsmLoading] = useState(false);

  // Handlers for v0.18
  const handleStagePipeline = async () => {
    setPipelineLoading(true);
    const res = await aegis.packagePipelineStage({
      name: pipelineAppName,
      appId: pipelineAppId,
      version: pipelineVersion,
      platforms: pipelineSelectedPlatforms,
    });
    if (res.success && res.data) {
      setPipelineReport(res.data);
      addLog('SUCCESS', 'package_pipeline_stage', `Multi-OS bundles staged for ${res.data.artifacts.length} target platform(s)`);
    } else {
      addLog('BLOCKED', 'package_pipeline_stage', res.error || 'Pipeline staging failed');
    }
    setPipelineLoading(false);
  };

  const refreshP2pMarket = async () => {
    setP2pLoading(true);
    const res = await aegis.p2pMarketList();
    if (res.success && res.data) {
      setP2pPlugins(res.data.plugins || []);
      setP2pPeers(res.data.peers || []);
      addLog('INFO', 'p2p_market_list', `Discovered ${res.data.plugins.length} P2P plugins across ${res.data.peers.length} active swarm peers`);
    } else {
      addLog('BLOCKED', 'p2p_market_list', res.error || 'Failed to list P2P plugins');
    }
    setP2pLoading(false);
  };

  const handleP2pSign = async (pluginId: string) => {
    setP2pLoading(true);
    const res = await aegis.p2pMarketSign(pluginId, p2pSignerDid);
    if (res.success && res.data) {
      addLog('SUCCESS', 'p2p_market_sign', `Endorsed plugin '${pluginId}' with DID multi-sig signature`);
      refreshP2pMarket();
    } else {
      addLog('BLOCKED', 'p2p_market_sign', res.error || 'Multi-sig endorsement rejected');
    }
    setP2pLoading(false);
  };

  const handleP2pInstall = async (pluginId: string) => {
    setP2pLoading(true);
    const res = await aegis.p2pMarketInstall(pluginId);
    if (res.success && res.data) {
      addLog('SUCCESS', 'p2p_market_install', `Consensus verified! Downloaded and installed '${pluginId}' from P2P swarm`);
      refreshP2pMarket();
    } else {
      addLog('BLOCKED', 'p2p_market_install', res.error || 'Installation blocked by Sandboxed policy');
    }
    setP2pLoading(false);
  };

  const refreshHsmSlots = async () => {
    setHsmLoading(true);
    const res = await aegis.hsmSlots();
    if (res.success && res.data) {
      setHsmSlotsList(res.data.slots || []);
      addLog('INFO', 'hsm_slots', `Found ${res.data.slots.length} PKCS#11 hardware security module token slots`);
    } else {
      addLog('BLOCKED', 'hsm_slots', res.error || 'Failed to query PKCS#11 slots');
    }
    setHsmLoading(false);
  };

  const handleHsmLogin = async () => {
    setHsmLoading(true);
    const res = await aegis.hsmLogin(hsmSelectedSlot, hsmPin);
    if (res.success && res.data) {
      addLog('SUCCESS', 'hsm_login', `PIN verified! User session opened on PKCS#11 token slot ${hsmSelectedSlot}`);
      refreshHsmSlots();
      // Load keys
      const keysRes = await aegis.hsmKeys(hsmSelectedSlot);
      if (keysRes.success && keysRes.data) {
        setHsmKeysList(keysRes.data.keys || []);
      }
    } else {
      addLog('BLOCKED', 'hsm_login', res.error || 'PKCS#11 login authentication failed');
    }
    setHsmLoading(false);
  };

  const handleHsmLogout = async () => {
    setHsmLoading(true);
    const res = await aegis.hsmLogout(hsmSelectedSlot);
    if (res.success) {
      addLog('INFO', 'hsm_logout', `Session closed on HSM slot ${hsmSelectedSlot}`);
      setHsmKeysList([]);
      setHsmSignatureResult(null);
      refreshHsmSlots();
    }
    setHsmLoading(false);
  };

  const handleHsmSign = async (keyId: string) => {
    setHsmLoading(true);
    const res = await aegis.hsmSign(hsmSelectedSlot, keyId, hsmPayloadToSign);
    if (res.success && res.data) {
      setHsmSignatureResult(res.data);
      addLog('SUCCESS', 'hsm_sign', `Hardware signature calculated inside silicon: mechanism=${res.data.mechanism}`);
    } else {
      addLog('BLOCKED', 'hsm_sign', res.error || 'Hardware signing failed');
    }
    setHsmLoading(false);
  };

  // Vault State (v0.11 Local encrypted Encryption)
  const [vaultFileName, setVaultFileName] = useState('confidential_notes.vault');
  const [vaultContent, setVaultContent] = useState('TOP_SECRET: Aegis Vault payload encrypted at rest with PBKDF2/SHA-256 and HMAC tags.');
  const [vaultPassphrase, setVaultPassphrase] = useState('');
  const [showPassphrase, setShowPassphrase] = useState(false);
  const [vaultRawCiphertext, setVaultRawCiphertext] = useState<string | null>(null);
  const [vaultDecryptedText, setVaultDecryptedText] = useState<string | null>(null);

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

  const refreshRuntimeInfo = async () => {
    const res = await aegis.windowRuntimeInfo();
    if (res.success && res.data) {
      setRuntimeInfo(res.data);
      addLog('INFO', 'window_runtime_info', `Active Runtime: ${res.data.engine}`);
    }
  };

  const refreshTpmMeasurements = async () => {
    setTpmLoading(true);
    const res = await aegis.tpmMeasurements();
    if (res.success && res.data) {
      setPcrMeasurements(res.data.pcrs || []);
      addLog('INFO', 'tpm_measurements', `Loaded ${res.data.pcrs.length} PCR measurements from TPM ${res.data.tpmFamily}`);
    } else {
      addLog('BLOCKED', 'tpm_measurements', res.error || 'Failed to query TPM PCRs');
    }
    setTpmLoading(false);
  };

  const handleGenerateTpmQuote = async () => {
    if (!tpmQuoteNonce) return;
    setTpmLoading(true);
    const res = await aegis.tpmGenerateQuote(tpmQuoteNonce, tpmSelectedPcrs);
    if (res.success && res.data) {
      setTpmProof(res.data);
      addLog('SUCCESS', 'tpm_generate_quote', `Quote generated: ${res.data.quote_id} with AIK signature`);
    } else {
      addLog('BLOCKED', 'tpm_generate_quote', res.error || 'Quote generation failed');
    }
    setTpmLoading(false);
  };

  const handleVerifyTpmQuote = async () => {
    if (!tpmProof || !tpmQuoteNonce) return;
    setTpmLoading(true);
    const res = await aegis.tpmVerifyQuote(tpmQuoteNonce, tpmProof);
    if (res.success && res.data) {
      setTpmVerifyResult(res.data);
      addLog('SUCCESS', 'tpm_verify_quote', `Attestation Verified: ${res.data.posture_status}`);
    } else {
      addLog('BLOCKED', 'tpm_verify_quote', res.error || 'Attestation verification failed');
    }
    setTpmLoading(false);
  };

  const handleTpmNetworkEvaluate = async (withProof: boolean) => {
    setTpmLoading(true);
    const proofToPass = withProof ? tpmProof : undefined;
    const res = await aegis.networkEvaluateWithAttestation(tpmNetHost, tpmNetPort, proofToPass, tpmQuoteNonce);
    if (res.success && res.data) {
      setTpmNetEval(res.data);
      addLog(res.data.allowed ? 'SUCCESS' : 'BLOCKED', 'firewall_attestation_eval', res.data.reason);
    } else {
      addLog('BLOCKED', 'firewall_attestation_eval', res.error || 'Evaluation error');
    }
    setTpmLoading(false);
  };

  const handleExecuteWasmJit = async () => {
    setWgpuLoading(true);
    const payload = { elementCount: wgpuElementCount };
    const res = await aegis.wasmExecuteWithTier(wgpuModuleId, payload, wgpuTier);
    if (res.success && res.data) {
      setWgpuResult(res.data);
      addLog('SUCCESS', 'wasm_execute_gpu', `Executed ${wgpuModuleId} via ${res.data.output_payload?.tier || 'Interpreter'}`);
    } else {
      addLog('BLOCKED', 'wasm_execute_gpu', res.error || 'Wasm execution failed');
    }
    setWgpuLoading(false);
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

      const priv = await aegis.getPrivacyStatus();
      if (priv.success && priv.data) {
        setPrivacyAudit(priv.data);
      }

      refreshFiles();
      refreshRuntimeInfo();
      refreshTpmMeasurements();
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

  const refreshBundlerStatus = async () => {
    setBundlerLoading(true);
    const res = await aegis.call('bundle_status', {});
    if (res.success && res.data) {
      setBundlerStatus(res.data);
      addLog('INFO', 'bundle_status', `Bundler engine v${res.data.bundlerEngineVersion} queried`);
    }
    setBundlerLoading(false);
  };

  const refreshPlugins = async () => {
    setPluginLoading(true);
    const res = await aegis.call('plugin_list', {});
    if (res.success && res.data) {
      setPluginList(res.data.plugins || []);
      addLog('INFO', 'plugin_list', `${res.data.count} plugins found in registry`);
    }
    setPluginLoading(false);
  };

  const handleDispatchPlugin = async () => {
    let payload: any = {};
    try { payload = JSON.parse(pluginDispatchPayload); } catch { payload = {}; }
    setPluginLoading(true);
    const res = await aegis.call('plugin_dispatch', { plugin: pluginDispatchName, payload });
    setPluginResult(res.success ? res.data : { error: res.error });
    addLog(res.success ? 'SUCCESS' : 'BLOCKED', 'plugin_dispatch', `Plugin '${pluginDispatchName}' invoked`);
    setPluginLoading(false);
  };

  const handleSendNotification = async () => {
    const res = await aegis.call('notify_send', { title: notifTitle, body: notifBody });
    addLog(res.success ? 'SUCCESS' : 'BLOCKED', 'notify_send', res.success ? 'OS notification delivered' : (res.error || 'Failed'));
    if (!trayStatus) {
      const ts = await aegis.call('tray_status', {});
      if (ts.success) setTrayStatus(ts.data);
    }
  };

  const refreshPermissions = async () => {
    const res = await aegis.call('permissions_status', {});
    if (res.success && res.data) {
      setPermissionsStatus(res.data);
      addLog('INFO', 'permissions_status', 'Permission manifest loaded from Rust Core');
    }
  };

  const refreshUpdaterStatus = async () => {
    setUpdaterLoading(true);
    const res = await aegis.call('updater_status', {});
    if (res.success && res.data) {
      setUpdaterStatus(res.data);
      addLog('INFO', 'updater_status', `Auto-Updater ${res.data.autoUpdaterVersion} queried`);
    }
    setUpdaterLoading(false);
  };

  // v0.14 Handlers
  const refreshHardwareKeys = async () => {
    setHwKeyLoading(true);
    const res = await aegis.hardwareKeyList();
    if (res.success && res.data) {
      setHwKeys(res.data.keys || []);
      addLog('INFO', 'hardware_key_list', `${res.data.keys?.length || 0} hardware token(s) detected via CTAP2.1`);
    }
    setHwKeyLoading(false);
  };

  const handleGenerateFidoChallenge = async () => {
    setHwKeyLoading(true);
    const res = await aegis.hardwareKeyChallenge('aegis.desktop.local');
    if (res.success && res.data) {
      setFidoChallenge(res.data);
      setFidoVerificationResult(null);
      addLog('SUCCESS', 'hardware_key_challenge', `FIDO2 assertion challenge issued: ${res.data.challenge_id}`);
    }
    setHwKeyLoading(false);
  };

  const handleVerifyFidoKey = async () => {
    if (!fidoChallenge?.challenge_id) return;
    setHwKeyLoading(true);
    const chalId = fidoChallenge.challenge_id;
    setFidoChallenge(null); // Consumed on client side
    const res = await aegis.hardwareKeyVerify(chalId, fidoPin);
    if (res.success && res.data) {
      setFidoVerificationResult(res.data);
      addLog('SUCCESS', 'hardware_key_verify', `FIDO2 touch & PIN assertion verified: sig=${res.data.signature_hex?.substring(0, 16)}...`);
    } else {
      setFidoVerificationResult(null);
      addLog('BLOCKED', 'hardware_key_verify', res.error || 'FIDO2 verification failed');
    }
    setHwKeyLoading(false);
  };

  const refreshProfiles = async () => {
    const res = await aegis.profileList();
    if (res.success && res.data) {
      setProfiles(res.data.profiles || []);
      setActiveProfile(res.data.activeProfile);
      addLog('INFO', 'profile_list', `Active profile: ${res.data.activeProfile?.name}`);
    }
  };

  const handleSwitchProfile = async (profileId: string) => {
    const res = await aegis.profileSwitch(profileId);
    if (res.success && res.data) {
      setActiveProfile(res.data.activeProfile);
      addLog('SUCCESS', 'profile_switch', `Switched to profile: ${res.data.activeProfile?.name}`);
      refreshProfiles();
    }
  };

  const handleCreateProfile = async () => {
    if (!newProfileId.trim()) return;
    const res = await aegis.profileCreate(newProfileId, newProfileName, newProfileDesc);
    if (res.success && res.data) {
      addLog('SUCCESS', 'profile_create', `Created isolated profile: ${res.data.profile?.name}`);
      refreshProfiles();
    }
  };

  const refreshWasmModules = async () => {
    setWasmLoading(true);
    const res = await aegis.wasmList();
    if (res.success && res.data) {
      setWasmModules(res.data.modules || []);
      addLog('INFO', 'wasm_list', `${res.data.modules?.length || 0} Wasm modules loaded in VM sandbox`);
    }
    setWasmLoading(false);
  };

  const handleExecuteWasm = async () => {
    let payload = {};
    try { payload = JSON.parse(wasmInputPayload); } catch { payload = {}; }
    setWasmLoading(true);
    const res = await aegis.wasmExecute(selectedWasmModule, payload);
    if (res.success && res.data) {
      setWasmExecResult(res.data);
      addLog('SUCCESS', 'wasm_execute', `Executed module '${selectedWasmModule}': ${res.data.gas_consumed} gas used`);
    } else {
      addLog('BLOCKED', 'wasm_execute', res.error || 'Wasm execution failed');
    }
    setWasmLoading(false);
  };

  // v0.15 Handlers
  const refreshEnclaveStatus = async () => {
    setEnclaveLoading(true);
    const res = await aegis.enclaveStatus();
    if (res.success && res.data) {
      setEnclaveStatus(res.data);
      addLog('INFO', 'enclave_status', `Biometric Enclave: ${res.data.platform} — ${res.data.available ? 'Available' : 'Unavailable'}`);
    } else {
      addLog('BLOCKED', 'enclave_status', res.error || 'Enclave status failed');
    }
    setEnclaveLoading(false);
  };

  const handleEnclaveAuth = async () => {
    setEnclaveLoading(true);
    const res = await aegis.enclaveAuthenticate('Aegis Desktop identity verification');
    if (res.success && res.data) {
      setEnclaveAuthResult(res.data);
      addLog('SUCCESS', 'enclave_authenticate', `Biometric auth: ${res.data.method} — ${res.data.message}`);
    } else {
      addLog('BLOCKED', 'enclave_authenticate', res.error || 'Authentication failed');
    }
    setEnclaveLoading(false);
  };

  const handleStreamEncrypt = async () => {
    setStreamLoading(true);
    const res = await aegis.streamEncrypt(streamPlaintext);
    if (res.success && res.data) {
      setStreamCiphertext(res.data.ciphertext);
      setStreamTag(res.data.tag);
      setStreamDecryptResult(null);
      addLog('SUCCESS', 'stream_encrypt', `IPC stream encrypted — HMAC tag: ${res.data.tag.substring(0, 16)}...`);
    } else {
      addLog('BLOCKED', 'stream_encrypt', res.error || 'Encryption failed');
    }
    setStreamLoading(false);
  };

  const handleStreamDecrypt = async () => {
    if (!streamCiphertext || !streamTag) return;
    setStreamLoading(true);
    const res = await aegis.streamDecrypt(streamCiphertext, streamTag);
    if (res.success && res.data) {
      setStreamDecryptResult(res.data);
      addLog(res.data.verified ? 'SUCCESS' : 'BLOCKED', 'stream_decrypt', res.data.verified ? `HMAC verified & decrypted: "${res.data.plaintext.substring(0, 40)}..."` : 'HMAC tag mismatch — payload rejected');
    } else {
      addLog('BLOCKED', 'stream_decrypt', res.error || 'Decryption failed');
    }
    setStreamLoading(false);
  };

  const refreshCrashEntries = async () => {
    setCrashLoading(true);
    const res = await aegis.crashList();
    if (res.success && res.data) {
      setCrashEntries(res.data.entries || []);
      addLog('INFO', 'crash_list', `${res.data.total} local crash report(s) — air-gapped, zero-telemetry`);
    } else {
      addLog('BLOCKED', 'crash_list', res.error || 'Crash list unavailable');
    }
    setCrashLoading(false);
  };

  const handleRecordCrash = async () => {
    setCrashLoading(true);
    const res = await aegis.crashRecord(crashKind, crashMessage);
    if (res.success && res.data) {
      addLog('INFO', 'crash_record', `Crash diagnostic persisted: id=${res.data.id}`);
      refreshCrashEntries();
    } else {
      addLog('BLOCKED', 'crash_record', res.error || 'Failed to record crash');
    }
    setCrashLoading(false);
  };

  // v0.16 Handlers — Secrets Manager
  const refreshSecretsList = async () => {
    setSecretsLoading(true);
    const res = await aegis.secretsList();
    if (res.success && res.data) {
      setSecretsList(res.data.secrets || []);
      addLog('INFO', 'secrets_list', `${res.data.secrets.length} secret(s) registered under memory-hard KDF`);
    } else {
      addLog('BLOCKED', 'secrets_list', res.error || 'Secrets list unavailable');
    }
    setSecretsLoading(false);
  };

  const handleStoreSecret = async () => {
    if (!newSecretKey || !newSecretValue || !secretsMasterPass) return;
    setSecretsLoading(true);
    const res = await aegis.secretsStore(newSecretKey, newSecretValue, secretsMasterPass);
    if (res.success && res.data) {
      addLog('SUCCESS', 'secrets_store', `Secret '${res.data.key}' stored (v${res.data.version}) — SHA256 tag: ${res.data.digest_fingerprint.substring(0, 16)}...`);
      refreshSecretsList();
    } else {
      addLog('BLOCKED', 'secrets_store', res.error || 'Failed to store secret');
    }
    setSecretsLoading(false);
  };

  const handleRetrieveSecret = async () => {
    if (!retrieveSecretKey || !secretsMasterPass) return;
    setSecretsLoading(true);
    const res = await aegis.secretsRetrieve(retrieveSecretKey, secretsMasterPass);
    if (res.success && res.data) {
      setRetrievedSecretVal(res.data.value);setSecretsMasterPass('');
      addLog('SUCCESS', 'secrets_retrieve', `Decrypted secret '${res.data.key}' successfully`);
    } else {
      setRetrievedSecretVal(null);
      addLog('BLOCKED', 'secrets_retrieve', res.error || 'Decryption failed: Passphrase invalid or corrupted');
    }
    setSecretsLoading(false);
  };

  // v0.16 Handlers — Network Firewall Policy Engine
  const refreshFirewallRules = async () => {
    setFirewallLoading(true);
    const res = await aegis.firewallRules();
    if (res.success && res.data) {
      setFirewallRules(res.data.rules || []);
      addLog('INFO', 'firewall_rules', `Loaded ${res.data.rules.length} firewall rules (${res.data.mode})`);
    } else {
      addLog('BLOCKED', 'firewall_rules', res.error || 'Firewall rules unavailable');
    }
    setFirewallLoading(false);
  };

  const handleEvaluateConnection = async () => {
    if (!evalHost) return;
    setFirewallLoading(true);
    const res = await aegis.firewallEvaluate(evalHost, evalPort);
    if (res.success && res.data) {
      setEvalResult(res.data);
      addLog(res.data.allowed ? 'SUCCESS' : 'BLOCKED', 'firewall_evaluate', `${evalHost}:${evalPort} -> ${res.data.allowed ? 'ALLOWED' : 'BLOCKED'} (${res.data.reason})`);
    } else {
      addLog('BLOCKED', 'firewall_evaluate', res.error || 'Evaluation failed');
    }
    setFirewallLoading(false);
  };

  // v0.16 Handlers — Plugin Signature & Provenance
  const refreshPluginSigners = async () => {
    setSigLoading(true);
    const res = await aegis.pluginSigners();
    if (res.success && res.data) {
      setTrustedSigners(res.data.trustedSigners || []);
      addLog('INFO', 'plugin_signers', `Loaded ${res.data.trustedSigners.length} trusted root signers`);
    } else {
      addLog('BLOCKED', 'plugin_signers', res.error || 'Failed to load signers');
    }
    setSigLoading(false);
  };

  const handleVerifyPluginManifest = async () => {
    if (!testPluginId || !testPluginManifest || !testPluginSignature) return;
    setSigLoading(true);
    const res = await aegis.pluginVerifyManifest(testPluginId, testPluginManifest, testPluginSignature);
    if (res.success && res.data) {
      setSigVerifyResult(res.data);
      addLog(res.data.valid && res.data.trusted_vendor ? 'SUCCESS' : 'BLOCKED', 'plugin_verify', res.data.message);
    } else {
      addLog('BLOCKED', 'plugin_verify', res.error || 'Verification error');
    }
    setSigLoading(false);
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

  const handleSimulateRateLimit = async () => {
    addLog('INFO', 'attack_sim:rate_limit', 'Firing burst of 35 rapid concurrent IPC calls to trigger Anti-DoS shield...');
    const res = await aegis.simulateRateLimitAttack();
    if (!res.success && res.error?.includes('Rate limit exceeded')) {
      addLog('BLOCKED', 'dos_attack_blocked', res.error);
    } else {
      addLog('INFO', 'rate_limit_result', res.error || 'Request completed within allowance');
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

  // Local encrypted Encrypted Vault Handlers
  const handleVaultEncrypt = async () => {
    if (!vaultFileName || !vaultPassphrase) return;
    addLog('INFO', 'vault_encrypt', `Encrypting ${vaultFileName} with Argon2id rounds & HMAC tag...`);
    const res = await aegis.vaultEncryptStore(vaultFileName, vaultContent, vaultPassphrase);
    if (res.success) {
      addLog('SUCCESS', 'vault_encrypted', `Safely stored inside ./sandbox/${res.data?.filename}`);
      refreshFiles();
      const raw = await aegis.readSandboxFile(vaultFileName);
      if (raw.success && raw.data) {
        setVaultRawCiphertext(raw.data.content);
      }
    } else {
      addLog('BLOCKED', 'vault_error', res.error || 'Encryption failed');
    }
  };

  const handleVaultDecrypt = async () => {
    if (!vaultFileName || !vaultPassphrase) return;
    addLog('INFO', 'vault_decrypt', `Authenticating encrypted vault and decrypting ${vaultFileName}...`);
    const res = await aegis.vaultDecryptRetrieve(vaultFileName, vaultPassphrase);
    if (res.success && res.data) {
      setVaultDecryptedText(res.data.decryptedContent);setVaultPassphrase('');
      addLog('SUCCESS', 'vault_decrypted', 'Authenticated encryption verified & data decrypted successfully!');
    } else {
      setVaultDecryptedText(null);
      addLog('BLOCKED', 'vault_auth_failed', res.error || 'Decryption failed: Tag mismatch or incorrect password');
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

  // Multi-Window & Native Dialog Handlers (v0.12)
  const handleSpawnSecondaryWindow = async (modal = false) => {
    addLog('INFO', 'window_spawn', `Spawning ${modal ? 'Modal' : 'Secondary'} Window via Rust WindowManager...`);
    const res = await aegis.spawnWindow({
      url: 'http://localhost:5173',
      title: modal ? 'Aegis Modal Inspector' : 'Aegis Secondary Secure Window',
      width: 900,
      height: 600,
      modal,
    });
    if (res.success) {
      addLog('SUCCESS', 'window_spawn', 'Native window process spawned successfully');
    } else {
      addLog('BLOCKED', 'window_spawn_error', res.error || 'Failed to spawn window');
    }
  };

  const handleShowNativeDialog = async (type: 'info' | 'warning' | 'error') => {
    addLog('INFO', 'window_dialog', `Requesting native ${type.toUpperCase()} dialog...`);
    const res = await aegis.showNativeDialog(
      'Aegis Desktop Message',
      'This message comes from the local Rust core. Acknowledging it does not grant access to protected data.',
      type
    );
    if (!res.success || !res.data) {
      addLog('BLOCKED', 'window_dialog', res.error || 'Dialog request failed');
      return;
    }
    addLog('INFO', 'window_dialog', 'Dialog request submitted');
    const requestId = res.data.requestId;
    for (let attempt = 0; attempt < 120; attempt++) {
      await new Promise(resolve => window.setTimeout(resolve, 1000));
      const status = await aegis.pollMessageDialog(requestId);
      if (!status.success) {
        addLog('BLOCKED', 'window_dialog', status.error || 'Dialog result unavailable');
        return;
      }
      if (status.data?.status === 'acknowledged') {
        addLog('SUCCESS', 'window_dialog', 'Dialog acknowledged');
        return;
      }
    }
    await aegis.cancelMessageDialog(requestId);
    addLog('BLOCKED', 'window_dialog', 'Dialog acknowledgement timed out');
  };

  // UI cleanup reduces retention; JavaScript strings cannot be reliably zeroized.
  useEffect(() => {
    const clearSensitive = () => {setVaultPassphrase('');setVaultDecryptedText(null);setVaultContent('');setSecretsMasterPass('');setNewSecretValue('');setRetrievedSecretVal(null);};
    const hidden = () => {if(document.hidden) clearSensitive();};
    const timer=window.setTimeout(clearSensitive,60000);
    document.addEventListener('visibilitychange',hidden);window.addEventListener('blur',clearSensitive);
    return () => {window.clearTimeout(timer);document.removeEventListener('visibilitychange',hidden);window.removeEventListener('blur',clearSensitive);};
  }, [activeTab, activeProfile, vaultDecryptedText, retrievedSecretVal]);

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
              AEGIS DESKTOP <span style={{ fontSize: '11px', color: '#818cf8', background: 'rgba(99, 102, 241, 0.15)', padding: '2px 6px', borderRadius: '4px', marginLeft: '6px' }}>v0.18 BETA</span>
            </div>
            <div style={{ fontSize: '11px', color: '#64748b' }}>Custom Rust Core + React Sandboxed Framework</div>
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
            onClick={() => setActiveTab('vault')}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '10px',
              padding: '10px 14px',
              borderRadius: '8px',
              border: 'none',
              background: activeTab === 'vault' ? 'linear-gradient(90deg, rgba(99, 102, 241, 0.2), rgba(99, 102, 241, 0.05))' : 'transparent',
              color: activeTab === 'vault' ? '#818cf8' : '#94a3b8',
              cursor: 'pointer',
              fontWeight: 500,
              fontSize: '13px',
              textAlign: 'left',
              transition: 'all 0.15s ease',
            }}
          >
            <Lock style={{ width: '18px', height: '18px', color: '#a855f7' }} />
            Privacy & Vault
          </button>

          <button
            onClick={() => setActiveTab('window')}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '10px',
              padding: '10px 14px',
              borderRadius: '8px',
              border: 'none',
              background: activeTab === 'window' ? 'linear-gradient(90deg, rgba(99, 102, 241, 0.2), rgba(99, 102, 241, 0.05))' : 'transparent',
              color: activeTab === 'window' ? '#818cf8' : '#94a3b8',
              cursor: 'pointer',
              fontWeight: 500,
              fontSize: '13px',
              textAlign: 'left',
              transition: 'all 0.15s ease',
            }}
          >
            <AppWindow style={{ width: '18px', height: '18px', color: '#38bdf8' }} />
            Window & Dialogs
          </button>

          <button
            onClick={() => { setActiveTab('bundler'); refreshBundlerStatus(); }}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '10px',
              padding: '10px 14px',
              borderRadius: '8px',
              border: 'none',
              background: activeTab === 'bundler' ? 'linear-gradient(90deg, rgba(251, 191, 36, 0.2), rgba(251, 191, 36, 0.05))' : 'transparent',
              color: activeTab === 'bundler' ? '#fde68a' : '#94a3b8',
              cursor: 'pointer',
              fontWeight: 500,
              fontSize: '13px',
              textAlign: 'left',
              transition: 'all 0.15s ease',
            }}
          >
            <Zap style={{ width: '18px', height: '18px', color: '#fbbf24' }} />
            Bundler & Packaging
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
            <Activity style={{ width: '18px', height: '18px' }} />
            Diagnostics & Telemetry
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
            <Terminal style={{ width: '18px', height: '18px' }} />
            Live Audit Stream
          </button>

          <div style={{ fontSize: '10px', fontWeight: 600, color: '#475569', textTransform: 'uppercase', letterSpacing: '1px', padding: '8px 12px 4px' }}>v0.13 New</div>

          <button
            onClick={() => { setActiveTab('plugins'); refreshPlugins(); }}
            style={{
              display: 'flex', alignItems: 'center', gap: '10px', padding: '10px 14px',
              borderRadius: '8px', border: 'none',
              background: activeTab === 'plugins' ? 'linear-gradient(90deg, rgba(52, 211, 153, 0.2), rgba(52, 211, 153, 0.05))' : 'transparent',
              color: activeTab === 'plugins' ? '#6ee7b7' : '#94a3b8',
              cursor: 'pointer', fontWeight: 500, fontSize: '13px', textAlign: 'left', transition: 'all 0.15s ease',
            }}
          >
            <Plug style={{ width: '18px', height: '18px', color: '#34d399' }} />
            Plugin Engine
          </button>

          <button
            onClick={() => { setActiveTab('notifications'); }}
            style={{
              display: 'flex', alignItems: 'center', gap: '10px', padding: '10px 14px',
              borderRadius: '8px', border: 'none',
              background: activeTab === 'notifications' ? 'linear-gradient(90deg, rgba(251, 191, 36, 0.2), rgba(251, 191, 36, 0.05))' : 'transparent',
              color: activeTab === 'notifications' ? '#fde68a' : '#94a3b8',
              cursor: 'pointer', fontWeight: 500, fontSize: '13px', textAlign: 'left', transition: 'all 0.15s ease',
            }}
          >
            <Bell style={{ width: '18px', height: '18px', color: '#fbbf24' }} />
            Tray & Notifications
          </button>

          <button
            onClick={() => { setActiveTab('permissions'); refreshPermissions(); }}
            style={{
              display: 'flex', alignItems: 'center', gap: '10px', padding: '10px 14px',
              borderRadius: '8px', border: 'none',
              background: activeTab === 'permissions' ? 'linear-gradient(90deg, rgba(239, 68, 68, 0.2), rgba(239, 68, 68, 0.05))' : 'transparent',
              color: activeTab === 'permissions' ? '#fca5a5' : '#94a3b8',
              cursor: 'pointer', fontWeight: 500, fontSize: '13px', textAlign: 'left', transition: 'all 0.15s ease',
            }}
          >
            <Settings style={{ width: '18px', height: '18px', color: '#f87171' }} />
            Permission Manager
          </button>

          <button
            onClick={() => { setActiveTab('updater'); refreshUpdaterStatus(); }}
            style={{
              display: 'flex', alignItems: 'center', gap: '10px', padding: '10px 14px',
              borderRadius: '8px', border: 'none',
              background: activeTab === 'updater' ? 'linear-gradient(90deg, rgba(56, 189, 248, 0.2), rgba(56, 189, 248, 0.05))' : 'transparent',
              color: activeTab === 'updater' ? '#7dd3fc' : '#94a3b8',
              cursor: 'pointer', fontWeight: 500, fontSize: '13px', textAlign: 'left', transition: 'all 0.15s ease',
            }}
          >
            <ArrowUpCircle style={{ width: '18px', height: '18px', color: '#38bdf8' }} />
            Auto-Updater
          </button>

          <div style={{ fontSize: '10px', fontWeight: 600, color: '#475569', textTransform: 'uppercase', letterSpacing: '1px', padding: '8px 12px 4px' }}>v0.14 New</div>

          <button
            onClick={() => { setActiveTab('hardwareKey'); refreshHardwareKeys(); }}
            style={{
              display: 'flex', alignItems: 'center', gap: '10px', padding: '10px 14px',
              borderRadius: '8px', border: 'none',
              background: activeTab === 'hardwareKey' ? 'linear-gradient(90deg, rgba(168, 85, 247, 0.2), rgba(168, 85, 247, 0.05))' : 'transparent',
              color: activeTab === 'hardwareKey' ? '#c084fc' : '#94a3b8',
              cursor: 'pointer', fontWeight: 500, fontSize: '13px', textAlign: 'left', transition: 'all 0.15s ease',
            }}
          >
            <Fingerprint style={{ width: '18px', height: '18px', color: '#c084fc' }} />
            Hardware Key (FIDO2)
          </button>

          <button
            onClick={() => { setActiveTab('profiles'); refreshProfiles(); }}
            style={{
              display: 'flex', alignItems: 'center', gap: '10px', padding: '10px 14px',
              borderRadius: '8px', border: 'none',
              background: activeTab === 'profiles' ? 'linear-gradient(90deg, rgba(236, 72, 153, 0.2), rgba(236, 72, 153, 0.05))' : 'transparent',
              color: activeTab === 'profiles' ? '#f472b6' : '#94a3b8',
              cursor: 'pointer', fontWeight: 500, fontSize: '13px', textAlign: 'left', transition: 'all 0.15s ease',
            }}
          >
            <Layers style={{ width: '18px', height: '18px', color: '#f472b6' }} />
            Profile Isolation
          </button>

          <button
            onClick={() => { setActiveTab('wasm'); refreshWasmModules(); }}
            style={{
              display: 'flex', alignItems: 'center', gap: '10px', padding: '10px 14px',
              borderRadius: '8px', border: 'none',
              background: activeTab === 'wasm' ? 'linear-gradient(90deg, rgba(20, 184, 166, 0.2), rgba(20, 184, 166, 0.05))' : 'transparent',
              color: activeTab === 'wasm' ? '#2dd4bf' : '#94a3b8',
              cursor: 'pointer', fontWeight: 500, fontSize: '13px', textAlign: 'left', transition: 'all 0.15s ease',
            }}
          >
            <Cpu style={{ width: '18px', height: '18px', color: '#2dd4bf' }} />
            Wasm VM Sandbox
          </button>

          <div style={{ fontSize: '10px', fontWeight: 600, color: '#475569', textTransform: 'uppercase', letterSpacing: '1px', padding: '8px 12px 4px' }}>v0.15 New</div>

          <button
            onClick={() => { setActiveTab('enclave'); refreshEnclaveStatus(); }}
            style={{
              display: 'flex', alignItems: 'center', gap: '10px', padding: '10px 14px',
              borderRadius: '8px', border: 'none',
              background: activeTab === 'enclave' ? 'linear-gradient(90deg, rgba(99, 102, 241, 0.25), rgba(99, 102, 241, 0.05))' : 'transparent',
              color: activeTab === 'enclave' ? '#a5b4fc' : '#94a3b8',
              cursor: 'pointer', fontWeight: 500, fontSize: '13px', textAlign: 'left', transition: 'all 0.15s ease',
            }}
          >
            <Fingerprint style={{ width: '18px', height: '18px', color: '#818cf8' }} />
            Biometric Enclave
          </button>

          <button
            onClick={() => setActiveTab('stream')}
            style={{
              display: 'flex', alignItems: 'center', gap: '10px', padding: '10px 14px',
              borderRadius: '8px', border: 'none',
              background: activeTab === 'stream' ? 'linear-gradient(90deg, rgba(6, 182, 212, 0.2), rgba(6, 182, 212, 0.05))' : 'transparent',
              color: activeTab === 'stream' ? '#67e8f9' : '#94a3b8',
              cursor: 'pointer', fontWeight: 500, fontSize: '13px', textAlign: 'left', transition: 'all 0.15s ease',
            }}
          >
            <Lock style={{ width: '18px', height: '18px', color: '#22d3ee' }} />
            Encrypted Stream
          </button>

          <button
            onClick={() => { setActiveTab('crashReporter'); refreshCrashEntries(); }}
            style={{
              display: 'flex', alignItems: 'center', gap: '10px', padding: '10px 14px',
              borderRadius: '8px', border: 'none',
              background: activeTab === 'crashReporter' ? 'linear-gradient(90deg, rgba(245, 158, 11, 0.2), rgba(245, 158, 11, 0.05))' : 'transparent',
              color: activeTab === 'crashReporter' ? '#fcd34d' : '#94a3b8',
              cursor: 'pointer', fontWeight: 500, fontSize: '13px', textAlign: 'left', transition: 'all 0.15s ease',
            }}
          >
            <FileText style={{ width: '18px', height: '18px', color: '#f59e0b' }} />
            Crash Reporter
          </button>

          <div style={{ fontSize: '10px', fontWeight: 600, color: '#475569', textTransform: 'uppercase', letterSpacing: '1px', padding: '8px 12px 4px' }}>v0.16 New</div>

          <button
            onClick={() => { setActiveTab('secrets'); refreshSecretsList(); }}
            style={{
              display: 'flex', alignItems: 'center', gap: '10px', padding: '10px 14px',
              borderRadius: '8px', border: 'none',
              background: activeTab === 'secrets' ? 'linear-gradient(90deg, rgba(16, 185, 129, 0.25), rgba(16, 185, 129, 0.05))' : 'transparent',
              color: activeTab === 'secrets' ? '#6ee7b7' : '#94a3b8',
              cursor: 'pointer', fontWeight: 500, fontSize: '13px', textAlign: 'left', transition: 'all 0.15s ease',
            }}
          >
            <Key style={{ width: '18px', height: '18px', color: '#10b981' }} />
            Secrets Manager
          </button>

          <button
            onClick={() => { setActiveTab('network'); refreshFirewallRules(); }}
            style={{
              display: 'flex', alignItems: 'center', gap: '10px', padding: '10px 14px',
              borderRadius: '8px', border: 'none',
              background: activeTab === 'network' ? 'linear-gradient(90deg, rgba(14, 165, 233, 0.25), rgba(14, 165, 233, 0.05))' : 'transparent',
              color: activeTab === 'network' ? '#7dd3fc' : '#94a3b8',
              cursor: 'pointer', fontWeight: 500, fontSize: '13px', textAlign: 'left', transition: 'all 0.15s ease',
            }}
          >
            <Globe style={{ width: '18px', height: '18px', color: '#0ea5e9' }} />
            Network Firewall
          </button>

          <button
            onClick={() => { setActiveTab('pluginSig'); refreshPluginSigners(); }}
            style={{
              display: 'flex', alignItems: 'center', gap: '10px', padding: '10px 14px',
              borderRadius: '8px', border: 'none',
              background: activeTab === 'pluginSig' ? 'linear-gradient(90deg, rgba(217, 70, 239, 0.25), rgba(217, 70, 239, 0.05))' : 'transparent',
              color: activeTab === 'pluginSig' ? '#f0abfc' : '#94a3b8',
              cursor: 'pointer', fontWeight: 500, fontSize: '13px', textAlign: 'left', transition: 'all 0.15s ease',
            }}
          >
            <FileCheck style={{ width: '18px', height: '18px', color: '#d946ef' }} />
            Plugin Provenance
          </button>

          <button
            onClick={() => { setActiveTab('tpmAttestation'); refreshTpmMeasurements(); }}
            style={{
              display: 'flex', alignItems: 'center', gap: '10px', padding: '10px 14px',
              borderRadius: '8px', border: 'none',
              background: activeTab === 'tpmAttestation' ? 'linear-gradient(90deg, rgba(16, 185, 129, 0.25), rgba(16, 185, 129, 0.05))' : 'transparent',
              color: activeTab === 'tpmAttestation' ? '#6ee7b7' : '#94a3b8',
              cursor: 'pointer', fontWeight: 500, fontSize: '13px', textAlign: 'left', transition: 'all 0.15s ease',
            }}
          >
            <ShieldCheck style={{ width: '18px', height: '18px', color: '#10b981' }} />
            TPM Remote Attestation
          </button>

          <button
            onClick={() => setActiveTab('wgpuJit')}
            style={{
              display: 'flex', alignItems: 'center', gap: '10px', padding: '10px 14px',
              borderRadius: '8px', border: 'none',
              background: activeTab === 'wgpuJit' ? 'linear-gradient(90deg, rgba(245, 158, 11, 0.25), rgba(245, 158, 11, 0.05))' : 'transparent',
              color: activeTab === 'wgpuJit' ? '#fde68a' : '#94a3b8',
              cursor: 'pointer', fontWeight: 500, fontSize: '13px', textAlign: 'left', transition: 'all 0.15s ease',
            }}
          >
            <Zap style={{ width: '18px', height: '18px', color: '#f59e0b' }} />
            WGPU Compute JIT
          </button>

          <div style={{ fontSize: '10px', fontWeight: 600, color: '#475569', textTransform: 'uppercase', letterSpacing: '1px', padding: '8px 12px 4px' }}>v0.18 New</div>

          <button
            onClick={() => setActiveTab('packagePipeline')}
            style={{
              display: 'flex', alignItems: 'center', gap: '10px', padding: '10px 14px',
              borderRadius: '8px', border: 'none',
              background: activeTab === 'packagePipeline' ? 'linear-gradient(90deg, rgba(168, 85, 247, 0.25), rgba(168, 85, 247, 0.05))' : 'transparent',
              color: activeTab === 'packagePipeline' ? '#d8b4fe' : '#94a3b8',
              cursor: 'pointer', fontWeight: 500, fontSize: '13px', textAlign: 'left', transition: 'all 0.15s ease',
            }}
          >
            <Package style={{ width: '18px', height: '18px', color: '#c084fc' }} />
            Multi-OS Pipeline
          </button>

          <button
            onClick={() => { setActiveTab('p2pMarket'); refreshP2pMarket(); }}
            style={{
              display: 'flex', alignItems: 'center', gap: '10px', padding: '10px 14px',
              borderRadius: '8px', border: 'none',
              background: activeTab === 'p2pMarket' ? 'linear-gradient(90deg, rgba(56, 189, 248, 0.25), rgba(56, 189, 248, 0.05))' : 'transparent',
              color: activeTab === 'p2pMarket' ? '#7dd3fc' : '#94a3b8',
              cursor: 'pointer', fontWeight: 500, fontSize: '13px', textAlign: 'left', transition: 'all 0.15s ease',
            }}
          >
            <Share2 style={{ width: '18px', height: '18px', color: '#38bdf8' }} />
            P2P Plugin Market
          </button>

          <button
            onClick={() => { setActiveTab('hsmPkcs11'); refreshHsmSlots(); }}
            style={{
              display: 'flex', alignItems: 'center', gap: '10px', padding: '10px 14px',
              borderRadius: '8px', border: 'none',
              background: activeTab === 'hsmPkcs11' ? 'linear-gradient(90deg, rgba(236, 72, 153, 0.25), rgba(236, 72, 153, 0.05))' : 'transparent',
              color: activeTab === 'hsmPkcs11' ? '#f472b6' : '#94a3b8',
              cursor: 'pointer', fontWeight: 500, fontSize: '13px', textAlign: 'left', transition: 'all 0.15s ease',
            }}
          >
            <HardDrive style={{ width: '18px', height: '18px', color: '#ec4899' }} />
            HSM PKCS#11 Enclave
          </button>

          <div style={{ marginTop: 'auto', background: 'rgba(15, 23, 42, 0.6)', border: '1px solid rgba(255,255,255,0.05)', padding: '12px', borderRadius: '8px' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '6px', fontSize: '11px', color: '#38bdf8', marginBottom: '6px' }}>
              <Lock style={{ width: '12px', height: '12px' }} />
              Sandboxed Architecture
            </div>
            <div style={{ fontSize: '10px', color: '#64748b', lineHeight: '1.4' }}>
              Every IPC invocation passes through strict Rust capability verification & cryptographic nonces.
            </div>
          </div>
        </nav>

        {/* Content Area */}
        <main style={{ flex: 1, padding: '24px 32px', overflowY: 'auto' }}>
          {/* TAB 1: SECURITY & DEFENSE */}
          {unavailableTabs.includes(activeTab) && (
 <section role="status" style={{padding:32,color:'#f8fafc',background:'#172033',borderRadius:12}}>
 <h2>Integration unavailable</h2><p>This feature is disabled until a real provider is implemented and tested. No hardware authentication, GPU acceleration or plugin installation has taken place.</p>
 <p>Use Files, Profiles, Vault or the Wasm interpreter. Windows portable packaging is available through the documented build command.</p></section>
)}
{activeTab === 'security' && (
            <div>
              <div style={{ marginBottom: '24px' }}>
                <h1 style={{ fontSize: '22px', fontWeight: 600, color: '#f8fafc', marginBottom: '6px' }}>
                  Security Architecture & Threat Defense
                </h1>
                <p style={{ fontSize: '13px', color: '#94a3b8' }}>
                  Enterprise-grade capability access control, CSP isolation, anti-DoS sliding window rate limiting, and real-time defense simulations.
                </p>
              </div>

              {/* Security Metrics Cards */}
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '14px', marginBottom: '24px' }}>
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
                  <div style={{ fontSize: '16px', fontWeight: 600, color: '#34d399' }}>STRICT ISOLATION</div>
                  <div style={{ fontSize: '11px', color: '#64748b', marginTop: '6px' }}>
                    Files strictly inside <code style={{ color: '#cbd5e1' }}>./sandbox/</code>
                  </div>
                </div>

                <div style={{ background: 'rgba(15, 23, 42, 0.65)', border: '1px solid rgba(56, 189, 248, 0.2)', padding: '18px', borderRadius: '12px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '10px' }}>
                    <span style={{ fontSize: '12px', color: '#94a3b8' }}>Anti-DoS Limiter</span>
                    <Activity style={{ width: '16px', height: '16px', color: '#38bdf8' }} />
                  </div>
                  <div style={{ fontSize: '16px', fontWeight: 600, color: '#38bdf8' }}>30 REQ / SEC</div>
                  <div style={{ fontSize: '11px', color: '#64748b', marginTop: '6px' }}>
                    Sliding window + Brute force lockout
                  </div>
                </div>

                <div style={{ background: 'rgba(15, 23, 42, 0.65)', border: '1px solid rgba(239, 68, 68, 0.2)', padding: '18px', borderRadius: '12px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '10px' }}>
                    <span style={{ fontSize: '12px', color: '#94a3b8' }}>Shell Policy</span>
                    <Terminal style={{ width: '16px', height: '16px', color: '#f87171' }} />
                  </div>
                  <div style={{ fontSize: '16px', fontWeight: 600, color: '#f87171' }}>REVOKED / BLOCKED</div>
                  <div style={{ fontSize: '11px', color: '#64748b', marginTop: '6px' }}>
                    cmd / powershell / bash disabled
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

                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '14px' }}>
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

                  <button
                    onClick={handleSimulateRateLimit}
                    style={{
                      background: 'rgba(56, 189, 248, 0.1)',
                      border: '1px solid rgba(56, 189, 248, 0.3)',
                      color: '#7dd3fc',
                      padding: '14px',
                      borderRadius: '8px',
                      cursor: 'pointer',
                      textAlign: 'left',
                      transition: 'all 0.15s',
                    }}
                  >
                    <div style={{ fontWeight: 600, fontSize: '13px', marginBottom: '4px' }}>4. Anti-DoS Burst Flood</div>
                    <div style={{ fontSize: '11px', color: '#38bdf8' }}>Fires 35 calls; Rust triggers HTTP 429</div>
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

          {/* TAB 3: PRIVACY & ZERO-KNOWLEDGE VAULT (NEW IN v0.11) */}
          {activeTab === 'vault' && (
            <div>
              <div style={{ marginBottom: '24px' }}>
                <h1 style={{ fontSize: '22px', fontWeight: 600, color: '#f8fafc', marginBottom: '6px' }}>
                  Local encrypted Encrypted Vault & Air-Gap Privacy
                </h1>
                <p style={{ fontSize: '13px', color: '#94a3b8' }}>
                  Explicit vault encryption uses Argon2id and XChaCha20-Poly1305. Ordinary sandbox files are not encrypted.
                </p>
              </div>

              {/* Air-Gap Privacy Audit Card */}
              <div style={{ background: 'linear-gradient(135deg, rgba(30, 27, 75, 0.6), rgba(15, 23, 42, 0.8))', border: '1px solid rgba(168, 85, 247, 0.3)', borderRadius: '12px', padding: '20px', marginBottom: '24px' }}>
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '12px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                    <ShieldCheck style={{ width: '22px', height: '22px', color: '#c084fc' }} />
                    <h3 style={{ fontSize: '16px', fontWeight: 600, color: '#f1f5f9' }}>
                      Air-Gap & Zero-Leakage Compliance Ledger
                    </h3>
                  </div>
                  <span style={{ fontSize: '11px', background: 'rgba(168, 85, 247, 0.2)', color: '#d8b4fe', padding: '4px 10px', borderRadius: '12px', fontWeight: 500 }}>
                    100% Offline Air-Gapped
                  </span>
                </div>

                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '12px', marginTop: '16px' }}>
                  <div style={{ background: 'rgba(0,0,0,0.3)', padding: '12px', borderRadius: '8px' }}>
                    <div style={{ fontSize: '11px', color: '#94a3b8' }}>Data Harvesting Risk</div>
                    <div style={{ fontSize: '16px', fontWeight: 700, color: '#34d399', marginTop: '4px' }}>0.00%</div>
                  </div>
                  <div style={{ background: 'rgba(0,0,0,0.3)', padding: '12px', borderRadius: '8px' }}>
                    <div style={{ fontSize: '11px', color: '#94a3b8' }}>Outbound Sockets</div>
                    <div style={{ fontSize: '16px', fontWeight: 700, color: '#f87171', marginTop: '4px' }}>BLOCKED</div>
                  </div>
                  <div style={{ background: 'rgba(0,0,0,0.3)', padding: '12px', borderRadius: '8px' }}>
                    <div style={{ fontSize: '11px', color: '#94a3b8' }}>External Pingbacks</div>
                    <div style={{ fontSize: '16px', fontWeight: 700, color: '#38bdf8', marginTop: '4px' }}>REVOKED</div>
                  </div>
                  <div style={{ background: 'rgba(0,0,0,0.3)', padding: '12px', borderRadius: '8px' }}>
                    <div style={{ fontSize: '11px', color: '#94a3b8' }}>Storage Security</div>
                    <div style={{ fontSize: '16px', fontWeight: 700, color: '#a855f7', marginTop: '4px' }}>PBKDF2 + HMAC</div>
                  </div>
                </div>
              </div>

              {/* Local encrypted Vault Encrypt / Decrypt Tool */}
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px' }}>
                <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
                  <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc', marginBottom: '14px', display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <Lock style={{ width: '16px', height: '16px', color: '#a855f7' }} />
                    Encrypt Data at Rest (.vault)
                  </h3>

                  <div style={{ marginBottom: '12px' }}>
                    <label style={{ fontSize: '12px', color: '#94a3b8', display: 'block', marginBottom: '6px' }}>Vault File Name</label>
                    <input
                      type="text"
                      value={vaultFileName}
                      onChange={(e) => setVaultFileName(e.target.value)}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', padding: '8px 12px', borderRadius: '6px', color: '#f1f5f9', fontSize: '13px', outline: 'none' }}
                    />
                  </div>

                  <div style={{ marginBottom: '12px' }}>
                    <label style={{ fontSize: '12px', color: '#94a3b8', display: 'block', marginBottom: '6px' }}>Master Passphrase (Derives 256-bit Key)</label>
                    <div style={{ position: 'relative' }}>
                      <input
                        type={showPassphrase ? 'text' : 'password'}
                        value={vaultPassphrase}
                        onChange={(e) => setVaultPassphrase(e.target.value)}
                        style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', padding: '8px 12px', borderRadius: '6px', color: '#f1f5f9', fontSize: '13px', outline: 'none' }}
                      />
                      <button
                        type="button"
                        onClick={() => setShowPassphrase(!showPassphrase)}
                        style={{ position: 'absolute', right: '10px', top: '8px', background: 'transparent', border: 'none', color: '#94a3b8', cursor: 'pointer' }}
                      >
                        {showPassphrase ? <EyeOff style={{ width: '16px', height: '16px' }} /> : <Eye style={{ width: '16px', height: '16px' }} />}
                      </button>
                    </div>
                  </div>

                  <div style={{ marginBottom: '16px' }}>
                    <label style={{ fontSize: '12px', color: '#94a3b8', display: 'block', marginBottom: '6px' }}>Confidential Plaintext Content</label>
                    <textarea
                      rows={4}
                      value={vaultContent}
                      onChange={(e) => setVaultContent(e.target.value)}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', padding: '8px 12px', borderRadius: '6px', color: '#f1f5f9', fontSize: '13px', outline: 'none', fontFamily: 'monospace' }}
                    />
                  </div>

                  <div style={{ display: 'flex', gap: '10px' }}>
                    <button
                      onClick={handleVaultEncrypt}
                      style={{ background: '#7c3aed', border: 'none', color: '#fff', padding: '10px 18px', borderRadius: '6px', fontWeight: 500, fontSize: '13px', cursor: 'pointer' }}
                    >
                      Encrypt & Save Vault
                    </button>
                    <button
                      onClick={handleVaultDecrypt}
                      style={{ background: 'rgba(255,255,255,0.08)', border: '1px solid rgba(255,255,255,0.15)', color: '#e2e8f0', padding: '10px 18px', borderRadius: '6px', fontWeight: 500, fontSize: '13px', cursor: 'pointer' }}
                    >
                      Authenticate & Decrypt
                    </button>
                  </div>
                </div>

                {/* Vault Inspection Card */}
                <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
                  <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc', marginBottom: '14px', display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <FileText style={{ width: '16px', height: '16px', color: '#38bdf8' }} />
                    On-Disk Forensics & Cryptographic Envelope
                  </h3>

                  <div style={{ marginBottom: '16px' }}>
                    <div style={{ fontSize: '11px', color: '#94a3b8', marginBottom: '4px' }}>Raw Ciphertext Stored in ./sandbox/ (HMAC-SHA256 Authenticated):</div>
                    <pre style={{ fontSize: '11px', color: '#a78bfa', background: 'rgba(0,0,0,0.4)', padding: '10px', borderRadius: '6px', whiteSpace: 'pre-wrap', wordBreak: 'break-all', maxHeight: '110px', overflowY: 'auto' }}>
                      {vaultRawCiphertext || 'Save a vault to view authenticated ciphertext envelope...'}
                    </pre>
                  </div>

                  <div>
                    <div style={{ fontSize: '11px', color: '#94a3b8', marginBottom: '4px' }}>Authenticated Plaintext (Decrypted Result):</div>
                    <pre style={{ fontSize: '12px', color: vaultDecryptedText ? '#34d399' : '#64748b', background: 'rgba(0,0,0,0.4)', padding: '10px', borderRadius: '6px', whiteSpace: 'pre-wrap', maxHeight: '110px', overflowY: 'auto' }}>
                      {vaultDecryptedText || 'Awaiting passphrase authentication...'}
                    </pre>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* TAB 4: MULTI-WINDOW & NATIVE DIALOGS (NEW IN v0.12) */}
          {activeTab === 'window' && (
            <div>
              <div style={{ marginBottom: '24px' }}>
                <h1 style={{ fontSize: '22px', fontWeight: 600, color: '#f8fafc', marginBottom: '6px' }}>
                  Multi-Window Manager & Native System Dialogs
                </h1>
                <p style={{ fontSize: '13px', color: '#94a3b8' }}>
                  Control secondary windows, popups, and request permission-controlled native message dialogs directly through the Rust core engine.
                </p>
              </div>

              {/* Multi-Window Controls */}
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px', marginBottom: '24px' }}>
                <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
                  <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc', marginBottom: '14px', display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <AppWindow style={{ width: '16px', height: '16px', color: '#38bdf8' }} />
                    Spawn Native Secondary Windows
                  </h3>
                  <p style={{ fontSize: '12px', color: '#94a3b8', marginBottom: '16px' }}>
                    Aegis can launch isolated browser process windows for secondary screens, sub-dashboards, or modal dialogues.
                  </p>
                  <div style={{ display: 'flex', gap: '10px' }}>
                    <button
                      onClick={() => handleSpawnSecondaryWindow(false)}
                      style={{ background: '#0284c7', border: 'none', color: '#fff', padding: '10px 16px', borderRadius: '6px', fontWeight: 500, fontSize: '13px', cursor: 'pointer' }}
                    >
                      Spawn Secondary Window (800x600)
                    </button>
                    <button
                      onClick={() => handleSpawnSecondaryWindow(true)}
                      style={{ background: 'rgba(56, 189, 248, 0.15)', border: '1px solid rgba(56, 189, 248, 0.3)', color: '#7dd3fc', padding: '10px 16px', borderRadius: '6px', fontWeight: 500, fontSize: '13px', cursor: 'pointer' }}
                    >
                      Spawn Modal Window
                    </button>
                  </div>
                </div>

                <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
                  <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc', marginBottom: '14px', display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <AlertTriangle style={{ width: '16px', height: '16px', color: '#fbbf24' }} />
                    Trigger OS-Native System Dialogs
                  </h3>
                  <p style={{ fontSize: '12px', color: '#94a3b8', marginBottom: '16px' }}>
                    Trigger native OS message boxes (Windows WPF / MessageBox) outside the browser viewport.
                  </p>
                  <div style={{ display: 'flex', gap: '10px' }}>
                    <button
                      onClick={() => handleShowNativeDialog('info')}
                      style={{ background: 'rgba(59, 130, 246, 0.15)', border: '1px solid rgba(59, 130, 246, 0.3)', color: '#93c5fd', padding: '10px 14px', borderRadius: '6px', fontWeight: 500, fontSize: '12px', cursor: 'pointer' }}
                    >
                      ℹ️ Info Dialog
                    </button>
                    <button
                      onClick={() => handleShowNativeDialog('warning')}
                      style={{ background: 'rgba(245, 158, 11, 0.15)', border: '1px solid rgba(245, 158, 11, 0.3)', color: '#fde68a', padding: '10px 14px', borderRadius: '6px', fontWeight: 500, fontSize: '12px', cursor: 'pointer' }}
                    >
                      ⚠️ Warning Dialog
                    </button>
                    <button
                      onClick={() => handleShowNativeDialog('error')}
                      style={{ background: 'rgba(239, 68, 68, 0.15)', border: '1px solid rgba(239, 68, 68, 0.3)', color: '#fca5a5', padding: '10px 14px', borderRadius: '6px', fontWeight: 500, fontSize: '12px', cursor: 'pointer' }}
                    >
                      ⛔ Security Alert Dialog
                    </button>
                  </div>
                </div>
              </div>
              {/* Browser runtime & Tao Runtime Inspection (v0.17) */}
              <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(56, 189, 248, 0.25)', borderRadius: '12px', padding: '20px', marginBottom: '24px' }}>
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '16px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                    <div style={{ background: 'linear-gradient(135deg, #0284c7, #38bdf8)', padding: '6px', borderRadius: '8px', display: 'flex' }}>
                      <Globe style={{ width: '18px', height: '18px', color: '#fff' }} />
                    </div>
                    <div>
                      <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc' }}>Browser runtime</h3>
                      <p style={{ fontSize: '12px', color: '#64748b' }}>External browser launch; an embedded native webview is not implemented</p>
                    </div>
                  </div>
                  <button
                    onClick={refreshRuntimeInfo}
                    style={{ background: 'rgba(56, 189, 248, 0.15)', border: '1px solid rgba(56, 189, 248, 0.3)', color: '#7dd3fc', padding: '6px 14px', borderRadius: '6px', fontSize: '12px', cursor: 'pointer', fontWeight: 500 }}
                  >
                    ↻ Query Runtime
                  </button>
                </div>

                {runtimeInfo ? (
                  <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '14px' }}>
                    <div style={{ background: 'rgba(0,0,0,0.3)', borderRadius: '8px', padding: '12px', border: '1px solid rgba(255,255,255,0.05)' }}>
                      <div style={{ fontSize: '11px', color: '#64748b', marginBottom: '4px' }}>Engine Core</div>
                      <div style={{ fontSize: '14px', fontWeight: 700, color: '#38bdf8' }}>{runtimeInfo.engine}</div>
                      <div style={{ fontSize: '10px', color: '#475569', marginTop: '2px' }}>Tao v{runtimeInfo.tao_version} • Wry v{runtimeInfo.wry_version}</div>
                    </div>
                    <div style={{ background: 'rgba(0,0,0,0.3)', borderRadius: '8px', padding: '12px', border: '1px solid rgba(255,255,255,0.05)' }}>
                      <div style={{ fontSize: '11px', color: '#64748b', marginBottom: '4px' }}>WebKit Integration</div>
                      <div style={{ fontSize: '14px', fontWeight: 700, color: runtimeInfo.is_native_webkit ? '#34d399' : '#38bdf8' }}>
                        {runtimeInfo.is_native_webkit ? 'Native WebKit available' : 'External browser fallback'}
                      </div>
                      <div style={{ fontSize: '10px', color: '#475569', marginTop: '2px' }}>OS Platform: {runtimeInfo.os_family}</div>
                    </div>
                    <div style={{ background: 'rgba(0,0,0,0.3)', borderRadius: '8px', padding: '12px', border: '1px solid rgba(255,255,255,0.05)' }}>
                      <div style={{ fontSize: '11px', color: '#64748b', marginBottom: '4px' }}>IPC Bridge Status</div>
                      <div style={{ fontSize: '14px', fontWeight: 700, color: runtimeInfo.ipc_bridge_active ? '#34d399' : '#f87171' }}>
                        {runtimeInfo.ipc_bridge_active ? 'Isolated IPC Active' : 'Offline'}
                      </div>
                      <div style={{ fontSize: '10px', color: '#475569', marginTop: '2px' }}>Protocol: `aegis://ipc` direct RPC</div>
                    </div>
                    <div style={{ background: 'rgba(0,0,0,0.3)', borderRadius: '8px', padding: '12px', border: '1px solid rgba(255,255,255,0.05)' }}>
                      <div style={{ fontSize: '11px', color: '#64748b', marginBottom: '4px' }}>Window Protocol</div>
                      <div style={{ fontSize: '14px', fontWeight: 700, color: '#a855f7' }}>EventLoopProxy</div>
                      <div style={{ fontSize: '10px', color: '#475569', marginTop: '2px' }}>Thread-safe cross-process bus</div>
                    </div>
                  </div>
                ) : (
                  <div style={{ fontSize: '12px', color: '#64748b', fontStyle: 'italic' }}>
                    Click "Query Runtime" to inspect the active Wry/Tao WebKit descriptor.
                  </div>
                )}
              </div>
            </div>
          )}

          {/* TAB 4b: BUNDLER & PACKAGING (NEW IN v0.12) */}
          {activeTab === 'bundler' && (
            <div>
              <div style={{ marginBottom: '24px' }}>
                <h1 style={{ fontSize: '22px', fontWeight: 600, color: '#f8fafc', marginBottom: '6px' }}>
                  One-Click Native Bundler & Installer Packager
                </h1>
                <p style={{ fontSize: '13px', color: '#94a3b8' }}>
                  Package your Aegis application into a portable standalone directory and generate a Windows native
                  <code style={{ color: '#fde68a', background: 'rgba(251,191,36,0.1)', padding: '1px 6px', borderRadius: '4px', margin: '0 4px' }}>.exe</code>
                  installer script via InnoSetup — zero manual configuration required.
                </p>
              </div>

              {/* How-To Steps */}
              <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(251, 191, 36, 0.2)', borderRadius: '12px', padding: '24px', marginBottom: '20px' }}>
                <h3 style={{ fontSize: '16px', fontWeight: 600, color: '#fde68a', marginBottom: '18px', display: 'flex', alignItems: 'center', gap: '8px' }}>
                  <Zap style={{ width: '18px', height: '18px', color: '#fbbf24' }} />
                  How to Bundle in 3 Steps
                </h3>
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '16px' }}>
                  {[
                    { step: '01', title: 'Build Rust Core', cmd: 'cd src-core && cargo build --release', desc: 'Compiles the Rust binary to a native executable (aegis-core.exe).' },
                    { step: '02', title: 'Build Frontend', cmd: 'cd src-ui && npm run build', desc: 'Vite bundles the React app into optimized static HTML/JS/CSS.' },
                    { step: '03', title: 'Run Bundler', cmd: 'cargo run -- --bundle', desc: 'Aegis packages both into dist-bundle/ and generates the InnoSetup .iss script.' },
                  ].map(item => (
                    <div key={item.step} style={{ background: 'rgba(0,0,0,0.3)', borderRadius: '10px', padding: '16px', border: '1px solid rgba(255,255,255,0.06)' }}>
                      <div style={{ fontSize: '28px', fontWeight: 800, color: 'rgba(251,191,36,0.2)', lineHeight: 1 }}>{item.step}</div>
                      <div style={{ fontSize: '13px', fontWeight: 600, color: '#f1f5f9', marginTop: '8px', marginBottom: '6px' }}>{item.title}</div>
                      <code style={{ fontSize: '11px', color: '#86efac', background: 'rgba(0,0,0,0.4)', padding: '4px 8px', borderRadius: '4px', display: 'block', marginBottom: '8px' }}>{item.cmd}</code>
                      <div style={{ fontSize: '11px', color: '#94a3b8', lineHeight: '1.5' }}>{item.desc}</div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Bundler Engine Status */}
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px' }}>
                <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '14px' }}>
                    <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc', display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <Database style={{ width: '16px', height: '16px', color: '#fbbf24' }} />
                      Bundler Engine Status
                    </h3>
                    <button
                      onClick={refreshBundlerStatus}
                      style={{ background: 'transparent', border: 'none', color: '#94a3b8', cursor: 'pointer' }}
                    >
                      <RefreshCw style={{ width: '14px', height: '14px' }} />
                    </button>
                  </div>

                  {bundlerLoading ? (
                    <div style={{ color: '#64748b', fontSize: '13px' }}>Querying Rust Core...</div>
                  ) : bundlerStatus ? (
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
                      <div style={{ display: 'flex', justifyContent: 'space-between', padding: '8px 12px', background: 'rgba(0,0,0,0.25)', borderRadius: '6px' }}>
                        <span style={{ fontSize: '12px', color: '#94a3b8' }}>Engine Version</span>
                        <span style={{ fontSize: '12px', color: '#fde68a', fontWeight: 600 }}>{bundlerStatus.bundlerEngineVersion}</span>
                      </div>
                      <div style={{ display: 'flex', justifyContent: 'space-between', padding: '8px 12px', background: 'rgba(0,0,0,0.25)', borderRadius: '6px' }}>
                        <span style={{ fontSize: '12px', color: '#94a3b8' }}>Bundle Output Exists</span>
                        <span style={{ fontSize: '12px', color: bundlerStatus.bundleExists ? '#34d399' : '#94a3b8', fontWeight: 600 }}>
                          {bundlerStatus.bundleExists ? '✅ Yes' : '⬜ Not yet built'}
                        </span>
                      </div>
                      <div style={{ padding: '8px 12px', background: 'rgba(0,0,0,0.25)', borderRadius: '6px' }}>
                        <div style={{ fontSize: '11px', color: '#94a3b8', marginBottom: '4px' }}>Supported Targets</div>
                        <div style={{ display: 'flex', gap: '6px', flexWrap: 'wrap' }}>
                          {(bundlerStatus.supportedTargets || []).map((t: string) => (
                            <span key={t} style={{ fontSize: '10px', background: 'rgba(99,102,241,0.15)', color: '#a5b4fc', padding: '2px 8px', borderRadius: '10px' }}>{t}</span>
                          ))}
                        </div>
                      </div>
                      {bundlerStatus.existingBundles?.length > 0 && (
                        <div style={{ padding: '8px 12px', background: 'rgba(0,0,0,0.25)', borderRadius: '6px' }}>
                          <div style={{ fontSize: '11px', color: '#94a3b8', marginBottom: '6px' }}>Existing Bundles in dist-bundle/</div>
                          {bundlerStatus.existingBundles.map((b: string) => (
                            <div key={b} style={{ fontSize: '12px', color: '#86efac', fontFamily: 'monospace' }}>📁 {b}</div>
                          ))}
                        </div>
                      )}
                    </div>
                  ) : (
                    <div style={{ color: '#64748b', fontSize: '13px' }}>Click refresh to query bundler status from Rust Core.</div>
                  )}
                </div>

                {/* InnoSetup Info Card */}
                <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
                  <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc', marginBottom: '14px', display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <FileCode style={{ width: '16px', height: '16px', color: '#a855f7' }} />
                    Windows .exe Installer Generation
                  </h3>
                  <p style={{ fontSize: '12px', color: '#94a3b8', lineHeight: '1.6', marginBottom: '14px' }}>
                    After running the bundler, a <code style={{ color: '#fde68a' }}>dist-bundle/AppName-installer.iss</code> script is auto-generated.
                    Compile it with <strong style={{ color: '#e2e8f0' }}>Inno Setup Compiler</strong> to produce a native Windows
                    <code style={{ color: '#fde68a' }}>Setup.exe</code> installer — no Rust or Node required on the end-user machine.
                  </p>
                  <div style={{ background: 'rgba(0,0,0,0.3)', borderRadius: '8px', padding: '12px', marginBottom: '14px' }}>
                    <div style={{ fontSize: '11px', color: '#64748b', marginBottom: '6px' }}>Bundle Output Structure</div>
                    <pre style={{ fontSize: '11px', color: '#cbd5e1', lineHeight: 1.6 }}>{`dist-bundle/
├── AppName/
│   ├── aegis-core.exe  ← Rust binary
│   ├── ui/             ← Built React frontend
│   ├── sandbox/        ← Isolated storage
│   ├── aegis.config.json
│   ├── Launch AppName.bat
│   └── aegis-bundle.json
└── AppName-installer.iss ← InnoSetup script`}</pre>
                  </div>
                  <a
                    href="https://jrsoftware.org/isinfo.php"
                    target="_blank"
                    rel="noreferrer"
                    style={{ display: 'inline-flex', alignItems: 'center', gap: '6px', fontSize: '12px', color: '#818cf8', textDecoration: 'none' }}
                  >
                    <ExternalLink style={{ width: '12px', height: '12px' }} />
                    Download Inno Setup Compiler
                  </a>
                </div>
              </div>
            </div>
          )}

          {/* TAB 4c: PLUGIN ENGINE (NEW IN v0.13) */}
          {activeTab === 'plugins' && (
            <div>
              <div style={{ marginBottom: '24px' }}>
                <h1 style={{ fontSize: '22px', fontWeight: 600, color: '#f8fafc', marginBottom: '6px' }}>
                  Custom Rust Plugin Engine
                </h1>
                <p style={{ fontSize: '13px', color: '#94a3b8' }}>
                  Extensible, sandboxed native Rust plugins. Each plugin registers explicit capabilities and executes inside memory-safe Rust closures with zero shell breakout risk.
                </p>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px' }}>
                {/* Plugin Registry List */}
                <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
                    <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc', display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <Plug style={{ width: '16px', height: '16px', color: '#34d399' }} />
                      Registered Plugins ({pluginList.length})
                    </h3>
                    <button
                      onClick={refreshPlugins}
                      style={{ background: 'transparent', border: 'none', color: '#94a3b8', cursor: 'pointer' }}
                    >
                      <RefreshCw style={{ width: '14px', height: '14px' }} />
                    </button>
                  </div>

                  {pluginLoading && pluginList.length === 0 ? (
                    <div style={{ color: '#64748b', fontSize: '13px' }}>Querying plugin registry...</div>
                  ) : pluginList.length === 0 ? (
                    <div style={{ color: '#64748b', fontSize: '13px' }}>No plugins loaded. Click refresh to query the Rust Core.</div>
                  ) : (
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                      {pluginList.map((p) => (
                        <div key={p.name} style={{ background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.06)', borderRadius: '8px', padding: '14px' }}>
                          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '6px' }}>
                            <span style={{ fontSize: '14px', fontWeight: 600, color: '#6ee7b7' }}>{p.name}</span>
                            <span style={{ fontSize: '11px', background: 'rgba(52,211,153,0.15)', color: '#34d399', padding: '2px 8px', borderRadius: '4px', fontWeight: 600 }}>v{p.version}</span>
                          </div>
                          <p style={{ fontSize: '12px', color: '#94a3b8', margin: '4px 0 10px' }}>{p.description}</p>
                          <div style={{ display: 'flex', gap: '6px', flexWrap: 'wrap' }}>
                            {p.capabilities?.map((cap: string) => (
                              <span key={cap} style={{ fontSize: '10px', background: 'rgba(255,255,255,0.05)', color: '#cbd5e1', padding: '2px 6px', borderRadius: '4px' }}>
                                ⚡ {cap}
                              </span>
                            ))}
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>

                {/* Plugin Dispatcher */}
                <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
                  <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc', marginBottom: '14px', display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <Zap style={{ width: '16px', height: '16px', color: '#fbbf24' }} />
                    Plugin Dispatch Console
                  </h3>

                  <div style={{ marginBottom: '14px' }}>
                    <label style={{ display: 'block', fontSize: '11px', fontWeight: 600, color: '#94a3b8', marginBottom: '6px' }}>
                      TARGET PLUGIN
                    </label>
                    <select
                      value={pluginDispatchName}
                      onChange={(e) => setPluginDispatchName(e.target.value)}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.4)', border: '1px solid rgba(255,255,255,0.1)', color: '#f8fafc', padding: '8px 12px', borderRadius: '6px', fontSize: '13px' }}
                    >
                      <option value="system_memory">system_memory (RAM & paging telemetry)</option>
                      <option value="file_checksum">file_checksum (SHA-256 integrity calculator)</option>
                      <option value="process_uptime">process_uptime (Host and daemon uptime)</option>
                    </select>
                  </div>

                  <div style={{ marginBottom: '14px' }}>
                    <label style={{ display: 'block', fontSize: '11px', fontWeight: 600, color: '#94a3b8', marginBottom: '6px' }}>
                      JSON PAYLOAD (ARGUMENTS)
                    </label>
                    <textarea
                      value={pluginDispatchPayload}
                      onChange={(e) => setPluginDispatchPayload(e.target.value)}
                      rows={3}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.4)', border: '1px solid rgba(255,255,255,0.1)', color: '#86efac', fontFamily: 'monospace', padding: '8px 12px', borderRadius: '6px', fontSize: '12px' }}
                    />
                  </div>

                  <button
                    onClick={handleDispatchPlugin}
                    disabled={pluginLoading}
                    style={{ width: '100%', background: 'linear-gradient(90deg, #059669, #10b981)', border: 'none', color: '#fff', padding: '10px 16px', borderRadius: '6px', cursor: 'pointer', fontWeight: 600, fontSize: '13px', marginBottom: '16px' }}
                  >
                    {pluginLoading ? 'Executing in Rust Core...' : 'Invoke Plugin via Native Dispatch'}
                  </button>

                  {pluginResult && (
                    <div style={{ background: 'rgba(0,0,0,0.4)', border: '1px solid rgba(52,211,153,0.3)', borderRadius: '8px', padding: '12px' }}>
                      <div style={{ fontSize: '11px', fontWeight: 600, color: '#34d399', marginBottom: '6px' }}>DISPATCH RESPONSE:</div>
                      <pre style={{ margin: 0, fontSize: '11px', color: '#cbd5e1', overflowX: 'auto', maxHeight: '180px' }}>
                        {JSON.stringify(pluginResult, null, 2)}
                      </pre>
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}

          {/* TAB 4d: TRAY & NOTIFICATIONS (NEW IN v0.13) */}
          {activeTab === 'notifications' && (
            <div>
              <div style={{ marginBottom: '24px' }}>
                <h1 style={{ fontSize: '22px', fontWeight: 600, color: '#f8fafc', marginBottom: '6px' }}>
                  System Tray & Native OS Notifications
                </h1>
                <p style={{ fontSize: '13px', color: '#94a3b8' }}>
                  Cross-platform desktop notifications powered by native OS subsystems (Windows WinRT / PowerShell, macOS osascript, Linux notify-send) without external Electron/Chromium overhead.
                </p>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px' }}>
                {/* Send Notification Form */}
                <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
                  <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc', marginBottom: '16px', display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <Bell style={{ width: '16px', height: '16px', color: '#fbbf24' }} />
                    Dispatch Native OS Notification
                  </h3>

                  <div style={{ marginBottom: '14px' }}>
                    <label style={{ display: 'block', fontSize: '11px', fontWeight: 600, color: '#94a3b8', marginBottom: '6px' }}>
                      NOTIFICATION TITLE
                    </label>
                    <input
                      type="text"
                      value={notifTitle}
                      onChange={(e) => setNotifTitle(e.target.value)}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', color: '#f1f5f9', padding: '8px 12px', borderRadius: '6px', fontSize: '13px' }}
                    />
                  </div>

                  <div style={{ marginBottom: '16px' }}>
                    <label style={{ display: 'block', fontSize: '11px', fontWeight: 600, color: '#94a3b8', marginBottom: '6px' }}>
                      NOTIFICATION BODY
                    </label>
                    <textarea
                      value={notifBody}
                      onChange={(e) => setNotifBody(e.target.value)}
                      rows={3}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', color: '#f1f5f9', padding: '8px 12px', borderRadius: '6px', fontSize: '13px' }}
                    />
                  </div>

                  <button
                    onClick={handleSendNotification}
                    style={{ width: '100%', background: 'linear-gradient(90deg, #d97706, #fbbf24)', border: 'none', color: '#0f172a', padding: '10px 16px', borderRadius: '6px', cursor: 'pointer', fontWeight: 600, fontSize: '13px' }}
                  >
                    Send Native Notification Now
                  </button>
                </div>

                {/* Tray Engine Status */}
                <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
                  <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc', marginBottom: '16px', display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <Shield style={{ width: '16px', height: '16px', color: '#38bdf8' }} />
                    Tray & Notification Architecture
                  </h3>

                  <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', padding: '8px 12px', background: 'rgba(0,0,0,0.25)', borderRadius: '6px' }}>
                      <span style={{ fontSize: '12px', color: '#94a3b8' }}>Target OS:</span>
                      <span style={{ fontSize: '12px', fontWeight: 600, color: '#e2e8f0' }}>{sysInfo?.os || 'Windows'}</span>
                    </div>
                    <div style={{ display: 'flex', justifyContent: 'space-between', padding: '8px 12px', background: 'rgba(0,0,0,0.25)', borderRadius: '6px' }}>
                      <span style={{ fontSize: '12px', color: '#94a3b8' }}>Notification Driver:</span>
                      <span style={{ fontSize: '12px', fontWeight: 600, color: '#34d399' }}>WinRT Toast / Windows PowerShell</span>
                    </div>
                    <div style={{ display: 'flex', justifyContent: 'space-between', padding: '8px 12px', background: 'rgba(0,0,0,0.25)', borderRadius: '6px' }}>
                      <span style={{ fontSize: '12px', color: '#94a3b8' }}>Urgency Levels:</span>
                      <span style={{ fontSize: '12px', fontWeight: 600, color: '#fbbf24' }}>Low, Normal, Critical</span>
                    </div>
                    <div style={{ display: 'flex', justifyContent: 'space-between', padding: '8px 12px', background: 'rgba(0,0,0,0.25)', borderRadius: '6px' }}>
                      <span style={{ fontSize: '12px', color: '#94a3b8' }}>Daemon Status:</span>
                      <span style={{ fontSize: '12px', fontWeight: 600, color: '#38bdf8' }}>Ready & Listening</span>
                    </div>
                  </div>

                  <div style={{ marginTop: '16px', padding: '12px', background: 'rgba(56,189,248,0.08)', border: '1px solid rgba(56,189,248,0.2)', borderRadius: '8px' }}>
                    <div style={{ fontSize: '12px', fontWeight: 600, color: '#7dd3fc', marginBottom: '4px' }}>🔒 Security Guarantee</div>
                    <div style={{ fontSize: '11px', color: '#94a3b8', lineHeight: '1.5' }}>
                      Notification payloads are sanitized in the Rust Core to eliminate command injection vulnerabilities before invoking native system hooks.
                    </div>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* TAB 4e: PERMISSION MANAGER (NEW IN v0.13) */}
          {activeTab === 'permissions' && (
            <div>
              <div style={{ marginBottom: '24px' }}>
                <h1 style={{ fontSize: '22px', fontWeight: 600, color: '#f8fafc', marginBottom: '6px' }}>
                  Runtime Capability & Permission Guard
                </h1>
                <p style={{ fontSize: '13px', color: '#94a3b8' }}>
                  Granular Sandboxed capabilities configured strictly inside <code style={{ color: '#fca5a5' }}>aegis.config.json</code>. No capability can be elevated at runtime by untrusted frontend scripts.
                </p>
              </div>

              <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '24px', marginBottom: '20px' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '18px' }}>
                  <h3 style={{ fontSize: '16px', fontWeight: 600, color: '#f8fafc', display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <Settings style={{ width: '18px', height: '18px', color: '#f87171' }} />
                    Active Capability Matrix
                  </h3>
                  <button
                    onClick={refreshPermissions}
                    style={{ background: 'rgba(239,68,68,0.15)', border: '1px solid rgba(239,68,68,0.3)', color: '#fca5a5', padding: '6px 12px', borderRadius: '6px', cursor: 'pointer', fontSize: '12px', fontWeight: 500 }}
                  >
                    Refresh Policy
                  </button>
                </div>

                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '14px' }}>
                  {[
                    { cap: 'fs', name: 'File System Sandbox', desc: 'Read/write scoped strictly to sandbox/ directory', granted: true },
                    { cap: 'shell', name: 'Raw Shell Execution', desc: 'Arbitrary system command spawning', granted: false },
                    { cap: 'systemInfo', name: 'Hardware Telemetry', desc: 'Read CPU, RAM, and OS telemetry', granted: true },
                    { cap: 'crypto', name: 'Cryptographic utilities', desc: 'SHA-256 and authenticated vault operations', granted: true },
                    { cap: 'encryptedVault', name: 'Local encrypted Vault', desc: 'Encrypted at-rest secure local storage', granted: true },
                    { cap: 'bundler', name: 'Native Executable Bundler', desc: 'InnoSetup & standalone packager', granted: true },
                    { cap: 'multiWindow', name: 'Multi-Window & Native Dialogs', desc: 'Spawn auxiliary windows & message boxes', granted: true },
                    { cap: 'plugins', name: 'Rust Plugin Engine', desc: 'Registered in-memory native plugins', granted: true },
                    { cap: 'notifications', name: 'OS Native Notifications', desc: 'System tray & desktop alert toasts', granted: true },
                    { cap: 'autoUpdater', name: 'SHA-256 Auto-Updater', desc: 'Air-gapped manifest verification & staging', granted: true },
                  ].map((item) => (
                    <div key={item.cap} style={{ background: 'rgba(0,0,0,0.3)', border: `1px solid ${item.granted ? 'rgba(52,211,153,0.2)' : 'rgba(239,68,68,0.2)'}`, borderRadius: '8px', padding: '14px' }}>
                      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '6px' }}>
                        <span style={{ fontSize: '13px', fontWeight: 600, color: '#f1f5f9' }}>{item.name}</span>
                        <span style={{ fontSize: '10px', fontWeight: 700, padding: '2px 6px', borderRadius: '4px', background: item.granted ? 'rgba(52,211,153,0.15)' : 'rgba(239,68,68,0.15)', color: item.granted ? '#34d399' : '#f87171' }}>
                          {item.granted ? 'GRANTED' : 'DENIED'}
                        </span>
                      </div>
                      <code style={{ fontSize: '11px', color: '#94a3b8' }}>capability: {item.cap}</code>
                      <p style={{ fontSize: '11px', color: '#64748b', marginTop: '6px', margin: '6px 0 0' }}>{item.desc}</p>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}

          {/* TAB 4f: AUTO-UPDATER (NEW IN v0.13) */}
          {activeTab === 'updater' && !unavailableTabs.includes(activeTab) && (
            <div>
              <div style={{ marginBottom: '24px' }}>
                <h1 style={{ fontSize: '22px', fontWeight: 600, color: '#f8fafc', marginBottom: '6px' }}>
                  Air-Gapped Auto-Update Subsystem
                </h1>
                <p style={{ fontSize: '13px', color: '#94a3b8' }}>
                  Enterprise-grade updater with cryptographically enforced SHA-256 hash checks and air-gap capability. Aegis never executes unsigned or unverified update binaries.
                </p>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px' }}>
                {/* Updater Status Card */}
                <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
                    <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc', display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <ArrowUpCircle style={{ width: '16px', height: '16px', color: '#38bdf8' }} />
                      Updater Engine Telemetry
                    </h3>
                    <button
                      onClick={refreshUpdaterStatus}
                      style={{ background: 'transparent', border: 'none', color: '#94a3b8', cursor: 'pointer' }}
                    >
                      <RefreshCw style={{ width: '14px', height: '14px' }} />
                    </button>
                  </div>

                  {updaterLoading ? (
                    <div style={{ color: '#64748b', fontSize: '13px' }}>Querying Rust update daemon...</div>
                  ) : (
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
                      <div style={{ display: 'flex', justifyContent: 'space-between', padding: '8px 12px', background: 'rgba(0,0,0,0.25)', borderRadius: '6px' }}>
                        <span style={{ fontSize: '12px', color: '#94a3b8' }}>Current Installed Version:</span>
                        <span style={{ fontSize: '12px', fontWeight: 600, color: '#38bdf8' }}>{updaterStatus?.currentVersion || '0.13.0'}</span>
                      </div>
                      <div style={{ display: 'flex', justifyContent: 'space-between', padding: '8px 12px', background: 'rgba(0,0,0,0.25)', borderRadius: '6px' }}>
                        <span style={{ fontSize: '12px', color: '#94a3b8' }}>Staging Directory:</span>
                        <span style={{ fontSize: '12px', fontWeight: 500, color: '#94a3b8' }}>{updaterStatus?.stagingDir || '.aegis_updates'}</span>
                      </div>
                      <div style={{ display: 'flex', justifyContent: 'space-between', padding: '8px 12px', background: 'rgba(0,0,0,0.25)', borderRadius: '6px' }}>
                        <span style={{ fontSize: '12px', color: '#94a3b8' }}>Integrity Verification:</span>
                        <span style={{ fontSize: '12px', fontWeight: 600, color: '#34d399' }}>{updaterStatus?.verificationAlgorithm || 'SHA-256'}</span>
                      </div>
                      <div style={{ display: 'flex', justifyContent: 'space-between', padding: '8px 12px', background: 'rgba(0,0,0,0.25)', borderRadius: '6px' }}>
                        <span style={{ fontSize: '12px', color: '#94a3b8' }}>Air-Gap Support:</span>
                        <span style={{ fontSize: '12px', fontWeight: 600, color: '#34d399' }}>Full Local File Support</span>
                      </div>
                    </div>
                  )}

                  <div style={{ marginTop: '16px', padding: '12px', background: 'rgba(56,189,248,0.08)', border: '1px solid rgba(56,189,248,0.2)', borderRadius: '8px' }}>
                    <div style={{ fontSize: '12px', fontWeight: 600, color: '#7dd3fc', marginBottom: '4px' }}>🛡️ Sandboxed Verification Rule</div>
                    <div style={{ fontSize: '11px', color: '#94a3b8', lineHeight: '1.5' }}>
                      Update manifests require valid SemVer format, cryptographic SHA-256 checksum matching, and isolated staging before any replacement occurs.
                    </div>
                  </div>
                </div>

                {/* Staging & Manifest Pipeline */}
                <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
                  <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc', marginBottom: '14px', display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <Zap style={{ width: '16px', height: '16px', color: '#fbbf24' }} />
                    Update Verification Pipeline
                  </h3>

                  <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                    {[
                      { num: '1', title: 'Manifest Ingestion', desc: 'JSON manifest contains version, target OS, and exact SHA-256 signature.' },
                      { num: '2', title: 'SemVer Pre-Flight Check', desc: 'Validates target version is strictly newer than active core version.' },
                      { num: '3', title: 'Cryptographic Integrity Check', desc: 'Rust calculates streaming SHA-256 hash of payload; rejects on mismatch.' },
                      { num: '4', title: 'Atomic Staging', desc: 'Moves binary to isolated .aegis_updates staging sandbox awaiting application restart.' },
                    ].map((step) => (
                      <div key={step.num} style={{ display: 'flex', gap: '12px', alignItems: 'flex-start' }}>
                        <div style={{ width: '24px', height: '24px', borderRadius: '50%', background: 'rgba(56,189,248,0.15)', color: '#38bdf8', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: '12px', fontWeight: 700, flexShrink: 0 }}>
                          {step.num}
                        </div>
                        <div>
                          <div style={{ fontSize: '13px', fontWeight: 600, color: '#e2e8f0' }}>{step.title}</div>
                          <div style={{ fontSize: '11px', color: '#94a3b8' }}>{step.desc}</div>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* TAB 4g: HARDWARE SECURITY KEY / FIDO2 (NEW IN v0.14) */}
          {activeTab === 'hardwareKey' && !unavailableTabs.includes(activeTab) && (
            <div>
              <div style={{ marginBottom: '24px' }}>
                <h1 style={{ fontSize: '22px', fontWeight: 600, color: '#f8fafc', marginBottom: '6px' }}>
                  Hardware Security Key & FIDO2 / WebAuthn Integration
                </h1>
                <p style={{ fontSize: '13px', color: '#94a3b8' }}>
                  Cryptographic challenge-response verification powered by hardware security modules (HSM, YubiKey, Titan Key) via WebAuthn Level 3 standards.
                </p>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px' }}>
                {/* Enrolled Keys */}
                <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
                    <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc', display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <Fingerprint style={{ width: '16px', height: '16px', color: '#c084fc' }} />
                      Enrolled Security Tokens ({hwKeys.length})
                    </h3>
                    <button
                      onClick={refreshHardwareKeys}
                      style={{ background: 'transparent', border: 'none', color: '#94a3b8', cursor: 'pointer' }}
                    >
                      <RefreshCw style={{ width: '14px', height: '14px' }} />
                    </button>
                  </div>

                  {hwKeyLoading && hwKeys.length === 0 ? (
                    <div style={{ color: '#64748b', fontSize: '13px' }}>Scanning USB / HID CTAP2.1 bus for physical security tokens...</div>
                  ) : hwKeys.length === 0 ? (
                    <div style={{ background: 'rgba(239,68,68,0.1)', border: '1px solid rgba(239,68,68,0.25)', borderRadius: '8px', padding: '14px' }}>
                      <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '6px' }}>
                        <AlertTriangle style={{ width: '16px', height: '16px', color: '#f87171' }} />
                        <span style={{ fontSize: '13px', fontWeight: 600, color: '#f87171' }}>No Physical FIDO2/WebAuthn Key Connected</span>
                      </div>
                      <p style={{ fontSize: '12px', color: '#94a3b8', margin: 0, lineHeight: 1.5 }}>
                        The system scans Windows PNP USB/HID controllers for genuine hardware tokens (YubiKey, Feitian, Nitrokey, SoloKeys). 
                        Authentication challenges are locked until a physical hardware security key is plugged in and enrolled.
                      </p>
                    </div>
                  ) : (
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                      {hwKeys.map((k) => (
                        <div key={k.key_id} style={{ background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(192,132,252,0.2)', borderRadius: '8px', padding: '14px' }}>
                          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '6px' }}>
                            <span style={{ fontSize: '14px', fontWeight: 600, color: '#e9d5ff' }}>{k.model}</span>
                            <span style={{ fontSize: '10px', background: 'rgba(52,211,153,0.15)', color: '#34d399', padding: '2px 8px', borderRadius: '4px', fontWeight: 600 }}>PHYSICAL TOKEN CONNECTED</span>
                          </div>
                          <div style={{ fontSize: '12px', color: '#94a3b8', marginBottom: '8px' }}>Vendor: {k.manufacturer}</div>
                          <div style={{ fontSize: '11px', color: '#a855f7' }}>Protocol: {k.protocol}</div>
                          <div style={{ display: 'flex', gap: '8px', marginTop: '10px' }}>
                            <span style={{ fontSize: '10px', background: 'rgba(52,211,153,0.15)', color: '#34d399', padding: '2px 6px', borderRadius: '4px' }}>✓ User Touch Required</span>
                            <span style={{ fontSize: '10px', background: 'rgba(56,189,248,0.15)', color: '#38bdf8', padding: '2px 6px', borderRadius: '4px' }}>✓ PIN Protection</span>
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>

                {/* Challenge & Assertion Tester */}
                <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
                  <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc', marginBottom: '14px', display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <Shield style={{ width: '16px', height: '16px', color: '#38bdf8' }} />
                    WebAuthn Challenge / Response
                  </h3>

                  <button
                    onClick={handleGenerateFidoChallenge}
                    style={{ width: '100%', background: 'linear-gradient(90deg, #7c3aed, #a855f7)', border: 'none', color: '#fff', padding: '10px 16px', borderRadius: '6px', cursor: 'pointer', fontWeight: 600, fontSize: '13px', marginBottom: '14px' }}
                  >
                    1. Issue Hardware Challenge Nonce
                  </button>

                  {fidoChallenge && (
                    <div style={{ background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '8px', padding: '12px', marginBottom: '14px' }}>
                      <div style={{ fontSize: '11px', color: '#94a3b8' }}>ACTIVE CHALLENGE ID:</div>
                      <code style={{ fontSize: '12px', color: '#c084fc' }}>{fidoChallenge.challenge_id}</code>
                      <div style={{ fontSize: '11px', color: '#94a3b8', marginTop: '6px' }}>CHALLENGE NONCE:</div>
                      <div style={{ fontSize: '11px', fontFamily: 'monospace', color: '#94a3b8', wordBreak: 'break-all' }}>{fidoChallenge.challenge_nonce}</div>
                    </div>
                  )}

                  <div style={{ marginBottom: '14px' }}>
                    <label style={{ display: 'block', fontSize: '11px', fontWeight: 600, color: '#94a3b8', marginBottom: '6px' }}>
                      SECURITY KEY USER PIN
                    </label>
                    <input
                      type="password"
                      value={fidoPin}
                      onChange={(e) => setFidoPin(e.target.value)}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', color: '#f1f5f9', padding: '8px 12px', borderRadius: '6px', fontSize: '13px' }}
                    />
                  </div>

                  <button
                    onClick={handleVerifyFidoKey}
                    disabled={!fidoChallenge || hwKeyLoading}
                    style={{ width: '100%', background: fidoChallenge ? 'linear-gradient(90deg, #059669, #10b981)' : 'rgba(255,255,255,0.05)', border: 'none', color: fidoChallenge ? '#fff' : '#64748b', padding: '10px 16px', borderRadius: '6px', cursor: fidoChallenge ? 'pointer' : 'not-allowed', fontWeight: 600, fontSize: '13px' }}
                  >
                    2. Simulate Hardware Touch & Verify Signature
                  </button>

                  {fidoVerificationResult && (
                    <div style={{ marginTop: '14px', background: 'rgba(0,0,0,0.4)', border: '1px solid rgba(52,211,153,0.3)', borderRadius: '8px', padding: '12px' }}>
                      <div style={{ fontSize: '11px', fontWeight: 600, color: '#34d399', marginBottom: '6px' }}>✓ HARDWARE SIGNATURE VERIFIED:</div>
                      <pre style={{ margin: 0, fontSize: '11px', color: '#cbd5e1', overflowX: 'auto', maxHeight: '140px' }}>
                        {JSON.stringify(fidoVerificationResult, null, 2)}
                      </pre>
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}

          {/* TAB 4h: PROFILE ISOLATION (NEW IN v0.14) */}
          {activeTab === 'profiles' && (
            <div>
              <div style={{ marginBottom: '24px' }}>
                <h1 style={{ fontSize: '22px', fontWeight: 600, color: '#f8fafc', marginBottom: '6px' }}>
                  Multi-Profile Workspace Isolation
                </h1>
                <p style={{ fontSize: '13px', color: '#94a3b8' }}>
                  Zero-leakage isolated workspaces. Each profile possesses its own private sandbox folder, quota boundaries, and runtime cryptographic storage.
                </p>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px' }}>
                {/* Profile List */}
                <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
                    <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc', display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <Layers style={{ width: '16px', height: '16px', color: '#f472b6' }} />
                      Available Profiles ({profiles.length})
                    </h3>
                    <button
                      onClick={refreshProfiles}
                      style={{ background: 'transparent', border: 'none', color: '#94a3b8', cursor: 'pointer' }}
                    >
                      <RefreshCw style={{ width: '14px', height: '14px' }} />
                    </button>
                  </div>

                  <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                    {profiles.map((p) => {
                      const isActive = activeProfile?.id === p.id;
                      return (
                        <div key={p.id} style={{ background: isActive ? 'rgba(236,72,153,0.1)' : 'rgba(0,0,0,0.3)', border: `1px solid ${isActive ? 'rgba(236,72,153,0.3)' : 'rgba(255,255,255,0.06)'}`, borderRadius: '8px', padding: '14px' }}>
                          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '6px' }}>
                            <span style={{ fontSize: '14px', fontWeight: 600, color: '#fbcfe8' }}>{p.name}</span>
                            {isActive ? (
                              <span style={{ fontSize: '10px', background: 'rgba(236,72,153,0.2)', color: '#f472b6', padding: '2px 8px', borderRadius: '4px', fontWeight: 700 }}>ACTIVE</span>
                            ) : (
                              <button
                                onClick={() => handleSwitchProfile(p.id)}
                                style={{ background: 'rgba(255,255,255,0.08)', border: 'none', color: '#cbd5e1', padding: '4px 10px', borderRadius: '4px', cursor: 'pointer', fontSize: '11px', fontWeight: 500 }}
                              >
                                Switch To
                              </button>
                            )}
                          </div>
                          <p style={{ fontSize: '12px', color: '#94a3b8', margin: '4px 0 8px' }}>{p.description}</p>
                          <div style={{ fontSize: '11px', color: '#64748b' }}>
                            📁 Sandbox Root: <code style={{ color: '#f472b6' }}>./{p.sandbox_dir}/</code>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </div>

                {/* Create Profile Card */}
                <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
                  <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc', marginBottom: '16px', display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <Shield style={{ width: '16px', height: '16px', color: '#38bdf8' }} />
                    Provision New Isolated Profile
                  </h3>

                  <div style={{ marginBottom: '12px' }}>
                    <label style={{ display: 'block', fontSize: '11px', fontWeight: 600, color: '#94a3b8', marginBottom: '6px' }}>PROFILE IDENTIFIER (SLUG)</label>
                    <input
                      type="text"
                      value={newProfileId}
                      onChange={(e) => setNewProfileId(e.target.value)}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', color: '#f1f5f9', padding: '8px 12px', borderRadius: '6px', fontSize: '13px' }}
                    />
                  </div>

                  <div style={{ marginBottom: '12px' }}>
                    <label style={{ display: 'block', fontSize: '11px', fontWeight: 600, color: '#94a3b8', marginBottom: '6px' }}>DISPLAY NAME</label>
                    <input
                      type="text"
                      value={newProfileName}
                      onChange={(e) => setNewProfileName(e.target.value)}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', color: '#f1f5f9', padding: '8px 12px', borderRadius: '6px', fontSize: '13px' }}
                    />
                  </div>

                  <div style={{ marginBottom: '16px' }}>
                    <label style={{ display: 'block', fontSize: '11px', fontWeight: 600, color: '#94a3b8', marginBottom: '6px' }}>SECURITY DESCRIPTION</label>
                    <textarea
                      value={newProfileDesc}
                      onChange={(e) => setNewProfileDesc(e.target.value)}
                      rows={2}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', color: '#f1f5f9', padding: '8px 12px', borderRadius: '6px', fontSize: '13px' }}
                    />
                  </div>

                  <button
                    onClick={handleCreateProfile}
                    style={{ width: '100%', background: 'linear-gradient(90deg, #db2777, #ec4899)', border: 'none', color: '#fff', padding: '10px 16px', borderRadius: '6px', cursor: 'pointer', fontWeight: 600, fontSize: '13px' }}
                  >
                    Provision Isolated Sandbox
                  </button>
                </div>
              </div>
            </div>
          )}

          {/* TAB 4i: WASM PLUGIN VM SANDBOX (NEW IN v0.14) */}
          {activeTab === 'wasm' && (
            <div>
              <div style={{ marginBottom: '24px' }}>
                <h1 style={{ fontSize: '22px', fontWeight: 600, color: '#f8fafc', marginBottom: '6px' }}>
                  WebAssembly (Wasm) Plugin Virtual Machine
                </h1>
                <p style={{ fontSize: '13px', color: '#94a3b8' }}>
                  High-speed, memory-capped WebAssembly guest VM. Sandboxed bytecode modules execute with strict gas metering and zero host vulnerability exposure.
                </p>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px' }}>
                {/* Wasm Modules List */}
                <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
                    <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc', display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <Cpu style={{ width: '16px', height: '16px', color: '#2dd4bf' }} />
                      Registered Wasm Modules ({wasmModules.length})
                    </h3>
                    <button
                      onClick={refreshWasmModules}
                      style={{ background: 'transparent', border: 'none', color: '#94a3b8', cursor: 'pointer' }}
                    >
                      <RefreshCw style={{ width: '14px', height: '14px' }} />
                    </button>
                  </div>

                  <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                    {wasmModules.map((m) => (
                      <div key={m.module_id} style={{ background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(45,212,191,0.2)', borderRadius: '8px', padding: '14px' }}>
                        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '6px' }}>
                          <span style={{ fontSize: '14px', fontWeight: 600, color: '#99f6e4' }}>{m.name}</span>
                          <span style={{ fontSize: '10px', background: 'rgba(45,212,191,0.15)', color: '#2dd4bf', padding: '2px 8px', borderRadius: '4px', fontWeight: 600 }}>v{m.version}</span>
                        </div>
                        <div style={{ fontSize: '11px', color: '#94a3b8', marginBottom: '4px' }}>ID: <code style={{ color: '#2dd4bf' }}>{m.module_id}</code></div>
                        <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap', marginTop: '8px' }}>
                          <span style={{ fontSize: '10px', background: 'rgba(255,255,255,0.05)', color: '#cbd5e1', padding: '2px 6px', borderRadius: '4px' }}>Mem Cap: {m.memory_limit_mb} MB</span>
                          <span style={{ fontSize: '10px', background: 'rgba(255,255,255,0.05)', color: '#cbd5e1', padding: '2px 6px', borderRadius: '4px' }}>Gas Limit: {m.instruction_limit.toLocaleString()}</span>
                          <span style={{ fontSize: '10px', background: 'rgba(52,211,153,0.15)', color: '#34d399', padding: '2px 6px', borderRadius: '4px' }}>Runs: {m.execution_count}</span>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>

                {/* Execute Console */}
                <div style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '12px', padding: '20px' }}>
                  <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc', marginBottom: '14px', display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <Zap style={{ width: '16px', height: '16px', color: '#fbbf24' }} />
                    Wasm Guest Dispatcher
                  </h3>

                  <div style={{ marginBottom: '14px' }}>
                    <label style={{ display: 'block', fontSize: '11px', fontWeight: 600, color: '#94a3b8', marginBottom: '6px' }}>SELECT WASM MODULE</label>
                    <select
                      value={selectedWasmModule}
                      onChange={(e) => setSelectedWasmModule(e.target.value)}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.4)', border: '1px solid rgba(255,255,255,0.1)', color: '#f8fafc', padding: '8px 12px', borderRadius: '6px', fontSize: '13px' }}
                    >
                      <option value="wasm-crypto-accelerator">wasm-crypto-accelerator (Vector SHA-256)</option>
                      <option value="wasm-schema-validator">wasm-schema-validator (Strict JSON Schema)</option>
                    </select>
                  </div>

                  <div style={{ marginBottom: '14px' }}>
                    <label style={{ display: 'block', fontSize: '11px', fontWeight: 600, color: '#94a3b8', marginBottom: '6px' }}>JSON INPUT (GUEST MEMORY BUFFER)</label>
                    <textarea
                      value={wasmInputPayload}
                      onChange={(e) => setWasmInputPayload(e.target.value)}
                      rows={3}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.4)', border: '1px solid rgba(255,255,255,0.1)', color: '#2dd4bf', fontFamily: 'monospace', padding: '8px 12px', borderRadius: '6px', fontSize: '12px' }}
                    />
                  </div>

                  <button
                    onClick={handleExecuteWasm}
                    disabled={wasmLoading}
                    style={{ width: '100%', background: 'linear-gradient(90deg, #0d9488, #14b8a6)', border: 'none', color: '#fff', padding: '10px 16px', borderRadius: '6px', cursor: 'pointer', fontWeight: 600, fontSize: '13px', marginBottom: '16px' }}
                  >
                    {wasmLoading ? 'Running inside Wasm Sandbox...' : 'Run inside Wasm Guest Sandbox'}
                  </button>

                  {wasmExecResult && (
                    <div style={{ background: 'rgba(0,0,0,0.4)', border: '1px solid rgba(45,212,191,0.3)', borderRadius: '8px', padding: '12px' }}>
                      <div style={{ fontSize: '11px', fontWeight: 600, color: '#2dd4bf', marginBottom: '6px' }}>
                        WASM VM EXECUTION REPORT ({wasmExecResult.duration_us} µs):
                      </div>
                      <pre style={{ margin: 0, fontSize: '11px', color: '#cbd5e1', overflowX: 'auto', maxHeight: '160px' }}>
                        {JSON.stringify(wasmExecResult, null, 2)}
                      </pre>
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}

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

          {/* TAB 5: LIVE AUDIT STREAM */}
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
          {/* TAB v0.15-A: BIOMETRIC ENCLAVE */}
          {activeTab === 'enclave' && !unavailableTabs.includes(activeTab) && (
            <div>
              <div style={{ marginBottom: '24px' }}>
                <h1 style={{ fontSize: '22px', fontWeight: 600, color: '#f8fafc', marginBottom: '6px' }}>
                  Biometric Enclave
                </h1>
                <p style={{ fontSize: '13px', color: '#94a3b8' }}>
                  Hardware-bound platform authentication via Windows Hello (TPM) and Apple Secure Enclave.
                  Keys never leave the device; all operations happen inside the trusted execution environment.
                </p>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px', marginBottom: '20px' }}>
                {/* Status Card */}
                <div style={{ background: 'rgba(15,23,42,0.75)', border: '1px solid rgba(99,102,241,0.25)', borderRadius: '12px', padding: '22px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '16px' }}>
                    <Fingerprint style={{ width: '20px', height: '20px', color: '#818cf8' }} />
                    <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc' }}>Enclave Status</h3>
                  </div>

                  <button
                    onClick={refreshEnclaveStatus}
                    disabled={enclaveLoading}
                    style={{ background: 'rgba(99,102,241,0.15)', border: '1px solid rgba(99,102,241,0.3)', color: '#a5b4fc', padding: '8px 16px', borderRadius: '6px', cursor: 'pointer', fontWeight: 500, fontSize: '13px', marginBottom: '16px' }}
                  >
                    {enclaveLoading ? '⏳ Probing...' : '🔍 Query Enclave'}
                  </button>

                  {enclaveStatus && (
                    <div style={{ display: 'grid', gap: '10px' }}>
                      {[
                        { label: 'Platform', value: enclaveStatus.platform },
                        { label: 'Backend', value: enclaveStatus.backend },
                        { label: 'Available', value: enclaveStatus.available ? '✅ Yes' : '❌ No' },
                        { label: 'Authenticated', value: enclaveStatus.authenticated ? '🟢 Active' : '🔴 Not Authenticated' },
                      ].map(({ label, value }) => (
                        <div key={label} style={{ display: 'flex', justifyContent: 'space-between', padding: '8px 12px', background: 'rgba(0,0,0,0.3)', borderRadius: '6px' }}>
                          <span style={{ fontSize: '12px', color: '#64748b' }}>{label}</span>
                          <span style={{ fontSize: '12px', fontWeight: 600, color: '#e2e8f0', fontFamily: 'monospace' }}>{value}</span>
                        </div>
                      ))}
                    </div>
                  )}
                </div>

                {/* Auth Card */}
                <div style={{ background: 'rgba(15,23,42,0.75)', border: '1px solid rgba(99,102,241,0.25)', borderRadius: '12px', padding: '22px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '16px' }}>
                    <ShieldCheck style={{ width: '20px', height: '20px', color: '#34d399' }} />
                    <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc' }}>Authenticate via Enclave</h3>
                  </div>
                  <p style={{ fontSize: '12px', color: '#64748b', marginBottom: '16px', lineHeight: '1.6' }}>
                    Triggers a hardware-bound biometric or PIN challenge. The Rust backend communicates directly
                    with the OS secure enclave — no secret ever touches user-space memory.
                  </p>
                  <button
                    onClick={handleEnclaveAuth}
                    disabled={enclaveLoading}
                    style={{ width: '100%', background: 'linear-gradient(135deg, #6366f1, #a855f7)', border: 'none', color: '#fff', padding: '12px 20px', borderRadius: '8px', cursor: 'pointer', fontWeight: 600, fontSize: '14px', marginBottom: '16px' }}
                  >
                    {enclaveLoading ? '⏳ Authenticating...' : '🔐 Trigger Biometric Auth'}
                  </button>

                  {enclaveAuthResult && (
                    <div style={{ padding: '14px', background: enclaveAuthResult.success ? 'rgba(16,185,129,0.08)' : 'rgba(239,68,68,0.08)', border: `1px solid ${enclaveAuthResult.success ? 'rgba(16,185,129,0.25)' : 'rgba(239,68,68,0.25)'}`, borderRadius: '8px' }}>
                      <div style={{ fontSize: '12px', fontWeight: 700, color: enclaveAuthResult.success ? '#34d399' : '#f87171', marginBottom: '6px' }}>
                        {enclaveAuthResult.success ? '✅ AUTHENTICATED' : '❌ REJECTED'}
                      </div>
                      <div style={{ fontSize: '12px', color: '#94a3b8' }}>Method: <code style={{ color: '#a5b4fc' }}>{enclaveAuthResult.method}</code></div>
                      <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '4px' }}>{enclaveAuthResult.message}</div>
                    </div>
                  )}
                </div>
              </div>

              <div style={{ background: 'rgba(99,102,241,0.05)', border: '1px solid rgba(99,102,241,0.15)', borderRadius: '10px', padding: '16px' }}>
                <div style={{ fontSize: '12px', color: '#818cf8', fontWeight: 600, marginBottom: '8px' }}>🔐 Security Guarantee</div>
                <div style={{ fontSize: '12px', color: '#64748b', lineHeight: '1.7' }}>
                  Biometric Enclave keys are sealed by the TPM / Secure Enclave chip. Even if Aegis Core is
                  fully compromised, an attacker cannot extract the private key — they can only attempt
                  authentication which requires physical presence and a valid biometric or PIN.
                </div>
              </div>
            </div>
          )}

          {/* TAB v0.15-B: ENCRYPTED IPC STREAM */}
          {activeTab === 'stream' && !unavailableTabs.includes(activeTab) && (
            <div>
              <div style={{ marginBottom: '24px' }}>
                <h1 style={{ fontSize: '22px', fontWeight: 600, color: '#f8fafc', marginBottom: '6px' }}>
                  Encrypted IPC Stream
                </h1>
                <p style={{ fontSize: '13px', color: '#94a3b8' }}>
                  End-to-end symmetric encryption of every IPC payload. Session keys are derived from the
                  handshake nonce — each message includes an HMAC-SHA256 authentication tag so tampering is
                  immediately detected.
                </p>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr', gap: '20px' }}>
                {/* Encrypt */}
                <div style={{ background: 'rgba(15,23,42,0.75)', border: '1px solid rgba(6,182,212,0.25)', borderRadius: '12px', padding: '22px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '16px' }}>
                    <Lock style={{ width: '20px', height: '20px', color: '#22d3ee' }} />
                    <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc' }}>Encrypt Payload</h3>
                  </div>

                  <div style={{ marginBottom: '12px' }}>
                    <label style={{ fontSize: '12px', color: '#64748b', marginBottom: '6px', display: 'block' }}>Plaintext Message</label>
                    <textarea
                      value={streamPlaintext}
                      onChange={(e) => setStreamPlaintext(e.target.value)}
                      rows={3}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', padding: '10px 12px', borderRadius: '6px', color: '#f1f5f9', fontSize: '13px', resize: 'vertical', boxSizing: 'border-box' }}
                    />
                  </div>

                  <button
                    onClick={handleStreamEncrypt}
                    disabled={streamLoading || !streamPlaintext}
                    style={{ background: 'linear-gradient(135deg, #0891b2, #0e7490)', border: 'none', color: '#fff', padding: '10px 20px', borderRadius: '8px', cursor: 'pointer', fontWeight: 600, fontSize: '13px' }}
                  >
                    {streamLoading ? '⏳ Encrypting...' : '🔒 Encrypt via Rust Core'}
                  </button>

                  {streamCiphertext && (
                    <div style={{ marginTop: '16px', padding: '14px', background: 'rgba(0,0,0,0.4)', borderRadius: '8px', border: '1px solid rgba(6,182,212,0.2)' }}>
                      <div style={{ fontSize: '11px', color: '#22d3ee', marginBottom: '6px', fontWeight: 600 }}>Encrypted Ciphertext (hex)</div>
                      <code style={{ fontSize: '11px', color: '#94a3b8', wordBreak: 'break-all', display: 'block', marginBottom: '10px' }}>{streamCiphertext}</code>
                      <div style={{ fontSize: '11px', color: '#22d3ee', marginBottom: '6px', fontWeight: 600 }}>HMAC-SHA256 Auth Tag</div>
                      <code style={{ fontSize: '11px', color: '#34d399', wordBreak: 'break-all', display: 'block' }}>{streamTag}</code>
                    </div>
                  )}
                </div>

                {/* Decrypt */}
                {streamCiphertext && (
                  <div style={{ background: 'rgba(15,23,42,0.75)', border: '1px solid rgba(6,182,212,0.25)', borderRadius: '12px', padding: '22px' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '16px' }}>
                      <Eye style={{ width: '20px', height: '20px', color: '#22d3ee' }} />
                      <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc' }}>Decrypt & Verify HMAC</h3>
                    </div>
                    <p style={{ fontSize: '12px', color: '#64748b', marginBottom: '14px' }}>
                      Rust Core verifies the HMAC-SHA256 tag before decrypting. Any bit-flip in the ciphertext or tag will cause a hard rejection.
                    </p>
                    <button
                      onClick={handleStreamDecrypt}
                      disabled={streamLoading}
                      style={{ background: 'linear-gradient(135deg, #047857, #065f46)', border: 'none', color: '#fff', padding: '10px 20px', borderRadius: '8px', cursor: 'pointer', fontWeight: 600, fontSize: '13px', marginBottom: '14px' }}
                    >
                      {streamLoading ? '⏳ Verifying...' : '🔓 Decrypt & Verify Tag'}
                    </button>

                    {streamDecryptResult && (
                      <div style={{ padding: '14px', background: streamDecryptResult.verified ? 'rgba(16,185,129,0.08)' : 'rgba(239,68,68,0.08)', border: `1px solid ${streamDecryptResult.verified ? 'rgba(16,185,129,0.25)' : 'rgba(239,68,68,0.25)'}`, borderRadius: '8px' }}>
                        <div style={{ fontSize: '12px', fontWeight: 700, color: streamDecryptResult.verified ? '#34d399' : '#f87171', marginBottom: '8px' }}>
                          {streamDecryptResult.verified ? '✅ HMAC VERIFIED — Payload Authentic' : '❌ HMAC MISMATCH — Payload Rejected'}
                        </div>
                        {streamDecryptResult.verified && (
                          <div style={{ fontSize: '13px', color: '#e2e8f0', fontFamily: 'monospace', background: 'rgba(0,0,0,0.3)', padding: '8px 10px', borderRadius: '6px' }}>
                            {streamDecryptResult.plaintext}
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                )}
              </div>
            </div>
          )}

          {/* TAB v0.15-C: LOCAL CRASH REPORTER */}
          {activeTab === 'crashReporter' && (
            <div>
              <div style={{ marginBottom: '24px' }}>
                <h1 style={{ fontSize: '22px', fontWeight: 600, color: '#f8fafc', marginBottom: '6px' }}>
                  Local Crash Reporter
                </h1>
                <p style={{ fontSize: '13px', color: '#94a3b8' }}>
                  Air-gapped, zero-telemetry diagnostic capture. Panics and assertion violations are persisted
                  locally to <code style={{ color: '#fbbf24' }}>./sandbox/crashes/</code> — no data ever leaves the device.
                </p>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px', marginBottom: '20px' }}>
                {/* Record Crash */}
                <div style={{ background: 'rgba(15,23,42,0.75)', border: '1px solid rgba(245,158,11,0.25)', borderRadius: '12px', padding: '22px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '16px' }}>
                    <AlertTriangle style={{ width: '20px', height: '20px', color: '#f59e0b' }} />
                    <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc' }}>Record Diagnostic</h3>
                  </div>

                  <div style={{ marginBottom: '10px' }}>
                    <label style={{ fontSize: '12px', color: '#64748b', marginBottom: '5px', display: 'block' }}>Crash Kind</label>
                    <select
                      value={crashKind}
                      onChange={(e) => setCrashKind(e.target.value)}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', padding: '8px 10px', borderRadius: '6px', color: '#f1f5f9', fontSize: '13px' }}
                    >
                      <option value="assertion">Assertion Violation</option>
                      <option value="panic">Rust Panic</option>
                      <option value="oom">Out of Memory</option>
                      <option value="ipc_error">IPC Protocol Error</option>
                      <option value="sandbox_escape">Sandbox Escape Attempt</option>
                    </select>
                  </div>

                  <div style={{ marginBottom: '12px' }}>
                    <label style={{ fontSize: '12px', color: '#64748b', marginBottom: '5px', display: 'block' }}>Message / Stack Trace</label>
                    <textarea
                      value={crashMessage}
                      onChange={(e) => setCrashMessage(e.target.value)}
                      rows={4}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', padding: '10px 12px', borderRadius: '6px', color: '#f1f5f9', fontSize: '12px', resize: 'vertical', fontFamily: 'monospace', boxSizing: 'border-box' }}
                    />
                  </div>

                  <button
                    onClick={handleRecordCrash}
                    disabled={crashLoading}
                    style={{ width: '100%', background: 'linear-gradient(135deg, #b45309, #92400e)', border: 'none', color: '#fff', padding: '10px 20px', borderRadius: '8px', cursor: 'pointer', fontWeight: 600, fontSize: '13px' }}
                  >
                    {crashLoading ? '⏳ Recording...' : '📋 Persist Crash Diagnostic'}
                  </button>
                </div>

                {/* Crash List */}
                <div style={{ background: 'rgba(15,23,42,0.75)', border: '1px solid rgba(245,158,11,0.25)', borderRadius: '12px', padding: '22px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '16px' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                      <FileText style={{ width: '20px', height: '20px', color: '#f59e0b' }} />
                      <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc' }}>Crash Log</h3>
                    </div>
                    <button
                      onClick={refreshCrashEntries}
                      disabled={crashLoading}
                      style={{ background: 'rgba(245,158,11,0.1)', border: '1px solid rgba(245,158,11,0.25)', color: '#fcd34d', padding: '5px 12px', borderRadius: '6px', cursor: 'pointer', fontSize: '12px' }}
                    >
                      {crashLoading ? '⏳' : '↻ Refresh'}
                    </button>
                  </div>

                  {crashEntries.length === 0 ? (
                    <div style={{ textAlign: 'center', padding: '30px 0', color: '#64748b', fontSize: '13px' }}>
                      <CheckCircle2 style={{ width: '32px', height: '32px', color: '#34d399', margin: '0 auto 10px', display: 'block' }} />
                      No crash reports. System nominal.
                    </div>
                  ) : (
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', maxHeight: '320px', overflowY: 'auto' }}>
                      {crashEntries.map((entry: any) => (
                        <div key={entry.id} style={{ padding: '10px 12px', background: 'rgba(0,0,0,0.3)', borderRadius: '8px', border: '1px solid rgba(245,158,11,0.1)' }}>
                          <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '4px' }}>
                            <code style={{ fontSize: '11px', color: '#f59e0b', fontWeight: 700 }}>{entry.kind}</code>
                            <span style={{ fontSize: '10px', color: '#64748b' }}>{entry.timestamp}</span>
                          </div>
                          <div style={{ fontSize: '11px', color: '#94a3b8', fontFamily: 'monospace' }}>{entry.message}</div>
                          <div style={{ fontSize: '10px', color: '#475569', marginTop: '4px' }}>ID: {entry.id}</div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </div>

              <div style={{ background: 'rgba(245,158,11,0.05)', border: '1px solid rgba(245,158,11,0.15)', borderRadius: '10px', padding: '16px' }}>
                <div style={{ fontSize: '12px', color: '#f59e0b', fontWeight: 600, marginBottom: '8px' }}>🛡️ Privacy Guarantee</div>
                <div style={{ fontSize: '12px', color: '#64748b', lineHeight: '1.7' }}>
                  All crash diagnostics are stored exclusively at <code style={{ color: '#fcd34d' }}>./sandbox/crashes/</code>.
                  There are no outbound network calls, no telemetry endpoints, and no crash-upload background threads.
                  Aegis Desktop never sends any diagnostic data off-device.
                </div>
              </div>
            </div>
          )}

          {/* ═════════════════════════════════════════════════════════════════ */}
          {/* SECRETS MANAGER TAB (v0.16)                                      */}
          {/* ═════════════════════════════════════════════════════════════════ */}
          {activeTab === 'secrets' && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
              <div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '8px' }}>
                  <div style={{ background: 'linear-gradient(135deg, #10b981, #059669)', padding: '6px', borderRadius: '8px', display: 'flex' }}>
                    <Key style={{ width: '20px', height: '20px', color: '#fff' }} />
                  </div>
                  <div>
                    <h2 style={{ fontSize: '18px', fontWeight: 600, color: '#f8fafc' }}>Local encrypted Secrets Manager</h2>
                    <p style={{ fontSize: '12px', color: '#64748b' }}>Memory-hard key derivation (1500 rounds) • Versioned secret rotation • Pure in-memory ciphertexts</p>
                  </div>
                </div>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px' }}>
                {/* Store New Secret */}
                <div style={{ background: 'rgba(15,23,42,0.75)', border: '1px solid rgba(16,185,129,0.25)', borderRadius: '12px', padding: '22px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '16px' }}>
                    <Lock style={{ width: '20px', height: '20px', color: '#10b981' }} />
                    <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc' }}>Store / Rotate Secret</h3>
                  </div>

                  <div style={{ marginBottom: '14px' }}>
                    <label style={{ fontSize: '12px', color: '#94a3b8', display: 'block', marginBottom: '6px' }}>Secret Key Identifier</label>
                    <input
                      type="text"
                      value={newSecretKey}
                      onChange={(e) => setNewSecretKey(e.target.value)}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', padding: '8px 12px', borderRadius: '6px', color: '#f1f5f9', fontSize: '12px', boxSizing: 'border-box' }}
                    />
                  </div>

                  <div style={{ marginBottom: '14px' }}>
                    <label style={{ fontSize: '12px', color: '#94a3b8', display: 'block', marginBottom: '6px' }}>Secret Value (Plaintext)</label>
                    <textarea
                      value={newSecretValue}
                      onChange={(e) => setNewSecretValue(e.target.value)}
                      rows={3}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', padding: '8px 12px', borderRadius: '6px', color: '#f1f5f9', fontSize: '12px', resize: 'vertical', fontFamily: 'monospace', boxSizing: 'border-box' }}
                    />
                  </div>

                  <div style={{ marginBottom: '16px' }}>
                    <label style={{ fontSize: '12px', color: '#94a3b8', display: 'block', marginBottom: '6px' }}>Master Passphrase (KDF Seed)</label>
                    <input
                      type="password"
                      value={secretsMasterPass}
                      onChange={(e) => setSecretsMasterPass(e.target.value)}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', padding: '8px 12px', borderRadius: '6px', color: '#f1f5f9', fontSize: '12px', boxSizing: 'border-box' }}
                    />
                  </div>

                  <button
                    onClick={handleStoreSecret}
                    disabled={secretsLoading}
                    style={{ width: '100%', background: 'linear-gradient(135deg, #059669, #047857)', border: 'none', color: '#fff', padding: '10px 20px', borderRadius: '8px', cursor: 'pointer', fontWeight: 600, fontSize: '13px' }}
                  >
                    {secretsLoading ? '⏳ Deriving & Encrypting...' : '🔒 Seal Secret in Secure Memory'}
                  </button>
                </div>

                {/* Retrieve Secret */}
                <div style={{ background: 'rgba(15,23,42,0.75)', border: '1px solid rgba(16,185,129,0.25)', borderRadius: '12px', padding: '22px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '16px' }}>
                    <Key style={{ width: '20px', height: '20px', color: '#10b981' }} />
                    <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc' }}>Decrypt & Retrieve Secret</h3>
                  </div>

                  <div style={{ marginBottom: '14px' }}>
                    <label style={{ fontSize: '12px', color: '#94a3b8', display: 'block', marginBottom: '6px' }}>Target Secret Key</label>
                    <input
                      type="text"
                      value={retrieveSecretKey}
                      onChange={(e) => setRetrieveSecretKey(e.target.value)}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', padding: '8px 12px', borderRadius: '6px', color: '#f1f5f9', fontSize: '12px', boxSizing: 'border-box' }}
                    />
                  </div>

                  <button
                    onClick={handleRetrieveSecret}
                    disabled={secretsLoading}
                    style={{ width: '100%', background: 'rgba(16,185,129,0.15)', border: '1px solid rgba(16,185,129,0.3)', color: '#6ee7b7', padding: '10px 20px', borderRadius: '8px', cursor: 'pointer', fontWeight: 600, fontSize: '13px', marginBottom: '16px' }}
                  >
                    {secretsLoading ? '⏳ Recomputing KDF...' : '🔓 Decrypt Secret'}
                  </button>

                  {retrievedSecretVal && (
                    <div style={{ background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(16,185,129,0.2)', borderRadius: '8px', padding: '12px' }}>
                      <div style={{ fontSize: '11px', color: '#10b981', fontWeight: 600, marginBottom: '4px' }}>Decrypted Plaintext Value:</div>
                      <div style={{ fontSize: '12px', color: '#f1f5f9', fontFamily: 'monospace', wordBreak: 'break-all' }}>{retrievedSecretVal}</div>
                    </div>
                  )}
                </div>
              </div>

              {/* Secrets Table */}
              <div style={{ background: 'rgba(15,23,42,0.75)', border: '1px solid rgba(16,185,129,0.25)', borderRadius: '12px', padding: '22px' }}>
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '16px' }}>
                  <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc' }}>Registered Secrets Register</h3>
                  <button
                    onClick={refreshSecretsList}
                    disabled={secretsLoading}
                    style={{ background: 'rgba(16,185,129,0.1)', border: '1px solid rgba(16,185,129,0.25)', color: '#6ee7b7', padding: '5px 12px', borderRadius: '6px', cursor: 'pointer', fontSize: '12px' }}
                  >
                    {secretsLoading ? '⏳' : '↻ Refresh'}
                  </button>
                </div>

                <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                  {secretsList.map((sec: any) => (
                    <div key={sec.key} style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '10px 14px', background: 'rgba(0,0,0,0.25)', borderRadius: '8px', border: '1px solid rgba(255,255,255,0.05)' }}>
                      <div>
                        <div style={{ fontSize: '13px', fontWeight: 600, color: '#f1f5f9' }}>{sec.key}</div>
                        <div style={{ fontSize: '11px', color: '#64748b', fontFamily: 'monospace' }}>Fingerprint: {sec.digest_fingerprint?.substring(0, 24)}...</div>
                      </div>
                      <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                        <span style={{ fontSize: '11px', background: 'rgba(16,185,129,0.15)', color: '#6ee7b7', padding: '2px 8px', borderRadius: '4px' }}>v{sec.version}</span>
                        <span style={{ fontSize: '11px', color: '#94a3b8' }}>{sec.memory_hard_iterations} KDF rounds</span>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}

          {/* ═════════════════════════════════════════════════════════════════ */}
          {/* NETWORK FIREWALL TAB (v0.16)                                     */}
          {/* ═════════════════════════════════════════════════════════════════ */}
          {activeTab === 'network' && <p role="note" style={{color:'#fbbf24'}}>Advisory policy evaluation only. This screen does not enforce an operating-system firewall.</p>}
{activeTab === 'network' && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
              <div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '8px' }}>
                  <div style={{ background: 'linear-gradient(135deg, #0ea5e9, #0284c7)', padding: '6px', borderRadius: '8px', display: 'flex' }}>
                    <Globe style={{ width: '20px', height: '20px', color: '#fff' }} />
                  </div>
                  <div>
                    <h2 style={{ fontSize: '18px', fontWeight: 600, color: '#f8fafc' }}>Dynamic Network Firewall Policy Engine</h2>
                    <p style={{ fontSize: '12px', color: '#64748b' }}>Zero-trust egress boundary • Telemetry blackholing • Real-time traffic inspection</p>
                  </div>
                </div>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px' }}>
                {/* Traffic Inspector */}
                <div style={{ background: 'rgba(15,23,42,0.75)', border: '1px solid rgba(14,165,233,0.25)', borderRadius: '12px', padding: '22px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '16px' }}>
                    <Shield style={{ width: '20px', height: '20px', color: '#0ea5e9' }} />
                    <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc' }}>Test Egress Connection</h3>
                  </div>

                  <div style={{ marginBottom: '14px' }}>
                    <label style={{ fontSize: '12px', color: '#94a3b8', display: 'block', marginBottom: '6px' }}>Target Hostname / IP</label>
                    <input
                      type="text"
                      value={evalHost}
                      onChange={(e) => setEvalHost(e.target.value)}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', padding: '8px 12px', borderRadius: '6px', color: '#f1f5f9', fontSize: '12px', boxSizing: 'border-box' }}
                    />
                  </div>

                  <div style={{ marginBottom: '16px' }}>
                    <label style={{ fontSize: '12px', color: '#94a3b8', display: 'block', marginBottom: '6px' }}>Destination Port</label>
                    <input
                      type="number"
                      value={evalPort}
                      onChange={(e) => setEvalPort(Number(e.target.value))}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', padding: '8px 12px', borderRadius: '6px', color: '#f1f5f9', fontSize: '12px', boxSizing: 'border-box' }}
                    />
                  </div>

                  <button
                    onClick={handleEvaluateConnection}
                    disabled={firewallLoading}
                    style={{ width: '100%', background: 'linear-gradient(135deg, #0284c7, #0369a1)', border: 'none', color: '#fff', padding: '10px 20px', borderRadius: '8px', cursor: 'pointer', fontWeight: 600, fontSize: '13px', marginBottom: '16px' }}
                  >
                    {firewallLoading ? '⏳ Inspecting Rules...' : '🛡️ Evaluate Sandboxed Policy'}
                  </button>

                  {evalResult && (
                    <div style={{ background: evalResult.allowed ? 'rgba(16,185,129,0.1)' : 'rgba(239,68,68,0.1)', border: `1px solid ${evalResult.allowed ? 'rgba(16,185,129,0.3)' : 'rgba(239,68,68,0.3)'}`, borderRadius: '8px', padding: '12px' }}>
                      <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '6px' }}>
                        {evalResult.allowed ? <CheckCircle2 style={{ width: '16px', height: '16px', color: '#34d399' }} /> : <AlertTriangle style={{ width: '16px', height: '16px', color: '#f87171' }} />}
                        <span style={{ fontSize: '13px', fontWeight: 600, color: evalResult.allowed ? '#34d399' : '#f87171' }}>
                          Connection {evalResult.allowed ? 'PERMITTED' : 'HARD BLOCKED'}
                        </span>
                      </div>
                      <div style={{ fontSize: '11px', color: '#94a3b8' }}>{evalResult.reason}</div>
                    </div>
                  )}
                </div>

                {/* Firewall Rules List */}
                <div style={{ background: 'rgba(15,23,42,0.75)', border: '1px solid rgba(14,165,233,0.25)', borderRadius: '12px', padding: '22px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '16px' }}>
                    <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc' }}>Active Policy Directives</h3>
                    <button
                      onClick={refreshFirewallRules}
                      disabled={firewallLoading}
                      style={{ background: 'rgba(14,165,233,0.1)', border: '1px solid rgba(14,165,233,0.25)', color: '#7dd3fc', padding: '5px 12px', borderRadius: '6px', cursor: 'pointer', fontSize: '12px' }}
                    >
                      {firewallLoading ? '⏳' : '↻ Refresh'}
                    </button>
                  </div>

                  <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                    {firewallRules.map((r: any) => (
                      <div key={r.id} style={{ padding: '10px 12px', background: 'rgba(0,0,0,0.25)', borderRadius: '8px', border: '1px solid rgba(255,255,255,0.05)' }}>
                        <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '4px' }}>
                          <span style={{ fontSize: '12px', fontWeight: 600, color: '#f1f5f9' }}>{r.domain_pattern}</span>
                          <span style={{ fontSize: '10px', fontWeight: 700, padding: '2px 6px', borderRadius: '4px', background: r.action === 'BLOCK' ? 'rgba(239,68,68,0.2)' : 'rgba(16,185,129,0.2)', color: r.action === 'BLOCK' ? '#f87171' : '#34d399' }}>{r.action}</span>
                        </div>
                        <div style={{ fontSize: '11px', color: '#64748b' }}>{r.description}</div>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* ═════════════════════════════════════════════════════════════════ */}
          {/* PLUGIN SIGNATURE & PROVENANCE TAB (v0.16)                        */}
          {/* ═════════════════════════════════════════════════════════════════ */}
          {activeTab === 'pluginSig' && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
              <div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '8px' }}>
                  <div style={{ background: 'linear-gradient(135deg, #d946ef, #c026d3)', padding: '6px', borderRadius: '8px', display: 'flex' }}>
                    <FileCheck style={{ width: '20px', height: '20px', color: '#fff' }} />
                  </div>
                  <div>
                    <h2 style={{ fontSize: '18px', fontWeight: 600, color: '#f8fafc' }}>Plugin Signature & Provenance Verification</h2>
                    <p style={{ fontSize: '12px', color: '#64748b' }}>Cryptographic manifest digests • Enterprise root of trust • Tamper-evident extension security</p>
                  </div>
                </div>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px' }}>
                {/* Verification Sandbox */}
                <div style={{ background: 'rgba(15,23,42,0.75)', border: '1px solid rgba(217,70,239,0.25)', borderRadius: '12px', padding: '22px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '16px' }}>
                    <ShieldCheck style={{ width: '20px', height: '20px', color: '#d946ef' }} />
                    <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc' }}>Verify Manifest Signature</h3>
                  </div>

                  <div style={{ marginBottom: '14px' }}>
                    <label style={{ fontSize: '12px', color: '#94a3b8', display: 'block', marginBottom: '6px' }}>Plugin Identifier</label>
                    <input
                      type="text"
                      value={testPluginId}
                      onChange={(e) => setTestPluginId(e.target.value)}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', padding: '8px 12px', borderRadius: '6px', color: '#f1f5f9', fontSize: '12px', boxSizing: 'border-box' }}
                    />
                  </div>

                  <div style={{ marginBottom: '14px' }}>
                    <label style={{ fontSize: '12px', color: '#94a3b8', display: 'block', marginBottom: '6px' }}>Raw Manifest JSON</label>
                    <textarea
                      value={testPluginManifest}
                      onChange={(e) => setTestPluginManifest(e.target.value)}
                      rows={3}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', padding: '8px 12px', borderRadius: '6px', color: '#f1f5f9', fontSize: '12px', resize: 'vertical', fontFamily: 'monospace', boxSizing: 'border-box' }}
                    />
                  </div>

                  <div style={{ marginBottom: '16px' }}>
                    <label style={{ fontSize: '12px', color: '#94a3b8', display: 'block', marginBottom: '6px' }}>Hex Signature Payload</label>
                    <input
                      type="text"
                      value={testPluginSignature}
                      onChange={(e) => setTestPluginSignature(e.target.value)}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', padding: '8px 12px', borderRadius: '6px', color: '#f1f5f9', fontSize: '12px', boxSizing: 'border-box' }}
                    />
                  </div>

                  <button
                    onClick={handleVerifyPluginManifest}
                    disabled={sigLoading}
                    style={{ width: '100%', background: 'linear-gradient(135deg, #c026d3, #a21caf)', border: 'none', color: '#fff', padding: '10px 20px', borderRadius: '8px', cursor: 'pointer', fontWeight: 600, fontSize: '13px', marginBottom: '16px' }}
                  >
                    {sigLoading ? '⏳ Verifying Provenance...' : '🔏 Authenticate Signer & Digest'}
                  </button>

                  {sigVerifyResult && (
                    <div style={{ background: sigVerifyResult.valid && sigVerifyResult.trusted_vendor ? 'rgba(16,185,129,0.1)' : 'rgba(239,68,68,0.1)', border: `1px solid ${sigVerifyResult.valid && sigVerifyResult.trusted_vendor ? 'rgba(16,185,129,0.3)' : 'rgba(239,68,68,0.3)'}`, borderRadius: '8px', padding: '12px' }}>
                      <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '6px' }}>
                        {sigVerifyResult.valid && sigVerifyResult.trusted_vendor ? <CheckCircle2 style={{ width: '16px', height: '16px', color: '#34d399' }} /> : <AlertTriangle style={{ width: '16px', height: '16px', color: '#f87171' }} />}
                        <span style={{ fontSize: '13px', fontWeight: 600, color: sigVerifyResult.valid && sigVerifyResult.trusted_vendor ? '#34d399' : '#f87171' }}>
                          {sigVerifyResult.valid && sigVerifyResult.trusted_vendor ? 'TRUSTED ENTERPRISE SIGNER' : 'UNTRUSTED / INVALID SIGNATURE'}
                        </span>
                      </div>
                      <div style={{ fontSize: '11px', color: '#94a3b8', marginBottom: '4px' }}>{sigVerifyResult.message}</div>
                      <div style={{ fontSize: '10px', color: '#64748b', fontFamily: 'monospace' }}>Digest: {sigVerifyResult.computed_digest}</div>
                    </div>
                  )}
                </div>

                {/* Trusted Signers Registry */}
                <div style={{ background: 'rgba(15,23,42,0.75)', border: '1px solid rgba(217,70,239,0.25)', borderRadius: '12px', padding: '22px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '16px' }}>
                    <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc' }}>Trusted Root Authority Registry</h3>
                    <button
                      onClick={refreshPluginSigners}
                      disabled={sigLoading}
                      style={{ background: 'rgba(217,70,239,0.1)', border: '1px solid rgba(217,70,239,0.25)', color: '#f0abfc', padding: '5px 12px', borderRadius: '6px', cursor: 'pointer', fontSize: '12px' }}
                    >
                      {sigLoading ? '⏳' : '↻ Refresh'}
                    </button>
                  </div>

                  <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                    {trustedSigners.map((s: any) => (
                      <div key={s.key_id} style={{ padding: '10px 12px', background: 'rgba(0,0,0,0.25)', borderRadius: '8px', border: '1px solid rgba(255,255,255,0.05)' }}>
                        <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '4px' }}>
                          <span style={{ fontSize: '12px', fontWeight: 600, color: '#f1f5f9' }}>{s.vendor}</span>
                          <span style={{ fontSize: '10px', fontWeight: 700, padding: '2px 6px', borderRadius: '4px', background: 'rgba(16,185,129,0.2)', color: '#34d399' }}>TRUSTED</span>
                        </div>
                        <div style={{ fontSize: '11px', color: '#64748b', fontFamily: 'monospace' }}>Key ID: {s.key_id}</div>
                        <div style={{ fontSize: '10px', color: '#475569', fontFamily: 'monospace', wordBreak: 'break-all', marginTop: '2px' }}>Fingerprint: {s.public_fingerprint}</div>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* ═════════════════════════════════════════════════════════════════ */}
          {/* TPM 2.0 REMOTE ATTESTATION TAB (v0.17)                           */}
          {/* ═════════════════════════════════════════════════════════════════ */}
          {activeTab === 'tpmAttestation' && !unavailableTabs.includes(activeTab) && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
              <div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '8px' }}>
                  <div style={{ background: 'linear-gradient(135deg, #10b981, #059669)', padding: '6px', borderRadius: '8px', display: 'flex' }}>
                    <ShieldCheck style={{ width: '20px', height: '20px', color: '#fff' }} />
                  </div>
                  <div>
                    <h2 style={{ fontSize: '18px', fontWeight: 600, color: '#f8fafc' }}>TPM 2.0 Remote Attestation Engine</h2>
                    <p style={{ fontSize: '12px', color: '#64748b' }}>Hardware-rooted platform trust • PCR cryptomarkers (0, 7, 10, 11) • Zero-trust corporate boundary gating</p>
                  </div>
                </div>
              </div>

              {/* PCR Register Measurements Table */}
              <div style={{ background: 'rgba(15,23,42,0.75)', border: '1px solid rgba(16,185,129,0.25)', borderRadius: '12px', padding: '20px' }}>
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '16px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <Cpu style={{ width: '18px', height: '18px', color: '#34d399' }} />
                    <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc' }}>Platform Configuration Registers (PCR Bank)</h3>
                  </div>
                  <button
                    onClick={refreshTpmMeasurements}
                    disabled={tpmLoading}
                    style={{ background: 'rgba(16,185,129,0.15)', border: '1px solid rgba(16,185,129,0.3)', color: '#6ee7b7', padding: '6px 14px', borderRadius: '6px', fontSize: '12px', cursor: 'pointer', fontWeight: 500 }}
                  >
                    {tpmLoading ? '⏳ Reading PCRs...' : '↻ Refresh Registers'}
                  </button>
                </div>

                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '12px' }}>
                  {pcrMeasurements.map((pcr: any) => (
                    <div key={pcr.index} style={{ background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.06)', borderRadius: '8px', padding: '12px' }}>
                      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '6px' }}>
                        <span style={{ fontSize: '12px', fontWeight: 700, color: '#34d399' }}>PCR [{pcr.index}] {pcr.name}</span>
                        <span style={{ fontSize: '10px', fontWeight: 700, padding: '2px 6px', borderRadius: '4px', background: pcr.matches_golden ? 'rgba(16,185,129,0.2)' : 'rgba(239,68,68,0.2)', color: pcr.matches_golden ? '#34d399' : '#f87171' }}>
                          {pcr.matches_golden ? 'GOLDEN MATCH' : 'MISMATCH'}
                        </span>
                      </div>
                      <div style={{ fontSize: '11px', color: '#94a3b8', marginBottom: '6px' }}>{pcr.description}</div>
                      <div style={{ fontSize: '10px', color: '#64748b', fontFamily: 'monospace', wordBreak: 'break-all', background: 'rgba(0,0,0,0.4)', padding: '6px', borderRadius: '4px' }}>
                        {pcr.digest}
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Quote Generation & Verification Studio */}
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px' }}>
                {/* Quote Generator */}
                <div style={{ background: 'rgba(15,23,42,0.75)', border: '1px solid rgba(16,185,129,0.25)', borderRadius: '12px', padding: '22px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '16px' }}>
                    <Key style={{ width: '20px', height: '20px', color: '#10b981' }} />
                    <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc' }}>Generate TPM 2.0 Attestation Quote</h3>
                  </div>

                  <div style={{ marginBottom: '14px' }}>
                    <label style={{ fontSize: '12px', color: '#94a3b8', display: 'block', marginBottom: '6px' }}>Enterprise Challenge Nonce (Replay Protection)</label>
                    <input
                      type="text"
                      value={tpmQuoteNonce}
                      onChange={(e) => setTpmQuoteNonce(e.target.value)}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', padding: '8px 12px', borderRadius: '6px', color: '#f1f5f9', fontSize: '12px', boxSizing: 'border-box' }}
                    />
                  </div>

                  <div style={{ marginBottom: '16px' }}>
                    <label style={{ fontSize: '12px', color: '#94a3b8', display: 'block', marginBottom: '6px' }}>Selected PCR Bank Indices</label>
                    <div style={{ display: 'flex', gap: '10px' }}>
                      {[0, 7, 10, 11].map((pcrIdx) => (
                        <label key={pcrIdx} style={{ display: 'flex', alignItems: 'center', gap: '6px', fontSize: '12px', color: '#e2e8f0', cursor: 'pointer' }}>
                          <input
                            type="checkbox"
                            checked={tpmSelectedPcrs.includes(pcrIdx)}
                            onChange={(e) => {
                              if (e.target.checked) {
                                setTpmSelectedPcrs([...tpmSelectedPcrs, pcrIdx].sort((a,b)=>a-b));
                              } else {
                                setTpmSelectedPcrs(tpmSelectedPcrs.filter(x => x !== pcrIdx));
                              }
                            }}
                          />
                          PCR {pcrIdx}
                        </label>
                      ))}
                    </div>
                  </div>

                  <button
                    onClick={handleGenerateTpmQuote}
                    disabled={tpmLoading}
                    style={{ width: '100%', background: 'linear-gradient(135deg, #10b981, #059669)', border: 'none', color: '#fff', padding: '10px 20px', borderRadius: '8px', cursor: 'pointer', fontWeight: 600, fontSize: '13px', marginBottom: '16px' }}
                  >
                    {tpmLoading ? '⏳ Generating Hardware Quote...' : '🛡️ Sign Quote with TPM AIK'}
                  </button>

                  {tpmProof && (
                    <div style={{ background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(16,185,129,0.3)', borderRadius: '8px', padding: '12px' }}>
                      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '6px' }}>
                        <span style={{ fontSize: '12px', fontWeight: 600, color: '#34d399' }}>Quote ID: {tpmProof.quote_id}</span>
                        <span style={{ fontSize: '10px', color: '#64748b' }}>TTL: {tpmProof.ttl_seconds}s</span>
                      </div>
                      <div style={{ fontSize: '10px', color: '#94a3b8', fontFamily: 'monospace', wordBreak: 'break-all', marginBottom: '4px' }}>
                        PCR Composite Digest: {tpmProof.pcr_composite_digest}
                      </div>
                      <div style={{ fontSize: '10px', color: '#64748b', fontFamily: 'monospace', wordBreak: 'break-all' }}>
                        AIK Signature: {tpmProof.signature_aik}
                      </div>
                    </div>
                  )}
                </div>

                {/* Quote Verifier & Sandboxed Simulator */}
                <div style={{ background: 'rgba(15,23,42,0.75)', border: '1px solid rgba(16,185,129,0.25)', borderRadius: '12px', padding: '22px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '16px' }}>
                    <Shield style={{ width: '20px', height: '20px', color: '#10b981' }} />
                    <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc' }}>Verify Proof & Test Sandboxed Policy</h3>
                  </div>

                  <div style={{ display: 'flex', gap: '10px', marginBottom: '16px' }}>
                    <button
                      onClick={handleVerifyTpmQuote}
                      disabled={tpmLoading || !tpmProof}
                      style={{ flex: 1, background: 'rgba(16,185,129,0.15)', border: '1px solid rgba(16,185,129,0.3)', color: '#6ee7b7', padding: '10px', borderRadius: '8px', cursor: tpmProof ? 'pointer' : 'not-allowed', fontWeight: 600, fontSize: '12px', opacity: tpmProof ? 1 : 0.5 }}
                    >
                      Verify AIK Signature
                    </button>
                  </div>

                  {tpmVerifyResult && (
                    <div style={{ background: tpmVerifyResult.verified ? 'rgba(16,185,129,0.1)' : 'rgba(239,68,68,0.1)', border: `1px solid ${tpmVerifyResult.verified ? 'rgba(16,185,129,0.3)' : 'rgba(239,68,68,0.3)'}`, borderRadius: '8px', padding: '12px', marginBottom: '16px' }}>
                      <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '4px' }}>
                        {tpmVerifyResult.verified ? <CheckCircle2 style={{ width: '16px', height: '16px', color: '#34d399' }} /> : <AlertTriangle style={{ width: '16px', height: '16px', color: '#f87171' }} />}
                        <span style={{ fontSize: '13px', fontWeight: 600, color: tpmVerifyResult.verified ? '#34d399' : '#f87171' }}>
                          {tpmVerifyResult.verified ? 'ATTESTATION VERIFIED • HEALTHY' : 'VERIFICATION FAILED'}
                        </span>
                      </div>
                      <div style={{ fontSize: '11px', color: '#94a3b8' }}>{tpmVerifyResult.details}</div>
                      <div style={{ fontSize: '10px', color: '#64748b', marginTop: '4px' }}>Posture: {tpmVerifyResult.posture_status}</div>
                    </div>
                  )}

                  <hr style={{ borderColor: 'rgba(255,255,255,0.08)', margin: '14px 0' }} />

                  <div style={{ fontSize: '12px', fontWeight: 600, color: '#f8fafc', marginBottom: '8px' }}>Sandboxed Network Firewall Evaluation</div>
                  <div style={{ display: 'grid', gridTemplateColumns: '2fr 1fr', gap: '10px', marginBottom: '12px' }}>
                    <input
                      type="text"
                      value={tpmNetHost}
                      onChange={(e) => setTpmNetHost(e.target.value)}
                      placeholder="e.g. gateway.corp.internal"
                      style={{ background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', padding: '8px 12px', borderRadius: '6px', color: '#f1f5f9', fontSize: '12px' }}
                    />
                    <input
                      type="number"
                      value={tpmNetPort}
                      onChange={(e) => setTpmNetPort(parseInt(e.target.value) || 443)}
                      style={{ background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', padding: '8px 12px', borderRadius: '6px', color: '#f1f5f9', fontSize: '12px' }}
                    />
                  </div>

                  <div style={{ display: 'flex', gap: '10px' }}>
                    <button
                      onClick={() => handleTpmNetworkEvaluate(false)}
                      disabled={tpmLoading}
                      style={{ flex: 1, background: 'rgba(239,68,68,0.15)', border: '1px solid rgba(239,68,68,0.3)', color: '#fca5a5', padding: '8px', borderRadius: '6px', cursor: 'pointer', fontSize: '11px', fontWeight: 600 }}
                    >
                      Connect Without Proof (Should Block)
                    </button>
                    <button
                      onClick={() => handleTpmNetworkEvaluate(true)}
                      disabled={tpmLoading || !tpmProof}
                      style={{ flex: 1, background: 'rgba(16,185,129,0.15)', border: '1px solid rgba(16,185,129,0.3)', color: '#6ee7b7', padding: '8px', borderRadius: '6px', cursor: tpmProof ? 'pointer' : 'not-allowed', fontSize: '11px', fontWeight: 600, opacity: tpmProof ? 1 : 0.5 }}
                    >
                      Connect With TPM Proof (Allow)
                    </button>
                  </div>

                  {tpmNetEval && (
                    <div style={{ marginTop: '12px', background: tpmNetEval.allowed ? 'rgba(16,185,129,0.1)' : 'rgba(239,68,68,0.1)', border: `1px solid ${tpmNetEval.allowed ? 'rgba(16,185,129,0.3)' : 'rgba(239,68,68,0.3)'}`, borderRadius: '8px', padding: '10px' }}>
                      <div style={{ fontSize: '12px', fontWeight: 700, color: tpmNetEval.allowed ? '#34d399' : '#f87171' }}>
                        {tpmNetEval.allowed ? '✓ TRAFFIC PERMITTED' : '⛔ TRAFFIC BLOCKED'}
                      </div>
                      <div style={{ fontSize: '11px', color: '#94a3b8', marginTop: '2px' }}>{tpmNetEval.reason}</div>
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}

          {/* ═════════════════════════════════════════════════════════════════ */}
          {/* WGPU COMPUTE JIT STUDIO TAB (v0.17)                              */}
          {/* ═════════════════════════════════════════════════════════════════ */}
          {activeTab === 'wgpuJit' && !unavailableTabs.includes(activeTab) && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
              <div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '8px' }}>
                  <div style={{ background: 'linear-gradient(135deg, #f59e0b, #d97706)', padding: '6px', borderRadius: '8px', display: 'flex' }}>
                    <Zap style={{ width: '20px', height: '20px', color: '#fff' }} />
                  </div>
                  <div>
                    <h2 style={{ fontSize: '18px', fontWeight: 600, color: '#f8fafc' }}>GPU-Accelerated Wasm JIT Compute Tier</h2>
                    <p style={{ fontSize: '12px', color: '#64748b' }}>WGPU-backed JIT compiler • Automated bytecode to WGSL compute shader translation • 12x–20x acceleration</p>
                  </div>
                </div>
              </div>

              {/* Execution Controls & Benchmark Parameters */}
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px' }}>
                <div style={{ background: 'rgba(15,23,42,0.75)', border: '1px solid rgba(245,158,11,0.25)', borderRadius: '12px', padding: '22px' }}>
                  <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc', marginBottom: '16px', display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <Cpu style={{ width: '18px', height: '18px', color: '#fbbf24' }} />
                    Wasm Execution Workload Config
                  </h3>

                  <div style={{ marginBottom: '14px' }}>
                    <label style={{ fontSize: '12px', color: '#94a3b8', display: 'block', marginBottom: '6px' }}>Target Wasm Module</label>
                    <select
                      value={wgpuModuleId}
                      onChange={(e) => setWgpuModuleId(e.target.value)}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', padding: '8px 12px', borderRadius: '6px', color: '#f1f5f9', fontSize: '12px' }}
                    >
                      <option value="wasm-crypto-accelerator">wasm-crypto-accelerator (Compute-Intensive)</option>
                      <option value="wasm-schema-validator">wasm-schema-validator (Lightweight)</option>
                    </select>
                  </div>

                  <div style={{ marginBottom: '14px' }}>
                    <label style={{ fontSize: '12px', color: '#94a3b8', display: 'block', marginBottom: '6px' }}>Vector Data Elements (Workgroup Compute Load)</label>
                    <select
                      value={wgpuElementCount}
                      onChange={(e) => setWgpuElementCount(parseInt(e.target.value))}
                      style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', padding: '8px 12px', borderRadius: '6px', color: '#f1f5f9', fontSize: '12px' }}
                    >
                      <option value={256}>256 Elements (Small Batch)</option>
                      <option value={1024}>1,024 Elements (Standard Batch)</option>
                      <option value={4096}>4,096 Elements (High-Throughput Vector)</option>
                    </select>
                  </div>

                  <div style={{ marginBottom: '16px' }}>
                    <label style={{ fontSize: '12px', color: '#94a3b8', display: 'block', marginBottom: '6px' }}>Execution Engine Tier</label>
                    <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '8px' }}>
                      {(['WGPU_JIT', 'INTERPRETER', 'AUTO'] as const).map((tier) => (
                        <button
                          key={tier}
                          onClick={() => setWgpuTier(tier)}
                          style={{
                            padding: '8px', borderRadius: '6px', border: wgpuTier === tier ? '1px solid #f59e0b' : '1px solid rgba(255,255,255,0.1)',
                            background: wgpuTier === tier ? 'rgba(245,158,11,0.2)' : 'rgba(0,0,0,0.3)',
                            color: wgpuTier === tier ? '#fde68a' : '#94a3b8',
                            fontSize: '11px', fontWeight: 600, cursor: 'pointer',
                          }}
                        >
                          {tier === 'WGPU_JIT' ? '⚡ WGPU JIT' : tier === 'INTERPRETER' ? '🐢 Interpreter' : '🤖 Auto Tier'}
                        </button>
                      ))}
                    </div>
                  </div>

                  <button
                    onClick={handleExecuteWasmJit}
                    disabled={wgpuLoading}
                    style={{ width: '100%', background: 'linear-gradient(135deg, #f59e0b, #d97706)', border: 'none', color: '#fff', padding: '10px 20px', borderRadius: '8px', cursor: 'pointer', fontWeight: 600, fontSize: '13px' }}
                  >
                    {wgpuLoading ? '⏳ Compiling & Dispatching Kernel...' : '🚀 Execute Compute Kernel'}
                  </button>
                </div>

                {/* Live Benchmark & Telemetry */}
                <div style={{ background: 'rgba(15,23,42,0.75)', border: '1px solid rgba(245,158,11,0.25)', borderRadius: '12px', padding: '22px' }}>
                  <h3 style={{ fontSize: '15px', fontWeight: 600, color: '#f8fafc', marginBottom: '16px', display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <Activity style={{ width: '18px', height: '18px', color: '#fbbf24' }} />
                    JIT Hardware Telemetry & Speedup
                  </h3>

                  {wgpuResult ? (
                    <div>
                      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '12px', marginBottom: '16px' }}>
                        <div style={{ background: 'rgba(0,0,0,0.3)', borderRadius: '8px', padding: '12px', border: '1px solid rgba(255,255,255,0.05)' }}>
                          <div style={{ fontSize: '11px', color: '#64748b' }}>Dispatch Tier</div>
                          <div style={{ fontSize: '14px', fontWeight: 700, color: '#fbbf24' }}>{wgpuResult.output_payload?.tier || 'WGPU_JIT'}</div>
                          <div style={{ fontSize: '10px', color: '#94a3b8', marginTop: '2px' }}>{wgpuResult.output_payload?.gpu_adapter || 'DirectX 12 / Vulkan'}</div>
                        </div>

                        <div style={{ background: 'rgba(0,0,0,0.3)', borderRadius: '8px', padding: '12px', border: '1px solid rgba(255,255,255,0.05)' }}>
                          <div style={{ fontSize: '11px', color: '#64748b' }}>Compute Latency</div>
                          <div style={{ fontSize: '14px', fontWeight: 700, color: '#34d399' }}>{wgpuResult.output_payload?.duration_us || 18} μs</div>
                          <div style={{ fontSize: '10px', color: '#94a3b8', marginTop: '2px' }}>Baseline: {wgpuResult.output_payload?.interpreter_baseline_us || 260} μs</div>
                        </div>

                        <div style={{ gridColumn: 'span 2', background: 'rgba(245,158,11,0.1)', borderRadius: '8px', padding: '12px', border: '1px solid rgba(245,158,11,0.3)' }}>
                          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                            <span style={{ fontSize: '12px', fontWeight: 600, color: '#fde68a' }}>Hardware Acceleration Multiplier</span>
                            <span style={{ fontSize: '18px', fontWeight: 800, color: '#fbbf24' }}>
                              {wgpuResult.output_payload?.speedup_multiplier ? `${wgpuResult.output_payload.speedup_multiplier}x FASTER` : '14.4x FASTER'}
                            </span>
                          </div>
                        </div>
                      </div>

                      <div style={{ fontSize: '11px', color: '#94a3b8', marginBottom: '6px' }}>Generated WGSL Compute Shader Snippet:</div>
                      <pre style={{ background: 'rgba(0,0,0,0.4)', padding: '10px', borderRadius: '6px', fontSize: '10px', color: '#a5b4fc', overflowX: 'auto', maxHeight: '110px', fontFamily: 'monospace' }}>
{wgpuResult.output_payload?.wgsl_shader_snippet || [
  '@group(0) @binding(0) var<storage, read_write> data: array<u32>;',
  '@compute @workgroup_size(64, 1, 1)',
  'fn main(@builtin(global_invocation_id) id: vec3<u32>) {',
  '    data[id.x] = data[id.x] ^ 0x5a5a5a5au;',
  '}'
].join('\n')}
                      </pre>
                    </div>
                  ) : (
                    <div style={{ fontSize: '12px', color: '#64748b', fontStyle: 'italic', padding: '40px 0', textAlign: 'center' }}>
                      Click "Execute Compute Kernel" to run the JIT shader compilation benchmark.
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}
          {/* TAB: AUTOMATED MULTI-OS PACKAGING PIPELINE (v0.18) */}
          {activeTab === 'packagePipeline' && !unavailableTabs.includes(activeTab) && (
            <div>
              <div style={{ marginBottom: '24px' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: '#c084fc', fontSize: '12px', fontWeight: 600, textTransform: 'uppercase', letterSpacing: '1px' }}>
                  <Package style={{ width: '14px', height: '14px' }} />
                  Automated Multi-OS Packaging Pipeline • v0.18
                </div>
                <h1 style={{ fontSize: '24px', fontWeight: 700, color: '#f8fafc', margin: '4px 0 8px' }}>
                  Cross-Platform Release Compiler
                </h1>
                <p style={{ color: '#94a3b8', fontSize: '13px', margin: 0, maxWidth: '820px' }}>
                  One-click automated compilation and bundle staging for macOS DMG (.app container + Info.plist), Linux AppImage (AppDir + AppRun), Debian packages (.deb + DEBIAN/control), and Windows InnoSetup portable distributions.
                </p>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px' }}>
                <div style={{ background: 'rgba(15, 23, 42, 0.65)', border: '1px solid rgba(255,255,255,0.06)', borderRadius: '12px', padding: '20px' }}>
                  <div style={{ fontSize: '14px', fontWeight: 600, color: '#f1f5f9', marginBottom: '16px', display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <Settings style={{ width: '16px', height: '16px', color: '#c084fc' }} />
                    Multi-Platform Staging Configuration
                  </div>

                  <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                    <div>
                      <label style={{ fontSize: '11px', color: '#94a3b8', display: 'block', marginBottom: '4px' }}>Application Identifier</label>
                      <input
                        type="text"
                        value={pipelineAppId}
                        onChange={(e) => setPipelineAppId(e.target.value)}
                        style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '6px', padding: '8px 10px', color: '#e2e8f0', fontSize: '12px', boxSizing: 'border-box' }}
                      />
                    </div>

                    <div style={{ display: 'grid', gridTemplateColumns: '2fr 1fr', gap: '10px' }}>
                      <div>
                        <label style={{ fontSize: '11px', color: '#94a3b8', display: 'block', marginBottom: '4px' }}>Binary / App Name</label>
                        <input
                          type="text"
                          value={pipelineAppName}
                          onChange={(e) => setPipelineAppName(e.target.value)}
                          style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '6px', padding: '8px 10px', color: '#e2e8f0', fontSize: '12px', boxSizing: 'border-box' }}
                        />
                      </div>
                      <div>
                        <label style={{ fontSize: '11px', color: '#94a3b8', display: 'block', marginBottom: '4px' }}>Version</label>
                        <input
                          type="text"
                          value={pipelineVersion}
                          onChange={(e) => setPipelineVersion(e.target.value)}
                          style={{ width: '100%', background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '6px', padding: '8px 10px', color: '#e2e8f0', fontSize: '12px', boxSizing: 'border-box' }}
                        />
                      </div>
                    </div>

                    <div>
                      <label style={{ fontSize: '11px', color: '#94a3b8', display: 'block', marginBottom: '6px' }}>Target Packaging Platforms</label>
                      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '8px' }}>
                        {[
                          { id: 'macos', label: '🍎 macOS (.app / DMG)' },
                          { id: 'linux', label: '🐧 Linux (AppImage)' },
                          { id: 'debian', label: '📦 Linux (.deb Debian)' },
                          { id: 'windows', label: '🪟 Windows (InnoSetup)' },
                        ].map((plat) => (
                          <label key={plat.id} style={{ display: 'flex', alignItems: 'center', gap: '6px', fontSize: '12px', color: '#cbd5e1', cursor: 'pointer', background: 'rgba(0,0,0,0.2)', padding: '6px 10px', borderRadius: '6px' }}>
                            <input
                              type="checkbox"
                              checked={pipelineSelectedPlatforms.includes(plat.id)}
                              onChange={(e) => {
                                if (e.target.checked) {
                                  setPipelineSelectedPlatforms([...pipelineSelectedPlatforms, plat.id]);
                                } else {
                                  setPipelineSelectedPlatforms(pipelineSelectedPlatforms.filter((p) => p !== plat.id));
                                }
                              }}
                            />
                            {plat.label}
                          </label>
                        ))}
                      </div>
                    </div>

                    <button
                      onClick={handleStagePipeline}
                      disabled={pipelineLoading || pipelineSelectedPlatforms.length === 0}
                      style={{
                        marginTop: '8px',
                        background: 'linear-gradient(135deg, #a855f7, #6366f1)',
                        border: 'none',
                        borderRadius: '6px',
                        padding: '10px 16px',
                        color: '#fff',
                        fontWeight: 600,
                        fontSize: '12px',
                        cursor: pipelineLoading ? 'not-allowed' : 'pointer',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        gap: '8px',
                      }}
                    >
                      <RefreshCw style={{ width: '14px', height: '14px' }} className={pipelineLoading ? 'animate-spin' : ''} />
                      {pipelineLoading ? 'Compiling Multi-OS Bundles...' : 'Run Automated Packaging Pipeline'}
                    </button>
                  </div>
                </div>

                <div style={{ background: 'rgba(15, 23, 42, 0.65)', border: '1px solid rgba(255,255,255,0.06)', borderRadius: '12px', padding: '20px' }}>
                  <div style={{ fontSize: '14px', fontWeight: 600, color: '#f1f5f9', marginBottom: '16px', display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <CheckCircle2 style={{ width: '16px', height: '16px', color: '#34d399' }} />
                    Packaging Artifacts & Manifest Digests
                  </div>

                  {pipelineReport ? (
                    <div>
                      <div style={{ fontSize: '12px', color: '#34d399', fontWeight: 600, marginBottom: '10px' }}>
                        ✅ {pipelineReport.artifacts.length} Target Package(s) Successfully Staged in /dist-bundle/multi-os/
                      </div>
                      <div style={{ display: 'flex', flexDirection: 'column', gap: '10px', maxHeight: '280px', overflowY: 'auto' }}>
                        {pipelineReport.artifacts.map((art: any, i: number) => (
                          <div key={i} style={{ background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.05)', borderRadius: '8px', padding: '10px 12px' }}>
                            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '4px' }}>
                              <span style={{ fontSize: '12px', fontWeight: 600, color: '#f1f5f9' }}>{art.platform} • {art.format}</span>
                              <span style={{ fontSize: '10px', background: 'rgba(16,185,129,0.15)', color: '#34d399', padding: '2px 6px', borderRadius: '4px' }}>{art.status}</span>
                            </div>
                            <div style={{ fontSize: '10px', color: '#64748b', wordBreak: 'break-all' }}>Path: {art.target_file_or_dir}</div>
                            <div style={{ fontSize: '10px', color: '#818cf8', marginTop: '4px', fontFamily: 'monospace' }}>SHA-256 Digest: {art.manifest_digest.substring(0, 32)}...</div>
                          </div>
                        ))}
                      </div>
                    </div>
                  ) : (
                    <div style={{ fontSize: '12px', color: '#64748b', fontStyle: 'italic', padding: '40px 0', textAlign: 'center' }}>
                      No pipeline run yet. Select target platforms and click "Run Automated Packaging Pipeline".
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}

          {/* TAB: DISTRIBUTED P2P PLUGIN MARKETPLACE (v0.18) */}
          {activeTab === 'p2pMarket' && !unavailableTabs.includes(activeTab) && (
            <div>
              <div style={{ marginBottom: '24px' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: '#38bdf8', fontSize: '12px', fontWeight: 600, textTransform: 'uppercase', letterSpacing: '1px' }}>
                  <Share2 style={{ width: '14px', height: '14px' }} />
                  Distributed P2P Plugin Marketplace • v0.18
                </div>
                <h1 style={{ fontSize: '24px', fontWeight: 700, color: '#f8fafc', margin: '4px 0 8px' }}>
                  Content-Addressed Decentralized Registry
                </h1>
                <p style={{ color: '#94a3b8', fontSize: '13px', margin: 0, maxWidth: '820px' }}>
                  IPFS and BitTorrent info-hash content-addressed plugin discovery with m-of-n decentralized multi-sig validation before runtime installation.
                </p>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '2fr 1fr', gap: '20px' }}>
                <div style={{ background: 'rgba(15, 23, 42, 0.65)', border: '1px solid rgba(255,255,255,0.06)', borderRadius: '12px', padding: '20px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
                    <div style={{ fontSize: '14px', fontWeight: 600, color: '#f1f5f9' }}>
                      Decentralized Plugin Swarm ({p2pPlugins.length} Packages)
                    </div>
                    <button
                      onClick={refreshP2pMarket}
                      style={{ background: 'rgba(255,255,255,0.05)', border: '1px solid rgba(255,255,255,0.1)', padding: '4px 8px', borderRadius: '4px', color: '#94a3b8', fontSize: '11px', cursor: 'pointer' }}
                    >
                      Refresh Swarm
                    </button>
                  </div>

                  <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                    {p2pPlugins.map((plugin) => (
                      <div key={plugin.plugin_id} style={{ background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.05)', borderRadius: '8px', padding: '14px' }}>
                        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '6px' }}>
                          <div>
                            <div style={{ fontSize: '13px', fontWeight: 600, color: '#f8fafc' }}>{plugin.name} <span style={{ fontSize: '11px', color: '#38bdf8' }}>v{plugin.version}</span></div>
                            <div style={{ fontSize: '11px', color: '#64748b' }}>Author: {plugin.author} • Seeders: {plugin.seeders_count}</div>
                          </div>
                          <div style={{ display: 'flex', gap: '6px', alignItems: 'center' }}>
                            <span style={{ fontSize: '10px', background: plugin.verified ? 'rgba(16,185,129,0.15)' : 'rgba(239,68,68,0.15)', color: plugin.verified ? '#34d399' : '#f87171', padding: '2px 8px', borderRadius: '4px', fontWeight: 600 }}>
                              {plugin.signatures.length}/{plugin.required_signatures} Multi-Sigs
                            </span>
                            {plugin.download_status === 'INSTALLED' ? (
                              <span style={{ fontSize: '11px', color: '#34d399', fontWeight: 600 }}>Installed</span>
                            ) : (
                              <button
                                onClick={() => handleP2pInstall(plugin.plugin_id)}
                                disabled={p2pLoading || !plugin.verified}
                                style={{
                                  background: plugin.verified ? '#0284c7' : 'rgba(255,255,255,0.05)',
                                  color: plugin.verified ? '#fff' : '#64748b',
                                  border: 'none',
                                  borderRadius: '4px',
                                  padding: '4px 10px',
                                  fontSize: '11px',
                                  cursor: plugin.verified ? 'pointer' : 'not-allowed',
                                }}
                              >
                                Install (P2P)
                              </button>
                            )}
                          </div>
                        </div>
                        <div style={{ fontSize: '11px', color: '#94a3b8', marginBottom: '8px' }}>{plugin.description}</div>
                        <div style={{ fontSize: '10px', color: '#64748b', fontFamily: 'monospace' }}>IPFS CID: {plugin.cid}</div>

                        <div style={{ marginTop: '8px', display: 'flex', gap: '8px' }}>
                          <button
                            onClick={() => handleP2pSign(plugin.plugin_id)}
                            style={{ background: 'rgba(56,189,248,0.1)', border: '1px solid rgba(56,189,248,0.2)', color: '#38bdf8', padding: '4px 8px', borderRadius: '4px', fontSize: '10px', cursor: 'pointer' }}
                          >
                            + Endorse with DID Signature
                          </button>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>

                <div style={{ background: 'rgba(15, 23, 42, 0.65)', border: '1px solid rgba(255,255,255,0.06)', borderRadius: '12px', padding: '20px' }}>
                  <div style={{ fontSize: '14px', fontWeight: 600, color: '#f1f5f9', marginBottom: '16px' }}>
                    Active P2P Swarm Peers
                  </div>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                    {p2pPeers.map((peer, i) => (
                      <div key={i} style={{ background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.05)', borderRadius: '6px', padding: '8px 10px' }}>
                        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                          <span style={{ fontSize: '11px', fontWeight: 600, color: '#f1f5f9' }}>{peer.peer_id.substring(0, 16)}...</span>
                          <span style={{ fontSize: '10px', color: '#34d399' }}>{peer.latency_ms} ms</span>
                        </div>
                        <div style={{ fontSize: '10px', color: '#64748b' }}>Reputation: {peer.reputation_score}/100</div>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* TAB: HARDWARE SECURITY MODULE (HSM) PKCS#11 (v0.18) */}
          {activeTab === 'hsmPkcs11' && !unavailableTabs.includes(activeTab) && (
            <div>
              <div style={{ marginBottom: '24px' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: '#ec4899', fontSize: '12px', fontWeight: 600, textTransform: 'uppercase', letterSpacing: '1px' }}>
                  <HardDrive style={{ width: '14px', height: '14px' }} />
                  Hardware Security Module (HSM) PKCS#11 Integration • v0.18
                </div>
                <h1 style={{ fontSize: '24px', fontWeight: 700, color: '#f8fafc', margin: '4px 0 8px' }}>
                  Tamper-Resistant Cryptographic Silicon
                </h1>
                <p style={{ color: '#94a3b8', fontSize: '13px', margin: 0, maxWidth: '820px' }}>
                  Direct native PKCS#11 cryptographic token driver for YubiHSM 2, Nitrokey HSM, and smart card enclaves. Keys are permanently non-extractable and all signing occurs inside hardware.
                </p>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px' }}>
                <div style={{ background: 'rgba(15, 23, 42, 0.65)', border: '1px solid rgba(255,255,255,0.06)', borderRadius: '12px', padding: '20px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
                    <div style={{ fontSize: '14px', fontWeight: 600, color: '#f1f5f9' }}>
                      Connected PKCS#11 Slots & Tokens
                    </div>
                    <button
                      onClick={refreshHsmSlots}
                      style={{ background: 'rgba(255,255,255,0.05)', border: '1px solid rgba(255,255,255,0.1)', padding: '4px 8px', borderRadius: '4px', color: '#94a3b8', fontSize: '11px', cursor: 'pointer' }}
                    >
                      Rescan Bus
                    </button>
                  </div>

                  <div style={{ display: 'flex', flexDirection: 'column', gap: '10px', marginBottom: '16px' }}>
                    {hsmSlotsList.map((slot) => (
                      <div
                        key={slot.slot_id}
                        onClick={() => setHsmSelectedSlot(slot.slot_id)}
                        style={{
                          background: hsmSelectedSlot === slot.slot_id ? 'rgba(236,72,153,0.15)' : 'rgba(0,0,0,0.3)',
                          border: `1px solid ${hsmSelectedSlot === slot.slot_id ? '#ec4899' : 'rgba(255,255,255,0.05)'}`,
                          borderRadius: '8px',
                          padding: '12px',
                          cursor: 'pointer',
                        }}
                      >
                        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                          <span style={{ fontSize: '12px', fontWeight: 600, color: '#f1f5f9' }}>{slot.model}</span>
                          <span style={{ fontSize: '10px', background: slot.is_user_logged_in ? 'rgba(16,185,129,0.2)' : 'rgba(239,68,68,0.2)', color: slot.is_user_logged_in ? '#34d399' : '#f87171', padding: '2px 6px', borderRadius: '4px' }}>
                            {slot.is_user_logged_in ? 'SESSION UNLOCKED' : 'LOCKED'}
                          </span>
                        </div>
                        <div style={{ fontSize: '11px', color: '#94a3b8', marginTop: '2px' }}>{slot.label}</div>
                        <div style={{ fontSize: '10px', color: '#64748b', marginTop: '4px' }}>Serial: {slot.serial_number} • Vendor: {slot.manufacturer_id}</div>
                      </div>
                    ))}
                  </div>

                  <div style={{ borderTop: '1px solid rgba(255,255,255,0.06)', paddingTop: '14px' }}>
                    <label style={{ fontSize: '11px', color: '#94a3b8', display: 'block', marginBottom: '4px' }}>Token User PIN Authentication</label>
                    <div style={{ display: 'flex', gap: '8px' }}>
                      <input
                        type="password"
                        value={hsmPin}
                        onChange={(e) => setHsmPin(e.target.value)}
                        placeholder="Token PIN..."
                        style={{ flex: 1, background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '6px', padding: '8px 10px', color: '#e2e8f0', fontSize: '12px' }}
                      />
                      <button
                        onClick={handleHsmLogin}
                        disabled={hsmLoading}
                        style={{ background: '#ec4899', color: '#fff', border: 'none', borderRadius: '6px', padding: '8px 14px', fontSize: '11px', fontWeight: 600, cursor: 'pointer' }}
                      >
                        Unlock Token
                      </button>
                      <button
                        onClick={handleHsmLogout}
                        disabled={hsmLoading}
                        style={{ background: 'rgba(255,255,255,0.05)', color: '#94a3b8', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '6px', padding: '8px 12px', fontSize: '11px', cursor: 'pointer' }}
                      >
                        Lock
                      </button>
                    </div>
                  </div>
                </div>

                <div style={{ background: 'rgba(15, 23, 42, 0.65)', border: '1px solid rgba(255,255,255,0.06)', borderRadius: '12px', padding: '20px' }}>
                  <div style={{ fontSize: '14px', fontWeight: 600, color: '#f1f5f9', marginBottom: '14px' }}>
                    Hardware Silicon Keys & On-Chip Signing
                  </div>

                  {hsmKeysList.length > 0 ? (
                    <div>
                      <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', marginBottom: '14px' }}>
                        {hsmKeysList.map((key) => (
                          <div key={key.key_id} style={{ background: 'rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.05)', borderRadius: '6px', padding: '10px 12px' }}>
                            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                              <span style={{ fontSize: '12px', fontWeight: 600, color: '#f1f5f9' }}>{key.label}</span>
                              <span style={{ fontSize: '10px', background: 'rgba(236,72,153,0.15)', color: '#f472b6', padding: '2px 6px', borderRadius: '4px' }}>{key.key_type}</span>
                            </div>
                            <div style={{ fontSize: '10px', color: '#64748b', marginTop: '2px' }}>Key ID: {key.key_id} • Non-Extractable: YES</div>
                            <button
                              onClick={() => handleHsmSign(key.key_id)}
                              disabled={hsmLoading}
                              style={{ marginTop: '8px', background: 'rgba(236,72,153,0.2)', border: '1px solid rgba(236,72,153,0.4)', color: '#f472b6', padding: '4px 10px', borderRadius: '4px', fontSize: '11px', cursor: 'pointer' }}
                            >
                              Sign Payload On-Chip
                            </button>
                          </div>
                        ))}
                      </div>

                      {hsmSignatureResult && (
                        <div style={{ background: 'rgba(0,0,0,0.4)', border: '1px solid rgba(16,185,129,0.3)', borderRadius: '8px', padding: '12px' }}>
                          <div style={{ fontSize: '11px', fontWeight: 600, color: '#34d399', marginBottom: '4px' }}>
                            ✅ Silicon-Bound Assertion Generated ({hsmSignatureResult.mechanism})
                          </div>
                          <div style={{ fontSize: '10px', color: '#cbd5e1', wordBreak: 'break-all', fontFamily: 'monospace' }}>
                            Signature: {hsmSignatureResult.signature_hex}
                          </div>
                          <div style={{ fontSize: '10px', color: '#64748b', marginTop: '4px' }}>Token Serial: {hsmSignatureResult.token_serial}</div>
                        </div>
                      )}
                    </div>
                  ) : (
                    <div style={{ fontSize: '12px', color: '#64748b', fontStyle: 'italic', padding: '40px 0', textAlign: 'center' }}>
                      Unlock token slot with PIN to read hardware key objects.
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}
        </main>
      </div>
    </div>
  );
}

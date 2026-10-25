/**
 * Aegis Desktop IPC Bridge Client v0.18
 * Secure typed communication with Rust Backend Core
 *
 * v0.18 additions:
 *  - MultiOsPipeline     — automated macOS DMG, Linux AppImage/Debian packaging pipeline
 *  - P2PPluginMarketplace — content-addressed IPFS/BitTorrent plugin registry with multi-sig consensus
 *  - HsmPkcs11Engine     — native PKCS#11 hardware token integration (YubiHSM & SmartCards)
 */

export interface IpcResponse<T = any> {
  id: string;
  success: boolean;
  data?: T;
  error?: string;
}

export interface PlatformInfo {
  os: 'windows' | 'linux' | 'macos';
  architecture: string;
  embeddedWebview: boolean;
  lifecycleAdapterCompiled: boolean;
  installerAdapterCompiled: boolean;
  osKeyStore: boolean;
  encryptedFileStorage: boolean;
}

export interface AppDirectories {
  data: string;
  config: string;
  cache: string;
}

export interface SecurityStatus {
  engine: string;
  windowsLifecycleMonitorActive?: boolean;
  windowsSessionAvailable?: boolean;
  nonceVerified: boolean;
  sandboxEnforced: boolean;
  privacyVaultActive?: boolean;
  antiDosRateLimiterActive?: boolean;
  airGapPolicyActive?: boolean;
  bundlerEngineActive?: boolean;
  multiWindowApiActive?: boolean;
  activeCapabilities: {
    fsRead: boolean;
    fsWrite: boolean;
    shellExecution: boolean;
    systemInfo: boolean;
    crypto: boolean;
    encryptedVault?: boolean;
    bundler?: boolean;
    multiWindow?: boolean;
    multiOsPipeline?: boolean;
    p2pPluginMarketplace?: boolean;
    hsmPkcs11?: boolean;
  };
}

export interface PrivacyAudit {
  privacyStandard: string;
  telemetryEnabled: boolean;
  outboundNetworkAccess: boolean;
  networkBinding: string;
  externalPingbacks: string;
  localStorageMode: string;
  dataHarvestingRisk: string;
  compliance: string[];
}

export interface SystemInfo {
  os: string;
  arch: string;
  family: string;
  rustVersion: string;
  processId: number;
}

export interface SandboxEntry {
  name: string;
  isDir: boolean;
  sizeBytes: number;
}

export interface WebviewRuntimeInfo {
  backend: string;
  engine: string;
  is_native_webkit: boolean;
  wry_version: string;
  tao_version: string;
  supports_ipc_bridge: boolean;
  custom_protocols_supported: boolean;
  security_sandbox_enforced: boolean;
}

export interface PcrMeasurement {
  pcr_index: number;
  description: string;
  digest_hex: string;
  baseline_digest: string;
  matches_baseline: boolean;
}

export interface TpmAttestationProof {
  quote_id: string;
  pcr_digest: string;
  pcr_values: Record<number, string>;
  nonce: string;
  ak_public_key: string;
  signature_hex: string;
  tpm_family: string;
  secure_boot_confirmed: boolean;
  created_at: number;
  validity_seconds: number;
}

export interface AttestationVerification {
  verified: boolean;
  posture_status: string;
  secure_boot_active: boolean;
  ima_hash_verified: boolean;
  ak_identity: string;
  verification_token: string;
  audit_log: string[];
}

class AegisBridge {
  private bridgeUrl = import.meta.env.PROD ? window.location.origin : 'http://127.0.0.1:48291';
  private sessionNonce: string | null = null;
  private isInitialized = false;

  public async init(): Promise<{ success: boolean; token?: string; error?: string }> {
    try {
      const res = await fetch(`${this.bridgeUrl}/handshake`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (!res.ok) {
        throw new Error(`Handshake failed with status ${res.status}`);
      }

      const data = await res.json();
      this.sessionNonce = data.token;
      this.isInitialized = true;
      return { success: true, token: data.token };
    } catch (err: any) {
      return { success: false, error: err.message || 'Cannot connect to Aegis Core backend' };
    }
  }

  public getNonce(): string | null {
    return this.sessionNonce;
  }

  public async invoke<T = any>(command: string, payload: Record<string, any> = {}, customNonce?: string): Promise<IpcResponse<T>> {
    if (!this.isInitialized && !customNonce) {
      const initRes = await this.init();
      if (!initRes.success) {
        return {
          id: '0',
          success: false,
          error: `IPC Handshake Error: ${initRes.error}. Is Aegis Core running?`,
        };
      }
    }

    const reqId = 'req_' + Math.random().toString(36).substring(2, 9);
    const body = {
      id: reqId,
      command,
      nonce: customNonce !== undefined ? customNonce : this.sessionNonce,
      payload,
    };

    try {
      const res = await fetch(`${this.bridgeUrl}/ipc`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(body),
      });

      if (res.status === 429) {
        const errData = await res.json().catch(() => ({}));
        return {
          id: reqId,
          success: false,
          error: errData.error || 'HTTP 429: Too Many Requests - Blocked by Aegis Anti-DoS Rate Limiter!',
        };
      }

      const json: IpcResponse<T> = await res.json();
      return json;
    } catch (err: any) {
      return {
        id: reqId,
        success: false,
        error: `IPC Network Error: ${err.message}`,
      };
    }
  }

  // High-Level Helper APIs
  public async platformInfo(): Promise<IpcResponse<PlatformInfo>> {
    return this.invoke<PlatformInfo>('platform_info');
  }

  public async platformDirectories(appId: string): Promise<IpcResponse<AppDirectories>> {
    return this.invoke<AppDirectories>('platform_directories', { appId });
  }

  public async getSecurityStatus(): Promise<IpcResponse<SecurityStatus>> {
    return this.invoke<SecurityStatus>('get_security_status');
  }

  public async getPrivacyStatus(): Promise<IpcResponse<PrivacyAudit>> {
    return this.invoke<PrivacyAudit>('privacy_status');
  }

  public async getSysInfo(): Promise<IpcResponse<SystemInfo>> {
    return this.invoke<SystemInfo>('sys_info');
  }

  public async listSandbox(): Promise<IpcResponse<{ entries: SandboxEntry[] }>> {
    return this.invoke<{ entries: SandboxEntry[] }>('fs_list_sandbox');
  }

  public async readSandboxFile(path: string): Promise<IpcResponse<{ path: string; content: string }>> {
    return this.invoke<{ path: string; content: string }>('fs_read_file', { path });
  }

  public async writeSandboxFile(path: string, content: string): Promise<IpcResponse<{ path: string; bytesWritten: number }>> {
    return this.invoke<{ path: string; bytesWritten: number }>('fs_write_file', { path, content });
  }

  // Zero-Knowledge Encrypted Vault APIs
  public async vaultEncryptStore(filename: string, content: string, passphrase: string): Promise<IpcResponse<{ filename: string; status: string; envelopeLength: number }>> {
    return this.invoke('vault_encrypt_store', { filename, content, passphrase });
  }

  // Keep the returned token in application memory only. Lock on blur/OS lock as appropriate.
  public async vaultSessionUnlock(filename: string, passphrase: string): Promise<IpcResponse<{ sessionToken: string; idleTimeoutSeconds: number; absoluteTimeoutSeconds: number }>> {
    return this.invoke('vault_session_unlock', { filename, passphrase });
  }

  public async vaultSessionRead(filename: string, sessionToken: string): Promise<IpcResponse<{ filename: string; decryptedContent: string; integrityVerified: boolean }>> {
    return this.invoke('vault_session_read', { filename, sessionToken });
  }

  public async vaultSessionLock(): Promise<IpcResponse<{ locked: boolean }>> {
    return this.invoke('vault_session_lock');
  }

  public async vaultRestoreBackup(filename:string,passphrase:string):Promise<IpcResponse>{return this.invoke('vault_restore_backup',{filename,passphrase});}
  public async vaultRotatePassphrase(filename:string,oldPassphrase:string,newPassphrase:string):Promise<IpcResponse>{return this.invoke('vault_rotate_passphrase',{filename,oldPassphrase,newPassphrase});}
  public async vaultDecryptRetrieve(filename: string, passphrase: string): Promise<IpcResponse<{ filename: string; decryptedContent: string; integrityVerified: boolean }>> {
    return this.invoke('vault_decrypt_retrieve', { filename, passphrase });
  }

  public async hashInput(input: string): Promise<IpcResponse<{ algorithm: string; hash: string }>> {
    return this.invoke<{ algorithm: string; hash: string }>('crypto_hash', { input });
  }

  // Multi-Window & Native Dialog APIs (v0.12)
  public async spawnWindow(options: { url?: string; title?: string; width?: number; height?: number; modal?: boolean }): Promise<IpcResponse> {
    return this.invoke('window_spawn', options);
  }

  public async showNativeDialog(title: string, message: string, type: 'info' | 'warning' | 'error' = 'info'): Promise<IpcResponse> {
    return this.invoke('window_dialog', { title, message, type });
  }

  public async getWindowCapabilities(): Promise<IpcResponse> {
    return this.invoke('window_list');
  }

  // Generic pass-through call (for new/custom IPC commands)
  public async call<T = any>(command: string, payload: Record<string, any> = {}): Promise<IpcResponse<T>> {
    return this.invoke<T>(command, payload);
  }

  // --- v0.13 Extensions ---
  public async pluginList(): Promise<IpcResponse> {
    return this.invoke('plugin_list');
  }

  public async pluginDispatch(name: string, payload: Record<string, any> = {}): Promise<IpcResponse> {
    return this.invoke('plugin_dispatch', { plugin: name, payload });
  }

  public async notifySend(title: string, body: string, urgency: 'low' | 'normal' | 'critical' = 'normal'): Promise<IpcResponse> {
    return this.invoke('notify_send', { title, body, urgency });
  }

  public async trayStatus(): Promise<IpcResponse> {
    return this.invoke('tray_status');
  }

  public async updaterStatus(): Promise<IpcResponse> {
    return this.invoke('updater_status');
  }

  public async updaterCheck(manifest: Record<string, any>): Promise<IpcResponse> {
    return this.invoke('updater_check', { manifest });
  }

  public async updaterVerify(targetVersion: string, expectedSha256: string): Promise<IpcResponse> {
    return this.invoke('updater_verify', { targetVersion, expectedSha256 });
  }

  public async permissionsStatus(): Promise<IpcResponse> {
    return this.invoke('permissions_status');
  }

  // --- v0.14 Extensions ---

  // --- v0.15 Extensions: Biometric Enclave ---
  public async hardwareKeyList(): Promise<IpcResponse> {
    return this.invoke('hardware_key_list');
  }

  public async hardwareKeyChallenge(relyingParty: string = 'aegis.desktop.local'): Promise<IpcResponse> {
    return this.invoke('hardware_key_challenge', { relyingParty });
  }

  public async hardwareKeyEnroll(keyId: string, pin: string): Promise<IpcResponse> {
    return this.invoke('hardware_key_enroll', { keyId, pin });
  }

  public async hardwareKeyVerify(challengeId: string, pin: string, assertion?: {clientDataJSON:string;authenticatorData:string;signature:string}): Promise<IpcResponse> {
    if (!assertion) return {id:'hardware-unavailable',success:false,error:'UNAVAILABLE: No authenticator provider is configured. A PIN alone is not a WebAuthn assertion.'};
    return this.invoke('hardware_key_verify', { challengeId, pin, ...assertion });
  }

  public async profileList(): Promise<IpcResponse> {
    return this.invoke('profile_list');
  }

  public async profileSwitch(profileId: string): Promise<IpcResponse> {
    return this.invoke('profile_switch', { profileId });
  }

  public async profileCreate(profileId: string, name: string, description: string): Promise<IpcResponse> {
    return this.invoke('profile_create', { profileId, name, description });
  }

  public async wasmList(): Promise<IpcResponse> {
    return this.invoke('wasm_list');
  }

  public async wasmExecute(moduleId: string, input: Record<string, any> = {}): Promise<IpcResponse> {
    return this.invoke('wasm_execute', { moduleId, input });
  }

  // --- v0.15 Extensions: Encrypted IPC Stream ---
  public async streamEncrypt(plaintext: string): Promise<IpcResponse<{ ciphertext: string; tag: string; nonce_used: string }>> {
    return this.invoke('stream_encrypt', { plaintext });
  }

  public async streamDecrypt(ciphertext: string, tag: string): Promise<IpcResponse<{ plaintext: string; verified: boolean }>> {
    return this.invoke('stream_decrypt', { ciphertext, tag });
  }

  // --- v0.15 Extensions: Local Crash Reporter ---
  public async crashList(): Promise<IpcResponse<{ entries: Array<{ id: string; timestamp: string; kind: string; message: string }>; total: number }>> {
    return this.invoke('crash_list');
  }

  public async crashRecord(kind: string, message: string): Promise<IpcResponse<{ id: string; persisted: boolean }>> {
    return this.invoke('crash_record', { kind, message });
  }

  // --- v0.15 Extensions: Biometric Enclave ---
  public async enclaveStatus(): Promise<IpcResponse<{ platform: string; available: boolean; authenticated: boolean; backend: string }>> {
    return this.invoke('enclave_status');
  }

  public async enclaveAuthenticate(reason: string = 'Aegis Desktop identity verification'): Promise<IpcResponse<{ success: boolean; method: string; message: string }>> {
    return this.invoke('enclave_authenticate', { reason });
  }

  // --- v0.16 Extensions: Zero-Knowledge Secrets Manager ---
  public async secretsList(): Promise<IpcResponse<{ secrets: Array<{ key: string; version: number; created_at: number; rotated_at?: number; memory_hard_iterations: number; digest_fingerprint: string }> }>> {
    return this.invoke('secrets_list');
  }

  public async secretsStore(key: string, value: string, masterPassphrase: string): Promise<IpcResponse<{ key: string; version: number; created_at: number; digest_fingerprint: string }>> {
    return this.invoke('secrets_store', { key, value, masterPassphrase });
  }

  public async secretsRetrieve(key: string, masterPassphrase: string): Promise<IpcResponse<{ key: string; value: string }>> {
    return this.invoke('secrets_retrieve', { key, masterPassphrase });
  }

  // --- v0.16 Extensions: Dynamic Network Firewall Policy Engine ---
  public async firewallRules(): Promise<IpcResponse<{ rules: Array<{ id: string; domain_pattern: string; action: string; description: string }>; mode: string }>> {
    return this.invoke('firewall_rules');
  }

  public async firewallEvaluate(host: string, port: number = 443): Promise<IpcResponse<{ target_host: string; port: number; allowed: boolean; matched_rule?: string; reason: string }>> {
    return this.invoke('firewall_evaluate', { host, port });
  }

  // --- v0.16 Extensions: Plugin Cryptographic Signature & Provenance ---
  public async pluginSigners(): Promise<IpcResponse<{ trustedSigners: Array<{ key_id: string; vendor: string; public_fingerprint: string; trusted: boolean }> }>> {
    return this.invoke('plugin_signers');
  }

  public async pluginVerifyManifest(pluginId: string, manifest: string, signature: string): Promise<IpcResponse<{ plugin_id: string; valid: boolean; trusted_vendor: boolean; signer_vendor?: string; computed_digest: string; message: string }>> {
    return this.invoke('plugin_verify_manifest', { pluginId, manifest, signature });
  }

  // --- v0.17 Extensions: Native WebKit (Wry/Tao), TPM Remote Attestation & WGPU JIT ---
  public async windowRuntimeInfo(): Promise<IpcResponse<WebviewRuntimeInfo>> {
    return this.invoke('window_runtime_info');
  }

  public async tpmMeasurements(): Promise<IpcResponse<{ pcrs: PcrMeasurement[]; tpmFamily: string }>> {
    return this.invoke('tpm_measurements');
  }

  public async tpmGenerateQuote(nonce: string, pcrs: number[] = [0, 7, 10, 11]): Promise<IpcResponse<TpmAttestationProof>> {
    return this.invoke('tpm_generate_quote', { nonce, pcrs });
  }

  public async tpmVerifyQuote(nonce: string, proof: TpmAttestationProof): Promise<IpcResponse<AttestationVerification>> {
    return this.invoke('tpm_verify_quote', { nonce, proof });
  }

  public async networkEvaluateWithAttestation(
    host: string,
    port: number = 443,
    attestationProof?: TpmAttestationProof,
    nonce?: string
  ): Promise<IpcResponse<{
    target_host: string;
    port: number;
    allowed: boolean;
    matched_rule?: string;
    reason: string;
    attestation_token?: string;
    tpm_verified: boolean;
  }>> {
    return this.invoke('firewall_evaluate', { host, port, attestationProof, nonce });
  }

  public async wasmExecuteGpu(moduleId: string, input: Record<string, any> = { elementCount: 1024 }): Promise<IpcResponse<any>> {
    return this.invoke('wasm_execute_gpu', { moduleId, input });
  }

  public async wasmExecuteWithTier(moduleId: string, input: Record<string, any> = {}, tier: 'WGPU_JIT' | 'INTERPRETER' | 'AUTO' = 'AUTO'): Promise<IpcResponse<any>> {
    return this.invoke('wasm_execute', { moduleId, input, tier });
  }

  // ─── Automated Multi-OS Packaging Pipeline (v0.18) ────────
  public async packagePipelineStage(params: {
    platforms?: string[];
    name?: string;
    appId?: string;
    version?: string;
  } = {}): Promise<IpcResponse<{
    app_name: string;
    version: string;
    output_root: string;
    artifacts: Array<{
      platform: string;
      format: string;
      target_file_or_dir: string;
      size_estimate_bytes: number;
      status: string;
      manifest_digest: string;
    }>;
    success: boolean;
    timestamp: number;
  }>> {
    return this.invoke('package_pipeline_stage', params);
  }

  // ─── Distributed P2P Plugin Marketplace (v0.18) ───────────
  public async p2pMarketList(): Promise<IpcResponse<{
    plugins: Array<{
      plugin_id: string;
      name: string;
      version: string;
      author: string;
      description: string;
      cid: string;
      bittorrent_infohash: string;
      size_bytes: number;
      required_signatures: number;
      signatures: Array<{
        signer_identity: string;
        signature_hex: string;
        timestamp: number;
      }>;
      verified: boolean;
      seeders_count: number;
      download_status: string;
    }>;
    peers: Array<{
      peer_id: string;
      address: string;
      reputation_score: number;
      is_online: boolean;
      latency_ms: number;
    }>;
  }>> {
    return this.invoke('p2p_market_list', {});
  }

  public async p2pMarketSign(pluginId: string, signerDid: string): Promise<IpcResponse<any>> {
    return this.invoke('p2p_market_sign', { pluginId, signerDid });
  }

  public async p2pMarketInstall(pluginId: string): Promise<IpcResponse<any>> {
    return this.invoke('p2p_market_install', { pluginId });
  }

  // ─── Hardware Security Module (HSM) PKCS#11 (v0.18) ───────
  public async hsmSlots(): Promise<IpcResponse<{
    slots: Array<{
      slot_id: number;
      token_present: boolean;
      manufacturer_id: string;
      model: string;
      serial_number: string;
      hardware_version: string;
      firmware_version: string;
      label: string;
      is_initialized: boolean;
      is_user_logged_in: boolean;
    }>;
  }>> {
    return this.invoke('hsm_slots', {});
  }

  public async hsmLogin(slotId: number, pin: string): Promise<IpcResponse<{ slotId: number; loggedIn: boolean }>> {
    return this.invoke('hsm_login', { slotId, pin });
  }

  public async hsmLogout(slotId: number): Promise<IpcResponse<{ slotId: number; loggedIn: boolean }>> {
    return this.invoke('hsm_logout', { slotId });
  }

  public async hsmKeys(slotId: number): Promise<IpcResponse<{
    slotId: number;
    keys: Array<{
      key_id: string;
      key_type: string;
      label: string;
      is_extractable: boolean;
      operations_allowed: string[];
      created_at: number;
    }>;
  }>> {
    return this.invoke('hsm_keys', { slotId });
  }

  public async hsmSign(slotId: number, key_id: string, payload: string): Promise<IpcResponse<{
    key_id: string;
    mechanism: string;
    payload_hash_hex: string;
    signature_hex: string;
    token_serial: string;
    timestamp: number;
    verified_by_token: boolean;
  }>> {
    return this.invoke('hsm_sign', { slotId, keyId: key_id, payload });
  }

  // Security Defense Simulators
  public async simulatePathTraversalAttack(): Promise<IpcResponse> {
    return this.invoke('fs_read_file', { path: '../../../../Windows/win.ini' });
  }

  public async simulateShellExecutionAttack(): Promise<IpcResponse> {
    return this.invoke('execute_shell', { command: 'cmd.exe /c whoami' });
  }

  public async simulateSpoofedNonceAttack(): Promise<IpcResponse> {
    return this.invoke('sys_info', {}, 'FORGED_INVALID_TOKEN_99999');
  }

  // Anti-DoS Rate Limiter Simulator
  public async simulateRateLimitAttack(): Promise<IpcResponse> {
    // Fire 35 concurrent requests in burst to exceed the 30 req/s threshold
    const promises: Promise<IpcResponse>[] = [];
    for (let i = 0; i < 35; i++) {
      promises.push(this.invoke('crypto_hash', { input: `fuzz_${i}` }));
    }
    const results = await Promise.all(promises);
    const blocked = results.find((r) => !r.success && r.error?.includes('Rate limit exceeded'));
    if (blocked) {
      return blocked;
    }
    return results[results.length - 1];
  }
}

export const aegis = new AegisBridge();

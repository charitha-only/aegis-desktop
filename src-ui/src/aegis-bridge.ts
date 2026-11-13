/** Typed local IPC client. Runtime capability checks remain authoritative.
 * Legacy helpers for unavailable providers return backend errors, not implementations.
 */

import {resolveBridgeOrigin} from './bridge-origin.mjs';
import {validCommandData} from './ipc-contract';

export interface IpcResponse<T = any> {
  id: string;
  success: boolean;
  data?: T;
  error?: string;
  errorCode?: 'HTTP_ERROR' | 'INVALID_RESPONSE' | 'NETWORK_ERROR' | 'TIMEOUT' | 'RESPONSE_TOO_LARGE';
}

export interface HandshakeResult {
  success: boolean;
  token?: string;
  error?: string;
  errorCode?: IpcResponse['errorCode'];
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

/** Managed child IDs are app-scoped handles, not credentials. */
export type ManagedWindowId = `child-${number}`;
export interface ManagedFocusRequest {
  windowId: ManagedWindowId;
  action: 'focus';
  requested: true;
  stateConfirmed: false;
}
export interface ManagedMinimizeRequest {
  windowId: ManagedWindowId;
  action: 'setMinimized';
  requested: true;
  requestedMinimized: boolean;
  stateConfirmed: false;
}
export interface ManagedMaximizeRequest {
  windowId: ManagedWindowId;
  action: 'setMaximized';
  requested: true;
  requestedMaximized: boolean;
  stateConfirmed: false;
}
export interface ManagedFullscreenRequest {
  windowId: ManagedWindowId;
  action: 'setFullscreen';
  requested: true;
  requestedFullscreen: boolean;
  stateConfirmed: false;
}
export interface ManagedTitleRequest {
  windowId: ManagedWindowId;
  action: 'setTitle';
  requested: true;
  title: string;
  stateConfirmed: false;
}
export interface WindowCapabilities {
  responseKind: 'capabilities';
  multiWindowSupported: boolean;
  managedWindowEnumerationAvailable: boolean;
  managedWindowCloseAvailable: boolean;
  externalBrowserLaunchAvailable: boolean;
  apiVersion: string;
  spawnMethods: string[];
  platformHint: string;
  capabilities: {modal: boolean; kiosk: boolean; alwaysOnTop: boolean; customSize: boolean; customUrl: boolean};
}
export interface ManagedWindowOptions {
  url?: string;
  title?: string;
  width?: number;
  height?: number;
  resizable?: boolean;
  decorations?: boolean;
}
export interface ManagedWindowCreated {
  windowId: ManagedWindowId;
  backend: 'NATIVE_EMBEDDED';
  documentReady: false;
}

export interface NativeDialogSubmission {
  requestId: string;
  status: 'pending';
  dialogType: 'info' | 'warning' | 'error';
  title: string;
  submitted: true;
  deliveryConfirmed: false;
}

export type FileSelectionResult =
  | {status: 'pending'}
  | {status: 'selected'; path: string}
  | {status: 'noSelection'};

export interface NotificationSubmission {
  requestId: string;
  status: 'pending';
  submitted: true;
  deliveryConfirmed: false;
}

export interface NotificationResult {
  status: 'pending' | 'providerAccepted';
  deliveryConfirmed: false;
}

export interface SecurityStatus {
  engine: string;
  windowsLifecycleMonitorActive?: boolean;
  windowsSessionAvailable?: boolean;
  embeddedWindowActive?: boolean;
  nonceVerified: boolean;
  sandboxEnforced: boolean;
  privacyVaultActive?: boolean;
  antiDosRateLimiterActive?: boolean;
  airGapPolicyActive?: boolean;
  bundlerEngineActive?: boolean;
  multiWindowApiActive?: boolean;
  activeCapabilities: {
    notifications?: boolean;
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

export interface UpdateJournal {
  journal_version: number;
  state: 'IDLE' | 'STAGED' | 'VERIFIED' | 'APPLIED' | 'ROLLED_BACK';
  active_version: string;
  target_version?: string;
  staged_file?: string;
  staged_sha256?: string;
  backup_file?: string;
  last_status: string;
}

export interface UpdaterStatus {
  autoUpdaterVersion: string;
  currentVersion: string;
  stagingDir: string;
  hasStagedUpdates: boolean;
  stagedFiles: string[];
  journal?: UpdateJournal;
  verificationAlgorithm: string;
  securityPolicy: string;
  airGapCompatible: boolean;
  autoApply: boolean;
  note: string;
}

export interface UpdateManifestInput {
  version: string;
  notes: string;
  download_url: string;
  sha256: string;
  released_at: string;
  min_version?: string;
  mandatory?: boolean;
}

export type UpdateCheckResult =
  | { status: 'update_available'; current: string; available: string; mandatory: boolean; notes: string }
  | { status: 'up_to_date'; version: string }
  | { status: 'parse_error'; reason: string };

class ResponseLimitError extends Error {}

/** Count actual decoded HTTP body bytes; Content-Length is not a trusted bound. */
async function boundedJson(response: Response, limit: number): Promise<any> {
  if (!response.body) throw new SyntaxError('Missing JSON response body');
  const reader = response.body.getReader();
  let bytes = new Uint8Array(Math.min(limit, 8192));
  let total = 0;
  try {
    while (true) {
      const {done, value} = await reader.read();
      if (done) break;
      if (value.byteLength > limit - total) throw new ResponseLimitError('IPC response exceeds byte limit');
      if (total + value.byteLength > bytes.byteLength) {
        const grown = new Uint8Array(Math.min(limit, Math.max(bytes.byteLength * 2, total + value.byteLength)));
        grown.set(bytes.subarray(0, total));
        bytes = grown;
      }
      bytes.set(value, total);
      total += value.byteLength;
    }
    return JSON.parse(new TextDecoder().decode(bytes.subarray(0, total)));
  } finally {
    await reader.cancel().catch(() => {});
    reader.releaseLock();
  }
}

export class AegisBridge {
  private readonly requestTimeoutMs: number;
  private readonly maxResponseBytes: number;
  constructor({requestTimeoutMs = 15000, maxResponseBytes = 16 * 1024 * 1024}: {requestTimeoutMs?: number; maxResponseBytes?: number} = {}) {
    if (!Number.isInteger(requestTimeoutMs) || requestTimeoutMs < 1 || requestTimeoutMs > 120000) {
      throw new Error('requestTimeoutMs must be an integer between 1 and 120000');
    }
    if (!Number.isInteger(maxResponseBytes) || maxResponseBytes < 1024 || maxResponseBytes > 64 * 1024 * 1024) {
      throw new Error('maxResponseBytes must be an integer between 1024 and 67108864');
    }
    this.requestTimeoutMs = requestTimeoutMs;
    this.maxResponseBytes = maxResponseBytes;
  }
  private bridgeOrigin(): string {
    return resolveBridgeOrigin({production: import.meta.env.PROD, appOrigin: import.meta.env.PROD ? window.location.origin : undefined, configuredOrigin: import.meta.env.VITE_AEGIS_BRIDGE_URL});
  }
  private sessionNonce: string | null = null;
  private isInitialized = false;
  private pendingInitialization: Promise<HandshakeResult> | null = null;

  public init(): Promise<HandshakeResult> {
    if (!this.pendingInitialization) {
      this.isInitialized = false;
      this.sessionNonce = null;
      this.pendingInitialization = this.initialize().finally(() => {this.pendingInitialization = null;});
    }
    return this.pendingInitialization;
  }

  private async initialize(): Promise<HandshakeResult> {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), this.requestTimeoutMs);
    let errorCode: IpcResponse['errorCode'] = 'NETWORK_ERROR';
    try {
      const res = await fetch(`${this.bridgeOrigin()}/handshake`, {
        method: 'GET',
        signal: controller.signal,
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (!res.ok) {
        errorCode = 'HTTP_ERROR';
        throw new Error(`Handshake failed with status ${res.status}`);
      }

      errorCode = 'INVALID_RESPONSE';
      const data = await boundedJson(res, 4096);
      if (!data || typeof data.token !== 'string' || !/^[a-f0-9]{64}$/.test(data.token)) {
        throw new Error('Invalid handshake response');
      }
      this.sessionNonce = data.token;
      this.isInitialized = true;
      return { success: true, token: data.token };
    } catch (err: any) {
      this.sessionNonce = null;
      this.isInitialized = false;
      return { success: false, error: controller.signal.aborted ? 'IPC handshake timed out' : (err.message || 'Cannot connect to Aegis Core backend'), errorCode: controller.signal.aborted ? 'TIMEOUT' : err instanceof ResponseLimitError ? 'RESPONSE_TOO_LARGE' : errorCode };
    } finally {clearTimeout(timer);controller.abort();}
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
          errorCode: initRes.errorCode,
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

    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), this.requestTimeoutMs);
    try {
      const res = await fetch(`${this.bridgeOrigin()}/ipc`, {
        method: 'POST',
        signal: controller.signal,
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(body),
      });

      if (res.status === 429) {
        const errData = await boundedJson(res, 4096).catch(error => {
          if (error instanceof SyntaxError) return {};
          throw error;
        });
        if (controller.signal.aborted) throw new Error('IPC response timed out');
        return {
          id: reqId,
          success: false,
          error: typeof errData?.error === 'string' ? errData.error : 'HTTP 429: Too Many Requests - Blocked by Aegis Anti-DoS Rate Limiter!',
          errorCode: 'HTTP_ERROR',
        };
      }
      if (!res.ok) return {id:reqId,success:false,error:`IPC HTTP error: ${res.status}`,errorCode:'HTTP_ERROR'};
      const json = await boundedJson(res, this.maxResponseBytes);
      if (!json || json.id !== reqId || typeof json.success !== 'boolean'
          || (json.success === false && typeof json.error !== 'string')) {
        return {id:reqId,success:false,error:'Invalid IPC response envelope',errorCode:'INVALID_RESPONSE'};
      }
      if (json.success && !validCommandData(command, json.data)) {
        return {id:reqId,success:false,error:`Invalid response data for ${command}`,errorCode:'INVALID_RESPONSE'};
      }
      return json;
    } catch (err: any) {
      return {
        id: reqId,
        success: false,
        error: controller.signal.aborted ? 'IPC request timed out; completion is unknown' : err instanceof ResponseLimitError ? 'IPC response exceeds byte limit; completion is unknown' : `IPC Network Error: ${err.message}`,
        errorCode: controller.signal.aborted ? 'TIMEOUT' : err instanceof ResponseLimitError ? 'RESPONSE_TOO_LARGE' : err instanceof SyntaxError ? 'INVALID_RESPONSE' : 'NETWORK_ERROR',
      };
    } finally {clearTimeout(timer);controller.abort();}
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

  public async writeSandboxFile(path: string, content: string): Promise<IpcResponse<{ path: string; bytesWritten: number; sha256?: string }>> {
    return this.invoke<{ path: string; bytesWritten: number; sha256?: string }>('fs_write_file', { path, content });
  }

  public async getFileHash(path: string): Promise<IpcResponse<{ path: string; sha256: string }>> {
    return this.invoke<{ path: string; sha256: string }>('fs_file_hash', { path });
  }

  public async verifyFileIntegrity(path: string, sha256: string): Promise<IpcResponse<{ path: string; verified: boolean }>> {
    return this.invoke<{ path: string; verified: boolean }>('fs_verify_integrity', { path, sha256 });
  }

  public async rotateHandshake(): Promise<IpcResponse<{ rotated: boolean; sessionToken: string; createdAt: number }>> {
    const res = await this.invoke<{ rotated: boolean; sessionToken: string; createdAt: number }>('handshake_rotate');
    if (res.success && res.data?.sessionToken) {
      this.sessionNonce = res.data.sessionToken;
    }
    return res;
  }

  // Auto-Updater APIs
  public async getUpdaterStatus(): Promise<IpcResponse<UpdaterStatus>> {
    return this.invoke<UpdaterStatus>('updater_status');
  }

  public async checkForUpdates(manifest: UpdateManifestInput): Promise<IpcResponse<UpdateCheckResult>> {
    return this.invoke<UpdateCheckResult>('updater_check', manifest);
  }

  public async verifyUpdatePayload(filePath: string, expectedSha256: string): Promise<IpcResponse<{ verified: boolean; filePath: string; sha256: string }>> {
    return this.invoke<{ verified: boolean; filePath: string; sha256: string }>('updater_verify', { filePath, expectedSha256 });
  }

  public async getUpdateJournal(): Promise<IpcResponse<UpdateJournal>> {
    return this.invoke<UpdateJournal>('updater_journal');
  }

  public async rollbackUpdate(): Promise<IpcResponse<{ rolledBack: boolean; journal: UpdateJournal }>> {
    return this.invoke<{ rolledBack: boolean; journal: UpdateJournal }>('updater_rollback');
  }

  public async stageUpdate(filePath: string, version: string, expectedSha256?: string): Promise<IpcResponse<{ staged_path: string; version: string; ready_to_apply: boolean }>> {
    return this.invoke<{ staged_path: string; version: string; ready_to_apply: boolean }>('updater_stage', { filePath, version, expectedSha256 });
  }

  public async stageUpdateBytes(filename: string, base64Data: string, version: string, expectedSha256?: string): Promise<IpcResponse<{ staged_path: string; version: string; ready_to_apply: boolean }>> {
    return this.invoke<{ staged_path: string; version: string; ready_to_apply: boolean }>('updater_stage_bytes', { filename, base64Data, version, expectedSha256 });
  }

  public async fetchLatestGitHubRelease(repo = 'charitha-only/aegis-desktop'): Promise<{
    tagName: string;
    version: string;
    name: string;
    body: string;
    publishedAt: string;
    htmlUrl: string;
    assets: Array<{ name: string; size: number; downloadUrl: string; contentType: string }>;
    isNewer: boolean;
  }> {
    const url = `https://api.github.com/repos/${repo}/releases/latest`;
    const res = await fetch(url, {
      headers: {
        Accept: 'application/vnd.github.v3+json',
      },
    });
    if (!res.ok) {
      throw new Error(`GitHub Releases API responded with status ${res.status}: ${res.statusText}`);
    }
    const data = await res.json();
    const tagName = data.tag_name || '';
    const cleanVer = tagName.replace(/^v/, '');
    const currentStatus = await this.getUpdaterStatus();
    const currentVer = currentStatus.data?.currentVersion || '1.0.0';

    const parse = (v: string) => v.replace(/^v/, '').split('.').map(n => parseInt(n, 10) || 0);
    const pa = parse(cleanVer);
    const pb = parse(currentVer);
    let isNewer = false;
    for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
      const na = pa[i] || 0;
      const nb = pb[i] || 0;
      if (na > nb) { isNewer = true; break; }
      if (na < nb) { isNewer = false; break; }
    }

    const assets = (data.assets || []).map((a: any) => ({
      name: a.name,
      size: a.size,
      downloadUrl: a.browser_download_url,
      contentType: a.content_type,
    }));

    return {
      tagName,
      version: cleanVer,
      name: data.name || tagName,
      body: data.body || '',
      publishedAt: data.published_at || '',
      htmlUrl: data.html_url || '',
      assets,
      isNewer,
    };
  }

  public async applyUpdate(): Promise<IpcResponse<{ applied: boolean; targetVersion: string; executable: string; backupFile: string; restartRequired: boolean; journal: UpdateJournal }>> {
    return this.invoke<{ applied: boolean; targetVersion: string; executable: string; backupFile: string; restartRequired: boolean; journal: UpdateJournal }>('updater_apply');
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

  public async createManagedWindow(options: ManagedWindowOptions = {}): Promise<IpcResponse<ManagedWindowCreated>> {
    return this.invoke('managed_window_create', options);
  }

  public async listManagedWindows(): Promise<IpcResponse<{windowIds: ManagedWindowId[]}>> {
    return this.invoke('managed_window_list');
  }

  public async closeManagedWindow(windowId: ManagedWindowId): Promise<IpcResponse<{windowId: ManagedWindowId; closed: true}>> {
    const result = await this.invoke<{windowId: ManagedWindowId; closed: true}>('managed_window_close', {windowId});
    if (result.success && result.data?.windowId !== windowId) {
      return {id: result.id, success: false, error: 'Managed close receipt does not match the requested window', errorCode: 'INVALID_RESPONSE'};
    }
    return result;
  }

  public async focusManagedWindow(windowId: ManagedWindowId): Promise<IpcResponse<ManagedFocusRequest>> {
    const result = await this.invoke<ManagedFocusRequest>('managed_window_focus', {windowId});
    if (result.success && result.data?.windowId !== windowId) return {id: result.id, success: false, error: 'Focus receipt does not match requested window', errorCode: 'INVALID_RESPONSE'};
    return result;
  }

  public async setManagedWindowMinimized(windowId: ManagedWindowId, minimized: boolean): Promise<IpcResponse<ManagedMinimizeRequest>> {
    const result = await this.invoke<ManagedMinimizeRequest>('managed_window_set_minimized', {windowId, minimized});
    if (result.success && (result.data?.windowId !== windowId || result.data?.requestedMinimized !== minimized)) return {id: result.id, success: false, error: 'Minimize receipt does not match requested window/state', errorCode: 'INVALID_RESPONSE'};
    return result;
  }

  public async setManagedWindowMaximized(windowId: ManagedWindowId, maximized: boolean): Promise<IpcResponse<ManagedMaximizeRequest>> {
    const result = await this.invoke<ManagedMaximizeRequest>('managed_window_set_maximized', {windowId, maximized});
    if (result.success && (result.data?.windowId !== windowId || result.data?.requestedMaximized !== maximized)) return {id: result.id, success: false, error: 'Maximize receipt does not match requested window/state', errorCode: 'INVALID_RESPONSE'};
    return result;
  }

  public async setManagedWindowFullscreen(windowId: ManagedWindowId, fullscreen: boolean): Promise<IpcResponse<ManagedFullscreenRequest>> {
    const result = await this.invoke<ManagedFullscreenRequest>('managed_window_set_fullscreen', {windowId, fullscreen});
    if (result.success && (result.data?.windowId !== windowId || result.data?.requestedFullscreen !== fullscreen)) return {id: result.id, success: false, error: 'Fullscreen receipt does not match requested window/state', errorCode: 'INVALID_RESPONSE'};
    return result;
  }

  public async setManagedWindowTitle(windowId: ManagedWindowId, title: string): Promise<IpcResponse<ManagedTitleRequest>> {
    const result = await this.invoke<ManagedTitleRequest>('managed_window_set_title', {windowId, title});
    if (result.success && (result.data?.windowId !== windowId || result.data?.title !== title)) return {id: result.id, success: false, error: 'Title receipt does not match requested window/title', errorCode: 'INVALID_RESPONSE'};
    return result;
  }

  public async showNativeDialog(title: string, message: string, type: 'info' | 'warning' | 'error' = 'info'): Promise<IpcResponse<NativeDialogSubmission>> {
    return this.invoke('window_dialog', { title, message, type });
  }

  public async openMessageDialog(title: string, message: string, type: 'info' | 'warning' | 'error' = 'info'): Promise<IpcResponse<NativeDialogSubmission>> {
    return this.invoke('message_dialog_open', {title, message, type});
  }

  public async pollMessageDialog(requestId: string): Promise<IpcResponse<{status: 'pending' | 'acknowledged'}>> {
    return this.invoke('message_dialog_poll', {requestId});
  }

  public async cancelMessageDialog(requestId: string): Promise<IpcResponse<{status: 'cancelled'}>> {
    return this.invoke('message_dialog_cancel', {requestId});
  }

  public async getWindowCapabilities(): Promise<IpcResponse<WindowCapabilities>> {
    return this.invoke('window_list');
  }

  public async writeClipboardText(text: string): Promise<IpcResponse<{written: true}>> {
    return this.invoke('clipboard_write_text', {text});
  }

  public async openFileDialog(title = 'Select a file'): Promise<IpcResponse<{requestId: string; status: 'pending'}>> {
    return this.invoke('file_dialog_open', {title});
  }

  public async pollFileDialog(requestId: string): Promise<IpcResponse<FileSelectionResult>> {
    return this.invoke('file_dialog_poll', {requestId});
  }

  public async cancelFileDialog(requestId: string): Promise<IpcResponse<{status: 'cancelled'}>> {
    return this.invoke('file_dialog_cancel', {requestId});
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

  public async pluginManifestList(): Promise<IpcResponse<{ manifests: any[]; count: number }>> {
    return this.invoke('plugin_manifest_list');
  }

  public async pluginManifestValidate(manifest: any): Promise<IpcResponse<{ valid: boolean; manifest?: any; error?: string }>> {
    return this.invoke('plugin_manifest_validate', { manifest });
  }

  public async pluginManifestRegister(manifest: any): Promise<IpcResponse<{ registered: boolean; pluginId: string }>> {
    return this.invoke('plugin_manifest_register', { manifest });
  }

  public async pluginSandboxedExecute(pluginId: string, action: string, payload: Record<string, any> = {}): Promise<IpcResponse<{ pluginId: string; action: string; result: any }>> {
    return this.invoke('plugin_sandboxed_execute', { pluginId, action, payload });
  }

  public async notifySend(title: string, body: string, urgency: 'low' | 'normal' | 'critical' = 'normal', timeoutMs?: number): Promise<IpcResponse<NotificationSubmission>> {
    return this.invoke('notify_send', { title, body, urgency, ...(timeoutMs === undefined ? {} : {timeout_ms: timeoutMs}) });
  }

  public async notifyPoll(requestId: string): Promise<IpcResponse<NotificationResult>> {
    return this.invoke('notify_poll', {requestId});
  }

  public async notifyCancel(requestId: string): Promise<IpcResponse<{status: 'cancelled'; deliveryConfirmed: false}>> {
    return this.invoke('notify_cancel', {requestId});
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

  public async crashRecord(kind: string, _message: string): Promise<IpcResponse<{ id: string; persisted: boolean }>> {
    // Free-form messages/traces are intentionally not sent or persisted.
    return this.invoke('crash_record', { errorKind: kind });
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

  // --- Banking-Grade Financial Suite APIs ---
  public async setScreenProtection(protectedState: boolean): Promise<IpcResponse<{
    protected: boolean;
    platform: string;
    affinity: string;
    applied_hwnds: number;
    anti_capture_active: boolean;
    description: string;
  }>> {
    return this.invoke('banking_screen_protection_set', { protected: protectedState });
  }

  public async getScreenProtectionStatus(): Promise<IpcResponse<{
    protected: boolean;
    platform: string;
    affinity: string;
    applied_hwnds: number;
    anti_capture_active: boolean;
    description: string;
  }>> {
    return this.invoke('banking_screen_protection_status');
  }

  public async getBankingBiometricStatus(): Promise<IpcResponse<{
    platform: string;
    provider: string;
    biometric_enrolled: boolean;
    hardware_backed: boolean;
    multi_factor_capable: boolean;
    supported_methods: string[];
  }>> {
    return this.invoke('banking_biometric_hardware_status');
  }

  public async requestBiometricChallenge(userId: string, operation: string, amount: number, currency = 'USD'): Promise<IpcResponse<{
    challenge_id: string;
    user_id: string;
    operation: string;
    amount: number;
    currency: string;
    nonce: string;
    created_at: number;
    expires_at: number;
  }>> {
    return this.invoke('banking_biometric_challenge_request', { userId, operation, amount, currency });
  }

  public async verifyBiometricAssertion(challengeId: string, userId: string, biometricPayload: string): Promise<IpcResponse<{
    token: string;
    user_id: string;
    operation: string;
    amount: number;
    currency: string;
    biometric_method: string;
    created_at: number;
    expires_at: number;
    consumed: boolean;
  }>> {
    return this.invoke('banking_biometric_verify', { challengeId, userId, biometricPayload });
  }

  public async proposeTransaction(
    makerId: string,
    fromAccount: string,
    toAccount: string,
    amount: number,
    currency: string,
    reference: string
  ): Promise<IpcResponse<{
    tx_id: string;
    maker_id: string;
    from_account: string;
    to_account: string;
    amount: number;
    currency: string;
    reference: string;
    created_at: number;
    canonical_hash: string;
    maker_signature: string;
    status: string;
  }>> {
    return this.invoke('banking_transaction_propose', {
      makerId,
      fromAccount,
      toAccount,
      amount,
      currency,
      reference,
    });
  }

  public async approveTransaction(txId: string, checkerId: string, stepUpToken?: string): Promise<IpcResponse<any>> {
    return this.invoke('banking_transaction_approve', { txId, checkerId, stepUpToken });
  }

  public async rejectTransaction(txId: string, checkerId: string, reason: string): Promise<IpcResponse<any>> {
    return this.invoke('banking_transaction_reject', { txId, checkerId, reason });
  }

  public async listTransactions(): Promise<IpcResponse<{ transactions: any[]; count: number }>> {
    return this.invoke('banking_transaction_list');
  }

  public async getLedger(): Promise<IpcResponse<{ ledger: any[]; blockCount: number }>> {
    return this.invoke('banking_ledger_get');
  }

  public async verifyLedger(): Promise<IpcResponse<{
    verified: boolean;
    total_blocks: number;
    genesis_hash: string;
    tip_hash: string;
    chained_integrity_confirmed: boolean;
    four_eyes_audit_compliant: boolean;
  }>> {
    return this.invoke('banking_ledger_verify');
  }

  // ── RASP Anti-Tampering & Memory Hook Protection ───────────────────────────
  public async evaluateRasp(): Promise<IpcResponse<{
    debugger_detected: boolean;
    memory_hooks_detected: boolean;
    suspicious_dlls_detected: boolean;
    hardware_breakpoints_detected: boolean;
    fail_closed_containment_active: boolean;
    system_integrity_tier: 'SECURE' | 'ELEVATED_RISK' | 'COMPROMISED_CONTAINED';
    evaluated_at: number;
    violations: string[];
  }>> {
    return this.invoke('rasp_evaluate');
  }

  public async resetRaspContainment(): Promise<IpcResponse<{ status: string }>> {
    return this.invoke('rasp_containment_reset');
  }

  // ── mTLS & Public Key Pinning Sentinel ─────────────────────────────────────
  public async evaluateNetworkPinning(
    host: string,
    certBytesHex: string,
    clientCertPresent = true
  ): Promise<IpcResponse<{
    host: string;
    pin_verified: boolean;
    mitm_interception_detected: boolean;
    mtls_client_authenticated: boolean;
    observed_spki_pin: string;
    expected_spki_pin?: string;
    status: string;
  }>> {
    return this.invoke('network_pinning_evaluate', { host, certBytesHex, clientCertPresent });
  }

  public async enrollPinnedEndpoint(endpoint: {
    host: string;
    port: number;
    expected_spki_pins: string[];
    require_mtls: boolean;
    allow_subdomains: boolean;
  }): Promise<IpcResponse<{ status: string; host: string }>> {
    return this.invoke('network_pinning_enroll', endpoint);
  }

  // ── Physical Hardware Security Key (FIDO2 / YubiKey) ────────────────────────
  public async enumerateFido2Tokens(): Promise<IpcResponse<{ devices: any[]; count: number }>> {
    return this.invoke('fido2_token_enumerate');
  }

  public async registerFido2Credential(userHandle: string, relyingParty = 'aegis.bank.internal'): Promise<IpcResponse<{
    credential_id: string;
    user_handle: string;
    relying_party: string;
    public_key_hex: string;
    sign_count: number;
    registered_at: number;
  }>> {
    return this.invoke('fido2_token_register', { userHandle, relyingParty });
  }

  public async signWithFido2Token(
    credentialId: string,
    clientDataHashHex: string,
    touchConfirmed: boolean
  ): Promise<IpcResponse<{
    credential_id: string;
    signature_hex: string;
    authenticator_data_hex: string;
    user_present: boolean;
    user_verified: boolean;
    counter: number;
  }>> {
    return this.invoke('fido2_token_sign', { credentialId, clientDataHashHex, touchConfirmed });
  }

  public async verifyFido2Signature(
    credentialId: string,
    clientDataHashHex: string,
    assertion: any
  ): Promise<IpcResponse<{ verified: boolean }>> {
    return this.invoke('fido2_token_verify', { credentialId, clientDataHashHex, assertion });
  }

  // ── Decentralized P2P Zero-Knowledge Sync (Noise Protocol) ──────────────────
  public async initiateP2pSync(): Promise<IpcResponse<{
    session_id: string;
    initiator_public_hex: string;
    ephemeral_nonce_hex: string;
    timestamp: number;
  }>> {
    return this.invoke('p2p_sync_init');
  }

  public async completeP2pSync(
    sessionId: string,
    initiatorPubHex: string,
    responderPubHex: string
  ): Promise<IpcResponse<{
    session_id: string;
    responder_public_hex: string;
    mutual_auth_tag_hex: string;
    session_established: boolean;
  }>> {
    return this.invoke('p2p_sync_complete', { sessionId, initiatorPubHex, responderPubHex });
  }

  public async encryptP2pBundle(
    sessionId: string,
    sequenceNumber: number,
    payloadHex: string
  ): Promise<IpcResponse<{
    session_id: string;
    sequence_number: number;
    ciphertext_hex: string;
    auth_tag_hex: string;
    nonce_hex: string;
  }>> {
    return this.invoke('p2p_sync_encrypt', { sessionId, sequenceNumber, payloadHex });
  }

  public async decryptP2pBundle(bundle: {
    session_id: string;
    sequence_number: number;
    ciphertext_hex: string;
    auth_tag_hex: string;
    nonce_hex: string;
  }): Promise<IpcResponse<{
    plaintextHex: string;
    plaintextUtf8: string;
    verified: boolean;
  }>> {
    return this.invoke('p2p_sync_decrypt', bundle);
  }

  // ── Pillar 1: Encrypted Embedded Database Engine ───────────────────────────
  public async queryEncryptedDb(table: string, filterCol?: string, filterVal?: string): Promise<IpcResponse<{
    table: string;
    rows: any[];
    count: number;
  }>> {
    return this.invoke('db_query', { table, filterCol, filterVal });
  }

  public async insertEncryptedDb(table: string, row: any): Promise<IpcResponse<{
    status: string;
    table: string;
    id: string;
    record: any;
  }>> {
    return this.invoke('db_insert', { table, row });
  }

  public async updateEncryptedDb(table: string, id: string, row: any): Promise<IpcResponse<{
    status: string;
    table: string;
    id: string;
    record: any;
  }>> {
    return this.invoke('db_update', { table, id, row });
  }

  public async deleteEncryptedDb(table: string, id: string): Promise<IpcResponse<{
    deleted: boolean;
    table: string;
    id: string;
  }>> {
    return this.invoke('db_delete', { table, id });
  }

  public async getEncryptedDbStats(): Promise<IpcResponse<{
    version: number;
    total_tables: number;
    total_records: number;
    estimated_size_bytes: number;
    key_fingerprint: string;
    tables: string[];
    in_transaction: boolean;
  }>> {
    return this.invoke('db_stats');
  }

  public async transactionEncryptedDb(action: 'begin' | 'commit' | 'rollback'): Promise<IpcResponse<{
    status: string;
  }>> {
    return this.invoke('db_transaction', { action });
  }

  // ── Pillar 2: Hardware TPM 2.0 PCR-Sealed Cryptographic Enclave ───────────
  public async getPcrEnclaveStatus(): Promise<IpcResponse<{
    tpm_version: string;
    hardware_detected: boolean;
    manufacturer: string;
    secure_boot_enforced: boolean;
    pcr_registers: Array<{ index: number; description: string; digest_hex: string }>;
    total_sealed_packages: number;
    defense_tier: string;
  }>> {
    return this.invoke('pcr_enclave_status');
  }

  public async sealPcrSecret(packageId: string, secret: string, pcrs?: number[]): Promise<IpcResponse<{
    package_id: string;
    created_at: number;
    required_pcrs: number[];
    pcr_composite_hash: string;
    nonce_hex: string;
    ciphertext_hex: string;
  }>> {
    return this.invoke('pcr_enclave_seal', { packageId, secret, pcrs });
  }

  public async unsealPcrSecret(packageId: string): Promise<IpcResponse<{
    packageId: string;
    secretUtf8: string;
    secretHex: string;
    verified: boolean;
  }>> {
    return this.invoke('pcr_enclave_unseal', { packageId });
  }

  public async simulatePcrTamper(pcrIndex: number = 7, mockCode?: string): Promise<IpcResponse<{
    tampered: boolean;
    pcrIndex: number;
    message: string;
  }>> {
    return this.invoke('pcr_enclave_tamper_sim', { pcrIndex, mockCode });
  }

  // ── Pillar 3: Enterprise Granular RBAC Engine ──────────────────────────────
  public async getRbacIdentity(): Promise<IpcResponse<{
    user_id: string;
    display_name: string;
    role: string;
    department: string;
    permissions: {
      can_initiate_transaction: boolean;
      can_approve_transaction: boolean;
      can_view_ledger: boolean;
      can_export_compliance: boolean;
      can_modify_security_policies: boolean;
      can_rotate_keys: boolean;
      can_access_database: boolean;
      max_single_transaction_cents: number;
    };
    session_id: string;
  }>> {
    return this.invoke('rbac_identity');
  }

  public async switchRbacRole(role: string, actorName?: string): Promise<IpcResponse<any>> {
    return this.invoke('rbac_switch_role', { role, actorName });
  }

  public async checkRbacPermission(action: string, resource: string = '*'): Promise<IpcResponse<{
    allowed: boolean;
    action: string;
    resource: string;
    reason: string;
  }>> {
    return this.invoke('rbac_check', { action, resource });
  }

  public async getRbacMatrix(): Promise<IpcResponse<{
    roles: Array<{
      role: string;
      display_name: string;
      description: string;
      permissions: any;
    }>;
    count: number;
  }>> {
    return this.invoke('rbac_matrix');
  }

  public async getRbacAuditLog(): Promise<IpcResponse<{
    auditLog: Array<{
      timestamp: number;
      actor_id: string;
      role: string;
      requested_action: string;
      target_resource: string;
      decision: string;
      reason: string;
    }>;
    count: number;
  }>> {
    return this.invoke('rbac_audit');
  }

  // ── Pillar 4: Outbound Network Firewall & Flow Monitor ─────────────────────
  public async getRecentTraffic(limit: number = 50): Promise<IpcResponse<{
    flows: Array<{
      id: string;
      timestamp: number;
      target_host: string;
      port: number;
      protocol: string;
      bytes_transferred: number;
      latency_ms: number;
      threat_score: number;
      decision: string;
      matched_rule: string;
      geo_region: string;
    }>;
    count: number;
  }>> {
    return this.invoke('traffic_recent', { limit });
  }

  public async getTrafficStats(): Promise<IpcResponse<{
    total_flows_inspected: number;
    total_allowed: number;
    total_blocked: number;
    total_bytes_transferred: number;
    active_threats_neutralized: number;
    killswitch_active: boolean;
    average_latency_ms: number;
  }>> {
    return this.invoke('traffic_stats');
  }

  public async toggleTrafficKillswitch(enabled: boolean): Promise<IpcResponse<{
    killswitchActive: boolean;
  }>> {
    return this.invoke('traffic_killswitch', { enabled });
  }

  public async simulateTrafficFlow(host: string, threat: number = 0): Promise<IpcResponse<any>> {
    return this.invoke('traffic_simulate', { host, threat });
  }

  public async clearTrafficHistory(): Promise<IpcResponse<{ status: string }>> {
    return this.invoke('traffic_clear');
  }

  // ── Pillar 5: Security Audit & Compliance Certificate Generator ─────────────
  public async generateComplianceCertificate(): Promise<IpcResponse<{
    payload: {
      certificate_id: string;
      timestamp: number;
      framework_version: string;
      standard: string;
      tier: string;
      score: string;
      total_controls: number;
      passed_controls: number;
      rasp_status: string;
      tpm_pcr_status: string;
      fido2_status: string;
      dwm_protection_status: string;
      maker_checker_sod_status: string;
      mtls_status: string;
      categories: Array<{
        category_id: string;
        name: string;
        level: string;
        controls_evaluated: number;
        controls_passed: number;
        status: string;
      }>;
    };
    payload_sha256: string;
    signature_hex: string;
    public_key_hex: string;
    printable_html: string;
  }>> {
    return this.invoke('compliance_generate');
  }

  public async verifyComplianceCertificate(
    payloadSha256: string,
    signatureHex: string,
    publicKeyHex: string
  ): Promise<IpcResponse<{ verified: boolean }>> {
    return this.invoke('compliance_verify', { payloadSha256, signatureHex, publicKeyHex });
  }

  // ── Differential Binary Delta Auto-Updater (Quantum-Safe Patching) ──────────
  public async verifyDeltaPatch(patch: string, baseData?: string): Promise<IpcResponse<{
    valid: boolean;
    magic: string;
    baseVersion: string;
    targetVersion: string;
    baseSha256Expected: string;
    baseSha256Computed: string;
    targetSha256: string;
    originalSize: number;
    patchedSize: number;
    operationsCount: number;
    patchSizeBytes: number;
    bytesSaved: number;
    compressionRatio: string;
  }>> {
    return this.invoke('updater_delta_verify', { patch, baseData });
  }

  public async createDeltaPatch(
    targetData: string,
    targetVersion: string = '1.4.0',
    baseData?: string
  ): Promise<IpcResponse<any>> {
    return this.invoke('updater_delta_create', { targetData, targetVersion, baseData });
  }

  public async applyDeltaPatch(patch: string, baseData?: string): Promise<IpcResponse<{
    success: boolean;
    baseVersion: string;
    targetVersion: string;
    targetSha256: string;
    originalSize: number;
    patchSize: number;
    reconstructedSize: number;
    bytesSaved: number;
    compressionRatio: string;
  }>> {
    return this.invoke('updater_delta_apply', { patch, baseData });
  }

  // ── Post-Quantum Hybrid Cryptography (NIST FIPS 203 ML-KEM-768 + X25519) ────
  public async generatePqKeypair(): Promise<IpcResponse<{
    algorithm: string;
    nist_standard: string;
    hybrid_public_key_hex: string;
    hybrid_secret_key_hex: string;
    classical_algorithm: string;
    pq_algorithm: string;
    created_at: number;
  }>> {
    return this.invoke('pq_kem_keypair');
  }

  public async encapsulatePqKem(hybridPublicKeyHex: string): Promise<IpcResponse<{
    algorithm: string;
    hybrid_ciphertext_hex: string;
    shared_secret_hex: string;
    classical_component: string;
    pq_component: string;
    kdf: string;
    security_bits: number;
  }>> {
    return this.invoke('pq_kem_encapsulate', { hybridPublicKeyHex });
  }

  public async decapsulatePqKem(
    hybridSecretKeyHex: string,
    hybridCiphertextHex: string
  ): Promise<IpcResponse<{
    algorithm: string;
    shared_secret_hex: string;
    verified: boolean;
    quantum_safe: boolean;
  }>> {
    return this.invoke('pq_kem_decapsulate', { hybridSecretKeyHex, hybridCiphertextHex });
  }

  public async getPqKemMetadata(): Promise<IpcResponse<any>> {
    return this.invoke('pq_kem_metadata');
  }

  // ── Smart Card & YubiKey PIV Slot Management (NIST SP 800-73-4) ─────────────
  public async getPivState(): Promise<IpcResponse<{
    card_present: boolean;
    reader_name: string;
    card_model: string;
    serial_number: string;
    firmware_version: string;
    pin_retries_remaining: number;
    is_locked_out: boolean;
    session_unlocked: boolean;
    slots: Array<{
      slot_id: string;
      name: string;
      purpose: string;
      key_algorithm: string;
      certificate_installed: boolean;
      cert_subject?: string;
      cert_serial?: string;
      cert_fingerprint_sha256?: string;
      pin_policy: string;
      touch_policy: string;
      is_unlocked: boolean;
    }>;
  }>> {
    return this.invoke('piv_state');
  }

  public async verifyPivPin(pin: string): Promise<IpcResponse<{
    authenticated: boolean;
    retriesRemaining: number;
    sessionUnlocked: boolean;
    message: string;
  }>> {
    return this.invoke('piv_pin_verify', { pin });
  }

  public async logoutPiv(): Promise<IpcResponse<{ status: string; sessionUnlocked: boolean }>> {
    return this.invoke('piv_logout');
  }

  public async signWithPiv(
    slotId: string,
    payload: string,
    pin?: string
  ): Promise<IpcResponse<{
    slot_id: string;
    mechanism: string;
    payload_sha256: string;
    signature_hex: string;
    token_serial: string;
    timestamp: number;
    non_repudiation_verified: boolean;
  }>> {
    return this.invoke('piv_sign', { slotId, payload, pin });
  }

  public async exportPivCertificate(slotId: string): Promise<IpcResponse<any>> {
    return this.invoke('piv_cert_export', { slotId });
  }

  // ── Native OS System Tray Daemon & Quick-Killswitch Controls ────────────────
  public async getTrayStatus(): Promise<IpcResponse<{
    apiVersion: string;
    daemonActive: boolean;
    securityBadge: string;
    quickKillswitchActive: boolean;
    pendingMakerCheckerApprovals: number;
    hardwareTokenConnected: boolean;
    tpmSealed: boolean;
    menuItems: Array<{
      id: string;
      label: string;
      item_type: string;
      enabled: boolean;
      checked?: boolean;
    }>;
    notificationBackend: string;
    lastEventTimestamp: number;
  }>> {
    return this.invoke('tray_status_get');
  }

  public async toggleTrayQuickKillswitch(enabled: boolean): Promise<IpcResponse<{
    killswitchActive: boolean;
    badge: string;
    status: string;
  }>> {
    return this.invoke('tray_quick_killswitch', { enabled });
  }

  public async triggerTrayMenuAction(actionId: string): Promise<IpcResponse<any>> {
    return this.invoke('tray_menu_action', { actionId });
  }
}

export const aegis = new AegisBridge();

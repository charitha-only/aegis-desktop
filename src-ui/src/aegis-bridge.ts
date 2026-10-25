/**
 * Aegis Desktop IPC Bridge Client v0.13
 * Secure typed communication with Rust Backend Core
 */

export interface IpcResponse<T = any> {
  id: string;
  success: boolean;
  data?: T;
  error?: string;
}

export interface SecurityStatus {
  engine: string;
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

class AegisBridge {
  private bridgeUrl = 'http://127.0.0.1:48291';
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

/**
 * Aegis Desktop IPC Bridge Client
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
  activeCapabilities: {
    fsRead: boolean;
    fsWrite: boolean;
    shellExecution: boolean;
    systemInfo: boolean;
    crypto: boolean;
  };
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

  public async hashInput(input: string): Promise<IpcResponse<{ algorithm: string; hash: string }>> {
    return this.invoke<{ algorithm: string; hash: string }>('crypto_hash', { input });
  }

  // Security Defense Simulators (Demonstrates blocking unauthorized actions)
  public async simulatePathTraversalAttack(): Promise<IpcResponse> {
    return this.invoke('fs_read_file', { path: '../../../../Windows/win.ini' });
  }

  public async simulateShellExecutionAttack(): Promise<IpcResponse> {
    return this.invoke('execute_shell', { command: 'cmd.exe /c whoami' });
  }

  public async simulateSpoofedNonceAttack(): Promise<IpcResponse> {
    return this.invoke('sys_info', {}, 'FORGED_INVALID_TOKEN_99999');
  }
}

export const aegis = new AegisBridge();

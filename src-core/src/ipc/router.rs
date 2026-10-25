use super::protocol::{IpcRequest, IpcResponse};
use crate::security::{HandshakeSession, SecurityEngine};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::sync::Arc;

pub struct IpcRouter {
    security: Arc<SecurityEngine>,
    session: Arc<HandshakeSession>,
}

impl IpcRouter {
    pub fn new(security: Arc<SecurityEngine>, session: Arc<HandshakeSession>) -> Self {
        Self { security, session }
    }

    pub fn handle_request(&self, req: IpcRequest) -> IpcResponse {
        // 1. Cryptographic Nonce Verification
        if !self.session.validate_nonce(&req.nonce) {
            return IpcResponse::err(&req.id, "Security Breach: Invalid cryptographic nonce. Request rejected!");
        }

        // 2. Command Dispatching with Capability Verification
        match req.command.as_str() {
            "get_security_status" => {
                let status = json!({
                    "engine": "Aegis Core 1.0 (Rust)",
                    "nonceVerified": true,
                    "sandboxEnforced": true,
                    "activeCapabilities": {
                        "fsRead": true,
                        "fsWrite": true,
                        "shellExecution": false,
                        "systemInfo": true,
                        "crypto": true
                    }
                });
                IpcResponse::ok(&req.id, status)
            }

            "sys_info" => {
                match self.security.check_system_info() {
                    Ok(_) => {
                        let info = json!({
                            "os": std::env::consts::OS,
                            "arch": std::env::consts::ARCH,
                            "family": std::env::consts::FAMILY,
                            "rustVersion": "1.99.1 (GNU)",
                            "processId": std::process::id()
                        });
                        IpcResponse::ok(&req.id, info)
                    }
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "fs_read_file" => {
                let path = match req.payload.get("path").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'path' parameter"),
                };

                match self.security.check_fs_read(path) {
                    Ok(safe_path) => match fs::read_to_string(&safe_path) {
                        Ok(content) => IpcResponse::ok(&req.id, json!({ "path": path, "content": content })),
                        Err(err) => IpcResponse::err(&req.id, &format!("Failed to read file: {}", err)),
                    },
                    Err(err) => IpcResponse::err(&req.id, &err),
                }
            }

            "fs_write_file" => {
                let path = match req.payload.get("path").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'path' parameter"),
                };
                let content = match req.payload.get("content").and_then(|c| c.as_str()) {
                    Some(c) => c,
                    None => return IpcResponse::err(&req.id, "Missing 'content' parameter"),
                };

                match self.security.check_fs_write(path) {
                    Ok(safe_path) => match fs::write(&safe_path, content) {
                        Ok(_) => IpcResponse::ok(&req.id, json!({ "path": path, "bytesWritten": content.len() })),
                        Err(err) => IpcResponse::err(&req.id, &format!("Failed to write file: {}", err)),
                    },
                    Err(err) => IpcResponse::err(&req.id, &err),
                }
            }

            "fs_list_sandbox" => {
                match self.security.check_fs_read(".") {
                    Ok(sandbox_dir) => {
                        let mut entries = Vec::new();
                        if let Ok(read_dir) = fs::read_dir(sandbox_dir) {
                            for entry in read_dir.flatten() {
                                if let Ok(name) = entry.file_name().into_string() {
                                    let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                                    entries.push(json!({
                                        "name": name,
                                        "isDir": is_dir,
                                        "sizeBytes": size
                                    }));
                                }
                            }
                        }
                        IpcResponse::ok(&req.id, json!({ "entries": entries }))
                    }
                    Err(err) => IpcResponse::err(&req.id, &err),
                }
            }

            "crypto_hash" => {
                match self.security.check_crypto() {
                    Ok(_) => {
                        let text = req.payload.get("input").and_then(|t| t.as_str()).unwrap_or("");
                        let mut hasher = Sha256::new();
                        hasher.update(text.as_bytes());
                        let hash = format!("{:x}", hasher.finalize());
                        IpcResponse::ok(&req.id, json!({ "algorithm": "SHA-256", "hash": hash }))
                    }
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "execute_shell" => {
                // Strictly blocked by capability policy
                match self.security.check_shell() {
                    Ok(_) => IpcResponse::err(&req.id, "Shell execution granted"),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            _ => IpcResponse::err(&req.id, &format!("Unknown IPC command: '{}'", req.command)),
        }
    }
}

use super::protocol::{IpcRequest, IpcResponse};
use crate::security::{HandshakeSession, PrivacyShield, SecurityEngine, VaultEngine};
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
                    "engine": "Aegis Core v0.11 (Rust)",
                    "nonceVerified": true,
                    "sandboxEnforced": true,
                    "privacyVaultActive": true,
                    "antiDosRateLimiterActive": true,
                    "airGapPolicyActive": true,
                    "activeCapabilities": {
                        "fsRead": true,
                        "fsWrite": true,
                        "shellExecution": false,
                        "systemInfo": true,
                        "crypto": true,
                        "encryptedVault": true
                    }
                });
                IpcResponse::ok(&req.id, status)
            }

            "privacy_status" => {
                IpcResponse::ok(&req.id, PrivacyShield::get_telemetry_audit())
            }

            "vault_encrypt_store" => {
                match self.security.check_crypto() {
                    Ok(_) => {
                        let filename = match req.payload.get("filename").and_then(|f| f.as_str()) {
                            Some(f) => f,
                            None => return IpcResponse::err(&req.id, "Missing 'filename' parameter"),
                        };
                        let content = match req.payload.get("content").and_then(|c| c.as_str()) {
                            Some(c) => c,
                            None => return IpcResponse::err(&req.id, "Missing 'content' parameter"),
                        };
                        let passphrase = match req.payload.get("passphrase").and_then(|p| p.as_str()) {
                            Some(p) => p,
                            None => return IpcResponse::err(&req.id, "Missing 'passphrase' parameter"),
                        };

                        if passphrase.len() < 4 {
                            return IpcResponse::err(&req.id, "Passphrase must be at least 4 characters long");
                        }

                        // Encrypt data with PBKDF2/SHA-256 stream cipher + HMAC-SHA256
                        let encrypted_envelope = VaultEngine::encrypt(content, passphrase);

                        // Ensure filename ends with .vault
                        let safe_filename = if filename.ends_with(".vault") {
                            filename.to_string()
                        } else {
                            format!("{}.vault", filename)
                        };

                        // Store safely inside isolated sandbox
                        match self.security.check_fs_write(&safe_filename) {
                            Ok(safe_path) => match fs::write(&safe_path, &encrypted_envelope) {
                                Ok(_) => IpcResponse::ok(
                                    &req.id,
                                    json!({
                                        "filename": safe_filename,
                                        "status": "Encrypted & Stored Successfully",
                                        "envelopeLength": encrypted_envelope.len()
                                    }),
                                ),
                                Err(err) => IpcResponse::err(&req.id, &format!("Failed to write encrypted vault: {}", err)),
                            },
                            Err(err) => IpcResponse::err(&req.id, &err),
                        }
                    }
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "vault_decrypt_retrieve" => {
                match self.security.check_crypto() {
                    Ok(_) => {
                        let filename = match req.payload.get("filename").and_then(|f| f.as_str()) {
                            Some(f) => f,
                            None => return IpcResponse::err(&req.id, "Missing 'filename' parameter"),
                        };
                        let passphrase = match req.payload.get("passphrase").and_then(|p| p.as_str()) {
                            Some(p) => p,
                            None => return IpcResponse::err(&req.id, "Missing 'passphrase' parameter"),
                        };

                        let safe_filename = if filename.ends_with(".vault") {
                            filename.to_string()
                        } else {
                            format!("{}.vault", filename)
                        };

                        // Read safely from sandbox
                        match self.security.check_fs_read(&safe_filename) {
                            Ok(safe_path) => match fs::read_to_string(&safe_path) {
                                Ok(envelope) => match VaultEngine::decrypt(&envelope, passphrase) {
                                    Ok(plaintext) => IpcResponse::ok(
                                        &req.id,
                                        json!({
                                            "filename": safe_filename,
                                            "decryptedContent": plaintext,
                                            "integrityVerified": true
                                        }),
                                    ),
                                    Err(err) => IpcResponse::err(&req.id, &err),
                                },
                                Err(err) => IpcResponse::err(&req.id, &format!("Failed to read vault file: {}", err)),
                            },
                            Err(err) => IpcResponse::err(&req.id, &err),
                        }
                    }
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
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

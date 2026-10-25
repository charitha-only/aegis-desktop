use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedSigner {
    pub key_id: String,
    pub vendor: String,
    pub public_fingerprint: String,
    pub trusted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureVerificationResult {
    pub plugin_id: String,
    pub valid: bool,
    pub trusted_vendor: bool,
    pub signer_vendor: Option<String>,
    pub computed_digest: String,
    pub message: String,
}

/// Only administrator-provisioned Ed25519 public keys are trusted. No embedded test roots.
pub struct SignatureVerifier {
    trusted_signers: Mutex<Vec<VerifiedSigner>>,
}
impl SignatureVerifier {
    pub fn new() -> Self {
        let signers = std::fs::read_to_string("trusted-signers.json")
            .ok()
            .and_then(|s| serde_json::from_str::<Vec<VerifiedSigner>>(&s).ok())
            .unwrap_or_default();
        Self {
            trusted_signers: Mutex::new(signers),
        }
    }
    pub fn list_trusted_signers(&self) -> Vec<VerifiedSigner> {
        self.trusted_signers.lock().unwrap().clone()
    }
    pub fn verify_plugin_manifest(
        &self,
        id: &str,
        manifest: &str,
        signature: &str,
    ) -> SignatureVerificationResult {
        let digest = hex::encode(Sha256::digest(manifest.as_bytes()));
        let signers = self.trusted_signers.lock().unwrap();
        let verified = signature.split_once(':').and_then(|(key_id, sig)| {
            let signer = signers.iter().find(|s| s.key_id == key_id && s.trusted)?;
            let public: [u8; 32] = hex::decode(&signer.public_fingerprint)
                .ok()?
                .try_into()
                .ok()?;
            let key = ed25519_dalek::VerifyingKey::from_bytes(&public).ok()?;
            let sig = ed25519_dalek::Signature::from_slice(&hex::decode(sig).ok()?).ok()?;
            let message = format!("AEGIS_PLUGIN_V1\0{}\0{}", id, manifest);
            key.verify_strict(message.as_bytes(), &sig).ok()?;
            Some(signer.vendor.clone())
        });
        SignatureVerificationResult{plugin_id:id.into(),valid:verified.is_some(),trusted_vendor:verified.is_some(),signer_vendor:verified,computed_digest:digest,message:"Ed25519 verification requires a provisioned trusted key and a signature over AEGIS_PLUGIN_V1\0<plugin_id>\0<manifest>".into()}
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Signer;
    #[test]
    fn real_signature_and_tamper() {
        let key = ed25519_dalek::SigningKey::from_bytes(&[42; 32]);
        let v = SignatureVerifier {
            trusted_signers: Mutex::new(vec![VerifiedSigner {
                key_id: "test".into(),
                vendor: "test".into(),
                public_fingerprint: hex::encode(key.verifying_key().as_bytes()),
                trusted: true,
            }]),
        };
        let message = "AEGIS_PLUGIN_V1\0plugin\0manifest";
        let sig = format!(
            "test:{}",
            hex::encode(key.sign(message.as_bytes()).to_bytes())
        );
        assert!(v.verify_plugin_manifest("plugin", "manifest", &sig).valid);
        assert!(!v.verify_plugin_manifest("plugin", "tampered", &sig).valid);
        assert!(!v.verify_plugin_manifest("other", "manifest", &sig).valid);
        assert!(
            !v.verify_plugin_manifest("plugin", "manifest", "signer_aegis_official-anything")
                .valid
        );
    }
}

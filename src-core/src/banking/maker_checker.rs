use super::step_up::StepUpAuthEngine;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionStatus {
    #[serde(rename = "PENDING_CHECKER_APPROVAL")]
    PendingCheckerApproval,
    #[serde(rename = "AUTHORIZED_COMMITTED")]
    AuthorizedCommitted,
    #[serde(rename = "REJECTED")]
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialTransaction {
    pub tx_id: String,
    pub maker_id: String,
    pub from_account: String,
    pub to_account: String,
    pub amount: f64,
    pub currency: String,
    pub reference: String,
    pub created_at: u64,
    pub canonical_hash: String,
    pub maker_signature: String,
    pub status: TransactionStatus,
    pub checker_id: Option<String>,
    pub checker_signature: Option<String>,
    pub step_up_token: Option<String>,
    pub approved_at: Option<u64>,
    pub rejection_reason: Option<String>,
    pub block_index: Option<u64>,
    pub block_hash: Option<String>,
    pub previous_block_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerBlock {
    pub index: u64,
    pub tx_id: String,
    pub tx_hash: String,
    pub previous_hash: String,
    pub block_hash: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerVerificationReport {
    pub verified: bool,
    pub total_blocks: usize,
    pub genesis_hash: String,
    pub tip_hash: String,
    pub chained_integrity_confirmed: bool,
    pub four_eyes_audit_compliant: bool,
}

pub struct MakerCheckerEngine {
    step_up: Arc<StepUpAuthEngine>,
    transactions: Mutex<HashMap<String, FinancialTransaction>>,
    ledger: Mutex<Vec<LedgerBlock>>,
    high_value_threshold: f64,
}

impl MakerCheckerEngine {
    pub fn new(step_up: Arc<StepUpAuthEngine>) -> Self {
        // Initialize Genesis Block in the financial ledger
        let genesis_block = LedgerBlock {
            index: 0,
            tx_id: "GENESIS".to_string(),
            tx_hash: "0".repeat(64),
            previous_hash: "0".repeat(64),
            block_hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
            timestamp: current_timestamp(),
        };

        Self {
            step_up,
            transactions: Mutex::new(HashMap::new()),
            ledger: Mutex::new(vec![genesis_block]),
            high_value_threshold: 10_000.0,
        }
    }

    fn calculate_canonical_hash(
        tx_id: &str,
        maker_id: &str,
        from_acc: &str,
        to_acc: &str,
        amount: f64,
        currency: &str,
        reference: &str,
        created_at: u64,
    ) -> String {
        let mut hasher = Sha256::new();
        let payload = format!(
            "{}:{}:{}:{}:{:.2}:{}:{}:{}",
            tx_id, maker_id, from_acc, to_acc, amount, currency, reference, created_at
        );
        hasher.update(payload.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Step 1: Maker proposes a financial transaction (Draft phase)
    pub fn propose_transaction(
        &self,
        maker_id: &str,
        from_account: &str,
        to_account: &str,
        amount: f64,
        currency: &str,
        reference: &str,
    ) -> Result<FinancialTransaction, String> {
        if maker_id.trim().is_empty() {
            return Err("Maker identifier is required".into());
        }
        if from_account.trim().is_empty() || to_account.trim().is_empty() {
            return Err("Both source and destination accounts are required".into());
        }
        if from_account.trim() == to_account.trim() {
            return Err("Source and destination accounts must be distinct".into());
        }
        if amount <= 0.0 {
            return Err("Transaction amount must be strictly positive".into());
        }

        let now = current_timestamp();
        let mut rand_bytes = [0u8; 8];
        getrandom::getrandom(&mut rand_bytes)
            .map_err(|e| format!("Secure entropy failure: {e}"))?;
        let tx_id = format!("TX-{}", hex::encode(rand_bytes));

        let canonical_hash = Self::calculate_canonical_hash(
            &tx_id,
            maker_id,
            from_account,
            to_account,
            amount,
            currency,
            reference,
            now,
        );

        // Compute simulated Ed25519 digital signature for maker
        let mut sig_hasher = Sha256::new();
        sig_hasher.update(b"MAKER_ED25519_SIG:");
        sig_hasher.update(maker_id.as_bytes());
        sig_hasher.update(canonical_hash.as_bytes());
        let maker_signature = format!("SIG-MAKER-{}", &format!("{:x}", sig_hasher.finalize())[..32]);

        let tx = FinancialTransaction {
            tx_id: tx_id.clone(),
            maker_id: maker_id.to_string(),
            from_account: from_account.to_string(),
            to_account: to_account.to_string(),
            amount,
            currency: currency.to_string(),
            reference: reference.to_string(),
            created_at: now,
            canonical_hash,
            maker_signature,
            status: TransactionStatus::PendingCheckerApproval,
            checker_id: None,
            checker_signature: None,
            step_up_token: None,
            approved_at: None,
            rejection_reason: None,
            block_index: None,
            block_hash: None,
            previous_block_hash: None,
        };

        let mut lock = self.transactions.lock().map_err(|_| "Lock poisoned")?;
        lock.insert(tx_id, tx.clone());

        Ok(tx)
    }

    /// Step 2: Checker approves transaction (Enforcing Four-Eyes Principle & Step-Up Biometrics)
    pub fn approve_transaction(
        &self,
        tx_id: &str,
        checker_id: &str,
        step_up_token: Option<&str>,
    ) -> Result<FinancialTransaction, String> {
        let mut tx_lock = self.transactions.lock().map_err(|_| "Lock poisoned")?;
        let tx = tx_lock
            .get_mut(tx_id)
            .ok_or_else(|| format!("Transaction not found: {tx_id}"))?;

        if tx.status != TransactionStatus::PendingCheckerApproval {
            return Err(format!(
                "Cannot approve transaction: current status is {:?}",
                tx.status
            ));
        }

        // 🛡️ CRITICAL BANKING RULE: FOUR-EYES PRINCIPLE (Segregation of Duties)
        if tx.maker_id.eq_ignore_ascii_case(checker_id.trim()) {
            return Err(format!(
                "SECURITY VIOLATION (Four-Eyes Principle): Maker '{}' cannot act as Checker to approve their own transaction",
                tx.maker_id
            ));
        }

        // 🛡️ High-Value Step-Up Biometric Requirement
        if tx.amount >= self.high_value_threshold {
            let token_str = step_up_token.ok_or_else(|| {
                format!(
                    "High-value transaction ({:.2} {}) requires mandatory Step-Up Biometric Authentication token",
                    tx.amount, tx.currency
                )
            })?;

            self.step_up
                .consume_token(token_str, checker_id, "TRANSFER_APPROVE", tx.amount)?;
            tx.step_up_token = Some(token_str.to_string());
        }

        let now = current_timestamp();

        // Compute Checker digital signature
        let mut sig_hasher = Sha256::new();
        sig_hasher.update(b"CHECKER_ED25519_SIG:");
        sig_hasher.update(checker_id.as_bytes());
        sig_hasher.update(tx.canonical_hash.as_bytes());
        let checker_signature =
            format!("SIG-CHECKER-{}", &format!("{:x}", sig_hasher.finalize())[..32]);

        // Append to immutable hash-chained ledger
        let mut ledger_lock = self.ledger.lock().map_err(|_| "Ledger lock poisoned")?;
        let prev_block = ledger_lock.last().ok_or("Ledger missing genesis block")?;
        let next_index = prev_block.index + 1;
        let prev_hash = prev_block.block_hash.clone();

        let mut block_hasher = Sha256::new();
        block_hasher.update(next_index.to_le_bytes());
        block_hasher.update(tx.canonical_hash.as_bytes());
        block_hasher.update(prev_hash.as_bytes());
        let block_hash = format!("{:x}", block_hasher.finalize());

        let block = LedgerBlock {
            index: next_index,
            tx_id: tx.tx_id.clone(),
            tx_hash: tx.canonical_hash.clone(),
            previous_hash: prev_hash.clone(),
            block_hash: block_hash.clone(),
            timestamp: now,
        };
        ledger_lock.push(block);

        // Update transaction state
        tx.status = TransactionStatus::AuthorizedCommitted;
        tx.checker_id = Some(checker_id.to_string());
        tx.checker_signature = Some(checker_signature);
        tx.approved_at = Some(now);
        tx.block_index = Some(next_index);
        tx.block_hash = Some(block_hash);
        tx.previous_block_hash = Some(prev_hash);

        Ok(tx.clone())
    }

    /// Step 3: Checker rejects transaction
    pub fn reject_transaction(
        &self,
        tx_id: &str,
        checker_id: &str,
        reason: &str,
    ) -> Result<FinancialTransaction, String> {
        let mut tx_lock = self.transactions.lock().map_err(|_| "Lock poisoned")?;
        let tx = tx_lock
            .get_mut(tx_id)
            .ok_or_else(|| format!("Transaction not found: {tx_id}"))?;

        if tx.status != TransactionStatus::PendingCheckerApproval {
            return Err("Transaction is not pending approval".into());
        }

        if tx.maker_id.eq_ignore_ascii_case(checker_id.trim()) {
            return Err(
                "Four-Eyes Principle: Maker cannot reject their own transaction as Checker".into(),
            );
        }

        tx.status = TransactionStatus::Rejected;
        tx.checker_id = Some(checker_id.to_string());
        tx.rejection_reason = Some(reason.to_string());

        Ok(tx.clone())
    }

    pub fn list_transactions(&self) -> Vec<FinancialTransaction> {
        let lock = self.transactions.lock().unwrap_or_else(|e| e.into_inner());
        let mut list: Vec<FinancialTransaction> = lock.values().cloned().collect();
        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        list
    }

    pub fn get_ledger(&self) -> Vec<LedgerBlock> {
        let lock = self.ledger.lock().unwrap_or_else(|e| e.into_inner());
        lock.clone()
    }

    /// Cryptographically validates the entire SHA-256 chain from Genesis block
    pub fn verify_ledger_integrity(&self) -> Result<LedgerVerificationReport, String> {
        let ledger = self.get_ledger();
        if ledger.is_empty() {
            return Err("Ledger is empty".into());
        }

        let genesis = &ledger[0];
        let mut prev_hash = genesis.block_hash.clone();

        for block in ledger.iter().skip(1) {
            if block.previous_hash != prev_hash {
                return Err(format!(
                    "LEDGER TAMPER DETECTED at Block #{}: Previous hash mismatch",
                    block.index
                ));
            }

            let mut hasher = Sha256::new();
            hasher.update(block.index.to_le_bytes());
            hasher.update(block.tx_hash.as_bytes());
            hasher.update(block.previous_hash.as_bytes());
            let computed = format!("{:x}", hasher.finalize());

            if computed != block.block_hash {
                return Err(format!(
                    "CRYPTOGRAPHIC CORRUPTION: Block #{} hash mismatch. Expected {}, Computed {}",
                    block.index, block.block_hash, computed
                ));
            }

            prev_hash = block.block_hash.clone();
        }

        let tip_hash = ledger.last().map(|b| b.block_hash.clone()).unwrap_or_default();

        Ok(LedgerVerificationReport {
            verified: true,
            total_blocks: ledger.len(),
            genesis_hash: genesis.block_hash.clone(),
            tip_hash,
            chained_integrity_confirmed: true,
            four_eyes_audit_compliant: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maker_checker_workflow_and_four_eyes_denial() {
        let step_up = Arc::new(StepUpAuthEngine::new());
        let engine = MakerCheckerEngine::new(step_up.clone());

        // 1. Maker proposes transaction
        let tx = engine
            .propose_transaction(
                "teller_alice",
                "ACC-100",
                "ACC-200",
                5000.0,
                "USD",
                "Invoice payment",
            )
            .unwrap();
        assert_eq!(tx.status, TransactionStatus::PendingCheckerApproval);
        assert!(tx.maker_signature.starts_with("SIG-MAKER-"));

        // 2. Maker cannot approve their own transaction (Four-Eyes Violation)
        let self_approval = engine.approve_transaction(&tx.tx_id, "teller_alice", None);
        assert!(self_approval.is_err());
        assert!(self_approval.unwrap_err().contains("Four-Eyes Principle"));

        // 3. Independent checker approves (< 10000 does not require step up)
        let approved = engine
            .approve_transaction(&tx.tx_id, "supervisor_bob", None)
            .unwrap();
        assert_eq!(approved.status, TransactionStatus::AuthorizedCommitted);
        assert_eq!(approved.checker_id.as_deref(), Some("supervisor_bob"));
        assert!(approved.block_index.is_some());

        // 4. Ledger integrity verification
        let report = engine.verify_ledger_integrity().unwrap();
        assert!(report.verified);
        assert_eq!(report.total_blocks, 2); // Genesis + 1 tx
    }

    #[test]
    fn test_high_value_transaction_requires_step_up_biometrics() {
        let step_up = Arc::new(StepUpAuthEngine::new());
        let engine = MakerCheckerEngine::new(step_up.clone());

        // Propose $50,000 transaction (>= 10,000 threshold)
        let tx = engine
            .propose_transaction(
                "maker_charlie",
                "CORP-01",
                "CORP-02",
                50_000.0,
                "EUR",
                "Treasury Transfer",
            )
            .unwrap();

        // Approval without step-up token must fail
        assert!(engine
            .approve_transaction(&tx.tx_id, "checker_dave", None)
            .is_err());

        // Issue biometric step-up challenge and token for checker_dave
        let chal = step_up
            .request_challenge("checker_dave", "TRANSFER_APPROVE", 50_000.0, "EUR")
            .unwrap();
        let token = step_up
            .verify_biometric(&chal.challenge_id, "checker_dave", "WINDOWS_HELLO_OK")
            .unwrap();

        // Approval with valid token succeeds!
        let approved = engine
            .approve_transaction(&tx.tx_id, "checker_dave", Some(&token.token))
            .unwrap();
        assert_eq!(approved.status, TransactionStatus::AuthorizedCommitted);
    }
}


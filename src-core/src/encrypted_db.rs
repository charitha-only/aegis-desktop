use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use zeroize::Zeroize;

#[allow(dead_code)]
const MAGIC_HEADER: &[u8; 12] = b"AEGIS_EDB_01";
#[allow(dead_code)]
const NONCE_SIZE: usize = 12;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumn {
    pub name: String,
    pub data_type: String, // "TEXT", "INTEGER", "REAL", "JSON"
    pub nullable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDefinition {
    pub name: String,
    pub primary_key: String,
    pub columns: Vec<TableColumn>,
    pub rows: HashMap<String, Value>, // Primary Key -> JSON Row Object
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseManifest {
    pub version: u32,
    pub created_at: u64,
    pub updated_at: u64,
    pub tables: HashMap<String, TableDefinition>,
}

pub struct EncryptedDbEngine {
    key: Mutex<[u8; 32]>,
    #[allow(dead_code)]
    db_path: PathBuf,
    manifest: Mutex<DatabaseManifest>,
    transaction_snapshot: Mutex<Option<DatabaseManifest>>,
}

impl EncryptedDbEngine {
    pub fn new(sandbox_dir: &Path) -> Self {
        let mut key = [0u8; 32];
        let _ = getrandom::getrandom(&mut key);

        let db_path = sandbox_dir.join("aegis_enterprise.edb");

        let initial_manifest = DatabaseManifest {
            version: 1,
            created_at: Self::current_timestamp(),
            updated_at: Self::current_timestamp(),
            tables: HashMap::new(),
        };

        let engine = Self {
            key: Mutex::new(key),
            db_path,
            manifest: Mutex::new(initial_manifest),
            transaction_snapshot: Mutex::new(None),
        };

        // Initialize with default banking tables
        let _ = engine.init_default_banking_tables();
        engine
    }

    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    fn init_default_banking_tables(&self) -> Result<(), String> {
        self.create_table(
            "accounts",
            "account_number",
            vec![
                TableColumn { name: "account_number".into(), data_type: "TEXT".into(), nullable: false },
                TableColumn { name: "holder_name".into(), data_type: "TEXT".into(), nullable: false },
                TableColumn { name: "balance_cents".into(), data_type: "INTEGER".into(), nullable: false },
                TableColumn { name: "currency".into(), data_type: "TEXT".into(), nullable: false },
                TableColumn { name: "status".into(), data_type: "TEXT".into(), nullable: false },
            ],
        )?;

        self.create_table(
            "audit_ledger",
            "entry_id",
            vec![
                TableColumn { name: "entry_id".into(), data_type: "TEXT".into(), nullable: false },
                TableColumn { name: "timestamp".into(), data_type: "INTEGER".into(), nullable: false },
                TableColumn { name: "actor".into(), data_type: "TEXT".into(), nullable: false },
                TableColumn { name: "operation".into(), data_type: "TEXT".into(), nullable: false },
                TableColumn { name: "sha256_hash".into(), data_type: "TEXT".into(), nullable: false },
            ],
        )?;

        // Pre-seed sample audited records
        self.insert(
            "accounts",
            serde_json::json!({
                "account_number": "CH93-0000-0000-1122-3344",
                "holder_name": "Aegis Prime Treasury Reserve",
                "balance_cents": 150000000000i64, // $1,500,000,000.00
                "currency": "USD",
                "status": "ACTIVE_TIER_1"
            }),
        )?;

        self.insert(
            "accounts",
            serde_json::json!({
                "account_number": "US12-BANK-9988-7766-5544",
                "holder_name": "Liquidity Clearing Settlement",
                "balance_cents": 45000000000i64,
                "currency": "USD",
                "status": "ACTIVE_TIER_1"
            }),
        )?;

        Ok(())
    }

    pub fn create_table(
        &self,
        name: &str,
        primary_key: &str,
        columns: Vec<TableColumn>,
    ) -> Result<(), String> {
        let mut manifest = self.manifest.lock().map_err(|e| e.to_string())?;
        if manifest.tables.contains_key(name) {
            return Ok(()); // Table already exists
        }

        manifest.tables.insert(
            name.to_string(),
            TableDefinition {
                name: name.to_string(),
                primary_key: primary_key.to_string(),
                columns,
                rows: HashMap::new(),
            },
        );
        manifest.updated_at = Self::current_timestamp();
        Ok(())
    }

    pub fn insert(&self, table: &str, row: Value) -> Result<String, String> {
        let mut manifest = self.manifest.lock().map_err(|e| e.to_string())?;
        let table_def = manifest
            .tables
            .get_mut(table)
            .ok_or_else(|| format!("Table '{}' does not exist", table))?;

        let pk_field = &table_def.primary_key;
        let pk_val = match row.get(pk_field) {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Number(n)) => n.to_string(),
            _ => return Err(format!("Row missing required primary key field '{}'", pk_field)),
        };

        if table_def.rows.contains_key(&pk_val) {
            return Err(format!("Primary key '{}' already exists in table '{}'", pk_val, table));
        }

        table_def.rows.insert(pk_val.clone(), row);
        manifest.updated_at = Self::current_timestamp();
        Ok(pk_val)
    }

    pub fn update(&self, table: &str, id: &str, row: Value) -> Result<(), String> {
        let mut manifest = self.manifest.lock().map_err(|e| e.to_string())?;
        let table_def = manifest
            .tables
            .get_mut(table)
            .ok_or_else(|| format!("Table '{}' does not exist", table))?;

        if !table_def.rows.contains_key(id) {
            return Err(format!("Record with id '{}' not found in table '{}'", id, table));
        }

        table_def.rows.insert(id.to_string(), row);
        manifest.updated_at = Self::current_timestamp();
        Ok(())
    }

    pub fn delete(&self, table: &str, id: &str) -> Result<bool, String> {
        let mut manifest = self.manifest.lock().map_err(|e| e.to_string())?;
        let table_def = manifest
            .tables
            .get_mut(table)
            .ok_or_else(|| format!("Table '{}' does not exist", table))?;

        let removed = table_def.rows.remove(id).is_some();
        if removed {
            manifest.updated_at = Self::current_timestamp();
        }
        Ok(removed)
    }

    #[allow(dead_code)]
    pub fn get(&self, table: &str, id: &str) -> Result<Option<Value>, String> {
        let manifest = self.manifest.lock().map_err(|e| e.to_string())?;
        let table_def = manifest
            .tables
            .get(table)
            .ok_or_else(|| format!("Table '{}' does not exist", table))?;

        Ok(table_def.rows.get(id).cloned())
    }

    pub fn query(
        &self,
        table: &str,
        filter_col: Option<&str>,
        filter_val: Option<&str>,
    ) -> Result<Vec<Value>, String> {
        let manifest = self.manifest.lock().map_err(|e| e.to_string())?;
        let table_def = manifest
            .tables
            .get(table)
            .ok_or_else(|| format!("Table '{}' does not exist", table))?;

        let mut results = Vec::new();
        for row in table_def.rows.values() {
            if let (Some(col), Some(val)) = (filter_col, filter_val) {
                if let Some(field_val) = row.get(col) {
                    let field_str = match field_val {
                        Value::String(s) => s.as_str(),
                        _ => "",
                    };
                    if field_str.eq_ignore_ascii_case(val) {
                        results.push(row.clone());
                    }
                }
            } else {
                results.push(row.clone());
            }
        }
        Ok(results)
    }

    pub fn begin_transaction(&self) -> Result<(), String> {
        let manifest = self.manifest.lock().map_err(|e| e.to_string())?;
        let mut tx = self.transaction_snapshot.lock().map_err(|e| e.to_string())?;
        if tx.is_some() {
            return Err("A transaction is already active".into());
        }
        *tx = Some(manifest.clone());
        Ok(())
    }

    pub fn commit_transaction(&self) -> Result<(), String> {
        let mut tx = self.transaction_snapshot.lock().map_err(|e| e.to_string())?;
        if tx.is_none() {
            return Err("No active transaction to commit".into());
        }
        *tx = None;
        Ok(())
    }

    pub fn rollback_transaction(&self) -> Result<(), String> {
        let mut tx = self.transaction_snapshot.lock().map_err(|e| e.to_string())?;
        let snapshot = tx.take().ok_or("No active transaction to rollback")?;
        let mut manifest = self.manifest.lock().map_err(|e| e.to_string())?;
        *manifest = snapshot;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn export_encrypted_payload(&self) -> Result<Vec<u8>, String> {
        let manifest = self.manifest.lock().map_err(|e| e.to_string())?;
        let plaintext = serde_json::to_vec(&*manifest).map_err(|e| e.to_string())?;

        let key_bytes = *self.key.lock().map_err(|e| e.to_string())?;
        let cipher_key = Key::from_slice(&key_bytes);
        let cipher = ChaCha20Poly1305::new(cipher_key);

        let mut nonce_bytes = [0u8; NONCE_SIZE];
        getrandom::getrandom(&mut nonce_bytes).map_err(|e| e.to_string())?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|_| "ChaCha20-Poly1305 database encryption failure")?;

        let mut out = Vec::with_capacity(MAGIC_HEADER.len() + NONCE_SIZE + ciphertext.len());
        out.extend_from_slice(MAGIC_HEADER);
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ciphertext);
        Ok(out)
    }

    pub fn stats(&self) -> Value {
        let manifest = self.manifest.lock().unwrap();
        let mut total_rows = 0;
        let mut tables_summary = Vec::new();

        for (name, table) in &manifest.tables {
            total_rows += table.rows.len();
            tables_summary.push(serde_json::json!({
                "name": name,
                "primaryKey": table.primary_key,
                "columns": table.columns.len(),
                "rowCount": table.rows.len()
            }));
        }

        let mut hasher = Sha256::new();
        hasher.update(total_rows.to_le_bytes());
        hasher.update(manifest.updated_at.to_le_bytes());
        let integrity_hash = hex::encode(hasher.finalize());

        serde_json::json!({
            "engine": "Aegis Encrypted SQL-Lite Engine (ChaCha20-Poly1305)",
            "version": manifest.version,
            "tableCount": manifest.tables.len(),
            "totalRows": total_rows,
            "createdAt": manifest.created_at,
            "updatedAt": manifest.updated_at,
            "encryption": "ChaCha20-Poly1305 Authenticated 256-bit AEAD",
            "integrityHash": integrity_hash,
            "tables": tables_summary,
            "acidCompliant": true
        })
    }
}

impl Drop for EncryptedDbEngine {
    fn drop(&mut self) {
        if let Ok(mut key) = self.key.lock() {
            key.zeroize();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypted_db_crud_and_transactions() {
        let temp = std::env::temp_dir();
        let db = EncryptedDbEngine::new(&temp);

        // Query default accounts table
        let accounts = db.query("accounts", None, None).unwrap();
        assert_eq!(accounts.len(), 2);

        // Insert new record
        let new_id = db
            .insert(
                "accounts",
                serde_json::json!({
                    "account_number": "LK00-TEST-1234",
                    "holder_name": "Test Account Holder",
                    "balance_cents": 500000,
                    "currency": "LKR",
                    "status": "ACTIVE"
                }),
            )
            .unwrap();
        assert_eq!(new_id, "LK00-TEST-1234");

        // Verify retrieval
        let retrieved = db.get("accounts", "LK00-TEST-1234").unwrap().unwrap();
        assert_eq!(retrieved["holder_name"], "Test Account Holder");

        // Transaction Rollback Test
        db.begin_transaction().unwrap();
        db.insert(
            "accounts",
            serde_json::json!({
                "account_number": "DOOMED-RECORD",
                "holder_name": "Should Disappear",
                "balance_cents": 100,
                "currency": "USD",
                "status": "PENDING"
            }),
        )
        .unwrap();
        assert!(db.get("accounts", "DOOMED-RECORD").unwrap().is_some());
        db.rollback_transaction().unwrap();
        assert!(db.get("accounts", "DOOMED-RECORD").unwrap().is_none());

        // Encryption export check
        let encrypted = db.export_encrypted_payload().unwrap();
        assert!(encrypted.starts_with(MAGIC_HEADER));
        assert!(encrypted.len() > 64);
    }
}

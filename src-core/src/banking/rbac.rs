use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoleType {
    SuperAdmin,
    ComplianceOfficer,
    BranchManager,
    SeniorTeller,
    Auditor,
}

impl RoleType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RoleType::SuperAdmin => "SuperAdmin",
            RoleType::ComplianceOfficer => "ComplianceOfficer",
            RoleType::BranchManager => "BranchManager",
            RoleType::SeniorTeller => "SeniorTeller",
            RoleType::Auditor => "Auditor",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            RoleType::SuperAdmin => "Super Admin (Global Security Officer)",
            RoleType::ComplianceOfficer => "Chief Compliance & Regulatory Officer",
            RoleType::BranchManager => "Branch Treasury & Settlement Manager",
            RoleType::SeniorTeller => "Senior Banking Officer / Teller",
            RoleType::Auditor => "External Regulatory & Independent Auditor",
        }
    }

    pub fn from_str(role: &str) -> Result<Self, String> {
        match role {
            "SuperAdmin" | "super_admin" => Ok(RoleType::SuperAdmin),
            "ComplianceOfficer" | "compliance_officer" => Ok(RoleType::ComplianceOfficer),
            "BranchManager" | "branch_manager" => Ok(RoleType::BranchManager),
            "SeniorTeller" | "senior_teller" => Ok(RoleType::SeniorTeller),
            "Auditor" | "auditor" => Ok(RoleType::Auditor),
            _ => Err(format!("Unknown RBAC role: {}", role)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RbacPermissions {
    pub can_initiate_transaction: bool,
    pub can_approve_transaction: bool,
    pub can_view_ledger: bool,
    pub can_export_compliance: bool,
    pub can_modify_security_policies: bool,
    pub can_rotate_keys: bool,
    pub can_access_database: bool,
    pub max_single_transaction_cents: u64,
}

impl RoleType {
    pub fn default_permissions(&self) -> RbacPermissions {
        match self {
            RoleType::SuperAdmin => RbacPermissions {
                can_initiate_transaction: false, // Strict SoD: admins cannot unilaterally initiate funds
                can_approve_transaction: false,  // Separation from treasury operations
                can_view_ledger: true,
                can_export_compliance: true,
                can_modify_security_policies: true,
                can_rotate_keys: true,
                can_access_database: true,
                max_single_transaction_cents: 0,
            },
            RoleType::ComplianceOfficer => RbacPermissions {
                can_initiate_transaction: false,
                can_approve_transaction: false,
                can_view_ledger: true,
                can_export_compliance: true,
                can_modify_security_policies: false,
                can_rotate_keys: false,
                can_access_database: true,
                max_single_transaction_cents: 0,
            },
            RoleType::BranchManager => RbacPermissions {
                can_initiate_transaction: false, // Manager approves, does not initiate
                can_approve_transaction: true,
                can_view_ledger: true,
                can_export_compliance: true,
                can_modify_security_policies: false,
                can_rotate_keys: false,
                can_access_database: true,
                max_single_transaction_cents: 10_000_000_00, // $10,000,000.00 approval limit
            },
            RoleType::SeniorTeller => RbacPermissions {
                can_initiate_transaction: true,
                can_approve_transaction: false, // Teller cannot approve
                can_view_ledger: true,
                can_export_compliance: false,
                can_modify_security_policies: false,
                can_rotate_keys: false,
                can_access_database: true,
                max_single_transaction_cents: 250_000_00, // $250,000.00 maker limit
            },
            RoleType::Auditor => RbacPermissions {
                can_initiate_transaction: false,
                can_approve_transaction: false,
                can_view_ledger: true,
                can_export_compliance: true,
                can_modify_security_policies: false,
                can_rotate_keys: false,
                can_access_database: false, // Read-only via audit reports, no raw DB manipulation
                max_single_transaction_cents: 0,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIdentity {
    pub user_id: String,
    pub display_name: String,
    pub role: RoleType,
    pub department: String,
    pub permissions: RbacPermissions,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessAttempt {
    pub timestamp: u64,
    pub actor_id: String,
    pub role: String,
    pub requested_action: String,
    pub target_resource: String,
    pub decision: String, // "PERMITTED" or "DENIED_PERMISSION" or "DENIED_SOD"
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleMatrixEntry {
    pub role: String,
    pub display_name: String,
    pub description: String,
    pub permissions: RbacPermissions,
}

pub struct RbacEngine {
    current_identity: Mutex<UserIdentity>,
    audit_log: Mutex<Vec<AccessAttempt>>,
}

impl RbacEngine {
    pub fn new() -> Self {
        let initial_role = RoleType::BranchManager;
        let initial_identity = UserIdentity {
            user_id: "USR-MGR-8891".to_string(),
            display_name: "Elena Rostova (Branch Treasury)".to_string(),
            role: initial_role,
            department: "Global Treasury & Settlements".to_string(),
            permissions: initial_role.default_permissions(),
            session_id: "SESS-SEC-9901-T2".to_string(),
        };

        Self {
            current_identity: Mutex::new(initial_identity),
            audit_log: Mutex::new(Vec::new()),
        }
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    pub fn get_identity(&self) -> UserIdentity {
        self.current_identity.lock().unwrap().clone()
    }

    pub fn switch_role(&self, new_role: RoleType, actor_name: Option<&str>) -> UserIdentity {
        let mut id = self.current_identity.lock().unwrap();
        let old_role = id.role;
        id.role = new_role;
        id.permissions = new_role.default_permissions();
        if let Some(name) = actor_name {
            id.display_name = name.to_string();
        } else {
            id.display_name = format!("Enterprise Actor ({})", new_role.as_str());
        }

        // Record audit entry for role elevation / switch
        let mut log = self.audit_log.lock().unwrap();
        log.push(AccessAttempt {
            timestamp: Self::current_timestamp(),
            actor_id: id.user_id.clone(),
            role: new_role.as_str().to_string(),
            requested_action: "ROLE_CHANGE".to_string(),
            target_resource: format!("from:{} to:{}", old_role.as_str(), new_role.as_str()),
            decision: "PERMITTED".to_string(),
            reason: format!("User assumed {} privileges under supervisory trail", new_role.as_str()),
        });

        id.clone()
    }

    pub fn check_permission(&self, action: &str, resource: &str) -> (bool, String) {
        let id = self.current_identity.lock().unwrap();
        let perms = &id.permissions;

        let (allowed, reason) = match action {
            "initiate_transaction" => {
                if perms.can_initiate_transaction {
                    (true, "Transaction origination permitted by SeniorTeller role".to_string())
                } else {
                    (false, format!("Role {} is strictly prohibited from initiating financial transfers (Segregation of Duties)", id.role.as_str()))
                }
            }
            "approve_transaction" => {
                if perms.can_approve_transaction {
                    (true, "Maker-Checker dual authorization capability verified".to_string())
                } else {
                    (false, format!("Role {} lacks managerial transaction approval capability", id.role.as_str()))
                }
            }
            "view_ledger" => {
                if perms.can_view_ledger {
                    (true, "Ledger inspection granted".to_string())
                } else {
                    (false, "Ledger access denied".to_string())
                }
            }
            "export_compliance" => {
                if perms.can_export_compliance {
                    (true, "Regulatory export authorization granted".to_string())
                } else {
                    (false, format!("Role {} cannot export regulatory audit digests", id.role.as_str()))
                }
            }
            "modify_security_policies" => {
                if perms.can_modify_security_policies {
                    (true, "Administrative policy reconfiguration authorized".to_string())
                } else {
                    (false, "Only SuperAdmin can modify framework security controls".to_string())
                }
            }
            "rotate_keys" => {
                if perms.can_rotate_keys {
                    (true, "Cryptographic key rotation authorized".to_string())
                } else {
                    (false, "Key rotation requires SuperAdmin authority".to_string())
                }
            }
            "access_database" => {
                if perms.can_access_database {
                    (true, "Encrypted database direct query access granted".to_string())
                } else {
                    (false, format!("Role {} is restricted from raw database access", id.role.as_str()))
                }
            }
            _ => (false, format!("Unrecognized capability check: {}", action)),
        };

        let decision = if allowed { "PERMITTED" } else { "DENIED_PERMISSION" };
        let mut log = self.audit_log.lock().unwrap();
        log.push(AccessAttempt {
            timestamp: Self::current_timestamp(),
            actor_id: id.user_id.clone(),
            role: id.role.as_str().to_string(),
            requested_action: action.to_string(),
            target_resource: resource.to_string(),
            decision: decision.to_string(),
            reason: reason.clone(),
        });

        (allowed, reason)
    }

    /// Enforces strict Maker-Checker Segregation of Duties.
    /// Under Basel III & PCI-DSS Section 7/8, the initiator and approver
    /// of a financial settlement CANNOT be the same cryptographic identity.
    #[allow(dead_code)]
    pub fn verify_maker_checker_sod(&self, maker_id: &str, checker_id: &str) -> Result<(), String> {
        let mut log = self.audit_log.lock().unwrap();
        let ts = Self::current_timestamp();

        if maker_id.trim().eq_ignore_ascii_case(checker_id.trim()) {
            let reason = format!(
                "Segregation of Duties VIOLATION: Maker ({}) and Checker ({}) cannot be the same entity.",
                maker_id, checker_id
            );
            log.push(AccessAttempt {
                timestamp: ts,
                actor_id: checker_id.to_string(),
                role: "BranchManager".to_string(),
                requested_action: "MAKER_CHECKER_APPROVAL".to_string(),
                target_resource: "financial_transfer".to_string(),
                decision: "DENIED_SOD".to_string(),
                reason: reason.clone(),
            });
            return Err(reason);
        }

        log.push(AccessAttempt {
            timestamp: ts,
            actor_id: checker_id.to_string(),
            role: "BranchManager".to_string(),
            requested_action: "MAKER_CHECKER_APPROVAL".to_string(),
            target_resource: "financial_transfer".to_string(),
            decision: "PERMITTED".to_string(),
            reason: format!("Maker ({}) != Checker ({}) dual-control validated.", maker_id, checker_id),
        });

        Ok(())
    }

    pub fn get_audit_log(&self) -> Vec<AccessAttempt> {
        self.audit_log.lock().unwrap().clone()
    }

    pub fn get_role_matrix(&self) -> Vec<RoleMatrixEntry> {
        vec![
            RoleMatrixEntry {
                role: RoleType::SuperAdmin.as_str().to_string(),
                display_name: RoleType::SuperAdmin.display_name().to_string(),
                description: "Security architecture, cryptographic key lifecycle, and zero-trust configuration. Prohibited from financial execution.".to_string(),
                permissions: RoleType::SuperAdmin.default_permissions(),
            },
            RoleMatrixEntry {
                role: RoleType::ComplianceOfficer.as_str().to_string(),
                display_name: RoleType::ComplianceOfficer.display_name().to_string(),
                description: "Regulatory reporting, ASVS/PCI-DSS compliance attestation, immutable audit trail export.".to_string(),
                permissions: RoleType::ComplianceOfficer.default_permissions(),
            },
            RoleMatrixEntry {
                role: RoleType::BranchManager.as_str().to_string(),
                display_name: RoleType::BranchManager.display_name().to_string(),
                description: "Maker-Checker dual approval authority up to $10,000,000. Prohibited from transaction origination.".to_string(),
                permissions: RoleType::BranchManager.default_permissions(),
            },
            RoleMatrixEntry {
                role: RoleType::SeniorTeller.as_str().to_string(),
                display_name: RoleType::SeniorTeller.display_name().to_string(),
                description: "Front-office transaction origination up to $250,000 maker limit. Prohibited from approving own transactions.".to_string(),
                permissions: RoleType::SeniorTeller.default_permissions(),
            },
            RoleMatrixEntry {
                role: RoleType::Auditor.as_str().to_string(),
                display_name: RoleType::Auditor.display_name().to_string(),
                description: "External independent verification. Strictly read-only access to tamper-evident ledgers.".to_string(),
                permissions: RoleType::Auditor.default_permissions(),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rbac_default_identity_and_roles() {
        let engine = RbacEngine::new();
        let id = engine.get_identity();
        assert_eq!(id.role, RoleType::BranchManager);
        assert!(id.permissions.can_approve_transaction);
        assert!(!id.permissions.can_initiate_transaction);
    }

    #[test]
    fn test_rbac_switch_to_senior_teller() {
        let engine = RbacEngine::new();
        let id = engine.switch_role(RoleType::SeniorTeller, Some("Kasun Perera"));
        assert_eq!(id.role, RoleType::SeniorTeller);
        assert_eq!(id.display_name, "Kasun Perera");
        assert!(id.permissions.can_initiate_transaction);
        assert!(!id.permissions.can_approve_transaction);

        let (allowed_init, _) = engine.check_permission("initiate_transaction", "tx_123");
        assert!(allowed_init);

        let (allowed_appr, _) = engine.check_permission("approve_transaction", "tx_123");
        assert!(!allowed_appr);
    }

    #[test]
    fn test_maker_checker_sod_enforcement() {
        let engine = RbacEngine::new();

        // Same maker and checker must be blocked
        let res_fail = engine.verify_maker_checker_sod("TELLER_01", "TELLER_01");
        assert!(res_fail.is_err());
        assert!(res_fail.unwrap_err().contains("Segregation of Duties VIOLATION"));

        // Different maker and checker is approved
        let res_ok = engine.verify_maker_checker_sod("TELLER_01", "MANAGER_09");
        assert!(res_ok.is_ok());

        let audit = engine.get_audit_log();
        assert!(audit.iter().any(|entry| entry.decision == "DENIED_SOD"));
        assert!(audit.iter().any(|entry| entry.decision == "PERMITTED"));
    }

    #[test]
    fn test_super_admin_cannot_execute_financial_operations() {
        let engine = RbacEngine::new();
        engine.switch_role(RoleType::SuperAdmin, None);

        let (can_init, _) = engine.check_permission("initiate_transaction", "acc_1");
        assert!(!can_init);

        let (can_appr, _) = engine.check_permission("approve_transaction", "acc_1");
        assert!(!can_appr);

        let (can_rotate, _) = engine.check_permission("rotate_keys", "master_key");
        assert!(can_rotate);
    }
}

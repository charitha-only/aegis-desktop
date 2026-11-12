pub mod maker_checker;
pub mod rbac;
pub mod screen_protection;
pub mod step_up;

#[allow(unused_imports)]
pub use maker_checker::{FinancialTransaction, MakerCheckerEngine, TransactionStatus};
#[allow(unused_imports)]
pub use rbac::{AccessAttempt, RbacEngine, RbacPermissions, RoleMatrixEntry, RoleType, UserIdentity};
pub use screen_protection::ScreenProtectionEngine;
pub use step_up::StepUpAuthEngine;

use std::sync::Arc;

pub struct BankingEngine {
    pub screen_protection: Arc<ScreenProtectionEngine>,
    pub maker_checker: Arc<MakerCheckerEngine>,
    pub step_up: Arc<StepUpAuthEngine>,
    pub rbac: Arc<RbacEngine>,
}

impl BankingEngine {
    pub fn new() -> Self {
        let step_up = Arc::new(StepUpAuthEngine::new());
        let maker_checker = Arc::new(MakerCheckerEngine::new(step_up.clone()));
        let screen_protection = Arc::new(ScreenProtectionEngine::new());
        let rbac = Arc::new(RbacEngine::new());
        Self {
            screen_protection,
            maker_checker,
            step_up,
            rbac,
        }
    }
}

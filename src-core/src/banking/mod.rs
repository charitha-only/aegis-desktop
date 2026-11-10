pub mod maker_checker;
pub mod screen_protection;
pub mod step_up;

#[allow(unused_imports)]
pub use maker_checker::{FinancialTransaction, MakerCheckerEngine, TransactionStatus};
pub use screen_protection::ScreenProtectionEngine;
pub use step_up::StepUpAuthEngine;

use std::sync::Arc;

pub struct BankingEngine {
    pub screen_protection: Arc<ScreenProtectionEngine>,
    pub maker_checker: Arc<MakerCheckerEngine>,
    pub step_up: Arc<StepUpAuthEngine>,
}

impl BankingEngine {
    pub fn new() -> Self {
        let step_up = Arc::new(StepUpAuthEngine::new());
        let maker_checker = Arc::new(MakerCheckerEngine::new(step_up.clone()));
        let screen_protection = Arc::new(ScreenProtectionEngine::new());
        Self {
            screen_protection,
            maker_checker,
            step_up,
        }
    }
}

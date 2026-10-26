//! Revoke-only lifecycle observer. This interface can never authorize a session.
use super::vault_session::VaultSessions;
use std::sync::Weak;

pub struct LifecycleObserver {
    sessions: Weak<VaultSessions>,
}

impl LifecycleObserver {
    pub fn new(sessions: Weak<VaultSessions>) -> Self {
        Self { sessions }
    }

    pub fn revoke(&self) {
        if let Some(sessions) = self.sessions.upgrade() {
            sessions.system_lock();
        }
    }

    pub fn is_needed(&self) -> bool {
        self.sessions.strong_count() != 0
    }
}

impl Drop for LifecycleObserver {
    fn drop(&mut self) {
        self.revoke();
    }
}

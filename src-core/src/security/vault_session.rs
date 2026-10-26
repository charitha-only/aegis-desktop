//! One process-local, vault-scoped unlocked session. Never persists credentials.
use super::VaultEngine;
use crate::secrets::SecretManager;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};
use zeroize::Zeroizing;

pub const IDLE_TIMEOUT: Duration = Duration::from_secs(60);
pub const ABSOLUTE_TIMEOUT: Duration = Duration::from_secs(300);

struct Unlocked {
    token: Zeroizing<String>,
    vault: String,
    password: Zeroizing<String>,
    opened: Instant,
    used: Instant,
    epoch: u64,
}
pub struct VaultSessions {
    state: Arc<Mutex<Option<Unlocked>>>,
    idle: Duration,
    absolute: Duration,
    secrets: Arc<SecretManager>,
    epoch: AtomicU64,
    monitor_required: AtomicBool,
    monitor_alive: AtomicBool,
}
impl VaultSessions {
    pub fn new(secrets: Arc<SecretManager>) -> Self {
        Self::with_limits(secrets, IDLE_TIMEOUT, ABSOLUTE_TIMEOUT)
    }
    pub fn with_limits(secrets: Arc<SecretManager>, idle: Duration, absolute: Duration) -> Self {
        let state = Arc::new(Mutex::new(None::<Unlocked>));
        let weak_state = Arc::downgrade(&state);
        let weak_secrets = Arc::downgrade(&secrets);
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_secs(1));
            let Some(state) = weak_state.upgrade() else {
                break;
            };
            let Some(secrets) = weak_secrets.upgrade() else {
                break;
            };
            let mut state = state.lock().unwrap_or_else(|e| e.into_inner());
            if state
                .as_ref()
                .is_some_and(|s| s.used.elapsed() >= idle || s.opened.elapsed() >= absolute)
            {
                *state = None;
                secrets.clear();
            }
        });
        Self {
            state,
            secrets,
            epoch: AtomicU64::new(0),
            monitor_required: AtomicBool::new(false),
            monitor_alive: AtomicBool::new(false),
            idle,
            absolute,
        }
    }
    pub fn unlock(
        &self,
        vault: &str,
        envelope: &str,
        password: &str,
    ) -> Result<Zeroizing<String>, String> {
        let mut state = self.state.lock().map_err(|_| "Session state unavailable")?;
        self.check_platform_or_clear(&mut state)?;
        let epoch = self.epoch.load(Ordering::SeqCst);
        let plaintext = Zeroizing::new(VaultEngine::decrypt(envelope, password)?);
        self.check_platform_or_clear(&mut state)?;
        if epoch != self.epoch.load(Ordering::SeqCst) {
            return Err("System session changed".into());
        }
        drop(plaintext);
        let mut random = Zeroizing::new([0u8; 32]);
        getrandom::getrandom(&mut *random).map_err(|_| "Secure randomness unavailable")?;
        let token = Zeroizing::new(hex::encode(&*random));
        self.secrets.clear();
        *state = Some(Unlocked {
            token: token.clone(),
            vault: vault.into(),
            password: Zeroizing::new(password.into()),
            opened: Instant::now(),
            used: Instant::now(),
            epoch,
        });
        Ok(token)
    }
    pub fn read(
        &self,
        token: &str,
        vault: &str,
        envelope: &str,
    ) -> Result<Zeroizing<String>, String> {
        let mut state = self.state.lock().map_err(|_| "Session state unavailable")?;
        if state
            .as_ref()
            .is_some_and(|s| s.used.elapsed() >= self.idle || s.opened.elapsed() >= self.absolute)
        {
            *state = None;
            self.secrets.clear();
        }
        self.check_platform_or_clear(&mut state)?;
        let unlocked = state.as_mut().ok_or("Vault is locked")?;
        if unlocked.epoch != self.epoch.load(Ordering::SeqCst) {
            return Err("System session changed".into());
        }
        let valid = token.len() == unlocked.token.len()
            && token
                .as_bytes()
                .iter()
                .zip(unlocked.token.as_bytes())
                .fold(0u8, |a, (x, y)| a | (x ^ y))
                == 0;
        if !valid || unlocked.vault != vault {
            return Err("Invalid vault session".into());
        }
        let result = VaultEngine::decrypt(envelope, &unlocked.password).map(Zeroizing::new);
        if self.check_platform().is_err()
            || unlocked.epoch != self.epoch.load(Ordering::SeqCst)
            || unlocked.used.elapsed() >= self.idle
            || unlocked.opened.elapsed() >= self.absolute
        {
            *state = None;
            self.secrets.clear();
            return Err("Vault session expired".into());
        }
        if result.is_ok() {
            unlocked.used = Instant::now();
        }
        result
    }
    pub fn require_platform_monitor(&self) {
        self.monitor_required.store(true, Ordering::SeqCst);
        self.system_lock();
    }
    pub fn platform_monitor_active(&self) -> bool {
        self.monitor_alive.load(Ordering::SeqCst)
    }
    pub fn set_monitor_alive(&self, alive: bool) {
        self.monitor_alive.store(alive, Ordering::SeqCst);
        if !alive {
            self.system_lock();
        }
    }
    fn check_platform(&self) -> Result<(), String> {
        if self.monitor_required.load(Ordering::SeqCst)
            && (!self.monitor_alive.load(Ordering::SeqCst)
                || !super::windows_lifecycle::desktop_available())
        {
            return Err("Operating-system session unavailable or locked".into());
        }
        Ok(())
    }
    fn check_platform_or_clear(&self, state: &mut Option<Unlocked>) -> Result<(), String> {
        if let Err(error) = self.check_platform() {
            // The caller already holds the state lock: system_lock() would deadlock.
            self.epoch.fetch_add(1, Ordering::SeqCst);
            *state = None;
            self.secrets.clear();
            return Err(error);
        }
        Ok(())
    }
    pub fn system_lock(&self) {
        self.epoch.fetch_add(1, Ordering::SeqCst);
        self.lock();
    }
    pub fn lock(&self) {
        // Even a poisoned lock must not leave retained credentials available.
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        *state = None;
        self.secrets.clear();
    }
}

impl Drop for VaultSessions {
    fn drop(&mut self) {
        self.lock();
    }
}

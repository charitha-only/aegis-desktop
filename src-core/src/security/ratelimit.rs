use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Sliding-Window Rate Limiter & Anti-DoS Shield
/// Protects the IPC loopback server from local malware fuzzing and brute-force nonce guessing.
pub struct RateLimiter {
    state: Mutex<RateLimiterState>,
    max_requests_per_window: usize,
    window_duration: Duration,
    lockout_duration: Duration,
}

struct RateLimiterState {
    request_timestamps: Vec<Instant>,
    failed_nonces: HashMap<String, (usize, Instant)>, // (consecutive_failures, last_failure)
}

impl RateLimiter {
    pub fn new(max_requests_per_window: usize, window_secs: u64, lockout_secs: u64) -> Self {
        Self {
            state: Mutex::new(RateLimiterState {
                request_timestamps: Vec::new(),
                failed_nonces: HashMap::new(),
            }),
            max_requests_per_window,
            window_duration: Duration::from_secs(window_secs),
            lockout_duration: Duration::from_secs(lockout_secs),
        }
    }

    /// Checks if a request is allowed under the sliding window limit
    pub fn check_request(&self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        let now = Instant::now();

        // Evict timestamps outside sliding window
        let window = self.window_duration;
        state
            .request_timestamps
            .retain(|&t| now.duration_since(t) <= window);

        if state.request_timestamps.len() >= self.max_requests_per_window {
            return Err(format!(
                "Rate limit exceeded: Max {} requests per {}s allowed by Aegis Anti-DoS Shield",
                self.max_requests_per_window,
                window.as_secs()
            ));
        }

        state.request_timestamps.push(now);
        Ok(())
    }

    /// Records a failed nonce attempt. If threshold exceeded, activates brute-force lockout.
    pub fn record_nonce_failure(&self, origin: &str) -> bool {
        let mut state = self.state.lock().unwrap();
        let now = Instant::now();
        let entry = state
            .failed_nonces
            .entry(origin.to_string())
            .or_insert((0, now));

        if now.duration_since(entry.1) > self.lockout_duration {
            entry.0 = 0; // reset expired penalty
        }

        entry.0 += 1;
        entry.1 = now;

        entry.0 >= 5 // Lockout trigger threshold
    }

    pub fn clear_nonce_failures(&self, origin: &str) {
        self.state.lock().unwrap().failed_nonces.remove(origin);
    }

    /// Checks if an origin is locked out due to suspected brute-force attacks
    pub fn is_locked_out(&self, origin: &str) -> bool {
        let state = self.state.lock().unwrap();
        if let Some(&(failures, last_time)) = state.failed_nonces.get(origin) {
            if failures >= 5 && Instant::now().duration_since(last_time) < self.lockout_duration {
                return true;
            }
        }
        false
    }
}

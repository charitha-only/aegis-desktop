use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Sliding-Window Rate Limiter & Anti-DoS Shield
///
/// Protects the IPC loopback server from local malware fuzzing and brute-force
/// nonce guessing. Uses a `VecDeque` ring buffer for O(1) amortized front-eviction
/// instead of the previous `Vec::retain` O(n) scan on every request.
pub struct RateLimiter {
    state: Mutex<RateLimiterState>,
    max_requests_per_window: usize,
    window_duration: Duration,
    lockout_duration: Duration,
    lockout_threshold: usize,
}

struct RateLimiterState {
    /// Ring buffer — oldest timestamps at the front, newest at the back.
    request_timestamps: VecDeque<Instant>,
    /// Per-origin nonce-failure tracking: (consecutive_failures, last_failure_time).
    failed_nonces: HashMap<String, (usize, Instant)>,
}

impl RateLimiter {
    pub fn new(max_requests_per_window: usize, window_secs: u64, lockout_secs: u64) -> Self {
        Self {
            state: Mutex::new(RateLimiterState {
                // Pre-allocate capacity to avoid reallocation under load.
                request_timestamps: VecDeque::with_capacity(max_requests_per_window + 1),
                failed_nonces: HashMap::new(),
            }),
            max_requests_per_window,
            window_duration: Duration::from_secs(window_secs),
            lockout_duration: Duration::from_secs(lockout_secs),
            lockout_threshold: 5,
        }
    }

    /// Checks if a request is allowed under the sliding window limit.
    ///
    /// Eviction is O(1) amortised: stale entries are popped from the front of the
    /// deque rather than scanning the whole buffer with `Vec::retain`.
    pub fn check_request(&self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        let now = Instant::now();
        let window = self.window_duration;

        // Pop expired entries from the front (they are strictly oldest-first).
        while state
            .request_timestamps
            .front()
            .is_some_and(|&t| now.duration_since(t) > window)
        {
            state.request_timestamps.pop_front();
        }

        if state.request_timestamps.len() >= self.max_requests_per_window {
            return Err(format!(
                "Rate limit exceeded: Max {} requests per {}s allowed by Aegis Anti-DoS Shield",
                self.max_requests_per_window,
                window.as_secs()
            ));
        }

        state.request_timestamps.push_back(now);
        Ok(())
    }

    /// Records a failed nonce attempt. Returns `true` when the lockout threshold
    /// has been crossed and the origin should be treated as locked out.
    pub fn record_nonce_failure(&self, origin: &str) -> bool {
        let mut state = self.state.lock().unwrap();
        let now = Instant::now();

        // Evict expired entries if table exceeds bound to prevent memory exhaustion DoS
        if state.failed_nonces.len() >= 1024 {
            let lockout = self.lockout_duration;
            state
                .failed_nonces
                .retain(|_, (_, last_time)| now.duration_since(*last_time) <= lockout);
        }

        let entry = state
            .failed_nonces
            .entry(origin.to_string())
            .or_insert((0, now));

        // Reset counter if the lockout window has expired since the last failure.
        if now.duration_since(entry.1) > self.lockout_duration {
            entry.0 = 0;
        }

        entry.0 += 1;
        entry.1 = now;

        entry.0 >= self.lockout_threshold
    }

    pub fn clear_nonce_failures(&self, origin: &str) {
        self.state.lock().unwrap().failed_nonces.remove(origin);
    }

    /// Returns `true` if the origin is currently in an active brute-force lockout.
    pub fn is_locked_out(&self, origin: &str) -> bool {
        let state = self.state.lock().unwrap();
        if let Some(&(failures, last_time)) = state.failed_nonces.get(origin) {
            return failures >= self.lockout_threshold
                && Instant::now().duration_since(last_time) < self.lockout_duration;
        }
        false
    }

    /// Returns remaining lockout duration in seconds if the origin is locked out.
    pub fn remaining_lockout_secs(&self, origin: &str) -> Option<u64> {
        let state = self.state.lock().unwrap();
        if let Some(&(failures, last_time)) = state.failed_nonces.get(origin) {
            if failures >= self.lockout_threshold {
                let elapsed = Instant::now().duration_since(last_time);
                if elapsed < self.lockout_duration {
                    return Some((self.lockout_duration - elapsed).as_secs().max(1));
                }
            }
        }
        None
    }

    /// Exposes live telemetry for security audits & diagnostics dashboards.
    pub fn get_telemetry(&self) -> serde_json::Value {
        let state = self.state.lock().unwrap();
        serde_json::json!({
            "activeRequestsInWindow": state.request_timestamps.len(),
            "maxRequestsPerWindow": self.max_requests_per_window,
            "windowDurationSecs": self.window_duration.as_secs(),
            "trackedFailedOrigins": state.failed_nonces.len(),
            "lockoutDurationSecs": self.lockout_duration.as_secs(),
            "lockoutThreshold": self.lockout_threshold,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limiter_sliding_window_and_dynamic_lockout() {
        let limiter = RateLimiter::new(5, 1, 10);
        for _ in 0..5 {
            assert!(limiter.check_request().is_ok());
        }
        assert!(limiter.check_request().is_err());

        // Lockout tracking
        let origin = "http://malicious-fuzzer.local";
        assert!(!limiter.is_locked_out(origin));
        assert_eq!(limiter.remaining_lockout_secs(origin), None);

        for _ in 0..4 {
            assert!(!limiter.record_nonce_failure(origin));
        }
        assert!(limiter.record_nonce_failure(origin));
        assert!(limiter.is_locked_out(origin));

        let remaining = limiter.remaining_lockout_secs(origin);
        assert!(remaining.is_some());
        assert!(remaining.unwrap() <= 10 && remaining.unwrap() >= 1);

        limiter.clear_nonce_failures(origin);
        assert!(!limiter.is_locked_out(origin));
        assert_eq!(limiter.remaining_lockout_secs(origin), None);
    }
}

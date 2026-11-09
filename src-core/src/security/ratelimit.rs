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
}

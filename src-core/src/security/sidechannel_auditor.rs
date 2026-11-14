//! Constant-Time Side-Channel Auditor (Dudect Statistical Test)
//!
//! Implements Welch's two-sample t-test ("Dude, is my code constant time?")
//! to mathematically evaluate cryptographic routines for timing side-channel leakage.
//! Asserts that execution distributions for secret vs public inputs do not diverge (|t| < 4.5).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Result of Welch's t-test evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DudectAuditReport {
    pub routine_name: String,
    pub total_samples: usize,
    pub class_a_mean_nanos: f64,
    pub class_b_mean_nanos: f64,
    pub t_statistic: f64,
    pub max_t_threshold: f64,
    pub is_constant_time: bool,
}

/// Side-Channel Statistical Auditor
pub struct SideChannelAuditor;

impl SideChannelAuditor {
    /// Execute Dudect Welch's t-test against a constant-time comparison closure
    pub fn audit_comparison_routine<F>(name: &str, samples_per_class: usize, mut compare_fn: F) -> DudectAuditReport
    where
        F: FnMut(&[u8], &[u8]) -> bool,
    {
        let key_len = 32;
        let fixed_key = vec![0x3Au8; key_len];

        // Class A: Always identical inputs
        let mut class_a_times = Vec::with_capacity(samples_per_class);
        // Class B: Variable random inputs
        let mut class_b_times = Vec::with_capacity(samples_per_class);

        // Warmup
        for _ in 0..50 {
            let _ = compare_fn(&fixed_key, &fixed_key);
        }

        // Interleaved sampling to mitigate thermal and frequency scaling bias
        for i in 0..samples_per_class {
            // Measure Class A
            let t0 = Instant::now();
            let res_a = compare_fn(&fixed_key, &fixed_key);
            let elapsed_a = t0.elapsed().as_nanos() as f64;
            class_a_times.push(elapsed_a);

            // Measure Class B
            let mut diff_key = fixed_key.clone();
            diff_key[i % key_len] ^= (i as u8).wrapping_add(1);
            let t1 = Instant::now();
            let res_b = compare_fn(&fixed_key, &diff_key);
            let elapsed_b = t1.elapsed().as_nanos() as f64;
            class_b_times.push(elapsed_b);

            // Prevent compiler optimization elimination
            std::hint::black_box(res_a);
            std::hint::black_box(res_b);
        }

        let mean_a = Self::compute_mean(&class_a_times);
        let mean_b = Self::compute_mean(&class_b_times);

        let var_a = Self::compute_variance(&class_a_times, mean_a);
        let var_b = Self::compute_variance(&class_b_times, mean_b);

        let n = samples_per_class as f64;
        let denom = ((var_a / n) + (var_b / n)).sqrt();
        let t_stat = if denom > 0.0 {
            ((mean_a - mean_b) / denom).abs()
        } else {
            0.0
        };

        // Standard Dudect threshold: |t| < 4.5 indicates no statistically significant timing divergence
        let max_threshold = 4.5;
        let is_constant_time = t_stat < max_threshold;

        DudectAuditReport {
            routine_name: name.to_string(),
            total_samples: samples_per_class * 2,
            class_a_mean_nanos: mean_a,
            class_b_mean_nanos: mean_b,
            t_statistic: t_stat,
            max_t_threshold: max_threshold,
            is_constant_time,
        }
    }

    fn compute_mean(data: &[f64]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        let sum: f64 = data.iter().sum();
        sum / (data.len() as f64)
    }

    fn compute_variance(data: &[f64], mean: f64) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }
        let sum_sq_diff: f64 = data.iter().map(|&x| (x - mean).powi(2)).sum();
        sum_sq_diff / ((data.len() - 1) as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dudect_constant_time_verification() {
        // Test constant-time bitwise slice comparison
        let report = SideChannelAuditor::audit_comparison_routine(
            "constant_time_slice_eq",
            1_000,
            |a, b| {
                if a.len() != b.len() {
                    return false;
                }
                let mut acc = 0u8;
                for (x, y) in a.iter().zip(b.iter()) {
                    acc |= x ^ y;
                }
                acc == 0
            },
        );

        assert_eq!(report.total_samples, 2_000);
        assert!(
            report.is_constant_time,
            "T-statistic {} exceeded threshold {}",
            report.t_statistic, report.max_t_threshold
        );
    }
}


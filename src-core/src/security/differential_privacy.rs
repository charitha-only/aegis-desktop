//! Differential Privacy Local Perturbation Engine (ε, δ-DP)
//!
//! Provides mathematically rigorous privacy guarantees for exporting performance,
//! SIEM telemetry, and financial statistics without disclosing individual user or account
//! attributes. Implements Laplace mechanism, Gaussian mechanism, and an ε-budget accountant.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyBudgetStatus {
    pub max_epsilon: f64,
    pub spent_epsilon: f64,
    pub remaining_epsilon: f64,
    pub queries_processed: u64,
    pub is_exhausted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerturbedMetricReport {
    pub original_metric_name: String,
    pub perturbed_value: f64,
    pub epsilon_consumed: f64,
    pub sensitivity: f64,
    pub mechanism: String,
    pub timestamp: u64,
}

pub struct DifferentialPrivacyEngine {
    max_epsilon: f64,
    spent_epsilon: f64,
    queries_count: u64,
}

impl DifferentialPrivacyEngine {
    pub fn new(max_epsilon: f64) -> Self {
        Self {
            max_epsilon: if max_epsilon <= 0.0 { 10.0 } else { max_epsilon },
            spent_epsilon: 0.0,
            queries_count: 0,
        }
    }

    pub fn get_budget_status(&self) -> PrivacyBudgetStatus {
        PrivacyBudgetStatus {
            max_epsilon: self.max_epsilon,
            spent_epsilon: self.spent_epsilon,
            remaining_epsilon: (self.max_epsilon - self.spent_epsilon).max(0.0),
            queries_processed: self.queries_count,
            is_exhausted: self.spent_epsilon >= self.max_epsilon,
        }
    }

    /// Perturbs a numerical value using the Laplace mechanism
    pub fn perturb_laplace(
        &mut self,
        metric_name: &str,
        true_value: f64,
        sensitivity: f64,
        epsilon: f64,
        timestamp: u64,
    ) -> Result<PerturbedMetricReport, &'static str> {
        if epsilon <= 0.0 {
            return Err("Epsilon must be strictly positive");
        }
        if sensitivity <= 0.0 {
            return Err("Sensitivity must be strictly positive");
        }
        if self.spent_epsilon + epsilon > self.max_epsilon {
            return Err("Privacy budget exhausted; metric query rejected");
        }

        // Draw uniform random bytes from CSPRNG
        let mut rand_bytes = [0u8; 8];
        getrandom::getrandom(&mut rand_bytes).map_err(|_| "Entropy generation failed")?;
        let rand_u64 = u64::from_le_bytes(rand_bytes);
        // Map to (0.0, 1.0)
        let u = (rand_u64 as f64 + 1.0) / (u64::MAX as f64 + 2.0);

        // Inverse CDF of Laplace distribution: scale b = sensitivity / epsilon
        let b = sensitivity / epsilon;
        let centered = u - 0.5;
        let sign = if centered >= 0.0 { 1.0 } else { -1.0 };
        let noise = -sign * b * (1.0 - 2.0 * centered.abs()).ln();

        self.spent_epsilon += epsilon;
        self.queries_count += 1;

        Ok(PerturbedMetricReport {
            original_metric_name: metric_name.to_string(),
            perturbed_value: true_value + noise,
            epsilon_consumed: epsilon,
            sensitivity,
            mechanism: "Laplace (Pure ε-DP)".to_string(),
            timestamp,
        })
    }

    /// Perturbs a numerical value using the Gaussian mechanism for (ε, δ)-DP
    #[allow(dead_code)]
    pub fn perturb_gaussian(
        &mut self,
        metric_name: &str,
        true_value: f64,
        sensitivity: f64,
        epsilon: f64,
        delta: f64,
        timestamp: u64,
    ) -> Result<PerturbedMetricReport, &'static str> {
        if epsilon <= 0.0 || delta <= 0.0 || delta >= 1.0 {
            return Err("Invalid (epsilon, delta) parameters");
        }
        if sensitivity <= 0.0 {
            return Err("Sensitivity must be strictly positive");
        }
        if self.spent_epsilon + epsilon > self.max_epsilon {
            return Err("Privacy budget exhausted; metric query rejected");
        }

        // Sigma calculation: sigma = sqrt(2 * ln(1.25 / delta)) * (sensitivity / epsilon)
        let sigma = (2.0 * (1.25 / delta).ln()).sqrt() * (sensitivity / epsilon);

        // Box-Muller transform for standard normal variable
        let mut r1_bytes = [0u8; 8];
        let mut r2_bytes = [0u8; 8];
        getrandom::getrandom(&mut r1_bytes).map_err(|_| "Entropy generation failed")?;
        getrandom::getrandom(&mut r2_bytes).map_err(|_| "Entropy generation failed")?;
        let u1 = (u64::from_le_bytes(r1_bytes) as f64 + 1.0) / (u64::MAX as f64 + 2.0);
        let u2 = (u64::from_le_bytes(r2_bytes) as f64 + 1.0) / (u64::MAX as f64 + 2.0);

        let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        let noise = z0 * sigma;

        self.spent_epsilon += epsilon;
        self.queries_count += 1;

        Ok(PerturbedMetricReport {
            original_metric_name: metric_name.to_string(),
            perturbed_value: true_value + noise,
            epsilon_consumed: epsilon,
            sensitivity,
            mechanism: format!("Gaussian ((ε={:.2}, δ={:.1e})-DP)", epsilon, delta),
            timestamp,
        })
    }

    /// Randomized response for boolean flags (e.g. "is_flagged", "is_high_net_worth")
    #[allow(dead_code)]
    pub fn randomized_response_boolean(&mut self, true_bit: bool, epsilon: f64) -> Result<bool, &'static str> {
        if epsilon <= 0.0 {
            return Err("Epsilon must be strictly positive");
        }
        if self.spent_epsilon + epsilon > self.max_epsilon {
            return Err("Privacy budget exhausted");
        }

        let p = epsilon.exp() / (1.0 + epsilon.exp());
        let mut rand_bytes = [0u8; 8];
        getrandom::getrandom(&mut rand_bytes).map_err(|_| "Entropy failed")?;
        let u = (u64::from_le_bytes(rand_bytes) as f64) / (u64::MAX as f64);

        self.spent_epsilon += epsilon;
        self.queries_count += 1;

        if u < p {
            Ok(true_bit)
        } else {
            Ok(!true_bit)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_differential_privacy_budget_tracking() {
        let mut dp = DifferentialPrivacyEngine::new(2.0);
        let initial = dp.get_budget_status();
        assert_eq!(initial.max_epsilon, 2.0);
        assert_eq!(initial.spent_epsilon, 0.0);
        assert!(!initial.is_exhausted);

        let report1 = dp.perturb_laplace("ipc_latency_micros", 150.0, 10.0, 1.0, 1000).unwrap();
        assert_eq!(report1.epsilon_consumed, 1.0);
        assert_eq!(dp.get_budget_status().spent_epsilon, 1.0);

        let report2 = dp.perturb_laplace("ipc_latency_micros", 150.0, 10.0, 1.0, 1001).unwrap();
        assert_eq!(report2.epsilon_consumed, 1.0);
        assert!(dp.get_budget_status().is_exhausted);

        // Next query should be blocked fail-closed
        let fail = dp.perturb_laplace("ipc_latency_micros", 150.0, 10.0, 0.5, 1002);
        assert!(fail.is_err());
    }

    #[test]
    fn test_gaussian_perturbation_validity() {
        let mut dp = DifferentialPrivacyEngine::new(5.0);
        let report = dp.perturb_gaussian("cpu_core_usage", 42.0, 5.0, 0.5, 1e-5, 2000).unwrap();

        assert_eq!(report.epsilon_consumed, 0.5);
        assert!(report.mechanism.contains("Gaussian"));
        assert!(report.perturbed_value.is_finite());
    }

    #[test]
    fn test_randomized_response_boolean() {
        let mut dp = DifferentialPrivacyEngine::new(10.0);
        let result = dp.randomized_response_boolean(true, 1.0);
        assert!(result.is_ok());
    }
}

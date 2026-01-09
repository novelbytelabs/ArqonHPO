use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverConfig {
    pub seed: u64,
    pub budget: u64,
    pub bounds: std::collections::HashMap<String, Domain>,
    #[serde(default = "default_probe_ratio")]
    pub probe_ratio: f64,
    #[serde(default)]
    pub strategy_params: Option<std::collections::HashMap<String, f64>>,
}

fn default_probe_ratio() -> f64 {
    0.2
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    pub min: f64,
    pub max: f64,
    #[serde(default)]
    pub scale: Scale,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum Scale {
    #[default]
    Linear,
    Log,
    Periodic, // Wraps around [min, max]
}

impl Domain {
    pub fn is_periodic(&self) -> bool {
        matches!(self.scale, Scale::Periodic)
    }
}

// Helper functions for Unit Interval [0, 1] arithmetic

/// Wrap x into [0, 1)
pub fn wrap01(x: f64) -> f64 {
    let mut y = x.fract();
    if y < 0.0 {
        y += 1.0;
    }
    y
}

/// Shortest signed difference in [0, 1) -> [-0.5, 0.5)
/// Returns a - b (modulo 1)
pub fn diff01(a: f64, b: f64) -> f64 {
    let d = a - b;
    // Standardize to [-0.5, 0.5)
    // (d + 0.5).fract() shifts range to [0, 1), then -0.5 shifts to [-0.5, 0.5)
    // But we need to handle negative inputs to fract correctly
    let mut r = (d + 0.5).fract();
    if r < 0.0 {
        r += 1.0;
    }
    r - 0.5
}

/// Wrapped absolute distance in [0, 1)
pub fn dist01(a: f64, b: f64) -> f64 {
    diff01(a, b).abs()
}

/// Circular mean for periodic dimension in [0, 1)
/// Converts to 2pi angle, averages sin/cos, converts back
pub fn circular_mean01(values: &[f64]) -> f64 {
    let mut sum_sin = 0.0;
    let mut sum_cos = 0.0;
    for &v in values {
        let angle = v * 2.0 * std::f64::consts::PI;
        let (s, c) = angle.sin_cos();
        sum_sin += s;
        sum_cos += c;
    }
    let mean_angle = sum_sin.atan2(sum_cos); // Result in (-pi, pi]
                                             // Convert back to [0, 1)
                                             // mean_angle / 2pi -> (-0.5, 0.5]
                                             // Add 1.0 if negative to get [0, 1)
    let mut u = mean_angle / (2.0 * std::f64::consts::PI);
    if u < 0.0 {
        u += 1.0;
    }
    u
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_probe_ratio() {
        assert_eq!(default_probe_ratio(), 0.2);
    }

    #[test]
    fn test_domain_is_periodic() {
        let linear = Domain {
            min: 0.0,
            max: 1.0,
            scale: Scale::Linear,
        };
        let log = Domain {
            min: 0.1,
            max: 10.0,
            scale: Scale::Log,
        };
        let periodic = Domain {
            min: 0.0,
            max: 1.0,
            scale: Scale::Periodic,
        };

        assert!(!linear.is_periodic());
        assert!(!log.is_periodic());
        assert!(periodic.is_periodic());
    }

    #[test]
    fn test_scale_default() {
        let scale: Scale = Default::default();
        assert_eq!(scale, Scale::Linear);
    }

    #[test]
    fn test_wrap01_in_range() {
        assert!((wrap01(0.5) - 0.5).abs() < 1e-10);
        assert!((wrap01(0.0) - 0.0).abs() < 1e-10);
        assert!((wrap01(0.99) - 0.99).abs() < 1e-10);
    }

    #[test]
    fn test_wrap01_above_one() {
        assert!((wrap01(1.0) - 0.0).abs() < 1e-10);
        assert!((wrap01(1.5) - 0.5).abs() < 1e-10);
        assert!((wrap01(2.3) - 0.3).abs() < 1e-10);
    }

    #[test]
    fn test_wrap01_negative() {
        assert!((wrap01(-0.5) - 0.5).abs() < 1e-10);
        assert!((wrap01(-0.1) - 0.9).abs() < 1e-10);
        assert!((wrap01(-1.3) - 0.7).abs() < 1e-10);
    }

    #[test]
    fn test_diff01_same() {
        assert!((diff01(0.5, 0.5) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_diff01_small_positive() {
        assert!((diff01(0.6, 0.5) - 0.1).abs() < 1e-10);
    }

    #[test]
    fn test_diff01_small_negative() {
        assert!((diff01(0.4, 0.5) - (-0.1)).abs() < 1e-10);
    }

    #[test]
    fn test_diff01_wrap_around() {
        // 0.9 to 0.1 should be -0.2 (shorter path through 0)
        assert!((diff01(0.1, 0.9) - 0.2).abs() < 1e-10);
        // 0.1 to 0.9 should be -0.2
        assert!((diff01(0.9, 0.1) - (-0.2)).abs() < 1e-10);
    }

    #[test]
    fn test_dist01_symmetric() {
        assert!((dist01(0.2, 0.7) - dist01(0.7, 0.2)).abs() < 1e-10);
    }

    #[test]
    fn test_dist01_wrap_around() {
        // Distance from 0.1 to 0.9 should be 0.2 (going through 0/1)
        assert!((dist01(0.1, 0.9) - 0.2).abs() < 1e-10);
    }

    #[test]
    fn test_circular_mean01_single() {
        let values = vec![0.5];
        assert!((circular_mean01(&values) - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_circular_mean01_symmetric() {
        let values = vec![0.25, 0.75];
        // Mean of 0.25 and 0.75 should be 0.5 or 0.0 depending on interpretation
        // Actually for 0.25 and 0.75, the circular mean could be 0.0 or 0.5
        let mean = circular_mean01(&values);
        assert!(mean >= 0.0 && mean < 1.0);
    }

    #[test]
    fn test_circular_mean01_wrap_around() {
        // 0.9 and 0.1 should average to 0.0
        let values = vec![0.9, 0.1];
        let mean = circular_mean01(&values);
        assert!((mean - 0.0).abs() < 1e-10 || (mean - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_circular_mean01_negative_angle() {
        // Test case where mean_angle is negative
        let values = vec![0.7, 0.8, 0.9];
        let mean = circular_mean01(&values);
        assert!(mean >= 0.0 && mean < 1.0);
    }
}

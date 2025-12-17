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

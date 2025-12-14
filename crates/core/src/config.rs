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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum Scale {
    #[default]
    Linear,
    Log,
}


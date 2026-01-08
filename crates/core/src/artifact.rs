use serde::{Deserialize, Serialize};

use crate::config::SolverConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunArtifact {
    pub run_id: String,
    pub seed: u64,
    pub budget: u64,
    pub config: SolverConfig,
    pub history: Vec<EvalTrace>,
    // Future: classification results, environment fingerprint
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalTrace {
    pub eval_id: u64,
    pub params: std::collections::HashMap<String, f64>,
    pub value: f64,
    pub cost: f64,
}

/// A simplified input for seeding (no eval_id required from user).
/// Used for warm-starting the solver with historical evaluations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedPoint {
    pub params: std::collections::HashMap<String, f64>,
    pub value: f64,
    pub cost: f64,
}

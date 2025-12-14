use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunArtifact {
    pub run_id: String,
    pub seed: u64,
    pub budget: u64,
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

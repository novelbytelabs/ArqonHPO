use crate::config::SolverConfig;
use crate::artifact::EvalTrace;
use std::collections::HashMap;

pub mod nelder_mead;
pub mod tpe;

/// Result of a strategy step.
pub enum StrategyAction {
    Evaluate(Vec<HashMap<String, f64>>), // Propose new points
    Wait, // Async/parallel support (future)
    Converged, // Strategy decided to stop
}

pub trait Strategy: Send + Sync {
    /// Generate next candidates based on history.
    fn step(&mut self, config: &SolverConfig, history: &[EvalTrace]) -> StrategyAction;
}

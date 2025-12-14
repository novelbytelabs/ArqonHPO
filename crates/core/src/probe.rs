use crate::config::{SolverConfig, Scale};
use crate::rng::get_rng;
use rand::Rng;
use std::collections::HashMap;

/// Result of a probe generation: a list of candidate parameters.
pub type Candidates = Vec<HashMap<String, f64>>;

pub trait Probe: Send + Sync {
    fn sample(&self, config: &SolverConfig) -> Candidates;
}

/// A deterministic Uniform Random probe.
/// 
/// Replaces Sobol for MVP to minimize dependencies while maintaining strict determinism.
pub struct UniformProbe;

impl Probe for UniformProbe {
    fn sample(&self, config: &SolverConfig) -> Candidates {
        let mut rng = get_rng(config.seed);
        let num_samples = (config.budget as f64 * config.probe_ratio).ceil() as usize;
        let mut candidates = Vec::with_capacity(num_samples);

        for _ in 0..num_samples {
            let mut point = HashMap::new();
            for (name, domain) in &config.bounds {
                let val = match domain.scale {
                    Scale::Linear => rng.random_range(domain.min..=domain.max),
                    Scale::Log => {
                        // linear sample in log space
                        let min_log = domain.min.ln();
                        let max_log = domain.max.ln();
                        let s = rng.random_range(min_log..=max_log);
                        s.exp()
                    }
                };
                point.insert(name.clone(), val);
            }
            candidates.push(point);
        }
        candidates
    }
}

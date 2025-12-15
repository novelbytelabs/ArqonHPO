use crate::artifact::EvalTrace;
use crate::classify::{Classify, Landscape, ResidualDecayClassifier, VarianceClassifier};
use crate::config::SolverConfig;
use crate::probe::{Probe, PrimeIndexProbe, PrimeSqrtSlopesRotProbe, PrimeSqrtSlopesRotConfig, UniformProbe};
use crate::strategies::nelder_mead::NelderMead;
// use crate::strategies::multi_start_nm::MultiStartNM;
use crate::strategies::tpe::TPE;
use crate::strategies::{Strategy, StrategyAction};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Probe,
    Classify,
    Refine(Landscape),
    Done,
}

/// Configuration for solver seeding behavior
#[derive(Debug, Clone)]
pub struct SeedingConfig {
    /// Number of top probe points to use for seeding (default: dim + 1)
    pub top_k: Option<usize>,
    /// Whether to use probe points to seed Nelder-Mead simplex
    pub seed_nm: bool,
}

impl Default for SeedingConfig {
    fn default() -> Self {
        Self {
            top_k: None, // Will default to dim + 1
            seed_nm: true,
        }
    }
}

pub struct Solver {
    pub config: SolverConfig,
    pub history: Vec<EvalTrace>,
    pub phase: Phase,
    pub probe: Box<dyn Probe>,
    pub classifier: Box<dyn Classify>,
    pub strategy: Option<Box<dyn Strategy>>,
    pub seeding: SeedingConfig,
}

impl Solver {
    /// Create a new solver with MVP defaults (UniformProbe, VarianceClassifier)
    pub fn new(config: SolverConfig) -> Self {
        Self {
            config,
            history: Vec::new(),
            phase: Phase::Probe,
            probe: Box::new(UniformProbe),
            classifier: Box::new(VarianceClassifier::default()),
            strategy: None,
            seeding: SeedingConfig::default(),
        }
    }

    /// Create a solver with a custom classifier
    pub fn with_classifier(config: SolverConfig, classifier: Box<dyn Classify>) -> Self {
        Self {
            config,
            history: Vec::new(),
            phase: Phase::Probe,
            probe: Box::new(UniformProbe),
            classifier,
            strategy: None,
            seeding: SeedingConfig::default(),
        }
    }

    /// Create a solver with the ResidualDecayClassifier (used in PCR)
    pub fn with_residual_decay(config: SolverConfig) -> Self {
        Self::with_classifier(config, Box::new(ResidualDecayClassifier::default()))
    }

    /// Creates a Solver with the PCR (Probe-Classify-Refine) strategy.
    ///
    /// This runs the complete ArqonHPO V2 algorithm:
    /// 1. **Probe**: Use `PrimeSqrtSlopesRotProbe` for low-discrepancy sampling with random spice.
    /// 2. **Classify**: Use `ResidualDecayClassifier` to detect structure (α > 0.5) vs chaos.
    /// 3. **Refine**: Use `Top-K` seeding to initialize the chosen strategy.
    ///    - Structured -> Nelder-Mead (initialized with best probe points)
    ///    - Chaotic -> TPE (initialized with all probe points)
    pub fn pcr(config: SolverConfig) -> Self {
        Self {
            config,
            history: Vec::new(),
            phase: Phase::Probe,
            probe: Box::new(PrimeSqrtSlopesRotProbe::default()),
            classifier: Box::new(ResidualDecayClassifier::default()),
            strategy: None,
            seeding: SeedingConfig {
                top_k: None,
                seed_nm: true,
            },
        }
    }

    /// Get top-k best probe points for seeding
    fn get_top_k_seed_points(&self, k: usize) -> Vec<HashMap<String, f64>> {
        let mut sorted: Vec<_> = self.history.iter().collect();
        sorted.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap_or(std::cmp::Ordering::Equal));
        
        sorted.iter()
            .take(k)
            .map(|t| t.params.clone())
            .collect()
    }

    /// Ask the solver what to do next.
    /// Returns a list of candidates to evaluate, or None if finished.
    #[tracing::instrument(skip(self))]
    pub fn ask(&mut self) -> Option<Vec<HashMap<String, f64>>> {
        loop {
            match self.phase {
                Phase::Probe => {
                    let probe_budget =
                        (self.config.budget as f64 * self.config.probe_ratio).ceil() as usize;
                    let current_count = self.history.len();

                    if current_count < probe_budget {
                        if current_count == 0 {
                            let candidates = self.probe.sample(&self.config);
                            return Some(candidates);
                        } else {
                            if self.history.len() >= probe_budget {
                                self.phase = Phase::Classify;
                                continue;
                            } else {
                                return None;
                            }
                        }
                    } else {
                        self.phase = Phase::Classify;
                    }
                }
                Phase::Classify => {
                    let (mode, _score) = self.classifier.classify(&self.history);
                    self.phase = Phase::Refine(mode);

                    // Factory Strategy with probe seeding
                    let dim = self.config.bounds.len();
                    match mode {
                        Landscape::Structured => {
                            // Update probe with low spice
                            let spice = PrimeSqrtSlopesRotConfig::adaptive_spice_for_landscape(false);
                            let p_config = PrimeSqrtSlopesRotConfig::with_spice(spice);
                            self.probe = Box::new(PrimeSqrtSlopesRotProbe::with_seed_and_config(self.config.seed, p_config));
                            
                            // Revert: Multi-Start NM caused starvation issues.
                            // Falling back to robust Single-Start NM.
                            let k = self.seeding.top_k.unwrap_or(dim + 1);
                            let seeds = self.get_top_k_seed_points(k);
                            
                            self.strategy = Some(Box::new(
                                NelderMead::with_seed_points(dim, seeds)
                            ));
                        }
                        Landscape::Chaotic => {
                            // Update probe with high spice
                            let spice = PrimeSqrtSlopesRotConfig::adaptive_spice_for_landscape(true);
                            let p_config = PrimeSqrtSlopesRotConfig::with_spice(spice);
                            self.probe = Box::new(PrimeSqrtSlopesRotProbe::with_seed_and_config(self.config.seed, p_config));
                            
                            // TPE uses Scott's Rule by default
                            self.strategy = Some(Box::new(TPE::new(dim)));
                        }
                    }
                    continue;
                }
                Phase::Refine(mode) => {
                    if let Some(strat) = &mut self.strategy {
                        if self.history.len() >= self.config.budget as usize {
                            self.phase = Phase::Done;
                            continue;
                        }
                        match strat.step(&self.config, &self.history) {
                            StrategyAction::Evaluate(points) => return Some(points),
                            StrategyAction::Wait => return None,
                            StrategyAction::Converged => {
                                self.phase = Phase::Done;
                                continue;
                            }
                        }
                    } else {
                        eprintln!("Strategy not wired for mode: {:?}", mode);
                        self.phase = Phase::Done;
                        return None;
                    }
                }
                Phase::Done => return None,
            }
        }
    }

    #[tracing::instrument(skip(self, eval_results))]
    pub fn tell(&mut self, eval_results: Vec<EvalTrace>) {
        self.history.extend(eval_results);
    }
}


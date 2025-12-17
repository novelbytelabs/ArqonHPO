use crate::artifact::EvalTrace;
use crate::classify::{Classify, Landscape, ResidualDecayClassifier, VarianceClassifier};
use crate::config::SolverConfig;
use crate::probe::{Probe, PrimeSqrtSlopesRotProbe, PrimeSqrtSlopesRotConfig, UniformProbe};
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
    /// Has the solver performed a CP restart?
    pub restarted: bool,
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
            restarted: false,
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
            restarted: false,
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
            classifier: Box::new(VarianceClassifier::default()),
            strategy: None,
            seeding: SeedingConfig {
                top_k: None,
                seed_nm: true,
            },
            restarted: false,
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
                    println!("[Machine] Classified as {:?} (Score: {:.4})", mode, _score);
                    self.phase = Phase::Refine(mode);

                    // Factory Strategy with probe seeding
                    let dim = self.config.bounds.len();
                    match mode {
                        Landscape::Structured => {
                            // Update probe with low spice
                            // Primary: No CP shift (None) -> 0% spice + pure QMC
                            let spice = PrimeSqrtSlopesRotConfig::adaptive_spice_for_landscape(false);
                            let p_config = PrimeSqrtSlopesRotConfig::with_spice(spice); // cp_shift is None (Δ=0)
                            self.probe = Box::new(PrimeSqrtSlopesRotProbe::with_seed_and_config(self.config.seed, p_config));
                            
                            // Revert: Multi-Start NM caused starvation issues.
                            // Falling back to robust Single-Start NM.
                            // Compute periodic mask for Nelder-Mead (must match sorted key order)
                            let mut keys: Vec<_> = self.config.bounds.keys().collect();
                            keys.sort();
                            let periodic_mask: Vec<bool> = keys.iter()
                                .map(|k| self.config.bounds.get(*k).map(|d| d.is_periodic()).unwrap_or(false))
                                .collect();

                            self.strategy = Some(Box::new(
                                NelderMead::new(dim, periodic_mask)
                            ));
                        }
                        Landscape::Chaotic => {
                            // Update probe with high spice
                            // Chaotic: CP shift always on
                            println!("[Machine] Chaotic mode -> Enabling CP Shift + Spice");
                            let spice = PrimeSqrtSlopesRotConfig::adaptive_spice_for_landscape(true);
                            
                            // Deterministic random CP shift for Chaotic
                            // Use seed_rotation logic from probe: seed * 1e9 + 0xDEAD_C0DE
                            let cp_seed = ((self.config.seed as f64 * 1e9) as u64).wrapping_add(0xDEAD_C0DE);
                            use rand::SeedableRng;
                            use rand::Rng;
                            let mut cp_rng = rand_chacha::ChaCha8Rng::seed_from_u64(cp_seed);
                            let cp_delta: Vec<f64> = (0..dim).map(|_| cp_rng.random()).collect();
                            
                            let p_config = PrimeSqrtSlopesRotConfig::with_spice(spice).with_cp_shift(cp_delta);
                            self.probe = Box::new(PrimeSqrtSlopesRotProbe::with_seed_and_config(self.config.seed, p_config));
                            
                            // TPE uses Scott's Rule by default
                            self.strategy = Some(Box::new(TPE::new(dim)));
                        }
                    }
                    continue;
                }
                Phase::Refine(mode) => {
                    // Check logic for Structured Fallback (CP Restart)
                    if let Landscape::Structured = mode {
                         if !self.restarted && self.history.len() >= (self.config.budget as f64 * 0.7) as usize {
                             // Trigger CP Restart!
                             println!("[Machine] Structured Fail-Safe Triggered! Restarting with CP Shift at param count {}", self.history.len());
                             self.restarted = true;
                             let dim = self.config.bounds.len();
                             
                             // Generate CP shift
                             let cp_seed = ((self.config.seed as f64 * 1.5e9) as u64).wrapping_add(0xBEEF_CAFE);
                             use rand::SeedableRng;
                             use rand::Rng;
                             let mut cp_rng = rand_chacha::ChaCha8Rng::seed_from_u64(cp_seed);
                             let cp_delta: Vec<f64> = (0..dim).map(|_| cp_rng.random()).collect();
                             
                             // Re-init probe with shift
                             let spice = PrimeSqrtSlopesRotConfig::adaptive_spice_for_landscape(true); // Maybe use chaotic spice (or just higher)? User said "CP restart"
                             let p_config = PrimeSqrtSlopesRotConfig::with_spice(spice).with_cp_shift(cp_delta);
                             self.probe = Box::new(PrimeSqrtSlopesRotProbe::with_seed_and_config(self.config.seed + 1, p_config)); // Seed+1 to get fresh points
                             
                             // Request new batch? Actually, we just need seeds.
                             // We can sample ~10 points from this new probe
                             let new_candidates = self.probe.sample(&self.config);
                             let rescue_batch = new_candidates.into_iter().take(15).collect::<Vec<_>>();
                             
                             // We must evaluate them first? 
                             // Wait, if we return them, the loop continues.
                             // But we need to RESTART the strategy AFTER evaluating.
                             // We can tell the strategy to wait? Or just return the points and set a flag "waiting for rescue batch"?
                             // Simplifying: Just evaluating them puts them in history.
                             // BUT NelderMead Top-K picks from history.
                             // So we just output them. AND we Reset Strategy.
                             
                             // Reset Strategy to New Nelder Mead
                             // But NM creates its own simplex. It needs the *data* from the rescue batch.
                             // We haven't evaluated rescue batch yet.
                             // So we output them. And we expect `tell` to happen.
                             // But `ask` is called again.
                             // So we need to detect "We just output rescue batch, now we need to re-init strategy".
                             // Simpler: Just Evaluate them. The strategy won't see them yet.
                             // When `ask` is called NEXT time, history has them.
                             // How to coordinate?
                             // We return Some(rescue_batch).
                             // We set `self.strategy = None` momentarily to force re-init next call?
                             // No, next call will enter `if let Some(strat)`.
                             // We should set a special flag or just re-init strategy NOW using *existing* history?
                             // No, existing history doesn't have rescue batch.
                             // So we return evaluate.
                             // Next time `ask` is called, we re-init strategy if `restarted` flag implies we just did it?
                             // Or add `Phase::Restartic`?
                             // Let's use `self.strategy = None` to signal "Need Re-init".
                             // But `ask` handles `None` by printing error.
                             // Let's replace strategy with a dummy or set Phase to a temp phase?
                             // User wants minimal diff.
                             
                             // Hack: Return the points.
                             // Set `self.restarted = true`. 
                             // We need to know when they are back.
                             // The Solver loop is simple. `ask` -> `tell` -> `ask`.
                             // So next `ask()` will see new history.
                             // BUT `NelderMead` maintains internal state. It ignores history after Init.
                             // So we MUST replace `self.strategy`.
                             // If we replace it NOW, it will try to Init from history *without* rescue batch.
                             // We need to replace it *after* rescue batch is evaluated.
                             // Since we can't track "batch done" easily without state...
                             // Maybe we just Re-init strategy NOW, but `NelderMead::Init` takes points from history.
                             // If we return points now, they aren't in history yet.
                             
                             // Alternative: Restart uses existing history? No, user wants CP shift points.
                             
                             // Correct flow:
                             // 1. Return rescue batch.
                             // 2. Set `self.strategy = None` (or a placeholder).
                             // 3. Next `ask()`: If strategy is None, Re-init NelderMead (Config D mode) and return its request.
                             
                             // Let's implement logic:
                             // If strategy is None in Refine: Re-create it (CP-aware picking).
                             self.strategy = None; 
                             return Some(rescue_batch);
                         }
                    }

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
                        // Strategy is None. This happens after CP Restart trigger returns points.
                        // Re-initialize Nelder Mead with Config D settings (Top K from Full History, which now includes CP points)
                        // Note: The history now has the CP points we just asked for (after user evaluated them).
                        // So Top-K will pick the best (which likely are the new CP points if valid).
                         let dim = self.config.bounds.len();
                         let k = self.seeding.top_k.unwrap_or(dim + 1);
                         
                         // Note: We don't filter history. We just let Top-K pick from everything.
                         // But we want to ensure we use CP logic?
                         // NelderMead::with_seed_points just takes seeds.
                         let _seeds = self.get_top_k_seed_points(k);
                         
                         // Compute periodic mask
                         let mut keys: Vec<_> = self.config.bounds.keys().collect();
                         keys.sort();
                         let periodic_mask: Vec<bool> = keys.iter()
                             .map(|k| self.config.bounds.get(*k).map(|d| d.is_periodic()).unwrap_or(false))
                             .collect();
                             
                         self.strategy = Some(Box::new(NelderMead::new(dim, periodic_mask)));
                         
                         // Immediately step the new strategy
                         continue; // Loop again to step
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


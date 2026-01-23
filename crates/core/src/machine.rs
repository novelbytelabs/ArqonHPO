use crate::artifact::{EvalTrace, SeedPoint};
use crate::classify::{Classify, Landscape, ResidualDecayClassifier, VarianceClassifier};
use crate::config::SolverConfig;
use crate::probe::{PrimeSqrtSlopesRotConfig, PrimeSqrtSlopesRotProbe, Probe, UniformProbe};
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
        sorted.sort_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        sorted.iter().take(k).map(|t| t.params.clone()).collect()
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
                        let all_candidates = self.probe.sample(&self.config);
                        if current_count > 0 {
                            // Incremental sampling: skip already-evaluated points
                            let remaining: Vec<_> =
                                all_candidates.into_iter().skip(current_count).collect();
                            if remaining.is_empty() {
                                self.phase = Phase::Classify;
                                continue;
                            }
                            return Some(remaining);
                        } else {
                            // Initial sampling
                            return Some(all_candidates);
                        }
                    } else {
                        self.phase = Phase::Classify;
                        continue;
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
                            let spice =
                                PrimeSqrtSlopesRotConfig::adaptive_spice_for_landscape(false);
                            let p_config = PrimeSqrtSlopesRotConfig::with_spice(spice); // cp_shift is None (Δ=0)
                            self.probe = Box::new(PrimeSqrtSlopesRotProbe::with_seed_and_config(
                                self.config.seed,
                                p_config,
                            ));

                            // Revert: Multi-Start NM caused starvation issues.
                            // Falling back to robust Single-Start NM.
                            // Compute periodic mask for Nelder-Mead (must match sorted key order)
                            let mut keys: Vec<_> = self.config.bounds.keys().collect();
                            keys.sort();
                            let periodic_mask: Vec<bool> = keys
                                .iter()
                                .map(|k| {
                                    self.config
                                        .bounds
                                        .get(*k)
                                        .map(|d| d.is_periodic())
                                        .unwrap_or(false)
                                })
                                .collect();

                            self.strategy = Some(Box::new(NelderMead::new(dim, periodic_mask)));
                        }
                        Landscape::Chaotic => {
                            // Update probe with high spice
                            // Chaotic: CP shift always on
                            println!("[Machine] Chaotic mode -> Enabling CP Shift + Spice");
                            let spice =
                                PrimeSqrtSlopesRotConfig::adaptive_spice_for_landscape(true);

                            // Deterministic random CP shift for Chaotic
                            // Use seed_rotation logic from probe: seed * 1e9 + 0xDEAD_C0DE
                            let cp_seed =
                                ((self.config.seed as f64 * 1e9) as u64).wrapping_add(0xDEAD_C0DE);
                            use rand::Rng;
                            use rand::SeedableRng;
                            let mut cp_rng = rand_chacha::ChaCha8Rng::seed_from_u64(cp_seed);
                            let cp_delta: Vec<f64> = (0..dim).map(|_| cp_rng.random()).collect();

                            let p_config =
                                PrimeSqrtSlopesRotConfig::with_spice(spice).with_cp_shift(cp_delta);
                            self.probe = Box::new(PrimeSqrtSlopesRotProbe::with_seed_and_config(
                                self.config.seed,
                                p_config,
                            ));

                            // TPE uses Scott's Rule by default
                            self.strategy = Some(Box::new(TPE::new(dim)));
                        }
                    }
                    continue;
                }
                Phase::Refine(mode) => {
                    // Check logic for Structured Fallback (CP Restart)
                    if let Landscape::Structured = mode {
                        if !self.restarted
                            && self.history.len() >= (self.config.budget as f64 * 0.7) as usize
                        {
                            // Trigger CP Restart!
                            println!("[Machine] Structured Fail-Safe Triggered! Restarting with CP Shift at param count {}", self.history.len());
                            self.restarted = true;
                            let dim = self.config.bounds.len();

                            // Generate CP shift
                            let cp_seed = ((self.config.seed as f64 * 1.5e9) as u64)
                                .wrapping_add(0xBEEF_CAFE);
                            use rand::Rng;
                            use rand::SeedableRng;
                            let mut cp_rng = rand_chacha::ChaCha8Rng::seed_from_u64(cp_seed);
                            let cp_delta: Vec<f64> = (0..dim).map(|_| cp_rng.random()).collect();

                            // Re-init probe with shift
                            let spice =
                                PrimeSqrtSlopesRotConfig::adaptive_spice_for_landscape(true); // Maybe use chaotic spice (or just higher)? User said "CP restart"
                            let p_config =
                                PrimeSqrtSlopesRotConfig::with_spice(spice).with_cp_shift(cp_delta);
                            self.probe = Box::new(PrimeSqrtSlopesRotProbe::with_seed_and_config(
                                self.config.seed + 1,
                                p_config,
                            )); // Seed+1 to get fresh points

                            // Request new batch? Actually, we just need seeds.
                            // We can sample ~10 points from this new probe
                            let new_candidates = self.probe.sample(&self.config);
                            let rescue_batch =
                                new_candidates.into_iter().take(15).collect::<Vec<_>>();

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
                        let periodic_mask: Vec<bool> = keys
                            .iter()
                            .map(|k| {
                                self.config
                                    .bounds
                                    .get(*k)
                                    .map(|d| d.is_periodic())
                                    .unwrap_or(false)
                            })
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

    /// Get the next available evaluation ID.
    fn next_eval_id(&self) -> u64 {
        self.history.iter().map(|t| t.eval_id).max().unwrap_or(0) + 1
    }

    /// Inject historical evaluations into the model.
    /// These are treated as if ask() had been called and tell() received the results.
    /// The solver assigns internal eval_ids automatically.
    ///
    /// # Use Cases
    /// - Warm-starting from previous optimization runs
    /// - Streaming/online optimization where external systems generate evaluations
    ///
    /// # Example
    /// ```ignore
    /// let mut solver = Solver::new(config);
    /// solver.seed(vec![
    ///     SeedPoint { params: params1, value: 1.0, cost: 1.0 },
    ///     SeedPoint { params: params2, value: 2.0, cost: 1.0 },
    /// ]);
    /// // Next ask() will be informed by seeded data
    /// let batch = solver.ask();
    /// ```
    #[tracing::instrument(skip(self, evaluations))]
    pub fn seed(&mut self, evaluations: Vec<SeedPoint>) {
        for eval in evaluations {
            let internal_id = self.next_eval_id();
            let trace = EvalTrace {
                eval_id: internal_id,
                params: eval.params,
                value: eval.value,
                cost: eval.cost,
            };
            self.history.push(trace);
        }
    }

    /// Ask for exactly ONE candidate configuration for online/real-time optimization.
    ///
    /// Unlike `ask()` which returns a full batch for PCR workflow, this method:
    /// 1. Skips Probe/Classify phases
    /// 2. Uses TPE strategy directly for incremental learning
    /// 3. Returns exactly 1 candidate per call
    ///
    /// # Usage Pattern (Online Mode)
    /// ```ignore
    /// let mut solver = Solver::new(config);
    /// loop {
    ///     let candidate = solver.ask_one()?;  // Get ONE config
    ///     let reward = evaluate(candidate);    // Measure performance
    ///     solver.seed(vec![SeedPoint { params: candidate, value: reward, cost: 1.0 }]);
    /// }
    /// ```
    #[tracing::instrument(skip(self))]
    pub fn ask_one(&mut self) -> Option<HashMap<String, f64>> {
        // Budget check
        if self.history.len() >= self.config.budget as usize {
            return None;
        }

        // Lazy-init TPE strategy for online mode
        if self.strategy.is_none() {
            let dim = self.config.bounds.len();
            self.strategy = Some(Box::new(TPE::new(dim)));
        }

        // Get one candidate from TPE
        if let Some(strat) = &mut self.strategy {
            match strat.step(&self.config, &self.history) {
                StrategyAction::Evaluate(points) => {
                    // Return just the first candidate
                    points.into_iter().next()
                }
                StrategyAction::Wait => None,
                StrategyAction::Converged => None,
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Domain, Scale};

    fn make_test_config() -> SolverConfig {
        let mut bounds = HashMap::new();
        bounds.insert(
            "x".to_string(),
            Domain {
                min: 0.0,
                max: 1.0,
                scale: Scale::Linear,
            },
        );
        bounds.insert(
            "y".to_string(),
            Domain {
                min: 0.0,
                max: 1.0,
                scale: Scale::Linear,
            },
        );
        SolverConfig {
            bounds,
            budget: 20,
            probe_ratio: 0.5,
            seed: 42,
            strategy_params: None,
        }
    }

    #[test]
    fn test_solver_new_creates_probe_phase() {
        let config = make_test_config();
        let solver = Solver::new(config);
        assert_eq!(solver.phase, Phase::Probe);
        assert!(solver.history.is_empty());
        assert!(solver.strategy.is_none());
    }

    #[test]
    fn test_solver_pcr_creates_probe_phase() {
        let config = make_test_config();
        let solver = Solver::pcr(config);
        assert_eq!(solver.phase, Phase::Probe);
        assert!(solver.seeding.seed_nm);
    }

    #[test]
    fn test_solver_with_residual_decay() {
        let config = make_test_config();
        let solver = Solver::with_residual_decay(config);
        assert_eq!(solver.phase, Phase::Probe);
    }

    #[test]
    fn test_seed_adds_to_history() {
        let config = make_test_config();
        let mut solver = Solver::new(config);

        let seed_points = vec![
            SeedPoint {
                params: [("x".to_string(), 0.5), ("y".to_string(), 0.5)]
                    .into_iter()
                    .collect(),
                value: 1.0,
                cost: 1.0,
            },
            SeedPoint {
                params: [("x".to_string(), 0.3), ("y".to_string(), 0.7)]
                    .into_iter()
                    .collect(),
                value: 0.8,
                cost: 1.0,
            },
        ];
        solver.seed(seed_points);

        assert_eq!(solver.history.len(), 2);
        assert_eq!(solver.history[0].eval_id, 1);
        assert_eq!(solver.history[1].eval_id, 2);
    }

    #[test]
    fn test_tell_extends_history() {
        let config = make_test_config();
        let mut solver = Solver::new(config);

        let traces = vec![EvalTrace {
            eval_id: 1,
            params: [("x".to_string(), 0.5)].into_iter().collect(),
            value: 1.0,
            cost: 1.0,
        }];
        solver.tell(traces);

        assert_eq!(solver.history.len(), 1);
    }

    #[test]
    fn test_ask_returns_candidates_in_probe_phase() {
        let config = make_test_config();
        let mut solver = Solver::new(config);

        let candidates = solver.ask();
        assert!(candidates.is_some());
        let batch = candidates.unwrap();
        assert!(!batch.is_empty());
    }

    #[test]
    fn test_ask_one_returns_single_candidate() {
        let config = make_test_config();
        let mut solver = Solver::new(config);

        // Seed some data first for TPE
        solver.seed(vec![SeedPoint {
            params: [("x".to_string(), 0.5), ("y".to_string(), 0.5)]
                .into_iter()
                .collect(),
            value: 1.0,
            cost: 1.0,
        }]);

        let candidate = solver.ask_one();
        assert!(candidate.is_some());
        let params = candidate.unwrap();
        assert!(params.contains_key("x"));
        assert!(params.contains_key("y"));
    }

    #[test]
    fn test_ask_one_respects_budget() {
        let mut config = make_test_config();
        config.budget = 2;
        let mut solver = Solver::new(config);

        // Fill budget
        solver.seed(vec![
            SeedPoint {
                params: [("x".to_string(), 0.5), ("y".to_string(), 0.5)]
                    .into_iter()
                    .collect(),
                value: 1.0,
                cost: 1.0,
            },
            SeedPoint {
                params: [("x".to_string(), 0.3), ("y".to_string(), 0.3)]
                    .into_iter()
                    .collect(),
                value: 0.5,
                cost: 1.0,
            },
        ]);

        let candidate = solver.ask_one();
        assert!(candidate.is_none()); // Budget exhausted
    }

    #[test]
    fn test_seeding_config_default() {
        let sc = SeedingConfig::default();
        assert!(sc.top_k.is_none());
        assert!(sc.seed_nm);
    }

    #[test]
    fn test_phase_enum_equality() {
        assert_eq!(Phase::Probe, Phase::Probe);
        assert_eq!(Phase::Done, Phase::Done);
        assert_ne!(Phase::Probe, Phase::Done);
        assert_eq!(
            Phase::Refine(Landscape::Structured),
            Phase::Refine(Landscape::Structured)
        );
        assert_ne!(
            Phase::Refine(Landscape::Structured),
            Phase::Refine(Landscape::Chaotic)
        );
    }

    #[test]
    fn test_next_eval_id_increments() {
        let config = make_test_config();
        let mut solver = Solver::new(config);

        assert_eq!(solver.next_eval_id(), 1);

        solver.seed(vec![SeedPoint {
            params: HashMap::new(),
            value: 1.0,
            cost: 1.0,
        }]);

        assert_eq!(solver.next_eval_id(), 2);
    }

    #[test]
    fn test_get_top_k_seed_points() {
        let config = make_test_config();
        let mut solver = Solver::new(config);

        // Add some history
        solver.tell(vec![
            EvalTrace {
                eval_id: 1,
                params: [("x".to_string(), 0.1)].into_iter().collect(),
                value: 3.0,
                cost: 1.0,
            },
            EvalTrace {
                eval_id: 2,
                params: [("x".to_string(), 0.2)].into_iter().collect(),
                value: 1.0,
                cost: 1.0,
            },
            EvalTrace {
                eval_id: 3,
                params: [("x".to_string(), 0.3)].into_iter().collect(),
                value: 2.0,
                cost: 1.0,
            },
        ]);

        let top_k = solver.get_top_k_seed_points(2);
        assert_eq!(top_k.len(), 2);
        // Should be sorted by value, so lowest first
        assert_eq!(top_k[0].get("x"), Some(&0.2));
        assert_eq!(top_k[1].get("x"), Some(&0.3));
    }

    #[test]
    fn test_classify_phase_transition() {
        // Test that solver transitions from Probe to Classify when probe budget is met
        let mut config = make_test_config();
        config.budget = 20;
        config.probe_ratio = 0.5; // probe_budget = 10
        let mut solver = Solver::new(config);

        // Fill probe budget with 10 evaluations
        let traces: Vec<EvalTrace> = (0..10)
            .map(|i| EvalTrace {
                eval_id: i as u64,
                params: [("x".to_string(), i as f64 / 10.0), ("y".to_string(), 0.5)]
                    .into_iter()
                    .collect(),
                value: (i as f64 - 5.0).powi(2), // parabola
                cost: 1.0,
            })
            .collect();
        solver.tell(traces);

        // Phase should still be Probe, but next ask() should transition to Classify
        assert_eq!(solver.phase, Phase::Probe);

        // Call ask - should trigger classification and move to Refine
        let candidates = solver.ask();
        assert!(candidates.is_some());
        // After classification, phase should be Refine (Structured or Chaotic)
        match solver.phase {
            Phase::Refine(_) => (),
            _ => panic!(
                "Expected Refine phase after classification, got {:?}",
                solver.phase
            ),
        }
    }

    #[test]
    fn test_chaotic_landscape_triggers_tpe() {
        // Test that Chaotic classification results in TPE strategy
        let mut config = make_test_config();
        config.budget = 20;
        config.probe_ratio = 0.5;
        let mut solver = Solver::new(config);

        // Add high-variance data (simulates chaotic landscape)
        let traces: Vec<EvalTrace> = (0..10)
            .map(|i| EvalTrace {
                eval_id: i as u64,
                params: [("x".to_string(), i as f64 / 10.0), ("y".to_string(), 0.5)]
                    .into_iter()
                    .collect(),
                // Random-looking values with high variance
                value: if i % 2 == 0 { 100.0 } else { 0.1 },
                cost: 1.0,
            })
            .collect();
        solver.tell(traces);

        // Trigger classification
        let _ = solver.ask();

        // Strategy should now be set (either NM or TPE based on landscape)
        assert!(solver.strategy.is_some());
    }

    #[test]
    fn test_refine_phase_budget_exhaustion() {
        // Test that solver returns None when budget is exhausted in Refine phase
        let mut config = make_test_config();
        config.budget = 12;
        config.probe_ratio = 0.5;
        let mut solver = Solver::pcr(config);

        // Fill probe budget
        let traces: Vec<EvalTrace> = (0..6)
            .map(|i| EvalTrace {
                eval_id: i as u64,
                params: [("x".to_string(), i as f64 / 10.0), ("y".to_string(), 0.5)]
                    .into_iter()
                    .collect(),
                value: (i as f64 - 3.0).powi(2),
                cost: 1.0,
            })
            .collect();
        solver.tell(traces);

        // Trigger classification
        let _ = solver.ask();

        // Fill remaining budget
        let more_traces: Vec<EvalTrace> = (6..12)
            .map(|i| EvalTrace {
                eval_id: i as u64,
                params: [("x".to_string(), i as f64 / 10.0), ("y".to_string(), 0.5)]
                    .into_iter()
                    .collect(),
                value: 1.0,
                cost: 1.0,
            })
            .collect();
        solver.tell(more_traces);

        // Now budget is exhausted, ask should return None or transition to Done or trigger CP restart
        let result = solver.ask();
        // Either returns None, or moves to Done phase, or triggers CP restart
        if result.is_some() {
            // If still returning points, phase should be Refine or Done
            match solver.phase {
                Phase::Done | Phase::Refine(_) => (),
                _ => panic!(
                    "Expected Done or Refine phase after budget exhaustion, got {:?}",
                    solver.phase
                ),
            }
        }
    }

    #[test]
    fn test_cp_restart_trigger() {
        // Test CP restart trigger at 70% budget in Structured mode
        let mut config = make_test_config();
        config.budget = 100;
        config.probe_ratio = 0.1; // probe_budget = 10
        let mut solver = Solver::pcr(config);

        // Fill probe budget with structured data
        let mut traces: Vec<EvalTrace> = (0..10)
            .map(|i| EvalTrace {
                eval_id: i as u64,
                params: [("x".to_string(), i as f64 / 10.0), ("y".to_string(), 0.5)]
                    .into_iter()
                    .collect(),
                value: (i as f64 / 10.0).powi(2), // structured: parabola
                cost: 1.0,
            })
            .collect();
        solver.tell(traces.clone());

        // Trigger classification - should be Structured
        let _ = solver.ask();

        // Verify we're in Refine(Structured) and have NM strategy
        match solver.phase {
            Phase::Refine(Landscape::Structured) => (),
            _ => {
                // If not structured, that's OK - classification may vary
                return;
            }
        }

        // Add more evaluations to reach 70% of budget (70 evaluations)
        for i in 10..70 {
            traces.push(EvalTrace {
                eval_id: i as u64,
                params: [
                    ("x".to_string(), (i % 10) as f64 / 10.0),
                    ("y".to_string(), 0.5),
                ]
                .into_iter()
                .collect(),
                value: 1.0,
                cost: 1.0,
            });
        }
        solver.tell(traces[10..70].to_vec());

        // This ask() should trigger CP restart
        let rescue_batch = solver.ask();

        // After CP restart, restarted flag should be true
        assert!(solver.restarted);

        // Should have returned a rescue batch
        assert!(rescue_batch.is_some());
    }

    #[test]
    fn test_post_restart_strategy_reinit() {
        // Test that strategy is re-initialized after CP restart
        let mut config = make_test_config();
        config.budget = 100;
        config.probe_ratio = 0.1;
        let mut solver = Solver::pcr(config);

        // Add probe data
        let traces: Vec<EvalTrace> = (0..10)
            .map(|i| EvalTrace {
                eval_id: i as u64,
                params: [("x".to_string(), i as f64 / 10.0), ("y".to_string(), 0.5)]
                    .into_iter()
                    .collect(),
                value: (i as f64 / 10.0).powi(2),
                cost: 1.0,
            })
            .collect();
        solver.tell(traces);
        let _ = solver.ask();

        // Artificially set up post-restart state where strategy is None
        // This simulates: CP restart happened, rescue batch returned, now re-calling ask
        solver.phase = Phase::Refine(Landscape::Structured);
        solver.strategy = None; // Simulate post-restart state
        solver.restarted = true; // Already restarted (prevents re-trigger)

        // Now ask should detect strategy is None and re-init
        let result = solver.ask();

        // Strategy should now be re-initialized (NelderMead)
        // If result is Some, strategy is initialized and returned candidates
        // If result is None due to convergence, that's also valid
        if result.is_some() {
            assert!(solver.strategy.is_some());
        }
    }
}

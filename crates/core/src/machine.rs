use crate::config::SolverConfig;
use crate::artifact::EvalTrace;
use crate::probe::{Probe, UniformProbe};
use crate::classify::{Classify, VarianceClassifier, Landscape};
use crate::strategies::{Strategy, StrategyAction};
use crate::strategies::nelder_mead::NelderMead;
use crate::strategies::tpe::TPE;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Probe,
    Classify,
    Refine(Landscape), 
    Done,
}

pub struct Solver {
    pub config: SolverConfig,
    pub history: Vec<EvalTrace>,
    pub phase: Phase,
    pub probe: Box<dyn Probe>,
    pub classifier: Box<dyn Classify>,
    pub strategy: Option<Box<dyn Strategy>>, // Only exists in Refine phase
}

impl Solver {
    pub fn new(config: SolverConfig) -> Self {
        Self {
            config,
            history: Vec::new(),
            phase: Phase::Probe,
            probe: Box::new(UniformProbe),
            classifier: Box::new(VarianceClassifier::default()),
            strategy: None,
        }
    }

    /// Ask the solver what to do next.
    /// Returns a list of candidates to evaluate, or None if finished.
    pub fn ask(&mut self) -> Option<Vec<HashMap<String, f64>>> {
        loop {
            match self.phase {
                Phase::Probe => {
                    let probe_budget = (self.config.budget as f64 * self.config.probe_ratio).ceil() as usize;
                    let current_count = self.history.len();
                    
                    if current_count < probe_budget {
                        // In MVP, UniformProbe generates *all* samples at once. 
                        // But we want to support iterative asking.
                        // UniformProbe implementation generated full set.
                        // Let's modify Probe trait or usages if we want chunked.
                        // For now, let's just generate candidates if we have none yet?
                        // Actually, if we just started, generate all probe samples.
                        if current_count == 0 {
                             let candidates = self.probe.sample(&self.config);
                             return Some(candidates);
                        } else {
                            // If we already yielded samples, we wait for them to be reported via tell()
                            // If we are here, it means we are waiting for results or done.
                            // If we have results for all probe samples, transition.
                             if self.history.len() >= probe_budget {
                                 self.phase = Phase::Classify;
                                 continue;
                             } else {
                                 // Waiting for user to report results.
                                 // Return empty/None implies "nothing to do yet"?
                                 // Or "Wait"? 
                                 // The user calls `ask` then `tell`. 
                                 // If they called `ask` and we yielded 10 points, they must `tell` 10 points.
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
                    
                    // Factory Strategy
                    match mode {
                        Landscape::Structured => {
                            self.strategy = Some(Box::new(NelderMead::new(self.config.bounds.len())));
                        }
                        Landscape::Chaotic => {
                            self.strategy = Some(Box::new(TPE::new(self.config.bounds.len())));
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
                         // Strategy not initialized.
                         // In full impl, `machine` would factory the strategy based on mode.
                         // For Phase 2 Checkpoint, we might just return None or TODO.
                         eprintln!("Strategy not wired for mode: {:?}", mode);
                         self.phase = Phase::Done;
                         return None;
                    }
                }
                Phase::Done => return None,
            }
        }
    }

    pub fn tell(&mut self, eval_results: Vec<EvalTrace>) {
        self.history.extend(eval_results);
    }
}

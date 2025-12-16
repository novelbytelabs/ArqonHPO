//! Omega Tier: Offline Discovery & Optimization
//!
//! This module defines the structures and flows for the "Outer Loop" (Offline).
//! Responsibilities:
//! 1. Generate Candidates (via LLM, Genetic Algo, or Grid Search).
//! 2. Evaluate Candidates (Heavy simulation/benchmarking).
//! 3. Promote Winners to the Online Variant Catalog.

use crate::variant_catalog::{Variant, VariantId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod observer;
pub use observer::{Observer, MockLlmObserver, ObserverContext};

/// A candidate solution proposed by the Discovery Tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidate {
    pub id: String,
    pub source: DiscoverySource,
    /// The proposed variant structure
    pub variant: Variant,
    /// Why this candidate was proposed
    pub hypothesis: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoverySource {
    Heuristic,
    GeneticAlgorithm,
    LlmObserver,
    GridSearch,
    Manual,
}

/// Result of an offline evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub candidate_id: String,
    pub score: f64,
    pub metrics: HashMap<String, f64>,
    pub passed_safety: bool,
    pub duration_ms: u64,
}

/// Trait for a component that can evaluate a candidate
/// (e.g., a Simulator, a Staging Env runner)
pub trait Evaluator {
    fn evaluate(&self, candidate: &Candidate) -> EvaluationResult;
}

/// The Discovery Loop (Omega Operator)
///
/// Iterates: Generate -> Evaluate -> Promote
pub struct DiscoveryLoop<E: Evaluator> {
    evaluator: E,
    candidates: Vec<Candidate>,
    promoted: Vec<Variant>,
}

impl<E: Evaluator> DiscoveryLoop<E> {
    pub fn new(evaluator: E) -> Self {
        Self {
            evaluator,
            candidates: Vec::new(),
            promoted: Vec::new(),
        }
    }

    pub fn add_candidate(&mut self, candidate: Candidate) {
        self.candidates.push(candidate);
    }

    /// Run one iteration of the discovery loop
    /// Returns the list of variants promoted in this step
    pub fn step(&mut self) -> Vec<Variant> {
        let mut new_promotions = Vec::new();

        // Drain candidates to process them
        let current_batch: Vec<Candidate> = self.candidates.drain(..).collect();

        for cand in current_batch {
            // 1. Evaluate "Offline" (simulated)
            let result = self.evaluator.evaluate(&cand);

            // 2. Check Criteria (Simple threshold for now)
            // In real system: compare vs baseline, check regression
            if result.passed_safety && result.score > 0.8 {
                // 3. Promote
                let mut var = cand.variant.clone();
                var.metadata
                    .insert("omega_score".to_string(), result.score.to_string());
                var.metadata
                    .insert("omega_source".to_string(), format!("{:?}", cand.source));

                self.promoted.push(var.clone());
                new_promotions.push(var);
            }
        }

        new_promotions
    }
}

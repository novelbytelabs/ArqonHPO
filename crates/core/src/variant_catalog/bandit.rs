//! Contextual Bandit for Variant Selection
//!
//! Uses Thompson Sampling with Beta distributions for exploration/exploitation.
//! Supports constraint-first eligibility filtering.

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Beta, Distribution};
use std::collections::HashMap;

use super::{Selection, SelectionReason, VariantId};

/// Configuration for the contextual bandit
#[derive(Debug, Clone)]
pub struct BanditConfig {
    /// Seed for deterministic behavior
    pub seed: u64,
    /// Base exploration probability (epsilon-greedy style fallback)
    pub exploration_prob: f64,
    /// Prior alpha for Beta distribution (successes + 1)
    pub prior_alpha: f64,
    /// Prior beta for Beta distribution (failures + 1)
    pub prior_beta: f64,
}

impl Default for BanditConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            exploration_prob: 0.1,
            prior_alpha: 1.0,
            prior_beta: 1.0,
        }
    }
}

/// Statistics for a single arm (variant)
#[derive(Debug, Clone)]
pub struct ArmStats {
    /// Number of times this arm was selected
    pub pulls: u64,
    /// Number of successes (reward > 0)
    pub successes: f64,
    /// Number of failures
    pub failures: f64,
    /// Running mean reward
    pub mean_reward: f64,
}

impl Default for ArmStats {
    fn default() -> Self {
        Self {
            pulls: 0,
            successes: 0.0,
            failures: 0.0,
            mean_reward: 0.0,
        }
    }
}

/// Contextual bandit for variant selection
///
/// Uses Thompson Sampling with Beta(alpha, beta) posteriors.
/// Each arm's alpha = prior_alpha + successes, beta = prior_beta + failures.
#[derive(Debug)]
pub struct ContextualBandit {
    rng: ChaCha8Rng,
    config: BanditConfig,
    arms: HashMap<VariantId, ArmStats>,
    last_selected: Option<VariantId>,
}

impl ContextualBandit {
    /// Create a new contextual bandit
    pub fn new(config: BanditConfig) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(config.seed),
            config,
            arms: HashMap::new(),
            last_selected: None,
        }
    }

    /// Select a variant from eligible candidates using Thompson Sampling
    ///
    /// # Algorithm
    /// 1. For each eligible arm, sample from Beta(alpha, beta)
    /// 2. Select the arm with the highest sample
    /// 3. With probability `exploration_prob`, select randomly instead
    pub fn select(
        &mut self,
        eligible: &[VariantId],
        default_id: Option<VariantId>,
    ) -> Option<Selection> {
        if eligible.is_empty() {
            // Fallback to default if no eligible variants
            return default_id.map(|id| Selection {
                variant_id: id,
                reason: SelectionReason::Fallback,
                exploration_prob: self.config.exploration_prob,
            });
        }

        if eligible.len() == 1 {
            let id = eligible[0];
            self.last_selected = Some(id);
            return Some(Selection {
                variant_id: id,
                reason: SelectionReason::OnlyEligible,
                exploration_prob: self.config.exploration_prob,
            });
        }

        // Epsilon-greedy exploration
        let explore: f64 = rand::Rng::random(&mut self.rng);
        if explore < self.config.exploration_prob {
            let idx = rand::Rng::random_range(&mut self.rng, 0..eligible.len());
            let id = eligible[idx];
            self.last_selected = Some(id);
            return Some(Selection {
                variant_id: id,
                reason: SelectionReason::Exploration,
                exploration_prob: self.config.exploration_prob,
            });
        }

        // Thompson Sampling: sample from posterior for each arm
        let mut best_id = eligible[0];
        let mut best_sample = f64::NEG_INFINITY;

        for &id in eligible {
            let stats = self.arms.get(&id).cloned().unwrap_or_default();

            let alpha = self.config.prior_alpha + stats.successes;
            let beta = self.config.prior_beta + stats.failures;

            // Sample from Beta(alpha, beta)
            let sample = if let Ok(dist) = Beta::new(alpha, beta) {
                dist.sample(&mut self.rng)
            } else {
                // Fallback if Beta params invalid
                0.5
            };

            if sample > best_sample {
                best_sample = sample;
                best_id = id;
            }
        }

        self.last_selected = Some(best_id);
        Some(Selection {
            variant_id: best_id,
            reason: SelectionReason::ThompsonSampling,
            exploration_prob: self.config.exploration_prob,
        })
    }

    /// Update the bandit with a reward signal
    ///
    /// Call this after observing the outcome of using the selected variant.
    /// Reward should be in [0, 1] where 1 is success, 0 is failure.
    pub fn update(&mut self, variant_id: VariantId, reward: f64) {
        let stats = self.arms.entry(variant_id).or_default();
        stats.pulls += 1;

        // Update running mean
        stats.mean_reward = stats.mean_reward + (reward - stats.mean_reward) / stats.pulls as f64;

        // Update Beta distribution parameters
        // Treat reward as Bernoulli success probability
        stats.successes += reward;
        stats.failures += 1.0 - reward;
    }

    /// Get statistics for a variant
    pub fn stats(&self, variant_id: VariantId) -> Option<&ArmStats> {
        self.arms.get(&variant_id)
    }

    /// Get the last selected variant
    pub fn last_selected(&self) -> Option<VariantId> {
        self.last_selected
    }

    /// Reset all arm statistics
    pub fn reset(&mut self) {
        self.arms.clear();
        self.last_selected = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bandit_select_single() {
        let mut bandit = ContextualBandit::new(BanditConfig::default());
        let eligible = vec![1];

        let selection = bandit.select(&eligible, None).unwrap();
        assert_eq!(selection.variant_id, 1);
        assert_eq!(selection.reason, SelectionReason::OnlyEligible);
    }

    #[test]
    fn test_bandit_select_multiple() {
        let mut bandit = ContextualBandit::new(BanditConfig {
            seed: 42,
            exploration_prob: 0.0, // Disable exploration for determinism
            ..Default::default()
        });

        let eligible = vec![1, 2, 3];

        // First selection should be from Thompson sampling
        let selection = bandit.select(&eligible, None).unwrap();
        assert_eq!(selection.reason, SelectionReason::ThompsonSampling);
        assert!(eligible.contains(&selection.variant_id));
    }

    #[test]
    fn test_bandit_update_and_learn() {
        let mut bandit = ContextualBandit::new(BanditConfig {
            seed: 42,
            exploration_prob: 0.0,
            ..Default::default()
        });

        let eligible = vec![1, 2];

        // Reward variant 1 heavily
        for _ in 0..100 {
            bandit.update(1, 1.0);
        }
        // Punish variant 2
        for _ in 0..100 {
            bandit.update(2, 0.0);
        }

        // Now selection should strongly prefer variant 1
        let mut count_1 = 0;
        for _ in 0..100 {
            let selection = bandit.select(&eligible, None).unwrap();
            if selection.variant_id == 1 {
                count_1 += 1;
            }
        }

        // Should select variant 1 almost always
        assert!(
            count_1 > 90,
            "Expected variant 1 to be selected >90 times, got {}",
            count_1
        );
    }

    #[test]
    fn test_bandit_fallback() {
        let mut bandit = ContextualBandit::new(BanditConfig::default());
        let eligible: Vec<VariantId> = vec![];

        let selection = bandit.select(&eligible, Some(99)).unwrap();
        assert_eq!(selection.variant_id, 99);
        assert_eq!(selection.reason, SelectionReason::Fallback);
    }

    #[test]
    fn test_bandit_deterministic() {
        let config = BanditConfig {
            seed: 12345,
            exploration_prob: 0.5,
            ..Default::default()
        };

        let mut bandit1 = ContextualBandit::new(config.clone());
        let mut bandit2 = ContextualBandit::new(config);

        let eligible = vec![1, 2, 3, 4, 5];

        // Same seed should give same selections
        for _ in 0..10 {
            let s1 = bandit1.select(&eligible, None).unwrap();
            let s2 = bandit2.select(&eligible, None).unwrap();
            assert_eq!(s1.variant_id, s2.variant_id);
        }
    }
}

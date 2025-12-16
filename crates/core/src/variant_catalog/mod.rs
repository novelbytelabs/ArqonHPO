//! Variant Catalog for Discrete Architecture Selection
//!
//! This module provides the "Approved Variant Catalog" pattern where
//! the online system can only SELECT among pre-approved variants,
//! never INVENT new ones.
//!
//! # Architecture
//!
//! ```text
//! Catalog → [ContextualBandit] → Eligible Variants → Selection → Apply
//! ```
//!
//! # Key Components
//!
//! - [`VariantCatalog`]: Collection of approved variants with metadata
//! - [`Variant`]: A single approved architecture/policy variant
//! - [`ContextualBandit`]: Thompson Sampling selector with constraint gating
//! - [`Context`]: Request/system context for bandit selection

mod bandit;
mod catalog;
mod context;

pub use bandit::{ArmStats, BanditConfig, ContextualBandit};
pub use catalog::{Variant, VariantCatalog, VariantConstraints, VariantId, VariantType};
pub use context::{Context, ContextFeature};

/// Result of variant selection
#[derive(Debug, Clone)]
pub struct Selection {
    /// Selected variant ID
    pub variant_id: VariantId,
    /// Why this variant was selected
    pub reason: SelectionReason,
    /// Exploration probability used
    pub exploration_prob: f64,
}

/// Reason for selecting a variant
#[derive(Debug, Clone, PartialEq)]
pub enum SelectionReason {
    /// Thompson sampling chose this arm
    ThompsonSampling,
    /// Exploration selected a random arm
    Exploration,
    /// Only one eligible variant
    OnlyEligible,
    /// Fallback to default
    Fallback,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_reason_debug() {
        let reason = SelectionReason::ThompsonSampling;
        assert_eq!(format!("{:?}", reason), "ThompsonSampling");
    }
}

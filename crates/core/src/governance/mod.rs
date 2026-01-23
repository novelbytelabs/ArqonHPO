//! Governance Protocol for ArqonBus Integration
//!
//! Defines the wire protocol for the "Propose" and "Enforce" loops.
//! Separation of Concerns:
//! - Data Plane (Tier 1/2) sends `Telemetry` and `Proposal`
//! - Control Plane (ArqonBus) sends `Approval` and `EnforcementAction`

use crate::variant_catalog::VariantId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod client;
pub use client::{GovernanceClient, MockBus};

mod rollout;
pub use rollout::RolloutTracker;

/// Message sent from Data Plane to Control Plane (ArqonBus)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UpstreamMessage {
    /// Routine telemetry digest
    Telemetry {
        timestamp_us: u64,
        node_id: String,
        metrics: HashMap<String, f64>,
    },
    /// Proposal for a new variant/parameter set (Tier 2 -> Bus)
    Proposal {
        id: String,
        timestamp_us: u64,
        variant_id: Option<VariantId>,
        params: Option<HashMap<String, f64>>,
        reason: String,
        /// Confidence in proposal (0.0 - 1.0)
        confidence: f64,
    },
    /// Alert/Violation report (Safety -> Bus)
    Alert {
        timestamp_us: u64,
        severity: AlertSeverity,
        message: String,
        context: HashMap<String, String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Message sent from Control Plane (ArqonBus) to Data Plane
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DownstreamMessage {
    /// Approval for a specific proposal
    Approval {
        proposal_id: String,
        /// Digest to ensure we're approving what we think we are
        proposal_digest: String,
        /// Gradual rollout config (if any)
        rollout: Option<RolloutConfig>,
    },
    /// Direct enforcement action (override)
    Enforce {
        action: EnforcementAction,
        reason: String,
    },
    /// Update to the Approved Variant Catalog
    CatalogUpdate {
        version: u64,
        /// URL or path to fetch new catalog
        source_uri: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolloutConfig {
    /// Percentage of traffic (0.0 - 1.0)
    pub initial_percent: f64,
    /// Increment per minute
    pub increment_per_minute: f64,
    /// Max percent
    pub max_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnforcementAction {
    /// Force specific variant
    SetVariant(VariantId),
    /// Force specific parameter values
    SetParams(HashMap<String, f64>),
    /// Reset to safe baseline
    EmergencyStop,
    /// Pause all Tier 2 adaptation
    PauseAdaptation,
    /// Resume Tier 2 adaptation
    ResumeAdaptation,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization() {
        let msg = UpstreamMessage::Proposal {
            id: "prop_123".to_string(),
            timestamp_us: 1000,
            variant_id: Some(1),
            params: None,
            reason: "latency spike".to_string(),
            confidence: 0.9,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("prop_123"));

        let decoded: UpstreamMessage = serde_json::from_str(&json).unwrap();
        match decoded {
            UpstreamMessage::Proposal { id, .. } => assert_eq!(id, "prop_123"),
            _ => panic!("Wrong type"),
        }
    }
}

//! Audit queue for non-blocking event logging.
//!
//! Constitution: VIII.5 - Audit-to-disk MUST be decoupled via ring buffer.
//! IX.2 - Events MUST include correlation IDs.

use crossbeam_queue::ArrayQueue;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Audit policy defining how events are recorded.
///
/// Constitution: Audit MUST be explicit, never bypassed silently.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuditPolicy {
    /// In-memory ring buffer only (always on, never blocks).
    /// Default for all tiers.
    InMemoryRequired,
    
    /// In-memory + async background flush to disk.
    /// Production default.
    InMemoryPlusAsyncDisk,
    
    /// In-memory only, no disk persistence.
    /// For benchmarks and constrained environments.
    InMemoryOnly,
    
    /// Audit disabled. **FORBIDDEN in Tier 1/2.**
    /// Only allowed in Tier Ω sandbox.
    Disable,
}

impl Default for AuditPolicy {
    fn default() -> Self {
        Self::InMemoryRequired
    }
}

/// Tier classification for enforcement rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tier {
    /// Core safety tier - Disable audit is FORBIDDEN.
    Tier1,
    /// Decision tier - Disable audit is FORBIDDEN.
    Tier2,
    /// Experimental sandbox - Disable audit is ALLOWED.
    TierOmega,
}

impl AuditPolicy {
    /// Validate that policy is allowed for the given tier.
    ///
    /// Constitution: Tier 1/2 MUST have audit enabled.
    pub fn validate_for_tier(self, tier: Tier) -> Result<(), AuditPolicyError> {
        match (self, tier) {
            (AuditPolicy::Disable, Tier::Tier1) => Err(AuditPolicyError::DisableNotAllowedInTier1),
            (AuditPolicy::Disable, Tier::Tier2) => Err(AuditPolicyError::DisableNotAllowedInTier2),
            (AuditPolicy::Disable, Tier::TierOmega) => Ok(()), // Allowed in sandbox
            _ => Ok(()),
        }
    }
}

/// Errors when validating audit policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuditPolicyError {
    /// Disable policy is forbidden in Tier 1.
    DisableNotAllowedInTier1,
    /// Disable policy is forbidden in Tier 2.
    DisableNotAllowedInTier2,
}

impl std::fmt::Display for AuditPolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DisableNotAllowedInTier1 => write!(f, "AuditPolicy::Disable is forbidden in Tier 1"),
            Self::DisableNotAllowedInTier2 => write!(f, "AuditPolicy::Disable is forbidden in Tier 2"),
        }
    }
}

impl std::error::Error for AuditPolicyError {}

/// Event types for audit logging.
#[derive(Clone, Debug)]
pub enum EventType {
    Digest,
    Proposal,
    Apply,
    Rollback,
    SafeModeEntered,
    SafeModeExited,
}

/// Structured audit event (fixed-size, no heap allocation).
///
/// Constitution: IX.2 - Events MUST include correlation IDs.
/// Hot path MUST NOT allocate.
#[derive(Clone, Debug)]
pub struct AuditEvent {
    pub event_type: EventType,
    pub timestamp_us: u64,
    pub run_id: u64,
    pub proposal_id: Option<u64>,
    pub config_version: u64,
    /// Static string payload (no heap allocation in hot path).
    pub payload: &'static str,
}

impl AuditEvent {
    /// Create a new audit event.
    pub fn new(
        event_type: EventType,
        timestamp_us: u64,
        run_id: u64,
        config_version: u64,
    ) -> Self {
        Self {
            event_type,
            timestamp_us,
            run_id,
            proposal_id: None,
            config_version,
            payload: "",
        }
    }

    /// Set proposal ID.
    pub fn with_proposal_id(mut self, id: u64) -> Self {
        self.proposal_id = Some(id);
        self
    }

    /// Set payload (static str only - no allocation).
    pub fn with_payload(mut self, payload: &'static str) -> Self {
        self.payload = payload;
        self
    }
}

/// Result of enqueue operation.
#[derive(Clone, Debug, PartialEq)]
pub enum EnqueueResult {
    /// Successfully enqueued.
    Ok,
    /// Queue above 80% capacity (warning).
    HighWaterMark,
    /// Queue is full (triggers SafeMode).
    Full,
}

/// Lock-free audit queue with drop counting.
///
/// Constitution: VIII.5 - No blocking I/O in hot path.
/// AC-17: When queue is full, adaptation halts; events are counted, never silently dropped.
pub struct AuditQueue {
    queue: Arc<ArrayQueue<AuditEvent>>,
    capacity: usize,
    high_water_mark: usize,
    /// Count of events that couldn't be enqueued (for monitoring).
    drop_count: AtomicU64,
}

impl AuditQueue {
    /// Create a new audit queue with given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: Arc::new(ArrayQueue::new(capacity)),
            capacity,
            high_water_mark: (capacity * 80) / 100,
            drop_count: AtomicU64::new(0),
        }
    }

    /// Enqueue an event (non-blocking).
    ///
    /// Returns Full if queue is at capacity. Never blocks.
    pub fn enqueue(&self, event: AuditEvent) -> EnqueueResult {
        match self.queue.push(event) {
            Ok(()) => {
                if self.queue.len() >= self.high_water_mark {
                    EnqueueResult::HighWaterMark
                } else {
                    EnqueueResult::Ok
                }
            }
            Err(_) => {
                self.drop_count.fetch_add(1, Ordering::Relaxed);
                EnqueueResult::Full
            }
        }
    }

    /// Drain events for async flush (cold path).
    pub fn drain(&self) -> Vec<AuditEvent> {
        let mut events = Vec::new();
        while let Some(event) = self.queue.pop() {
            events.push(event);
        }
        events
    }

    /// Current queue length.
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Check if queue is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Queue capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    
    /// Number of events that couldn't be enqueued (for monitoring).
    pub fn drop_count(&self) -> u64 {
        self.drop_count.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::new(EventType::Apply, 1000, 42, 5)
            .with_proposal_id(123)
            .with_payload("delta applied");
        
        assert!(matches!(event.event_type, EventType::Apply));
        assert_eq!(event.run_id, 42);
        assert_eq!(event.proposal_id, Some(123));
    }

    #[test]
    fn test_audit_queue_enqueue() {
        let queue = AuditQueue::new(10);
        
        let event = AuditEvent::new(EventType::Digest, 1000, 1, 1);
        assert_eq!(queue.enqueue(event), EnqueueResult::Ok);
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_audit_queue_full() {
        let queue = AuditQueue::new(2);
        
        queue.enqueue(AuditEvent::new(EventType::Digest, 1, 1, 1));
        queue.enqueue(AuditEvent::new(EventType::Digest, 2, 1, 1));
        
        let result = queue.enqueue(AuditEvent::new(EventType::Digest, 3, 1, 1));
        assert_eq!(result, EnqueueResult::Full);
    }

    #[test]
    fn test_audit_queue_drain() {
        let queue = AuditQueue::new(10);
        
        queue.enqueue(AuditEvent::new(EventType::Digest, 1, 1, 1));
        queue.enqueue(AuditEvent::new(EventType::Apply, 2, 1, 1));
        
        let events = queue.drain();
        assert_eq!(events.len(), 2);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_no_silent_drops() {
        let queue = AuditQueue::new(100);
        
        // Enqueue 100 events
        for i in 0..100 {
            let result = queue.enqueue(AuditEvent::new(EventType::Proposal, i, 1, 1));
            assert_ne!(result, EnqueueResult::Full, "Event {} should not be dropped", i);
        }
        
        // 101st should fail
        let result = queue.enqueue(AuditEvent::new(EventType::Proposal, 100, 1, 1));
        assert_eq!(result, EnqueueResult::Full);
        
        // Drain and verify count
        let events = queue.drain();
        assert_eq!(events.len(), 100, "Exactly 100 events should be present");
    }

    #[test]
    fn test_audit_policy_tier1_forbids_disable() {
        let policy = AuditPolicy::Disable;
        let result = policy.validate_for_tier(Tier::Tier1);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AuditPolicyError::DisableNotAllowedInTier1);
    }

    #[test]
    fn test_audit_policy_tier2_forbids_disable() {
        let policy = AuditPolicy::Disable;
        let result = policy.validate_for_tier(Tier::Tier2);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AuditPolicyError::DisableNotAllowedInTier2);
    }

    #[test]
    fn test_audit_policy_tier_omega_allows_disable() {
        let policy = AuditPolicy::Disable;
        let result = policy.validate_for_tier(Tier::TierOmega);
        assert!(result.is_ok());
    }

    #[test]
    fn test_audit_policy_inmemory_allowed_all_tiers() {
        for policy in [AuditPolicy::InMemoryRequired, AuditPolicy::InMemoryPlusAsyncDisk, AuditPolicy::InMemoryOnly] {
            assert!(policy.validate_for_tier(Tier::Tier1).is_ok());
            assert!(policy.validate_for_tier(Tier::Tier2).is_ok());
            assert!(policy.validate_for_tier(Tier::TierOmega).is_ok());
        }
    }

    #[test]
    fn test_saturation_drop_count() {
        let queue = AuditQueue::new(5);
        
        // Fill queue
        for i in 0..5 {
            queue.enqueue(AuditEvent::new(EventType::Digest, i, 1, 1));
        }
        assert_eq!(queue.drop_count(), 0);
        
        // Try to enqueue 10 more (all should fail)
        for i in 5..15 {
            let result = queue.enqueue(AuditEvent::new(EventType::Digest, i, 1, 1));
            assert_eq!(result, EnqueueResult::Full);
        }
        
        // Should have counted 10 drops
        assert_eq!(queue.drop_count(), 10);
        
        // Queue still has original 5 events
        assert_eq!(queue.len(), 5);
    }

    #[test]
    fn test_default_audit_policy() {
        let policy = AuditPolicy::default();
        assert_eq!(policy, AuditPolicy::InMemoryRequired);
    }
}


//! Audit queue for non-blocking event logging.
//!
//! Constitution: VIII.5 - Audit-to-disk MUST be decoupled via ring buffer.
//! IX.2 - Events MUST include correlation IDs.

use crossbeam_queue::ArrayQueue;
use std::sync::Arc;

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

/// Structured audit event.
///
/// Constitution: IX.2 - Events MUST include correlation IDs.
#[derive(Clone, Debug)]
pub struct AuditEvent {
    pub event_type: EventType,
    pub timestamp_us: u64,
    pub run_id: u64,
    pub proposal_id: Option<u64>,
    pub config_version: u64,
    pub payload: String,
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
            payload: String::new(),
        }
    }

    /// Set proposal ID.
    pub fn with_proposal_id(mut self, id: u64) -> Self {
        self.proposal_id = Some(id);
        self
    }

    /// Set payload.
    pub fn with_payload(mut self, payload: impl Into<String>) -> Self {
        self.payload = payload.into();
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

/// Lock-free audit queue.
///
/// Constitution: VIII.5 - No blocking I/O in hot path.
/// AC-17: When queue is full, adaptation halts; never silently drops.
pub struct AuditQueue {
    queue: Arc<ArrayQueue<AuditEvent>>,
    capacity: usize,
    high_water_mark: usize,
}

impl AuditQueue {
    /// Create a new audit queue with given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: Arc::new(ArrayQueue::new(capacity)),
            capacity,
            high_water_mark: (capacity * 80) / 100,
        }
    }

    /// Enqueue an event (non-blocking).
    pub fn enqueue(&self, event: AuditEvent) -> EnqueueResult {
        match self.queue.push(event) {
            Ok(()) => {
                if self.queue.len() >= self.high_water_mark {
                    EnqueueResult::HighWaterMark
                } else {
                    EnqueueResult::Ok
                }
            }
            Err(_) => EnqueueResult::Full,
        }
    }

    /// Drain events for async flush.
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
}

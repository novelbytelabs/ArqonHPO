//! Telemetry types and ring buffer
//!
//! Compact telemetry digests for the adaptive engine.
//! The ring buffer provides lock-free push for high-frequency telemetry.

use std::collections::VecDeque;

/// Compact telemetry digest
///
/// Contains only the essential metrics needed for optimization.
/// This should be small enough for high-frequency emission.
#[derive(Debug, Clone, Default)]
pub struct TelemetryDigest {
    /// Timestamp in microseconds
    pub timestamp_us: u64,
    /// Primary objective value (what we're optimizing)
    pub objective_value: f64,
    /// Optional: latency p99 in microseconds
    pub latency_p99_us: Option<u64>,
    /// Optional: throughput (requests/second)
    pub throughput_rps: Option<f64>,
    /// Optional: error rate (0.0 - 1.0)
    pub error_rate: Option<f64>,
    /// Optional: memory usage in bytes
    pub memory_bytes: Option<u64>,
    /// Optional: constraint margin (positive = within constraints)
    pub constraint_margin: Option<f64>,
}

impl TelemetryDigest {
    /// Create a minimal digest with just the objective value
    pub fn objective(value: f64) -> Self {
        Self {
            objective_value: value,
            ..Default::default()
        }
    }
    
    /// Create a digest with timestamp and objective
    pub fn with_timestamp(timestamp_us: u64, objective_value: f64) -> Self {
        Self {
            timestamp_us,
            objective_value,
            ..Default::default()
        }
    }
}

/// Fixed-size ring buffer for telemetry digests
///
/// Simple VecDeque-based implementation. For production, consider
/// a true lock-free SPSC ring buffer.
#[derive(Debug)]
pub struct TelemetryRingBuffer {
    buffer: VecDeque<TelemetryDigest>,
    capacity: usize,
}

impl TelemetryRingBuffer {
    /// Create a new ring buffer with given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }
    
    /// Push a new digest, evicting oldest if full
    pub fn push(&mut self, digest: TelemetryDigest) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(digest);
    }
    
    /// Get the most recent digest
    pub fn latest(&self) -> Option<&TelemetryDigest> {
        self.buffer.back()
    }
    
    /// Get recent N digests (most recent first)
    pub fn recent(&self, n: usize) -> impl Iterator<Item = &TelemetryDigest> {
        self.buffer.iter().rev().take(n)
    }
    
    /// Compute mean objective over recent history
    pub fn mean_objective(&self, window: usize) -> Option<f64> {
        let values: Vec<f64> = self.recent(window)
            .map(|d| d.objective_value)
            .collect();
        
        if values.is_empty() {
            None
        } else {
            Some(values.iter().sum::<f64>() / values.len() as f64)
        }
    }
    
    /// Length of buffer
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    
    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
    
    /// Clear all digests
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ring_buffer_push() {
        let mut buffer = TelemetryRingBuffer::new(3);
        
        buffer.push(TelemetryDigest::objective(1.0));
        buffer.push(TelemetryDigest::objective(2.0));
        buffer.push(TelemetryDigest::objective(3.0));
        
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.latest().unwrap().objective_value, 3.0);
    }
    
    #[test]
    fn test_ring_buffer_eviction() {
        let mut buffer = TelemetryRingBuffer::new(2);
        
        buffer.push(TelemetryDigest::objective(1.0));
        buffer.push(TelemetryDigest::objective(2.0));
        buffer.push(TelemetryDigest::objective(3.0));
        
        // Should have evicted 1.0
        assert_eq!(buffer.len(), 2);
        let values: Vec<f64> = buffer.recent(10)
            .map(|d| d.objective_value)
            .collect();
        assert_eq!(values, vec![3.0, 2.0]);
    }
    
    #[test]
    fn test_mean_objective() {
        let mut buffer = TelemetryRingBuffer::new(10);
        
        buffer.push(TelemetryDigest::objective(1.0));
        buffer.push(TelemetryDigest::objective(2.0));
        buffer.push(TelemetryDigest::objective(3.0));
        
        assert_eq!(buffer.mean_objective(10), Some(2.0));
        assert_eq!(buffer.mean_objective(2), Some(2.5)); // (3+2)/2
    }
}

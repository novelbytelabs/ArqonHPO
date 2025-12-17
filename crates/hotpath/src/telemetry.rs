//! Telemetry types for streaming metrics.
//!
//! Constitution: II.19 - Telemetry Digest Contract (≤128 bytes)

use std::mem::size_of;

/// Digest validity after staleness and generation checks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DigestValidity {
    /// Digest is valid and usable.
    Valid,
    /// Digest is from wrong config generation.
    WrongGeneration,
    /// Digest is too old (beyond max_digest_age).
    TooOld,
    /// Digest arrived during settle period.
    Settling,
}

/// Compact telemetry from the data plane.
///
/// Constitution: II.19 - MUST be ≤128 bytes with mandatory timestamp_us and objective_value.
#[derive(Clone, Debug, Default)]
pub struct TelemetryDigest {
    /// Timestamp in microseconds (required).
    pub timestamp_us: u64,
    /// Objective function value (required).
    pub objective_value: f64,
    /// Config generation observed by the data plane at emit time (required for correlation).
    pub config_generation: u64,
    /// Optional: p99 latency in microseconds.
    pub latency_p99_us: Option<u64>,
    /// Optional: throughput in requests per second.
    pub throughput_rps: Option<f64>,
    /// Optional: error rate (0.0 to 1.0).
    pub error_rate: Option<f64>,
    /// Optional: constraint margin (positive = satisfied, negative = violated).
    pub constraint_margin: Option<f64>,
}

// Compile-time size assertion (AC-9)
const _: () = assert!(size_of::<TelemetryDigest>() <= 128, "TelemetryDigest must be ≤128 bytes");

impl TelemetryDigest {
    /// Create a new digest with required fields only.
    pub fn new(timestamp_us: u64, objective_value: f64, config_generation: u64) -> Self {
        Self {
            timestamp_us,
            objective_value,
            config_generation,
            ..Default::default()
        }
    }

    /// Validate digest against expected generation and timing windows.
    pub fn validate(
        &self,
        expected_generation: u64,
        apply_timestamp_us: u64,
        settle_time_us: u64,
        max_age_us: u64,
        now_us: u64,
    ) -> DigestValidity {
        if self.config_generation != expected_generation {
            return DigestValidity::WrongGeneration;
        }
        if self.timestamp_us < apply_timestamp_us + settle_time_us {
            return DigestValidity::Settling;
        }
        if now_us > self.timestamp_us && now_us - self.timestamp_us > max_age_us {
            return DigestValidity::TooOld;
        }
        DigestValidity::Valid
    }
}

/// Fixed-capacity ring buffer for telemetry digests.
///
/// Constitution: VIII.5 - No allocation after init, O(1) push.
pub struct TelemetryRingBuffer {
    buffer: Box<[Option<TelemetryDigest>]>,
    capacity: usize,
    head: usize,
    len: usize,
    drop_count: u64,
}

impl TelemetryRingBuffer {
    /// Create a new ring buffer with fixed capacity.
    pub fn new(capacity: usize) -> Self {
        let buffer: Vec<Option<TelemetryDigest>> = (0..capacity).map(|_| None).collect();
        Self {
            buffer: buffer.into_boxed_slice(),
            capacity,
            head: 0,
            len: 0,
            drop_count: 0,
        }
    }

    /// Push a digest, evicting oldest if at capacity.
    pub fn push(&mut self, digest: TelemetryDigest) {
        let slot = (self.head + self.len) % self.capacity;
        
        if self.len == self.capacity {
            // Overwrite oldest
            self.head = (self.head + 1) % self.capacity;
            self.drop_count += 1;
        } else {
            self.len += 1;
        }
        
        self.buffer[slot] = Some(digest);
    }

    /// Get the number of digests in the buffer.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get count of dropped digests due to overflow.
    pub fn drop_count(&self) -> u64 {
        self.drop_count
    }

    /// Clear all digests.
    pub fn clear(&mut self) {
        for slot in self.buffer.iter_mut() {
            *slot = None;
        }
        self.head = 0;
        self.len = 0;
    }

    /// Iterate over digests from oldest to newest.
    pub fn iter(&self) -> impl Iterator<Item = &TelemetryDigest> {
        (0..self.len)
            .map(move |i| (self.head + i) % self.capacity)
            .filter_map(|idx| self.buffer[idx].as_ref())
    }

    /// Collect valid digests matching expected generation.
    pub fn collect_valid(
        &self,
        expected_generation: u64,
        apply_timestamp_us: u64,
        settle_time_us: u64,
        max_age_us: u64,
        now_us: u64,
    ) -> Vec<&TelemetryDigest> {
        self.iter()
            .filter(|d| {
                d.validate(expected_generation, apply_timestamp_us, settle_time_us, max_age_us, now_us)
                    == DigestValidity::Valid
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digest_size() {
        assert!(size_of::<TelemetryDigest>() <= 128);
    }

    #[test]
    fn test_digest_validation() {
        let digest = TelemetryDigest::new(1000, 0.5, 1);
        
        // Valid case
        assert_eq!(
            digest.validate(1, 500, 100, 10000, 1500),
            DigestValidity::Valid
        );
        
        // Wrong generation
        assert_eq!(
            digest.validate(2, 500, 100, 10000, 1500),
            DigestValidity::WrongGeneration
        );
        
        // Settling period
        assert_eq!(
            digest.validate(1, 950, 100, 10000, 1500),
            DigestValidity::Settling
        );
    }

    #[test]
    fn test_ring_buffer_overflow() {
        let mut buf = TelemetryRingBuffer::new(3);
        
        buf.push(TelemetryDigest::new(1, 0.1, 1));
        buf.push(TelemetryDigest::new(2, 0.2, 1));
        buf.push(TelemetryDigest::new(3, 0.3, 1));
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.drop_count(), 0);
        
        buf.push(TelemetryDigest::new(4, 0.4, 1));
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.drop_count(), 1);
        
        // Oldest should be evicted (timestamp 1)
        let timestamps: Vec<u64> = buf.iter().map(|d| d.timestamp_us).collect();
        assert_eq!(timestamps, vec![2, 3, 4]);
    }
}

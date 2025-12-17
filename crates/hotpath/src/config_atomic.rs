//! Configuration types for atomic parameter management.
//!
//! Constitution: II.18 - Atomic Configuration Contract

use smallvec::SmallVec;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

/// Stable parameter identifier (u16 = up to 65K params).
///
/// Used internally in the hot path to avoid string operations.
pub type ParamId = u16;

/// Dense parameter vector, stack-allocated for ≤16 params.
///
/// This avoids heap allocation in the hot path for typical workloads.
pub type ParamVec = SmallVec<[f64; 16]>;

/// Create a ParamVec from a slice (for testing and initialization).
pub fn param_vec(values: &[f64]) -> ParamVec {
    ParamVec::from_slice(values)
}

/// Mapping between human-readable parameter names and dense IDs.
///
/// Created once at initialization and immutable thereafter.
/// Implemented without HashMap to satisfy Hot Path invariants.
#[derive(Clone, Debug)]
pub struct ParamRegistry {
    // Sorted (Name, ParamId) dictionary for O(log N) lookup
    name_map: Vec<(String, ParamId)>,
    // Dense lookup by ID (index = ParamId)
    id_to_name: Vec<String>,
}

impl ParamRegistry {
    /// Create a new registry from parameter names.
    pub fn new(names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let id_to_name: Vec<String> = names.into_iter().map(|n| n.into()).collect();

        let mut name_map: Vec<(String, ParamId)> = id_to_name
            .iter()
            .enumerate()
            .map(|(i, name)| (name.clone(), i as ParamId))
            .collect();

        // Sort by name for binary search
        name_map.sort_by(|a, b| a.0.cmp(&b.0));

        Self {
            name_map,
            id_to_name,
        }
    }

    /// Look up a parameter ID by name.
    pub fn get_id(&self, name: &str) -> Option<ParamId> {
        self.name_map
            .binary_search_by(|(n, _)| n.as_str().cmp(name))
            .ok()
            .map(|idx| self.name_map[idx].1)
    }

    /// Look up a parameter name by ID.
    pub fn get_name(&self, id: ParamId) -> Option<&str> {
        self.id_to_name.get(id as usize).map(|s| s.as_str())
    }

    /// Number of registered parameters.
    pub fn len(&self) -> usize {
        self.id_to_name.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.id_to_name.is_empty()
    }

    /// Convert a named map (as slice) to a ParamVec.
    /// Used at boundary.
    pub fn to_param_vec(&self, map: &[(String, f64)]) -> ParamVec {
        let mut vec = ParamVec::with_capacity(self.len());
        // Initialize with default (0.0?) or require all params?
        // For simplicity, we can scan. Optimal: sort input if needed.
        // But for boundary conversion, a naive lookup is acceptable or
        // we assume map is large.

        // Actually, we must produce a dense vector ordered by ID.
        // We can iterate IDs and look up in the map (O(N*M) or O(N log M) if map sorted).
        // Since input is slice, O(N*M).
        // Better: sort input map temporarily?

        // Let's assume the caller provides what we need.
        // But `to_param_vec` usually takes a HashMap.
        // If we strictly ban HashMap types in signatures... use `&[(String, f64)]`.

        // Naive for now:
        for name in &self.id_to_name {
            let val = map
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| v)
                .copied()
                .unwrap_or(0.0);
            vec.push(val);
        }
        vec
    }

    /// Convert a ParamVec to a list of named values.
    /// Used at boundary.
    pub fn to_kv(&self, vec: &ParamVec) -> Vec<(String, f64)> {
        self.id_to_name
            .iter()
            .zip(vec.iter())
            .map(|(name, &value)| (name.clone(), value))
            .collect()
    }
}

/// Immutable configuration snapshot.
///
/// Constitution: II.18 - Config swaps MUST be atomic with monotonic generation counter.
#[derive(Clone, Debug)]
pub struct ConfigSnapshot {
    /// Parameter values as a dense vector.
    pub params: ParamVec,
    /// Monotonically increasing generation counter.
    pub generation: u64,
}

impl ConfigSnapshot {
    /// Create a new snapshot with initial parameters and generation 0.
    pub fn new(params: ParamVec) -> Self {
        Self {
            params,
            generation: 0,
        }
    }

    /// Create a snapshot with a specific generation.
    pub fn with_generation(params: ParamVec, generation: u64) -> Self {
        Self { params, generation }
    }
}

/// Thread-safe atomic configuration.
///
/// Constitution: II.18 - Atomic Configuration Contract
/// - Atomicity: RwLock<Arc<ConfigSnapshot>>
/// - Generation counter: monotonically increasing
/// - Zero-alloc hot path: snapshot() = Arc clone only
/// - Thread safety: Send + Sync
pub struct AtomicConfig {
    inner: RwLock<Arc<ConfigSnapshot>>,
    generation: AtomicU64,
    baseline: RwLock<Option<Arc<ConfigSnapshot>>>,
}

impl AtomicConfig {
    /// Create a new atomic config with initial parameters.
    pub fn new(params: ParamVec) -> Self {
        let snapshot = Arc::new(ConfigSnapshot::new(params));
        Self {
            inner: RwLock::new(snapshot),
            generation: AtomicU64::new(0),
            baseline: RwLock::new(None),
        }
    }

    /// Get current configuration snapshot (zero-copy via Arc clone).
    pub fn snapshot(&self) -> Arc<ConfigSnapshot> {
        self.inner.read().unwrap().clone()
    }

    /// Get current generation counter.
    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }

    /// Swap in a new configuration, incrementing the generation counter.
    ///
    /// Returns the new generation.
    pub fn swap(&self, new_params: ParamVec) -> u64 {
        let new_gen = self.generation.fetch_add(1, Ordering::AcqRel) + 1;
        let new_snapshot = Arc::new(ConfigSnapshot::with_generation(new_params, new_gen));
        *self.inner.write().unwrap() = new_snapshot;
        new_gen
    }

    /// Set the current config as the baseline for rollback.
    pub fn set_baseline(&self) {
        let current = self.snapshot();
        *self.baseline.write().unwrap() = Some(current);
    }

    /// Rollback to the baseline configuration.
    ///
    /// Returns the new generation, or None if no baseline is set.
    pub fn rollback(&self) -> Option<u64> {
        let baseline = self.baseline.read().unwrap().clone()?;
        let new_gen = self.generation.fetch_add(1, Ordering::AcqRel) + 1;
        let new_snapshot = Arc::new(ConfigSnapshot::with_generation(
            baseline.params.clone(),
            new_gen,
        ));
        *self.inner.write().unwrap() = new_snapshot;
        Some(new_gen)
    }
}

// Ensure AtomicConfig is Send + Sync (AC-10)
static_assertions::assert_impl_all!(AtomicConfig: Send, Sync);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_registry() {
        let registry = ParamRegistry::new(["alpha", "beta", "gamma"]);
        assert_eq!(registry.len(), 3);
        assert_eq!(registry.get_id("alpha"), Some(0));
        assert_eq!(registry.get_id("beta"), Some(1));
        assert_eq!(registry.get_name(2), Some("gamma"));
    }

    #[test]
    fn test_config_snapshot_generation() {
        let params = ParamVec::from_slice(&[1.0, 2.0, 3.0]);
        let snapshot = ConfigSnapshot::with_generation(params, 42);
        assert_eq!(snapshot.generation, 42);
    }

    #[test]
    fn test_atomic_config_swap_increments_generation() {
        let config = AtomicConfig::new(ParamVec::from_slice(&[0.5]));
        assert_eq!(config.generation(), 0);

        let gen1 = config.swap(ParamVec::from_slice(&[0.6]));
        assert_eq!(gen1, 1);
        assert_eq!(config.generation(), 1);

        let gen2 = config.swap(ParamVec::from_slice(&[0.7]));
        assert_eq!(gen2, 2);
        assert_eq!(config.generation(), 2);
    }

    #[test]
    fn test_atomic_config_rollback() {
        let config = AtomicConfig::new(ParamVec::from_slice(&[0.5]));
        config.set_baseline();

        config.swap(ParamVec::from_slice(&[0.9]));
        assert_eq!(config.snapshot().params[0], 0.9);

        let gen = config.rollback().unwrap();
        assert_eq!(config.snapshot().params[0], 0.5);
        assert_eq!(gen, 2);
    }
}

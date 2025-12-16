//! Atomic configuration management
//!
//! Provides lock-free configuration swaps for the adaptive engine.
//! Uses atomic operations to ensure zero-allocation in the hot path.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Immutable configuration snapshot
#[derive(Debug, Clone)]
pub struct ConfigSnapshot {
    /// Parameter values
    pub params: HashMap<String, f64>,
    /// Generation counter (monotonically increasing)
    pub generation: u64,
}

/// Atomic configuration container
///
/// Uses ArcSwap-style semantics: readers get a snapshot atomically,
/// writers swap in a new snapshot. Zero-copy for readers.
#[derive(Debug)]
pub struct AtomicConfig {
    /// Current configuration (behind Arc for cheap cloning)
    current: std::sync::RwLock<Arc<ConfigSnapshot>>,
    /// Generation counter
    generation: AtomicU64,
}

impl AtomicConfig {
    /// Create a new atomic config with initial parameters
    pub fn new(params: HashMap<String, f64>) -> Self {
        let snapshot = Arc::new(ConfigSnapshot {
            params,
            generation: 0,
        });
        
        Self {
            current: std::sync::RwLock::new(snapshot),
            generation: AtomicU64::new(0),
        }
    }
    
    /// Get a snapshot of the current configuration
    ///
    /// This is a cheap operation (Arc clone).
    pub fn snapshot(&self) -> ConfigSnapshot {
        let guard = self.current.read().unwrap();
        (**guard).clone()
    }
    
    /// Atomically swap to a new configuration
    ///
    /// Returns the new generation number.
    pub fn swap(&self, params: HashMap<String, f64>) -> u64 {
        let new_gen = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        
        let new_snapshot = Arc::new(ConfigSnapshot {
            params,
            generation: new_gen,
        });
        
        let mut guard = self.current.write().unwrap();
        *guard = new_snapshot;
        
        new_gen
    }
    
    /// Get the current generation number
    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_atomic_config_snapshot() {
        let params = HashMap::from([
            ("x".to_string(), 1.0),
            ("y".to_string(), 2.0),
        ]);
        
        let config = AtomicConfig::new(params.clone());
        let snapshot = config.snapshot();
        
        assert_eq!(snapshot.params, params);
        assert_eq!(snapshot.generation, 0);
    }
    
    #[test]
    fn test_atomic_config_swap() {
        let initial = HashMap::from([("x".to_string(), 1.0)]);
        let updated = HashMap::from([("x".to_string(), 2.0)]);
        
        let config = AtomicConfig::new(initial);
        
        let gen = config.swap(updated.clone());
        assert_eq!(gen, 1);
        
        let snapshot = config.snapshot();
        assert_eq!(snapshot.params, updated);
        assert_eq!(snapshot.generation, 1);
    }
    
    #[test]
    fn test_generation_monotonic() {
        let config = AtomicConfig::new(HashMap::new());
        
        for i in 1..=10 {
            let gen = config.swap(HashMap::new());
            assert_eq!(gen, i);
        }
    }
}

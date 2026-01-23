//! Context features for contextual bandit selection
//!
//! Defines the context (system state) that influences variant selection.

use std::collections::HashMap;

/// A single context feature
#[derive(Debug, Clone)]
pub enum ContextFeature {
    /// Numeric feature (load, memory pressure, etc.)
    Numeric(f64),
    /// Categorical feature (GPU type, request class, etc.)
    Categorical(String),
    /// Boolean feature (flag, condition, etc.)
    Boolean(bool),
}

/// System/request context for variant selection
#[derive(Debug, Clone, Default)]
pub struct Context {
    /// Current load factor (0.0 - 1.0)
    pub load_factor: f64,
    /// Current latency budget in microseconds
    pub latency_budget_us: u64,
    /// Current memory budget in bytes
    pub memory_budget_bytes: u64,
    /// Minimum quality requirement (0.0 - 1.0)
    pub min_quality: f64,
    /// Cost budget per request
    pub cost_budget: f64,
    /// GPU type being used
    pub gpu_type: String,
    /// Request class (e.g., "chat", "completion", "embedding")
    pub request_class: String,
    /// Batch size
    pub batch_size: u32,
    /// Additional custom features
    pub features: HashMap<String, ContextFeature>,
}

impl Context {
    /// Create a new context with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder: set load factor
    pub fn with_load(mut self, load: f64) -> Self {
        self.load_factor = load;
        self
    }

    /// Builder: set latency budget
    pub fn with_latency_budget(mut self, us: u64) -> Self {
        self.latency_budget_us = us;
        self
    }

    /// Builder: set memory budget
    pub fn with_memory_budget(mut self, bytes: u64) -> Self {
        self.memory_budget_bytes = bytes;
        self
    }

    /// Builder: set GPU type
    pub fn with_gpu(mut self, gpu_type: &str) -> Self {
        self.gpu_type = gpu_type.to_string();
        self
    }

    /// Builder: set request class
    pub fn with_request_class(mut self, class: &str) -> Self {
        self.request_class = class.to_string();
        self
    }

    /// Builder: set batch size
    pub fn with_batch_size(mut self, size: u32) -> Self {
        self.batch_size = size;
        self
    }

    /// Add a custom feature
    pub fn with_feature(mut self, name: &str, feature: ContextFeature) -> Self {
        self.features.insert(name.to_string(), feature);
        self
    }

    /// Convert context to a feature vector for ML models
    pub fn to_feature_vector(&self) -> Vec<f64> {
        // Basic numeric features
        let mut features = vec![
            self.load_factor,
            self.latency_budget_us as f64 / 1_000_000.0, // Normalize to seconds
            self.memory_budget_bytes as f64 / 1_000_000_000.0, // Normalize to GB
            self.min_quality,
            self.cost_budget,
            self.batch_size as f64 / 128.0, // Normalize by typical max batch
        ];

        // Add custom numeric features
        for feature in self.features.values() {
            if let ContextFeature::Numeric(v) = feature {
                features.push(*v);
            }
        }

        features
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_builder() {
        let ctx = Context::new()
            .with_load(0.8)
            .with_latency_budget(10_000)
            .with_gpu("a100")
            .with_batch_size(32);

        assert_eq!(ctx.load_factor, 0.8);
        assert_eq!(ctx.latency_budget_us, 10_000);
        assert_eq!(ctx.gpu_type, "a100");
        assert_eq!(ctx.batch_size, 32);
    }

    #[test]
    fn test_context_feature_vector() {
        let ctx = Context::new()
            .with_load(0.5)
            .with_latency_budget(1_000_000) // 1 second
            .with_memory_budget(1_000_000_000) // 1 GB
            .with_batch_size(64);

        let features = ctx.to_feature_vector();

        assert!((features[0] - 0.5).abs() < 0.001); // load
        assert!((features[1] - 1.0).abs() < 0.001); // latency normalized
        assert!((features[2] - 1.0).abs() < 0.001); // memory normalized
    }

    #[test]
    fn test_custom_feature() {
        let ctx = Context::new()
            .with_feature("queue_depth", ContextFeature::Numeric(42.0))
            .with_feature("is_priority", ContextFeature::Boolean(true));

        assert!(ctx.features.contains_key("queue_depth"));
        assert!(ctx.features.contains_key("is_priority"));
    }
}

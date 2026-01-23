//! Variant and Catalog data structures
//!
//! Defines the schema for approved variants and the catalog that holds them.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a variant
pub type VariantId = u32;

/// Constraints that a variant must satisfy
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VariantConstraints {
    /// Maximum latency p99 in microseconds (None = no limit)
    pub max_latency_p99_us: Option<u64>,
    /// Maximum memory usage in bytes (None = no limit)
    pub max_memory_bytes: Option<u64>,
    /// Minimum accuracy/quality score (None = no limit)
    pub min_quality_score: Option<f64>,
    /// Maximum cost per request (None = no limit)
    pub max_cost_per_request: Option<f64>,
    /// Required GPU types (empty = any)
    pub required_gpu_types: Vec<String>,
}

impl VariantConstraints {
    /// Check if constraints are satisfied given current metrics
    pub fn is_satisfied(
        &self,
        latency_p99_us: u64,
        memory_bytes: u64,
        quality: f64,
        cost: f64,
        gpu_type: &str,
    ) -> bool {
        if let Some(max) = self.max_latency_p99_us {
            if latency_p99_us > max {
                return false;
            }
        }
        if let Some(max) = self.max_memory_bytes {
            if memory_bytes > max {
                return false;
            }
        }
        if let Some(min) = self.min_quality_score {
            if quality < min {
                return false;
            }
        }
        if let Some(max) = self.max_cost_per_request {
            if cost > max {
                return false;
            }
        }
        if !self.required_gpu_types.is_empty()
            && !self.required_gpu_types.contains(&gpu_type.to_string())
        {
            return false;
        }
        true
    }
}

/// A single approved variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variant {
    /// Unique identifier
    pub id: VariantId,
    /// Human-readable name
    pub name: String,
    /// Version string
    pub version: String,
    /// Type of variant (quantization, kernel, adapter, etc.)
    pub variant_type: VariantType,
    /// Constraints this variant can satisfy
    pub constraints: VariantConstraints,
    /// Expected performance characteristics
    pub expected_latency_us: u64,
    /// Whether this is the default/fallback variant
    pub is_default: bool,
    /// Arbitrary metadata
    pub metadata: HashMap<String, String>,
}

/// Types of variants supported
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VariantType {
    /// Quantization profile (fp16, int8, int4)
    Quantization,
    /// Compiled kernel variant
    Kernel,
    /// Early-exit depth profile
    EarlyExit,
    /// Adapter bundle (LoRA sets)
    Adapter,
    /// MoE routing policy
    RoutingPolicy,
    /// KV-cache eviction strategy
    CachePolicy,
    /// Custom variant type
    Custom(String),
}

/// Catalog of approved variants
#[derive(Debug)]
pub struct VariantCatalog {
    variants: HashMap<VariantId, Variant>,
    by_type: HashMap<VariantType, Vec<VariantId>>,
    default_id: Option<VariantId>,
    next_id: VariantId,
}

impl VariantCatalog {
    /// Create a new empty catalog
    pub fn new() -> Self {
        Self {
            variants: HashMap::new(),
            by_type: HashMap::new(),
            default_id: None,
            next_id: 1,
        }
    }

    /// Add a variant to the catalog
    pub fn add(&mut self, mut variant: Variant) -> VariantId {
        let id = self.next_id;
        self.next_id += 1;
        variant.id = id;

        if variant.is_default {
            self.default_id = Some(id);
        }

        self.by_type
            .entry(variant.variant_type.clone())
            .or_default()
            .push(id);

        self.variants.insert(id, variant);
        id
    }

    /// Get a variant by ID
    pub fn get(&self, id: VariantId) -> Option<&Variant> {
        self.variants.get(&id)
    }

    /// Get the default variant
    pub fn default_variant(&self) -> Option<&Variant> {
        self.default_id.and_then(|id| self.variants.get(&id))
    }

    /// Get all variants of a specific type
    pub fn by_type(&self, variant_type: &VariantType) -> Vec<&Variant> {
        self.by_type
            .get(variant_type)
            .map(|ids| ids.iter().filter_map(|id| self.variants.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get all variant IDs
    pub fn all_ids(&self) -> Vec<VariantId> {
        self.variants.keys().copied().collect()
    }

    /// Number of variants in catalog
    pub fn len(&self) -> usize {
        self.variants.len()
    }

    /// Check if catalog is empty
    pub fn is_empty(&self) -> bool {
        self.variants.is_empty()
    }

    /// Filter variants that satisfy given constraints
    pub fn filter_eligible(
        &self,
        latency_budget_us: u64,
        memory_budget_bytes: u64,
        min_quality: f64,
        cost_budget: f64,
        gpu_type: &str,
    ) -> Vec<VariantId> {
        self.variants
            .values()
            .filter(|v| {
                v.constraints.is_satisfied(
                    latency_budget_us,
                    memory_budget_bytes,
                    min_quality,
                    cost_budget,
                    gpu_type,
                )
            })
            .map(|v| v.id)
            .collect()
    }
}

impl Default for VariantCatalog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_variant(name: &str, is_default: bool) -> Variant {
        Variant {
            id: 0, // Will be assigned by catalog
            name: name.to_string(),
            version: "1.0".to_string(),
            variant_type: VariantType::Quantization,
            constraints: VariantConstraints::default(),
            expected_latency_us: 1000,
            is_default,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_catalog_add_and_get() {
        let mut catalog = VariantCatalog::new();

        let v1 = create_test_variant("fp16", true);
        let id1 = catalog.add(v1);

        let v2 = create_test_variant("int8", false);
        let id2 = catalog.add(v2);

        assert_eq!(catalog.len(), 2);
        assert_eq!(catalog.get(id1).unwrap().name, "fp16");
        assert_eq!(catalog.get(id2).unwrap().name, "int8");
    }

    #[test]
    fn test_catalog_default() {
        let mut catalog = VariantCatalog::new();

        catalog.add(create_test_variant("fp16", true));
        catalog.add(create_test_variant("int8", false));

        let default = catalog.default_variant().unwrap();
        assert_eq!(default.name, "fp16");
    }

    #[test]
    fn test_catalog_by_type() {
        let mut catalog = VariantCatalog::new();

        let mut v1 = create_test_variant("fp16", false);
        v1.variant_type = VariantType::Quantization;
        catalog.add(v1);

        let mut v2 = create_test_variant("flash_attn", false);
        v2.variant_type = VariantType::Kernel;
        catalog.add(v2);

        let quants = catalog.by_type(&VariantType::Quantization);
        assert_eq!(quants.len(), 1);
        assert_eq!(quants[0].name, "fp16");
    }

    #[test]
    fn test_constraint_satisfaction() {
        let constraints = VariantConstraints {
            max_latency_p99_us: Some(1000),
            max_memory_bytes: Some(1_000_000),
            min_quality_score: Some(0.9),
            max_cost_per_request: Some(0.01),
            required_gpu_types: vec!["a100".to_string()],
        };

        // All satisfied
        assert!(constraints.is_satisfied(500, 500_000, 0.95, 0.005, "a100"));

        // Latency too high
        assert!(!constraints.is_satisfied(1500, 500_000, 0.95, 0.005, "a100"));

        // Wrong GPU
        assert!(!constraints.is_satisfied(500, 500_000, 0.95, 0.005, "t4"));
    }
}

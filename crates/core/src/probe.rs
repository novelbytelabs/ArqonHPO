use crate::config::{Scale, SolverConfig};
use crate::rng::get_rng;
use rand::Rng;
use std::collections::HashMap;

/// Result of a probe generation: a list of candidate parameters.
pub type Candidates = Vec<HashMap<String, f64>>;

pub trait Probe: Send + Sync {
    fn sample(&self, config: &SolverConfig) -> Candidates;
}

/// A deterministic Uniform Random probe.
///
/// Replaces Sobol for MVP to minimize dependencies while maintaining strict determinism.
pub struct UniformProbe;

impl Probe for UniformProbe {
    fn sample(&self, config: &SolverConfig) -> Candidates {
        let mut rng = get_rng(config.seed);
        let num_samples = (config.budget as f64 * config.probe_ratio).ceil() as usize;
        let mut candidates = Vec::with_capacity(num_samples);

        for _ in 0..num_samples {
            let mut point = HashMap::new();
            for (name, domain) in &config.bounds {
                let val = match domain.scale {
                    Scale::Linear | Scale::Periodic => rng.random_range(domain.min..=domain.max),
                    Scale::Log => {
                        // linear sample in log space
                        let min_log = domain.min.ln();
                        let max_log = domain.max.ln();
                        let s = rng.random_range(min_log..=max_log);
                        s.exp()
                    }
                };
                point.insert(name.clone(), val);
            }
            candidates.push(point);
        }
        candidates
    }
}

// ============================================================================
// Prime-Index Probe (PCR Algorithm)
// ============================================================================

/// Prime-Index Probe for multi-scale structure detection (PCR methodology).
///
/// Samples at prime ratios (2/N, 3/N, 5/N, 7/N, ...) to avoid aliasing
/// and provide coverage across multiple scales simultaneously.
///
/// This is superior to uniform random sampling for detecting landscape structure
/// because prime ratios are mutually coprime and don't share common harmonics.
#[derive(Default)]
pub struct PrimeIndexProbe {
    /// Max number of primes to use (default: use as many as needed for sample count)
    pub max_primes: Option<usize>,
}

impl PrimeIndexProbe {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with limited number of primes
    pub fn with_max_primes(max_primes: usize) -> Self {
        Self {
            max_primes: Some(max_primes),
        }
    }

    /// Generate primes up to limit using Sieve of Eratosthenes
    pub fn sieve_of_eratosthenes(limit: usize) -> Vec<usize> {
        if limit < 2 {
            return vec![];
        }

        let mut is_prime = vec![true; limit + 1];
        is_prime[0] = false;
        is_prime[1] = false;

        let sqrt_limit = (limit as f64).sqrt() as usize;
        for i in 2..=sqrt_limit {
            if is_prime[i] {
                for j in ((i * i)..=limit).step_by(i) {
                    is_prime[j] = false;
                }
            }
        }

        is_prime
            .iter()
            .enumerate()
            .filter_map(|(i, &prime)| if prime { Some(i) } else { None })
            .collect()
    }

    /// Get first n primes
    pub fn first_n_primes(n: usize) -> Vec<usize> {
        if n == 0 {
            return vec![];
        }

        // Estimate upper bound using prime number theorem: p_n ~ n * ln(n)
        let upper_bound = if n < 6 {
            15
        } else {
            let n_f = n as f64;
            (n_f * (n_f.ln() + n_f.ln().ln() + 2.0)) as usize
        };

        let primes = Self::sieve_of_eratosthenes(upper_bound);
        primes.into_iter().take(n).collect()
    }

    /// Generate sample positions using prime ratios
    fn generate_prime_positions(&self, num_samples: usize) -> Vec<f64> {
        let primes = match self.max_primes {
            Some(max) => Self::first_n_primes(max.min(num_samples)),
            None => Self::first_n_primes(num_samples),
        };

        // Use prime ratios: p_i / N where N is a large base
        let n = 1000.0; // Base resolution
        primes
            .iter()
            .map(|&p| (p as f64 / n) % 1.0) // Normalize to [0, 1)
            .collect()
    }
}

impl Probe for PrimeIndexProbe {
    fn sample(&self, config: &SolverConfig) -> Candidates {
        let mut rng = get_rng(config.seed);
        let num_samples = (config.budget as f64 * config.probe_ratio).ceil() as usize;

        // Generate prime-indexed positions for each dimension
        let positions = self.generate_prime_positions(num_samples);

        // Sort dimension keys for deterministic ordering
        let mut keys: Vec<_> = config.bounds.keys().cloned().collect();
        keys.sort();

        let mut candidates = Vec::with_capacity(num_samples);

        for (i, &pos) in positions.iter().enumerate() {
            let mut point = HashMap::new();

            for (dim_idx, name) in keys.iter().enumerate() {
                if let Some(domain) = config.bounds.get(name) {
                    // Use different prime-indexed offset for each dimension
                    // This provides multi-scale coverage across all dimensions
                    let dim_offset = (dim_idx + 1) as f64 * 0.618033988749895; // Golden ratio offset
                    let adjusted_pos = (pos + dim_offset * (i as f64 / num_samples as f64)) % 1.0;

                    let val = match domain.scale {
                        Scale::Linear | Scale::Periodic => {
                            domain.min + adjusted_pos * (domain.max - domain.min)
                        }
                        Scale::Log => {
                            let min_log = domain.min.ln();
                            let max_log = domain.max.ln();
                            (min_log + adjusted_pos * (max_log - min_log))
                                .exp()
                                .clamp(domain.min, domain.max)
                        }
                    };
                    point.insert(name.clone(), val);
                }
            }
            candidates.push(point);
        }

        // Add small random perturbation for robustness (optional)
        // This prevents exact aliasing while maintaining the multi-scale property
        for candidate in candidates.iter_mut() {
            for (name, value) in candidate.iter_mut() {
                if let Some(domain) = config.bounds.get(name) {
                    let range = domain.max - domain.min;
                    let perturbation = rng.random_range(-0.01..=0.01) * range;
                    *value = (*value + perturbation).clamp(domain.min, domain.max);
                }
            }
        }

        candidates
    }
}

// ============================================================================
// Prime-Sqrt-Slopes-Rot Probe (Validated Kronecker Sequence)
// ============================================================================

/// Configuration for PrimeSqrtSlopesRotProbe
#[derive(Debug, Clone)]
pub struct PrimeSqrtSlopesRotConfig {
    /// Starting index in prime table for slope primes (default: 50)
    pub prime_offset: usize,
    /// Starting index in prime table for rotation primes (default: 200)  
    pub rot_offset: usize,
    /// Irrational multiplier for rotation phase (default: sqrt(2) - 1)
    pub rot_alpha: f64,
    /// Fraction of samples to replace with random points for multimodal robustness (default: 0.1)
    pub random_spice_ratio: f64,
    /// Cranley-Patterson shift vector (randomized QMC)
    pub cp_shift: Option<Vec<f64>>,
}

impl PrimeSqrtSlopesRotConfig {
    /// Compute adaptive spice ratio based on landscape classification
    ///
    /// - Structured: 1% spice (minimal random noise, rely on CP shift)
    /// - Chaotic: 20% spice (moderate exploration)
    /// - Unknown: 10% spice (balanced default)
    pub fn adaptive_spice_for_landscape(is_chaotic: bool) -> f64 {
        if is_chaotic {
            0.20
        } else {
            0.00
        }
    }

    /// Create config with custom spice ratio
    pub fn with_spice(spice_ratio: f64) -> Self {
        Self {
            random_spice_ratio: spice_ratio.clamp(0.0, 1.0),
            ..Self::default()
        }
    }

    /// Create config with CP shift
    pub fn with_cp_shift(mut self, shift: Vec<f64>) -> Self {
        self.cp_shift = Some(shift);
        self
    }
}

impl Default for PrimeSqrtSlopesRotConfig {
    fn default() -> Self {
        Self {
            prime_offset: 50,
            rot_offset: 200,
            rot_alpha: std::f64::consts::SQRT_2 - 1.0, // ≈ 0.4142...
            random_spice_ratio: 0.1,                   // 10% random points for multimodal hedge
            cp_shift: None,
        }
    }
}

/// Prime-Sqrt-Slopes-Rot Probe (Kronecker/Weyl sequence).
///
/// Generates low-discrepancy samples using:
/// ```text
/// x_{i,d} = frac(i * sqrt(p_{d+prime_offset}) + frac(p_{d+rot_offset} * rot_alpha))
/// ```
///
/// **Key properties:**
/// - **Anytime**: No dependence on total N (prefixes are meaningful)
/// - **Deterministic**: Same seed produces identical samples
/// - **Low discrepancy**: Competitive with scrambled Sobol/Halton
/// - **No collisions**: Unlike p/1000, uses irrational slopes
/// - **No striping**: Per-dimension independent phases
///
/// This is the recommended probe for ArqonHPO per the Prime-Driven
/// Low-Discrepancy Sampling technical specification.
pub struct PrimeSqrtSlopesRotProbe {
    config: PrimeSqrtSlopesRotConfig,
    /// Seed-based global rotation (differentiates runs while preserving determinism)
    seed_rotation: f64,
}

impl Default for PrimeSqrtSlopesRotProbe {
    fn default() -> Self {
        Self::new()
    }
}

impl PrimeSqrtSlopesRotProbe {
    pub fn new() -> Self {
        Self {
            config: PrimeSqrtSlopesRotConfig::default(),
            seed_rotation: 0.0,
        }
    }

    pub fn with_config(config: PrimeSqrtSlopesRotConfig) -> Self {
        Self {
            config,
            seed_rotation: 0.0,
        }
    }

    pub fn with_seed(seed: u64) -> Self {
        // Derive a [0, 1) rotation from the seed for run differentiation
        // Using a simple hash-like transformation
        let seed_rot = ((seed as f64) * std::f64::consts::FRAC_1_PI) % 1.0;
        Self {
            config: PrimeSqrtSlopesRotConfig::default(),
            seed_rotation: seed_rot,
        }
    }

    pub fn with_seed_and_config(seed: u64, config: PrimeSqrtSlopesRotConfig) -> Self {
        let seed_rot = ((seed as f64) * std::f64::consts::FRAC_1_PI) % 1.0;
        Self {
            config,
            seed_rotation: seed_rot,
        }
    }

    /// Prepare geometry (primes, slopes, rotations, sorted keys) for the given config
    fn prepare_geometry(
        &self,
        config: &SolverConfig,
    ) -> (Vec<usize>, Vec<f64>, Vec<f64>, Vec<String>) {
        // Sort dimension keys for deterministic ordering
        let mut keys: Vec<_> = config.bounds.keys().cloned().collect();
        keys.sort();
        let num_dims = keys.len();

        // Generate enough primes for all dimensions
        let primes_needed = self.config.rot_offset.max(self.config.prime_offset) + num_dims + 10;
        let primes = PrimeIndexProbe::first_n_primes(primes_needed);

        let slopes: Vec<f64> = (0..num_dims)
            .map(|d| {
                let prime_idx = self.config.prime_offset + d;
                let prime = primes
                    .get(prime_idx)
                    .copied()
                    .unwrap_or(primes.last().copied().unwrap_or(2));
                (prime as f64).sqrt()
            })
            .collect();

        let rotations: Vec<f64> = (0..num_dims)
            .map(|d| {
                let prime_idx = self.config.rot_offset + d;
                let prime = primes
                    .get(prime_idx)
                    .copied()
                    .unwrap_or(primes.last().copied().unwrap_or(2));
                (prime as f64 * self.config.rot_alpha) % 1.0
            })
            .collect();

        (primes, slopes, rotations, keys)
    }

    /// Generate a single pure LDS point at the given global index (Sharding API)
    ///
    /// This is stateless, deterministic, and collision-free.
    /// Does NOT include anchors, spice, or CP shift.
    pub fn sample_at(&self, index: usize, config: &SolverConfig) -> HashMap<String, f64> {
        let (_, slopes, rotations, keys) = self.prepare_geometry(config);
        self.generate_point_at(index, &keys, &slopes, &rotations, config)
    }

    /// Generate a range of pure LDS points [start, start+count) (Sharding API)
    ///
    /// This is the preferred method for workers to request a shard of trials.
    pub fn sample_range(
        &self,
        start: usize,
        count: usize,
        config: &SolverConfig,
    ) -> Vec<HashMap<String, f64>> {
        let (_, slopes, rotations, keys) = self.prepare_geometry(config);
        (0..count)
            .map(|offset| {
                self.generate_point_at(start + offset, &keys, &slopes, &rotations, config)
            })
            .collect()
    }

    fn generate_point_at(
        &self,
        i: usize,
        keys: &[String],
        slopes: &[f64],
        rotations: &[f64],
        config: &SolverConfig,
    ) -> HashMap<String, f64> {
        let mut point = HashMap::new();

        for (dim_idx, name) in keys.iter().enumerate() {
            if let Some(domain) = config.bounds.get(name) {
                // Fast unit_value from precomputed slopes/rotations
                let unit_pos =
                    ((i + 1) as f64 * slopes[dim_idx] + rotations[dim_idx] + self.seed_rotation)
                        % 1.0;
                let unit_pos = if unit_pos < 0.0 {
                    unit_pos + 1.0
                } else {
                    unit_pos
                };

                let val = match domain.scale {
                    Scale::Linear | Scale::Periodic => {
                        domain.min + unit_pos * (domain.max - domain.min)
                    }
                    Scale::Log => {
                        let min_log = domain.min.ln();
                        let max_log = domain.max.ln();
                        // Clamp to handle floating-point precision (fixes TD-002)
                        (min_log + unit_pos * (max_log - min_log))
                            .exp()
                            .clamp(domain.min, domain.max)
                    }
                };
                point.insert(name.clone(), val);
            }
        }
        point
    }
}

impl Probe for PrimeSqrtSlopesRotProbe {
    fn sample(&self, config: &SolverConfig) -> Candidates {
        let (_, slopes, rotations, keys) = self.prepare_geometry(config);

        // Calculate budget based on config
        let num_samples = (config.budget as f64 * config.probe_ratio).ceil() as usize;

        // Determine how many points to spice with random
        let num_random = (num_samples as f64 * self.config.random_spice_ratio).floor() as usize;

        // Reserve space for deterministic anchors (Origin + Center)
        let num_anchors = 2;
        let num_qmc = num_samples.saturating_sub(num_random + num_anchors);

        // PHASE 6: Apply Cranley-Patterson shift if provided in config
        // "Structured Primary: Δ=0" -> Config will have cp_shift = None (or Some(dataset to 0))
        // "Chaotic/Fallback: Δ=random" -> Config will have cp_shift = Some(random)
        let cp_delta = self
            .config
            .cp_shift
            .clone()
            .unwrap_or_else(|| vec![0.0; keys.len()]);

        let mut candidates = Vec::with_capacity(num_samples);

        // 1. Inject Deterministic Anchors (Origin + Center)
        let anchors_unit = [0.0, 0.5];
        for unit_pos in anchors_unit {
            let mut point = HashMap::new();
            for name in keys.iter() {
                if let Some(domain) = config.bounds.get(name) {
                    let val = match domain.scale {
                        Scale::Linear | Scale::Periodic => {
                            domain.min + unit_pos * (domain.max - domain.min)
                        }
                        Scale::Log => {
                            let min_log = domain.min.ln();
                            let max_log = domain.max.ln();
                            (min_log + unit_pos * (max_log - min_log))
                                .exp()
                                .clamp(domain.min, domain.max)
                        }
                    };
                    point.insert(name.clone(), val);
                }
            }
            candidates.push(point);
            if candidates.len() >= num_samples {
                break;
            }
        }

        // 2. Generate QMC (prime-sqrt-slopes-rot) points using precomputed values
        for i in 0..num_qmc {
            let mut point = HashMap::new();

            for (dim_idx, name) in keys.iter().enumerate() {
                if let Some(domain) = config.bounds.get(name) {
                    // Fast unit_value using precomputed slopes/rotations
                    let base_pos = ((i + 1) as f64 * slopes[dim_idx]
                        + rotations[dim_idx]
                        + self.seed_rotation)
                        % 1.0;

                    // Apply Cranley-Patterson shift for randomized QMC
                    let shifted_pos = (base_pos + cp_delta[dim_idx]).fract();
                    let unit_pos = if shifted_pos < 0.0 {
                        shifted_pos + 1.0
                    } else {
                        shifted_pos
                    };

                    let val = match domain.scale {
                        Scale::Linear | Scale::Periodic => {
                            domain.min + unit_pos * (domain.max - domain.min)
                        }
                        Scale::Log => {
                            let min_log = domain.min.ln();
                            let max_log = domain.max.ln();
                            (min_log + unit_pos * (max_log - min_log))
                                .exp()
                                .clamp(domain.min, domain.max)
                        }
                    };
                    point.insert(name.clone(), val);
                }
            }
            candidates.push(point);
        }

        // Add random spice points for multimodal robustness
        // Use seed_rotation to derive deterministic random seed
        let random_seed = (self.seed_rotation * 1e9) as u64;
        use rand::SeedableRng;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(random_seed);

        for _ in 0..num_random {
            let mut point = HashMap::new();
            for name in keys.iter() {
                if let Some(domain) = config.bounds.get(name) {
                    let unit_pos: f64 = rng.random();
                    let val = match domain.scale {
                        Scale::Linear | Scale::Periodic => {
                            domain.min + unit_pos * (domain.max - domain.min)
                        }
                        Scale::Log => {
                            let min_log = domain.min.ln();
                            let max_log = domain.max.ln();
                            (min_log + unit_pos * (max_log - min_log))
                                .exp()
                                .clamp(domain.min, domain.max)
                        }
                    };
                    point.insert(name.clone(), val);
                }
            }
            candidates.push(point);
        }

        candidates
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Domain;

    fn test_config() -> SolverConfig {
        let mut bounds = HashMap::new();
        bounds.insert(
            "x".to_string(),
            Domain {
                min: -5.0,
                max: 5.0,
                scale: Scale::Linear,
            },
        );

        SolverConfig {
            bounds,
            budget: 50,
            seed: 42,
            probe_ratio: 0.2,
            strategy_params: None,
        }
    }

    #[test]
    fn test_sieve_of_eratosthenes() {
        let primes = PrimeIndexProbe::sieve_of_eratosthenes(30);
        assert_eq!(primes, vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29]);
    }

    #[test]
    fn test_first_n_primes() {
        let primes = PrimeIndexProbe::first_n_primes(5);
        assert_eq!(primes, vec![2, 3, 5, 7, 11]);
    }

    #[test]
    fn test_prime_index_probe_deterministic() {
        let config = test_config();
        let probe = PrimeIndexProbe::default();

        let samples1 = probe.sample(&config);
        let samples2 = probe.sample(&config);

        assert_eq!(samples1.len(), samples2.len());

        for (s1, s2) in samples1.iter().zip(samples2.iter()) {
            let x1 = s1.get("x").unwrap();
            let x2 = s2.get("x").unwrap();
            assert!(
                (x1 - x2).abs() < 1e-10,
                "Same seed should produce same samples"
            );
        }
    }

    #[test]
    fn test_prime_index_probe_respects_bounds() {
        let config = test_config();
        let probe = PrimeIndexProbe::default();

        let samples = probe.sample(&config);

        for sample in samples {
            let x = sample.get("x").unwrap();
            assert!(*x >= -5.0 && *x <= 5.0, "Sample should be within bounds");
        }
    }

    #[test]
    fn test_prime_index_multi_scale_coverage() {
        // Prime ratios should not alias - check that samples are spread across range
        let config = test_config();
        let probe = PrimeIndexProbe::default();

        let samples = probe.sample(&config);
        let values: Vec<f64> = samples.iter().map(|s| *s.get("x").unwrap()).collect();

        // Check samples cover multiple regions
        let min_val = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_val = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // Should cover at least 50% of the range
        let coverage = (max_val - min_val) / 10.0; // 10.0 is the total range
        assert!(
            coverage > 0.5,
            "Prime samples should cover at least 50% of range"
        );
    }

    // ============================================================================
    // NEW: PrimeSqrtSlopesRotProbe Tests (Validated Algorithm)
    // ============================================================================

    fn test_config_multi_dim() -> SolverConfig {
        let mut bounds = HashMap::new();
        bounds.insert(
            "x".to_string(),
            Domain {
                min: 0.0,
                max: 1.0,
                scale: Scale::Linear,
            },
        );
        bounds.insert(
            "y".to_string(),
            Domain {
                min: 0.0,
                max: 1.0,
                scale: Scale::Linear,
            },
        );

        SolverConfig {
            bounds,
            budget: 256,
            seed: 42,
            probe_ratio: 1.0, // Use full budget for probe
            strategy_params: None,
        }
    }

    #[test]
    fn test_prime_sqrt_slopes_rot_deterministic() {
        let config = test_config();
        let probe = PrimeSqrtSlopesRotProbe::new();

        let samples1 = probe.sample(&config);
        let samples2 = probe.sample(&config);

        assert_eq!(samples1.len(), samples2.len());

        for (s1, s2) in samples1.iter().zip(samples2.iter()) {
            let x1 = s1.get("x").unwrap();
            let x2 = s2.get("x").unwrap();
            assert!(
                (x1 - x2).abs() < 1e-10,
                "Same config should produce identical samples"
            );
        }
    }

    #[test]
    fn test_prime_sqrt_slopes_rot_respects_bounds() {
        let config = test_config();
        let probe = PrimeSqrtSlopesRotProbe::new();

        let samples = probe.sample(&config);

        for sample in samples {
            let x = sample.get("x").unwrap();
            assert!(
                *x >= -5.0 && *x <= 5.0,
                "Sample should be within bounds: got {}",
                x
            );
        }
    }

    #[test]
    fn test_prime_sqrt_slopes_rot_no_collisions() {
        // At N=256, the new probe should have NO collisions (unlike legacy p/1000)
        let config = test_config_multi_dim();
        let probe = PrimeSqrtSlopesRotProbe::new();

        let samples = probe.sample(&config);

        // Extract normalized positions (already in [0,1] due to config)
        let positions: Vec<(f64, f64)> = samples
            .iter()
            .map(|s| (*s.get("x").unwrap(), *s.get("y").unwrap()))
            .collect();

        // Check for near-duplicate positions (collision threshold: 0.001)
        let thresh = 0.001;
        let mut collision_count = 0;
        for i in 0..positions.len() {
            for j in (i + 1)..positions.len() {
                let dist_sq = (positions[i].0 - positions[j].0).powi(2)
                    + (positions[i].1 - positions[j].1).powi(2);
                if dist_sq < thresh * thresh {
                    collision_count += 1;
                }
            }
        }

        // No collisions expected with irrational slopes
        assert_eq!(
            collision_count, 0,
            "PrimeSqrtSlopesRotProbe should have no near-collisions, found {}",
            collision_count
        );
    }

    #[test]
    fn test_prime_sqrt_slopes_rot_good_coverage() {
        // Samples should be spread across the unit square
        let config = test_config_multi_dim();
        let probe = PrimeSqrtSlopesRotProbe::new();

        let samples = probe.sample(&config);

        // Check that all quadrants have samples
        let mut quadrants = [0u32; 4];
        for s in &samples {
            let x = *s.get("x").unwrap();
            let y = *s.get("y").unwrap();
            let q = match (x < 0.5, y < 0.5) {
                (true, true) => 0,
                (false, true) => 1,
                (true, false) => 2,
                (false, false) => 3,
            };
            quadrants[q] += 1;
        }

        // Each quadrant should have at least 10% of samples
        let min_expected = (samples.len() / 10) as u32;
        for (i, &count) in quadrants.iter().enumerate() {
            assert!(
                count >= min_expected,
                "Quadrant {} has only {} samples (expected >= {})",
                i,
                count,
                min_expected
            );
        }
    }

    #[test]
    fn test_prime_sqrt_slopes_rot_seed_differentiation() {
        let config = test_config();

        // Fixed TD-001: Compare sum of differences across all samples
        let probe1 = PrimeSqrtSlopesRotProbe::with_seed(42);
        let probe2 = PrimeSqrtSlopesRotProbe::with_seed(12345); // Use more different seed

        let samples1 = probe1.sample(&config);
        let samples2 = probe2.sample(&config);

        // Different seeds should produce different sample sets
        // Compare sum of absolute differences across all samples
        let total_diff: f64 = samples1
            .iter()
            .zip(samples2.iter())
            .map(|(s1, s2)| {
                let x1 = *s1.get("x").unwrap();
                let x2 = *s2.get("x").unwrap();
                (x1 - x2).abs()
            })
            .sum();

        assert!(
            total_diff > 0.1,
            "Different seeds should produce different samples (total diff: {})",
            total_diff
        );
    }

    #[test]
    fn test_prime_sqrt_slopes_rot_log_scale() {
        let mut bounds = HashMap::new();
        bounds.insert(
            "lr".to_string(),
            Domain {
                min: 1e-5,
                max: 1e-1,
                scale: Scale::Log,
            },
        );

        let config = SolverConfig {
            bounds,
            budget: 100,
            seed: 42,
            probe_ratio: 0.5,
            strategy_params: None,
        };

        let probe = PrimeSqrtSlopesRotProbe::new();
        let samples = probe.sample(&config);

        // Fixed TD-002: Log-scale sampling now uses clamp() to handle floating-point precision
        for sample in samples {
            let lr = *sample.get("lr").unwrap();
            assert!(
                lr >= 1e-5 && lr <= 1e-1,
                "Log-scale sample should be within bounds: got {}",
                lr
            );
        }
    }

    #[test]
    fn test_prime_index_probe_with_max_primes() {
        let probe = PrimeIndexProbe::with_max_primes(5);
        assert_eq!(probe.max_primes, Some(5));

        let config = test_config();
        let samples = probe.sample(&config);
        assert!(!samples.is_empty());
    }

    #[test]
    fn test_uniform_probe_log_scale() {
        // Test UniformProbe with Log scale
        let mut bounds = HashMap::new();
        bounds.insert(
            "lr".to_string(),
            Domain {
                min: 1e-4,
                max: 1e-1,
                scale: Scale::Log,
            },
        );

        let config = SolverConfig {
            bounds,
            budget: 50,
            seed: 42,
            probe_ratio: 0.2,
            strategy_params: None,
        };

        let probe = UniformProbe;
        let samples = probe.sample(&config);

        for sample in samples {
            let lr = *sample.get("lr").unwrap();
            assert!(
                lr >= 1e-4 && lr <= 1e-1,
                "UniformProbe log-scale sample should be within bounds: got {}",
                lr
            );
        }
    }

    #[test]
    fn test_prime_sqrt_sample_at_sharding_api() {
        // Test the sample_at sharding API
        let probe = PrimeSqrtSlopesRotProbe::with_seed(42);
        let config = test_config();

        // Sample at specific indices
        let point0 = probe.sample_at(0, &config);
        let point10 = probe.sample_at(10, &config);

        // Same index should give same point
        let point0_again = probe.sample_at(0, &config);
        assert_eq!(point0.get("x"), point0_again.get("x"));

        // Different indices should give different points
        assert_ne!(point0.get("x"), point10.get("x"));
    }

    #[test]
    fn test_prime_sqrt_sample_range_sharding_api() {
        // Test the sample_range sharding API
        let probe = PrimeSqrtSlopesRotProbe::with_seed(42);
        let config = test_config();

        let range1 = probe.sample_range(0, 5, &config);
        let range2 = probe.sample_range(5, 5, &config);

        assert_eq!(range1.len(), 5);
        assert_eq!(range2.len(), 5);

        // Non-overlapping ranges should be different
        assert_ne!(range1[0].get("x"), range2[0].get("x"));
    }

    #[test]
    fn test_prime_sqrt_with_config() {
        let custom_config = PrimeSqrtSlopesRotConfig {
            prime_offset: 100,
            rot_offset: 300,
            rot_alpha: 0.5,
            random_spice_ratio: 0.2,
            cp_shift: None,
        };

        let probe = PrimeSqrtSlopesRotProbe::with_config(custom_config);
        let config = test_config();
        let samples = probe.sample(&config);
        assert!(!samples.is_empty());
    }

    #[test]
    fn test_prime_sqrt_with_cp_shift() {
        let base_config = PrimeSqrtSlopesRotConfig::with_spice(0.1);
        let shifted_config = base_config.with_cp_shift(vec![0.25, 0.75]);

        assert!(shifted_config.cp_shift.is_some());
        let shift = shifted_config.cp_shift.unwrap();
        assert_eq!(shift.len(), 2);
        assert!((shift[0] - 0.25).abs() < 1e-10);
    }

    #[test]
    fn test_adaptive_spice_for_landscape() {
        let chaotic_spice = PrimeSqrtSlopesRotConfig::adaptive_spice_for_landscape(true);
        let structured_spice = PrimeSqrtSlopesRotConfig::adaptive_spice_for_landscape(false);

        assert!((chaotic_spice - 0.20).abs() < 1e-10);
        assert!((structured_spice - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_sieve_of_eratosthenes_edge_cases() {
        // Test with limit < 2
        let primes_zero = PrimeIndexProbe::sieve_of_eratosthenes(0);
        assert!(primes_zero.is_empty());

        let primes_one = PrimeIndexProbe::sieve_of_eratosthenes(1);
        assert!(primes_one.is_empty());

        let primes_two = PrimeIndexProbe::sieve_of_eratosthenes(2);
        assert_eq!(primes_two, vec![2]);
    }

    #[test]
    fn test_first_n_primes_edge_cases() {
        let primes_zero = PrimeIndexProbe::first_n_primes(0);
        assert!(primes_zero.is_empty());

        let primes_one = PrimeIndexProbe::first_n_primes(1);
        assert_eq!(primes_one, vec![2]);
    }
}

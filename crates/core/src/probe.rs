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
                    Scale::Linear => rng.random_range(domain.min..=domain.max),
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
// Prime-Index Probe (RPZL Algorithm)
// ============================================================================

/// Prime-Index Probe for multi-scale structure detection (RPZL methodology).
///
/// Samples at prime ratios (2/N, 3/N, 5/N, 7/N, ...) to avoid aliasing
/// and provide coverage across multiple scales simultaneously.
///
/// This is superior to uniform random sampling for detecting landscape structure
/// because prime ratios are mutually coprime and don't share common harmonics.
pub struct PrimeIndexProbe {
    /// Max number of primes to use (default: use as many as needed for sample count)
    pub max_primes: Option<usize>,
}

impl Default for PrimeIndexProbe {
    fn default() -> Self {
        Self { max_primes: None }
    }
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
                        Scale::Linear => {
                            domain.min + adjusted_pos * (domain.max - domain.min)
                        }
                        Scale::Log => {
                            let min_log = domain.min.ln();
                            let max_log = domain.max.ln();
                            (min_log + adjusted_pos * (max_log - min_log)).exp()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Domain;

    fn test_config() -> SolverConfig {
        let mut bounds = HashMap::new();
        bounds.insert("x".to_string(), Domain {
            min: -5.0,
            max: 5.0,
            scale: Scale::Linear,
        });
        
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
            assert!((x1 - x2).abs() < 1e-10, "Same seed should produce same samples");
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
        assert!(coverage > 0.5, "Prime samples should cover at least 50% of range");
    }
}

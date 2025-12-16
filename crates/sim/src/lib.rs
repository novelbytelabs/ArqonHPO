use arqonhpo_core::variant_catalog::Variant;
use std::collections::HashMap;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand::Rng;

pub const ROWS: usize = 64;
pub const COLS: usize = 64;

/// A simulated workload representing the "Answer-Emergent Universe"
pub struct Universe {
    pub grid: Vec<Vec<f64>>,
    pub rng: ChaCha8Rng,
    // Tunable parameters
    pub diffusion_rate: f64,
    pub noise_level: f64,
    // Rule configuration
    pub kernel_size: usize,
}

impl Universe {
    pub fn new(seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut grid = vec![vec![0.0; COLS]; ROWS];
        
        // Initialize with random state
        for r in 0..ROWS {
            for c in 0..COLS {
                // Compatibility for rand 0.9 or 0.8
                grid[r][c] = rng.random();
            }
        }
        
        Self {
            grid,
            rng,
            diffusion_rate: 0.1,
            noise_level: 0.01,
            kernel_size: 1,
        }
    }
    
    /// Apply "physics" parameters from external tuner
    pub fn apply_physics(&mut self, params: &HashMap<String, f64>) {
        if let Some(v) = params.get("diffusion_rate") {
            self.diffusion_rate = *v;
        }
        if let Some(v) = params.get("noise_level") {
            self.noise_level = *v;
        }
    }
    
    /// Apply discrete variant selection
    pub fn apply_variant(&mut self, variant: &Variant) {
        // Mock: extract kernel size from variant metadata or name
        if variant.name.contains("kernel_3x3") {
            self.kernel_size = 1; // Radius 1 = 3x3
        } else if variant.name.contains("kernel_5x5") {
            self.kernel_size = 2; // Radius 2 = 5x5
        } else {
            self.kernel_size = 1;
        }
        // Metadata overrides?
        if let Some(s) = variant.metadata.get("kernel_radius") {
            if let Ok(v) = s.parse() {
                self.kernel_size = v;
            }
        }
    }
    
    /// Step the simulation
    pub fn step(&mut self) -> f64 {
        let mut new_grid = self.grid.clone();
        
        for r in 0..ROWS {
            for c in 0..COLS {
                // Diffusion with configured kernel
                let mut sum = 0.0;
                let mut count = 0.0;
                
                let range = self.kernel_size as isize;
                for dr in -range..=range {
                    for dc in -range..=range {
                        let nr = (r as isize + dr).rem_euclid(ROWS as isize) as usize;
                        let nc = (c as isize + dc).rem_euclid(COLS as isize) as usize;
                        sum += self.grid[nr][nc];
                        count += 1.0;
                    }
                }
                
                let avg = sum / count;
                let current = self.grid[r][c];
                
                // Update rule: diff + noise
                let diff = avg - current;
                let noise: f64 = self.rng.random_range(-self.noise_level..=self.noise_level);
                
                new_grid[r][c] = current + (self.diffusion_rate * diff) + noise;
                new_grid[r][c] = new_grid[r][c].clamp(0.0, 1.0);
            }
        }
        
        // Quality metric: Stability
        let mut total_delta = 0.0;
        for r in 0..ROWS {
            for c in 0..COLS {
                total_delta += (new_grid[r][c] - self.grid[r][c]).abs();
            }
        }
        
        self.grid = new_grid;
        
        // Return 1.0 - mean_delta (normalized stability)
        let mean_delta = total_delta / (ROWS * COLS) as f64;
        (1.0 - mean_delta).max(0.0)
    }

    /// Inject a massive disturbance (random noise) to test recovery
    pub fn inject_shock(&mut self) {
        for r in 0..ROWS {
            for c in 0..COLS {
                self.grid[r][c] = self.rng.random(); // Full randomization
            }
        }
    }
}

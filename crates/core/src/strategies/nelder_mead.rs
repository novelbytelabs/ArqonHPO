use crate::artifact::EvalTrace;
use crate::config::SolverConfig;
use crate::strategies::{Strategy, StrategyAction};
use std::collections::HashMap;

/// Standard Nelder-Mead coefficients per spec clarification (2025-12-14)
pub struct NMCoefficients {
    /// Reflection coefficient (α = 1.0)
    pub alpha: f64,
    /// Expansion coefficient (γ = 2.0)
    pub gamma: f64,
    /// Contraction coefficient (ρ = 0.5)
    pub rho: f64,
    /// Shrink coefficient (σ = 0.5)
    pub sigma: f64,
}

impl Default for NMCoefficients {
    fn default() -> Self {
        Self {
            alpha: 1.0,
            gamma: 2.0,
            rho: 0.5,
            sigma: 0.5,
        }
    }
}

/// Nelder-Mead state machine
#[derive(Debug, Clone)]
enum NMState {
    /// Initial state, needs to build simplex
    Init,
    /// Waiting for reflection point evaluation
    Reflection {
        centroid: Vec<f64>,
        reflection: Vec<f64>,
        best: f64,
        second_worst: f64,
        worst: f64,
    },
    /// Waiting for expansion point evaluation
    Expansion {
        centroid: Vec<f64>,
        reflection: Vec<f64>,
        expansion: Vec<f64>,
        reflection_value: f64,
    },
    /// Waiting for outside contraction point evaluation
    OutsideContraction {
        centroid: Vec<f64>,
        contraction: Vec<f64>,
        reflection_value: f64,
    },
    /// Waiting for inside contraction point evaluation  
    InsideContraction {
        centroid: Vec<f64>,
        contraction: Vec<f64>,
    },
    /// Shrink: evaluate all shrunk points
    Shrink {
        shrunk_points: Vec<Vec<f64>>,
        shrunk_idx: usize,
    },
    /// Building initial simplex: waiting for vertex evaluations
    SimplexBuild {
        /// Number of vertices already evaluated (excluding anchor)
        evals_received: usize,
    },
    /// Converged
    Converged,
}

pub struct NelderMead {
    dim: usize,
    state: NMState,
    /// Simplex: (value, params_vector)
    simplex: Vec<(f64, Vec<f64>)>,
    /// Coefficients for NM operations
    coeffs: NMCoefficients,
    /// Convergence tolerance (simplex diameter)
    tolerance: f64,
    /// Seed points from probe phase
    seed_points: Option<Vec<HashMap<String, f64>>>,
}

impl NelderMead {
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            state: NMState::Init,
            simplex: Vec::new(),
            coeffs: NMCoefficients::default(),
            tolerance: 1e-8,
            seed_points: None,
        }
    }

    /// Create NM with seed points from probe results (Top-K seeding)
    pub fn with_seed_points(dim: usize, seed_points: Vec<HashMap<String, f64>>) -> Self {
        Self {
            dim,
            state: NMState::Init,
            simplex: Vec::new(),
            coeffs: NMCoefficients::default(),
            tolerance: 1e-8,
            seed_points: Some(seed_points),
        }
    }

    /// Create NM with custom coefficients
    pub fn with_coefficients(dim: usize, coeffs: NMCoefficients) -> Self {
        Self {
            dim,
            state: NMState::Init,
            simplex: Vec::new(),
            coeffs,
            tolerance: 1e-8,
            seed_points: None,
        }
    }

    fn dict_to_vec(&self, params: &HashMap<String, f64>, keys: &[String]) -> Vec<f64> {
        keys.iter()
            .map(|k| *params.get(k).unwrap_or(&0.0))
            .collect()
    }

    fn vec_to_dict(&self, vec: &[f64], keys: &[String]) -> HashMap<String, f64> {
        let mut map = HashMap::new();
        for (i, k) in keys.iter().enumerate() {
            if i < vec.len() {
                map.insert(k.clone(), vec[i]);
            }
        }
        map
    }

    /// Clamp vector to bounds
    fn clamp_to_bounds(&self, vec: &mut [f64], config: &SolverConfig, keys: &[String]) {
        for (i, k) in keys.iter().enumerate() {
            if i < vec.len() {
                if let Some(domain) = config.bounds.get(k) {
                    vec[i] = vec[i].clamp(domain.min, domain.max);
                }
            }
        }
    }

    /// Compute centroid of all points except the worst (last)
    fn compute_centroid(&self) -> Vec<f64> {
        let n = self.dim;
        let mut centroid = vec![0.0; n];
        for simplex_point in self.simplex.iter().take(n) {
            for (j, c) in centroid.iter_mut().enumerate() {
                if j < simplex_point.1.len() {
                    *c += simplex_point.1[j];
                }
            }
        }
        for c in centroid.iter_mut() {
            *c /= n as f64;
        }
        centroid
    }

    /// Compute reflection point: r = c + α*(c - worst)
    fn compute_reflection(&self, centroid: &[f64], worst: &[f64]) -> Vec<f64> {
        centroid
            .iter()
            .zip(worst.iter())
            .map(|(&c, &w)| c + self.coeffs.alpha * (c - w))
            .collect()
    }

    /// Compute expansion point: e = c + γ*(r - c)
    fn compute_expansion(&self, centroid: &[f64], reflection: &[f64]) -> Vec<f64> {
        centroid
            .iter()
            .zip(reflection.iter())
            .map(|(&c, &r)| c + self.coeffs.gamma * (r - c))
            .collect()
    }

    /// Compute outside contraction: c_o = c + ρ*(r - c)
    fn compute_outside_contraction(&self, centroid: &[f64], reflection: &[f64]) -> Vec<f64> {
        centroid
            .iter()
            .zip(reflection.iter())
            .map(|(&c, &r)| c + self.coeffs.rho * (r - c))
            .collect()
    }

    /// Compute inside contraction: c_i = c - ρ*(c - worst) = c + ρ*(worst - c)
    fn compute_inside_contraction(&self, centroid: &[f64], worst: &[f64]) -> Vec<f64> {
        centroid
            .iter()
            .zip(worst.iter())
            .map(|(&c, &w)| c + self.coeffs.rho * (w - c))
            .collect()
    }

    /// Compute shrunk points: x_i = x_best + σ*(x_i - x_best)
    fn compute_shrunk_points(&self) -> Vec<Vec<f64>> {
        let best = &self.simplex[0].1;
        self.simplex
            .iter()
            .skip(1) // Skip best point
            .map(|(_, xi)| {
                best.iter()
                    .zip(xi.iter())
                    .map(|(&b, &x)| b + self.coeffs.sigma * (x - b))
                    .collect()
            })
            .collect()
    }

    /// Check if simplex has converged (diameter < tolerance)
    fn check_convergence(&self) -> bool {
        if self.simplex.len() < 2 {
            return false;
        }
        let best = &self.simplex[0].1;
        let worst = &self.simplex.last().unwrap().1;
        
        // Compute max distance from best to any other point
        let diameter: f64 = best
            .iter()
            .zip(worst.iter())
            .map(|(&b, &w)| (b - w).abs())
            .fold(0.0, f64::max);
        
        diameter < self.tolerance
    }

    /// Sort simplex by objective value (ascending - minimization)
    fn sort_simplex(&mut self) {
        self.simplex.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    }
}

impl Strategy for NelderMead {
    fn step(&mut self, config: &SolverConfig, history: &[EvalTrace]) -> StrategyAction {
        // Collect keys for deterministic ordering
        let mut keys: Vec<String> = config.bounds.keys().cloned().collect();
        keys.sort();
        self.dim = keys.len();
        let n = self.dim;

        match &self.state {
            NMState::Init => {
                // Build simplex from N+1 best points in history
                let mut sorted: Vec<_> = history.iter().collect();
                sorted.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap_or(std::cmp::Ordering::Equal));

                if sorted.len() < n + 1 {
                    return StrategyAction::Wait;
                }

                self.simplex.clear();
                
                // PHASE 3: Use regular axis-aligned simplex around best seed (not probe points)
                // This avoids clustered/degenerate simplices from farthest-point selection
                
                // 1. Get best point as anchor
                let best_trace = &sorted[0];
                let best_vec = self.dict_to_vec(&best_trace.params, &keys);
                self.simplex.push((best_trace.value, best_vec.clone()));
                
                // 2. Build N additional vertices: anchor + scale * e_k (axis-aligned)
                // Scale: 0.05 in unit space (given mapping real=(u*2-1)*5, this is ~0.5 in real space)
                let scale = 0.05;
                
                for dim_idx in 0..n {
                    let mut vertex = best_vec.clone();
                    
                    // Perturb in positive direction, clamped to [0, 1]
                    let new_val = (vertex[dim_idx] + scale).min(1.0);
                    
                    // If we hit upper bound, try negative direction
                    if (new_val - vertex[dim_idx]).abs() < 1e-6 {
                        vertex[dim_idx] = (vertex[dim_idx] - scale).max(0.0);
                    } else {
                        vertex[dim_idx] = new_val;
                    }
                    
                    // We need to evaluate this point - add placeholder value (will be updated)
                    // Actually, we should request evaluation of these points first
                    self.simplex.push((f64::INFINITY, vertex));
                }
                
                // Request evaluation of the N new simplex vertices (not the anchor, it's from history)
                let new_vertices: Vec<_> = self.simplex.iter()
                    .skip(1) // Skip anchor (already evaluated in history)
                    .map(|(_, v)| self.vec_to_dict(v, &keys))
                    .collect();
                
                if !new_vertices.is_empty() {
                    // Transition to SimplexBuild state to receive evaluations
                    self.state = NMState::SimplexBuild { evals_received: 0 };
                    return StrategyAction::Evaluate(new_vertices);
                }
                
                self.sort_simplex();

                // Check convergence before proceeding
                if self.check_convergence() {
                    self.state = NMState::Converged;
                    return StrategyAction::Converged;
                }

                // Compute reflection point
                let centroid = self.compute_centroid();
                let worst = &self.simplex[n].1;
                let mut reflection = self.compute_reflection(&centroid, worst);
                self.clamp_to_bounds(&mut reflection, config, &keys);

                let best = self.simplex[0].0;
                let second_worst = self.simplex[n - 1].0;
                let worst_val = self.simplex[n].0;

                self.state = NMState::Reflection {
                    centroid,
                    reflection: reflection.clone(),
                    best,
                    second_worst,
                    worst: worst_val,
                };

                StrategyAction::Evaluate(vec![self.vec_to_dict(&reflection, &keys)])
            }

            NMState::Reflection { centroid, reflection, best, second_worst, worst } => {
                let reflection_val = history.last().map(|t| t.value).unwrap_or(*worst);

                if reflection_val < *best {
                    // Try expansion
                    let mut expansion = self.compute_expansion(centroid, reflection);
                    self.clamp_to_bounds(&mut expansion, config, &keys);

                    self.state = NMState::Expansion {
                        centroid: centroid.clone(),
                        reflection: reflection.clone(),
                        expansion: expansion.clone(),
                        reflection_value: reflection_val,
                    };
                    StrategyAction::Evaluate(vec![self.vec_to_dict(&expansion, &keys)])
                } else if reflection_val < *second_worst {
                    // Accept reflection
                    let n = self.dim;
                    self.simplex[n] = (reflection_val, reflection.clone());
                    self.state = NMState::Init;
                    self.step(config, history) // Immediate restart
                } else if reflection_val < *worst {
                    // Try outside contraction
                    let mut contraction = self.compute_outside_contraction(centroid, reflection);
                    self.clamp_to_bounds(&mut contraction, config, &keys);

                    self.state = NMState::OutsideContraction {
                        centroid: centroid.clone(),
                        contraction: contraction.clone(),
                        reflection_value: reflection_val,
                    };
                    StrategyAction::Evaluate(vec![self.vec_to_dict(&contraction, &keys)])
                } else {
                    // Try inside contraction
                    let worst_pt = &self.simplex[n].1;
                    let mut contraction = self.compute_inside_contraction(centroid, worst_pt);
                    self.clamp_to_bounds(&mut contraction, config, &keys);

                    self.state = NMState::InsideContraction {
                        centroid: centroid.clone(),
                        contraction: contraction.clone(),
                    };
                    StrategyAction::Evaluate(vec![self.vec_to_dict(&contraction, &keys)])
                }
            }

            NMState::Expansion { centroid: _, reflection, expansion: _, reflection_value } => {
                let expansion_val = history.last().map(|t| t.value).unwrap_or(*reflection_value);
                let expansion_pt = history.last().map(|t| self.dict_to_vec(&t.params, &keys)).unwrap_or_else(Vec::new);

                if expansion_val < *reflection_value {
                    // Accept expansion
                    self.simplex[n] = (expansion_val, expansion_pt);
                } else {
                    // Accept reflection
                    self.simplex[n] = (*reflection_value, reflection.clone());
                }
                self.state = NMState::Init;
                self.step(config, history)
            }

            NMState::OutsideContraction { centroid: _, contraction, reflection_value } => {
                let contraction_val = history.last().map(|t| t.value).unwrap_or(*reflection_value);

                if contraction_val <= *reflection_value {
                    // Accept outside contraction
                    self.simplex[n] = (contraction_val, contraction.clone());
                    self.state = NMState::Init;
                    self.step(config, history)
                } else {
                    // Shrink
                    let shrunk = self.compute_shrunk_points();
                    if shrunk.is_empty() {
                        self.state = NMState::Init;
                        return self.step(config, history);
                    }
                    let first_shrunk = shrunk[0].clone();
                    self.state = NMState::Shrink {
                        shrunk_points: shrunk,
                        shrunk_idx: 0,
                    };
                    StrategyAction::Evaluate(vec![self.vec_to_dict(&first_shrunk, &keys)])
                }
            }

            NMState::InsideContraction { centroid: _, contraction } => {
                let contraction_val = history.last().map(|t| t.value).unwrap_or(f64::INFINITY);
                let worst_val = self.simplex[n].0;

                if contraction_val < worst_val {
                    // Accept inside contraction
                    self.simplex[n] = (contraction_val, contraction.clone());
                    self.state = NMState::Init;
                    self.step(config, history)
                } else {
                    // Shrink
                    let shrunk = self.compute_shrunk_points();
                    if shrunk.is_empty() {
                        self.state = NMState::Init;
                        return self.step(config, history);
                    }
                    let first_shrunk = shrunk[0].clone();
                    self.state = NMState::Shrink {
                        shrunk_points: shrunk,
                        shrunk_idx: 0,
                    };
                    StrategyAction::Evaluate(vec![self.vec_to_dict(&first_shrunk, &keys)])
                }
            }

            NMState::Shrink { shrunk_points, shrunk_idx } => {
                // Record the shrunk point we just evaluated
                if let Some(last) = history.last() {
                    let idx = shrunk_idx + 1; // +1 because index 0 is best (unchanged)
                    if idx < self.simplex.len() {
                        self.simplex[idx] = (last.value, self.dict_to_vec(&last.params, &keys));
                    }
                }

                let next_idx = shrunk_idx + 1;
                if next_idx < shrunk_points.len() {
                    // Evaluate next shrunk point
                    let next_shrunk = shrunk_points[next_idx].clone();
                    self.state = NMState::Shrink {
                        shrunk_points: shrunk_points.clone(),
                        shrunk_idx: next_idx,
                    };
                    StrategyAction::Evaluate(vec![self.vec_to_dict(&next_shrunk, &keys)])
                } else {
                    // Shrink complete, restart
                    self.state = NMState::Init;
                    self.step(config, history)
                }
            }

            NMState::SimplexBuild { evals_received } => {
                // Receive evaluations for the N new simplex vertices
                // The evaluations come in order, matching the vertices we requested
                let num_vertices_needed = n; // N vertices to evaluate (anchor already done)
                
                // Update simplex values from history (last N entries from this batch)
                // Find the evaluation results in history that correspond to our vertices
                let start_idx = history.len().saturating_sub(num_vertices_needed - *evals_received);
                
                for (i, eval) in history.iter().skip(start_idx).enumerate() {
                    let simplex_idx = *evals_received + i + 1; // +1 to skip anchor
                    if simplex_idx < self.simplex.len() {
                        self.simplex[simplex_idx].0 = eval.value;
                    }
                }
                
                // All vertices evaluated, sort and proceed to reflection
                self.sort_simplex();
                
                // Check convergence
                if self.check_convergence() {
                    self.state = NMState::Converged;
                    return StrategyAction::Converged;
                }
                
                // Compute first reflection
                let centroid = self.compute_centroid();
                let worst = &self.simplex[n].1;
                let mut reflection = self.compute_reflection(&centroid, worst);
                self.clamp_to_bounds(&mut reflection, config, &keys);
                
                let best = self.simplex[0].0;
                let second_worst = self.simplex[n - 1].0;
                let worst_val = self.simplex[n].0;
                
                self.state = NMState::Reflection {
                    centroid,
                    reflection: reflection.clone(),
                    best,
                    second_worst,
                    worst: worst_val,
                };
                
                StrategyAction::Evaluate(vec![self.vec_to_dict(&reflection, &keys)])
            }

            NMState::Converged => StrategyAction::Converged,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nm_coefficients_default() {
        let coeffs = NMCoefficients::default();
        assert_eq!(coeffs.alpha, 1.0);
        assert_eq!(coeffs.gamma, 2.0);
        assert_eq!(coeffs.rho, 0.5);
        assert_eq!(coeffs.sigma, 0.5);
    }

    #[test]
    fn test_nm_reflection_calculation() {
        let nm = NelderMead::new(2);
        let centroid = vec![1.0, 1.0];
        let worst = vec![0.0, 0.0];
        
        let reflection = nm.compute_reflection(&centroid, &worst);
        
        // r = c + α*(c - w) = [1,1] + 1.0*([1,1] - [0,0]) = [2, 2]
        assert_eq!(reflection, vec![2.0, 2.0]);
    }

    #[test]
    fn test_nm_expansion_calculation() {
        let nm = NelderMead::new(2);
        let centroid = vec![1.0, 1.0];
        let reflection = vec![2.0, 2.0];
        
        let expansion = nm.compute_expansion(&centroid, &reflection);
        
        // e = c + γ*(r - c) = [1,1] + 2.0*([2,2] - [1,1]) = [3, 3]
        assert_eq!(expansion, vec![3.0, 3.0]);
    }

    #[test]
    fn test_nm_contraction_calculation() {
        let nm = NelderMead::new(2);
        let centroid = vec![1.0, 1.0];
        let reflection = vec![2.0, 2.0];
        
        let contraction = nm.compute_outside_contraction(&centroid, &reflection);
        
        // c_o = c + ρ*(r - c) = [1,1] + 0.5*([2,2] - [1,1]) = [1.5, 1.5]
        assert_eq!(contraction, vec![1.5, 1.5]);
    }

    #[test]
    fn test_nm_inside_contraction_calculation() {
        let nm = NelderMead::new(2);
        let centroid = vec![1.0, 1.0];
        let worst = vec![0.0, 0.0];
        
        let contraction = nm.compute_inside_contraction(&centroid, &worst);
        
        // c_i = c + ρ*(w - c) = [1,1] + 0.5*([0,0] - [1,1]) = [0.5, 0.5]
        assert_eq!(contraction, vec![0.5, 0.5]);
    }

    #[test]
    fn test_nm_with_seed_points() {
        let seeds = vec![
            {
                let mut m = HashMap::new();
                m.insert("x".to_string(), 1.0);
                m
            },
        ];
        let nm = NelderMead::with_seed_points(1, seeds.clone());
        
        assert!(nm.seed_points.is_some());
        assert_eq!(nm.seed_points.as_ref().unwrap().len(), 1);
    }
}

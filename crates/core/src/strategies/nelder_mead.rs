use crate::artifact::EvalTrace;
use crate::config::{circular_mean01, diff01, dist01, wrap01, SolverConfig};
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
        #[allow(dead_code)]
        centroid: Vec<f64>,
        reflection: Vec<f64>,
        #[allow(dead_code)]
        expansion: Vec<f64>,
        reflection_value: f64,
    },
    /// Waiting for outside contraction point evaluation
    OutsideContraction {
        #[allow(dead_code)]
        centroid: Vec<f64>,
        contraction: Vec<f64>,
        reflection_value: f64,
    },
    /// Waiting for inside contraction point evaluation  
    InsideContraction {
        #[allow(dead_code)]
        centroid: Vec<f64>,
        contraction: Vec<f64>,
    },
    /// Shrink: evaluate all shrunk points
    Shrink {
        shrunk_points: Vec<Vec<f64>>,
        shrunk_idx: usize,
    },
    /// Coordinate prepass: greedy descent before simplex
    CoordinatePrepass {
        /// Current best point
        best_point: Vec<f64>,
        /// Current best value
        best_value: f64,
        /// Delta values to try [0.05, 0.01]
        deltas: Vec<f64>,
        /// Current delta index
        delta_idx: usize,
        /// Current dimension index
        dim_idx: usize,
        /// Pending candidate points to evaluate (+δ, -δ)
        pending: Vec<Vec<f64>>,
        /// For multi-seed: remaining seeds to try after this one
        remaining_seeds: Vec<(f64, Vec<f64>)>,
        /// For multi-seed: best refined result so far
        global_best: Option<(f64, Vec<f64>)>,
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
    pub(crate) simplex: Vec<(f64, Vec<f64>)>,
    /// Coefficients for NM operations
    coeffs: NMCoefficients,
    /// Convergence tolerance (simplex diameter)
    pub tolerance: f64,
    /// Mask for periodic dimensions (true = periodic, false = linear)
    pub periodic_mask: Vec<bool>,
}

impl NelderMead {
    pub fn new(dim: usize, periodic_mask: Vec<bool>) -> Self {
        Self {
            dim,
            state: NMState::Init,
            simplex: Vec::new(),
            coeffs: NMCoefficients::default(),
            tolerance: 1e-8,
            periodic_mask,
        }
    }

    /// Create NM with seed points from probe results (Top-K seeding)
    pub fn with_seed_points(
        dim: usize,
        seeds: Vec<(f64, Vec<f64>)>,
        periodic_mask: Vec<bool>,
    ) -> Self {
        Self {
            dim,
            state: NMState::Init,
            simplex: seeds,
            coeffs: NMCoefficients::default(),
            tolerance: 1e-8,
            periodic_mask,
        }
    }

    /// Create NM with custom coefficients
    pub fn with_coefficients(dim: usize, coeffs: NMCoefficients, periodic_mask: Vec<bool>) -> Self {
        Self {
            dim,
            state: NMState::Init,
            simplex: Vec::new(),
            coeffs,
            tolerance: 1e-8,
            periodic_mask,
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

    /// Clamp vector to bounds (or wrap if periodic)
    fn clamp_to_bounds(&self, vec: &mut [f64], config: &SolverConfig, keys: &[String]) {
        for (i, k) in keys.iter().enumerate() {
            if i < vec.len() {
                if let Some(domain) = config.bounds.get(k) {
                    if domain.is_periodic() {
                        vec[i] = wrap01(vec[i]);
                    } else {
                        vec[i] = vec[i].clamp(domain.min, domain.max);
                    }
                }
            }
        }
    }

    /// Compute centroid of all points except the worst (last)
    fn compute_centroid(&self) -> Vec<f64> {
        let n = self.dim;
        let mut centroid = vec![0.0; n];

        // Use circular_mean01 for periodic, arithmetic mean for linear
        for dim_idx in 0..n {
            let is_periodic = self.periodic_mask.get(dim_idx).copied().unwrap_or(false);
            if is_periodic {
                let values: Vec<f64> = self
                    .simplex
                    .iter()
                    .take(n)
                    .map(|(_, v)| v[dim_idx])
                    .collect();
                centroid[dim_idx] = circular_mean01(&values);
            } else {
                let sum: f64 = self.simplex.iter().take(n).map(|(_, v)| v[dim_idx]).sum();
                centroid[dim_idx] = sum / n as f64;
            }
        }
        centroid
    }

    /// Compute reflection point: r = c + α*(c - worst)
    pub(crate) fn compute_reflection(&self, centroid: &[f64], worst: &[f64]) -> Vec<f64> {
        centroid
            .iter()
            .zip(worst.iter())
            .enumerate()
            .map(|(i, (&c, &w))| {
                let is_periodic = self.periodic_mask.get(i).copied().unwrap_or(false);
                if is_periodic {
                    // Periodic: wrap(c + α * diff(c, w))
                    wrap01(c + self.coeffs.alpha * diff01(c, w))
                } else {
                    c + self.coeffs.alpha * (c - w)
                }
            })
            .collect()
    }

    /// Compute expansion point: e = c + γ*(r - c)
    pub(crate) fn compute_expansion(&self, centroid: &[f64], reflection: &[f64]) -> Vec<f64> {
        centroid
            .iter()
            .zip(reflection.iter())
            .enumerate()
            .map(|(i, (&c, &r))| {
                let is_periodic = self.periodic_mask.get(i).copied().unwrap_or(false);
                if is_periodic {
                    wrap01(c + self.coeffs.gamma * diff01(r, c))
                } else {
                    c + self.coeffs.gamma * (r - c)
                }
            })
            .collect()
    }

    /// Compute outside contraction: c_o = c + ρ*(r - c)
    pub(crate) fn compute_outside_contraction(&self, centroid: &[f64], reflection: &[f64]) -> Vec<f64> {
        centroid
            .iter()
            .zip(reflection.iter())
            .enumerate()
            .map(|(i, (&c, &r))| {
                let is_periodic = self.periodic_mask.get(i).copied().unwrap_or(false);
                if is_periodic {
                    wrap01(c + self.coeffs.rho * diff01(r, c))
                } else {
                    c + self.coeffs.rho * (r - c)
                }
            })
            .collect()
    }

    /// Compute inside contraction: c_i = c + ρ*(worst - c)
    pub(crate) fn compute_inside_contraction(&self, centroid: &[f64], worst: &[f64]) -> Vec<f64> {
        centroid
            .iter()
            .zip(worst.iter())
            .enumerate()
            .map(|(i, (&c, &w))| {
                let is_periodic = self.periodic_mask.get(i).copied().unwrap_or(false);
                if is_periodic {
                    wrap01(c + self.coeffs.rho * diff01(w, c))
                } else {
                    c + self.coeffs.rho * (w - c)
                }
            })
            .collect()
    }

    /// Compute shrunk points: x_i = x_best + σ*(x_i - x_best)
    pub(crate) fn compute_shrunk_points(&self) -> Vec<Vec<f64>> {
        let best = &self.simplex[0].1;
        self.simplex
            .iter()
            .skip(1)
            .map(|(_, xi)| {
                best.iter()
                    .zip(xi.iter())
                    .enumerate()
                    .map(|(i, (&b, &x))| {
                        let is_periodic = self.periodic_mask.get(i).copied().unwrap_or(false);
                        if is_periodic {
                            wrap01(b + self.coeffs.sigma * diff01(x, b))
                        } else {
                            b + self.coeffs.sigma * (x - b)
                        }
                    })
                    .collect()
            })
            .collect()
    }

    /// Check if simplex has converged (diameter < tolerance)
    pub(crate) fn check_convergence(&self) -> bool {
        if self.simplex.len() < 2 {
            return false;
        }
        let best = &self.simplex[0].1;
        let worst = &self.simplex.last().unwrap().1;

        // Compute max distance from best to any other point
        let diameter: f64 = best
            .iter()
            .zip(worst.iter())
            .enumerate()
            .map(|(i, (&b, &w))| {
                let is_periodic = self.periodic_mask.get(i).copied().unwrap_or(false);
                if is_periodic {
                    dist01(b, w)
                } else {
                    (b - w).abs()
                }
            })
            .fold(0.0, f64::max);

        diameter < self.tolerance
    }

    /// Sort simplex by objective value (ascending - minimization)
    fn sort_simplex(&mut self) {
        self.simplex
            .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
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
                // PHASE 5: Multi-seed prepass - pick K=3 diverse seeds from top candidates
                let mut sorted: Vec<_> = history.iter().collect();
                sorted.sort_by(|a, b| {
                    a.value
                        .partial_cmp(&b.value)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                if sorted.is_empty() {
                    return StrategyAction::Wait;
                }

                // Select K=3 diverse seeds from top-10 using farthest-point selection
                // [A/B CONFIG C/D: Top-K ENABLED]
                let k = 3;
                let pool_size = 10.min(sorted.len());
                let mut seeds: Vec<(f64, Vec<f64>)> = Vec::new();

                // Always include best point
                let best = &sorted[0];
                seeds.push((best.value, self.dict_to_vec(&best.params, &keys)));

                // Farthest-point selection for remaining seeds
                for _ in 1..k {
                    if seeds.len() >= pool_size {
                        break;
                    }

                    let mut best_min_dist = -1.0;
                    let mut best_idx = 1;

                    #[allow(clippy::needless_range_loop)]
                    for i in 1..pool_size {
                        let candidate = self.dict_to_vec(&sorted[i].params, &keys);

                        // Check if already selected
                        let already_selected = seeds.iter().any(|(_, s)| {
                            s.iter()
                                .zip(candidate.iter())
                                .all(|(a, b)| (a - b).abs() < 1e-9)
                        });
                        if already_selected {
                            continue;
                        }

                        // Compute min distance to all selected seeds
                        let min_dist = seeds
                            .iter()
                            .map(|(_, s)| {
                                s.iter()
                                    .zip(candidate.iter())
                                    .map(|(a, b)| (a - b).powi(2))
                                    .sum::<f64>()
                                    .sqrt()
                            })
                            .fold(f64::INFINITY, f64::min);

                        if min_dist > best_min_dist {
                            best_min_dist = min_dist;
                            best_idx = i;
                        }
                    }

                    if best_min_dist > 0.0 {
                        let selected = &sorted[best_idx];
                        seeds.push((selected.value, self.dict_to_vec(&selected.params, &keys)));
                    }
                }

                // Start coordinate prepass on first seed
                let (seed_val, seed_vec) = seeds.remove(0);
                let remaining_seeds = seeds;

                // Simplified prepass: just δ=0.05 (one pass for speed)
                let deltas = vec![0.05];
                let delta = deltas[0];

                let mut plus = seed_vec.clone();
                let mut minus = seed_vec.clone();
                plus[0] = (plus[0] + delta).min(1.0);
                minus[0] = (minus[0] - delta).max(0.0);

                let pending = vec![plus.clone(), minus.clone()];
                let candidates: Vec<_> =
                    pending.iter().map(|v| self.vec_to_dict(v, &keys)).collect();

                self.state = NMState::CoordinatePrepass {
                    best_point: seed_vec,
                    best_value: seed_val,
                    deltas,
                    delta_idx: 0,
                    dim_idx: 0,
                    pending,
                    remaining_seeds,
                    global_best: None,
                };

                StrategyAction::Evaluate(candidates)
            }

            NMState::CoordinatePrepass {
                best_point,
                best_value,
                deltas,
                delta_idx,
                dim_idx,
                pending,
                remaining_seeds,
                global_best,
            } => {
                // Process evaluation results: take greedy improving move
                let mut current_best = best_point.clone();
                let mut current_val = *best_value;

                // Check if any pending point improved
                for eval in history.iter().rev().take(pending.len()) {
                    if eval.value < current_val {
                        current_best = self.dict_to_vec(&eval.params, &keys);
                        current_val = eval.value;
                    }
                }

                // Move to next dimension
                let next_dim = dim_idx + 1;

                if next_dim < n {
                    // Continue with current delta, next dimension
                    let delta = deltas[*delta_idx];
                    let mut plus = current_best.clone();
                    let mut minus = current_best.clone();
                    plus[next_dim] = (plus[next_dim] + delta).min(1.0);
                    minus[next_dim] = (minus[next_dim] - delta).max(0.0);

                    let new_pending = vec![plus.clone(), minus.clone()];
                    let candidates: Vec<_> = new_pending
                        .iter()
                        .map(|v| self.vec_to_dict(v, &keys))
                        .collect();

                    self.state = NMState::CoordinatePrepass {
                        best_point: current_best,
                        best_value: current_val,
                        deltas: deltas.clone(),
                        delta_idx: *delta_idx,
                        dim_idx: next_dim,
                        pending: new_pending,
                        remaining_seeds: remaining_seeds.clone(),
                        global_best: global_best.clone(),
                    };

                    StrategyAction::Evaluate(candidates)
                } else {
                    // Finished all dimensions for this seed
                    // Update global best
                    let new_global_best = match global_best {
                        Some((gv, gp)) if *gv < current_val => Some((*gv, gp.clone())),
                        _ => Some((current_val, current_best.clone())),
                    };

                    // Check if more seeds to try
                    let mut remaining = remaining_seeds.clone();
                    if !remaining.is_empty() {
                        // Start prepass on next seed
                        let (next_val, next_vec) = remaining.remove(0);
                        let delta = deltas[0];

                        let mut plus = next_vec.clone();
                        let mut minus = next_vec.clone();
                        plus[0] = (plus[0] + delta).min(1.0);
                        minus[0] = (minus[0] - delta).max(0.0);

                        let new_pending = vec![plus.clone(), minus.clone()];
                        let candidates: Vec<_> = new_pending
                            .iter()
                            .map(|v| self.vec_to_dict(v, &keys))
                            .collect();

                        self.state = NMState::CoordinatePrepass {
                            best_point: next_vec,
                            best_value: next_val,
                            deltas: deltas.clone(),
                            delta_idx: 0,
                            dim_idx: 0,
                            pending: new_pending,
                            remaining_seeds: remaining,
                            global_best: new_global_best,
                        };

                        StrategyAction::Evaluate(candidates)
                    } else {
                        // All seeds processed - use global best for simplex
                        let (final_val, final_point) =
                            new_global_best.unwrap_or((current_val, current_best));

                        self.simplex.clear();
                        self.simplex.push((final_val, final_point.clone()));

                        // Build axis-aligned simplex around best refined point
                        let scale = 0.05;
                        for dim_idx in 0..n {
                            let mut vertex = final_point.clone();
                            let new_val = (vertex[dim_idx] + scale).min(1.0);
                            if (new_val - vertex[dim_idx]).abs() < 1e-6 {
                                vertex[dim_idx] = (vertex[dim_idx] - scale).max(0.0);
                            } else {
                                vertex[dim_idx] = new_val;
                            }
                            self.simplex.push((f64::INFINITY, vertex));
                        }

                        // Request evaluation of simplex vertices
                        let new_vertices: Vec<_> = self
                            .simplex
                            .iter()
                            .skip(1)
                            .map(|(_, v)| self.vec_to_dict(v, &keys))
                            .collect();

                        self.state = NMState::SimplexBuild { evals_received: 0 };
                        StrategyAction::Evaluate(new_vertices)
                    }
                }
            }

            NMState::SimplexBuild { evals_received } => {
                // (Previous SimplexBuild logic moved here from Init)
                // Simplex vertices already built, receive evaluations
                let num_vertices_needed = n;
                let start_idx = history
                    .len()
                    .saturating_sub(num_vertices_needed - *evals_received);

                for (i, eval) in history.iter().skip(start_idx).enumerate() {
                    let simplex_idx = *evals_received + i + 1;
                    if simplex_idx < self.simplex.len() {
                        self.simplex[simplex_idx].0 = eval.value;
                    }
                }

                self.sort_simplex();

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

            NMState::Reflection {
                centroid,
                reflection,
                best,
                second_worst,
                worst,
            } => {
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

            NMState::Expansion {
                centroid: _,
                reflection,
                expansion: _,
                reflection_value,
            } => {
                let expansion_val = history.last().map(|t| t.value).unwrap_or(*reflection_value);
                let expansion_pt = history
                    .last()
                    .map(|t| self.dict_to_vec(&t.params, &keys))
                    .unwrap_or_default();

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

            NMState::OutsideContraction {
                centroid: _,
                contraction,
                reflection_value,
            } => {
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

            NMState::InsideContraction {
                centroid: _,
                contraction,
            } => {
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

            NMState::Shrink {
                shrunk_points,
                shrunk_idx,
            } => {
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
        let dim = 2;
        let nm = NelderMead::new(dim, vec![false; dim]);
        let centroid = vec![1.0, 1.0];
        let worst = vec![0.0, 0.0];

        let reflection = nm.compute_reflection(&centroid, &worst);

        // r = c + α*(c - w) = [1,1] + 1.0*([1,1] - [0,0]) = [2, 2]
        assert_eq!(reflection, vec![2.0, 2.0]);
    }

    #[test]
    fn test_nm_expansion_calculation() {
        let dim = 2;
        let nm = NelderMead::new(dim, vec![false; dim]);
        let centroid = vec![1.0, 1.0];
        let reflection = vec![2.0, 2.0];

        let expansion = nm.compute_expansion(&centroid, &reflection);

        // e = c + γ*(r - c) = [1,1] + 2.0*([2,2] - [1,1]) = [3, 3]
        assert_eq!(expansion, vec![3.0, 3.0]);
    }

    #[test]
    fn test_nm_contraction_calculation() {
        let dim = 2;
        let nm = NelderMead::new(dim, vec![false; dim]);
        let centroid = vec![1.0, 1.0];
        let reflection = vec![2.0, 2.0];

        let contraction = nm.compute_outside_contraction(&centroid, &reflection);

        // c_o = c + ρ*(r - c) = [1,1] + 0.5*([2,2] - [1,1]) = [1.5, 1.5]
        assert_eq!(contraction, vec![1.5, 1.5]);
    }

    #[test]
    fn test_nm_inside_contraction_calculation() {
        let dim = 2;
        let nm = NelderMead::new(dim, vec![false; dim]);
        let centroid = vec![1.0, 1.0];
        let worst = vec![0.0, 0.0];

        let contraction = nm.compute_inside_contraction(&centroid, &worst);

        // c_i = c + ρ*(w - c) = [1,1] + 0.5*([0,0] - [1,1]) = [0.5, 0.5]
        assert_eq!(contraction, vec![0.5, 0.5]);
    }

    #[test]
    fn test_nm_with_seed_points() {
        let seed_val = 0.5;
        let seed_vec = vec![1.0_f64];
        let seeds = vec![(seed_val, seed_vec)];

        let nm = NelderMead::with_seed_points(1, seeds.clone(), vec![false; 1]);

        assert_eq!(nm.simplex.len(), 1);
        assert_eq!(nm.simplex[0].0, 0.5);
    }
}

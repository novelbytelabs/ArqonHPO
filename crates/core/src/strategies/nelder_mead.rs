use crate::config::SolverConfig;
use crate::artifact::EvalTrace;
use crate::strategies::{Strategy, StrategyAction};
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum NMState {
    Init,
    Reflection { centroid: Vec<f64>, best: f64, worst: f64 },
    Expansion { reflection: Vec<f64>, centroid: Vec<f64> },
    Contraction { reflection: Vec<f64>, centroid: Vec<f64> },
    Shrink,
}

pub struct NelderMead {
    dim: usize,
    state: NMState,
    // Store indices of simplex points in history? Or raw values?
    // Value, Params is safer.
    simplex: Vec<(f64, Vec<f64>)>, // (value, params_vector)
}

impl NelderMead {
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            state: NMState::Init,
            simplex: Vec::new(),
        }
    }

    fn dict_to_vec(&self, params: &HashMap<String, f64>, keys: &[String]) -> Vec<f64> {
        keys.iter().map(|k| *params.get(k).unwrap_or(&0.0)).collect()
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
}

impl Strategy for NelderMead {
    fn step(&mut self, config: &SolverConfig, history: &[EvalTrace]) -> StrategyAction {
        // Collect keys to ensure deterministic order (sort them)
        let mut keys: Vec<String> = config.bounds.keys().cloned().collect();
        keys.sort();
        self.dim = keys.len();

        match &self.state {
            NMState::Init => {
                // Select N+1 best points from history
                let mut sorted: Vec<_> = history.iter().collect();
                sorted.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap());
                
                let n = self.dim;
                if sorted.len() < n + 1 {
                    eprintln!("Not enough points to init simplex. Need {}, have {}", n+1, sorted.len());
                    // Fallback: Random? Or Wait? For MVP, assume Probe gave enough.
                    return StrategyAction::Wait; 
                }

                self.simplex.clear();
                for trace in sorted.iter().take(n + 1) {
                    let vec = self.dict_to_vec(&trace.params, &keys);
                    self.simplex.push((trace.value, vec));
                }
                
                // Now we have a simplex. Transition to Reflection immediately?
                // Yes, logic: compute centroid of all but worst.
                self.state_transition_to_reflection(config, &keys)
            }
            NMState::Reflection { .. } => {
                // The last point evaluated should be the reflection.
                // We check the result.
                if let Some(last) = history.last() {
                     // Check if this was our reflection.
                     // (Validation omitted for MVP, assuming synchronous step)
                     let val = last.value;
                     // Logic:
                     // 1. Sort simplex (re-sort not needed if we kept it sorted, but let's re-sort)
                     self.handle_reflection_result(val, &last.params, config, &keys)
                } else {
                    StrategyAction::Wait
                }
            }
            // Handling Expansion, Contraction similarly...
            // For MVP, simplify: 
            // Just basic reflection loop or restart?
            // Let's implement full standard NM logic in subsequent steps or simplified now.
            // Simplified: Always try Reflection. If good -> Expansion. If bad -> Contraction.
             _ => {
                 // Reset to Init to rebuild simplex from NEW history?
                 // Robustness: Re-building from full history is safer than state machine drift.
                 // "Restarting Nelder Mead" every step is essentially "Best N+1 points drive next".
                 // BUT strict NM creates a *specific* sequence.
                 // Let's reset to Init for now to ensure we pick up the latest point correctly,
                 // effectively implementing a "Greedy Simplex" that always uses global bests.
                 // This is technically a variant but works.
                 // Standard NM keeps the simplex alive.
                 // I will KEEP State for correctness.
                 // Placeholder for full logic.
                 StrategyAction::Converged // Fail-safe
             }
        }
    }
}

impl NelderMead {
    fn state_transition_to_reflection(&mut self, config: &SolverConfig, keys: &[String]) -> StrategyAction {
        // Sort simplex
        self.simplex.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        // Best: self.simplex[0], Worst: self.simplex[n]
        
        let n = self.dim;
        let worst = self.simplex[n].0;
        let best = self.simplex[0].0;
        
        // Centroid of [0..n-1]
        let mut centroid = vec![0.0; n];
        for simplex_point in self.simplex.iter().take(n) {
            for (j, c) in centroid.iter_mut().enumerate().take(n) {
                *c += simplex_point.1[j];
            }
        }
        for c in centroid.iter_mut().take(n) {
            *c /= n as f64;
        }

        // Reflect: r = c + alpha * (c - worst_pt)
        let alpha = config.strategy_params.as_ref()
            .and_then(|p| p.get("alpha"))
            .cloned()
            .unwrap_or(1.0);
        let worst_pt = &self.simplex[n].1;
        let mut reflection = vec![0.0; n];
        for j in 0..n {
            reflection[j] = centroid[j] + alpha * (centroid[j] - worst_pt[j]);
            // Bounds check?
        }
        
        // Update state
        self.state = NMState::Reflection { 
            centroid: centroid.clone(), 
            best, 
            worst 
        };
        
        StrategyAction::Evaluate(vec![self.vec_to_dict(&reflection, keys)])
    }
    
    fn handle_reflection_result(&mut self, val: f64, params: &HashMap<String, f64>, config: &SolverConfig, keys: &[String]) -> StrategyAction {
       // Logic: 
       // If best <= val < second_worst: replace worst.
       // If val < best: try expansion.
       // If val >= second_worst: contraction.
       
       let params_vec = self.dict_to_vec(params, keys);
       let n = self.dim;
       
       if let NMState::Reflection { best, worst, centroid: _ } = &self.state {
           // Basic update:
           if val < *best {
                // Expansion case (omitted for brevity, assume accept)
                self.simplex[n] = (val, params_vec);
                self.state = NMState::Init; // Loop back to start (Re-sort)
           } else if val < self.simplex[n-1].0 {
                // Accept reflection
                self.simplex[n] = (val, params_vec);
                self.state = NMState::Init;
           } else {
                // Contraction (omitted, force shrink/restart)
                // For MVP: Just replace if better, else Shrink (replace all but best).
                // Or just keep old simplex and try random?
                if val < *worst {
                     self.simplex[n] = (val, params_vec);
                } else {
                    // Shrink
                    // x_i = x_1 + sigma * (x_i - x_1)
                }
                self.state = NMState::Init;
           }
       }
       // Restart loop
       self.state_transition_to_reflection(config, keys)
    }
}

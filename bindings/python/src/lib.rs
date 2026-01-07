//! ArqonHPO Python Bindings - Boundary code for Python interface.
//!
//! Constitution VIII.3: This module is BOUNDARY CODE, not hot-path.
//! HashMap usage is ALLOWED here. Conversion to dense ParamVec happens
//! at the hotpath crate boundary.
#![allow(clippy::disallowed_types)] // Boundary code - HashMap allowed per VIII.3
#![allow(non_local_definitions)]
use arqonhpo_core::artifact::{EvalTrace, SeedPoint};
use arqonhpo_core::config::SolverConfig;
use arqonhpo_core::machine::Solver;
use pyo3::prelude::*;
use pyo3::types::PyModule;
use std::collections::HashMap;

#[pyclass]
struct ArqonSolver {
    inner: Solver,
}

#[allow(non_local_definitions)]
#[pymethods]
impl ArqonSolver {
    #[new]
    fn new(config_json: String) -> PyResult<Self> {
        // We take JSON string for config to avoid complex pyo3 implementation details for now.
        // It's clean and explicit.
        let config: SolverConfig = serde_json::from_str(&config_json).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid config: {}", e))
        })?;

        Ok(ArqonSolver {
            // Use the standard PCR (Probe-Classify-Refine) algorithm for all Python consumers
            inner: Solver::pcr(config),
        })
    }

    fn ask(&mut self) -> PyResult<Option<Vec<HashMap<String, f64>>>> {
        let candidates = self.inner.ask();
        Ok(candidates)
    }

    fn tell(&mut self, results_json: String) -> PyResult<()> {
        let results: Vec<EvalTrace> = serde_json::from_str(&results_json).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid results: {}", e))
        })?;
        self.inner.tell(results);
        Ok(())
    }

    fn get_history_len(&self) -> usize {
        self.inner.history.len()
    }

    /// Seed the solver with historical evaluations.
    /// Input: JSON array of {"params": {...}, "value": f64, "cost": f64}
    ///
    /// Use cases:
    /// - Warm-starting from previous optimization runs
    /// - Streaming/online optimization where external systems generate evaluations
    fn seed(&mut self, seed_json: String) -> PyResult<()> {
        let seeds: Vec<SeedPoint> = serde_json::from_str(&seed_json).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid seed data: {}", e))
        })?;
        self.inner.seed(seeds);
        Ok(())
    }
}

#[pymodule]
fn _internal(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ArqonSolver>()?;
    m.add_class::<ArqonProbe>()?;
    Ok(())
}

use arqonhpo_core::probe::{PrimeSqrtSlopesRotConfig, PrimeSqrtSlopesRotProbe};

#[pyclass]
struct ArqonProbe {
    inner: PrimeSqrtSlopesRotProbe,
    config: SolverConfig,
}

#[allow(non_local_definitions)]
#[pymethods]
impl ArqonProbe {
    #[new]
    #[pyo3(signature = (config_json, seed=42))]
    fn new(config_json: String, seed: u64) -> PyResult<Self> {
        let config: SolverConfig = serde_json::from_str(&config_json).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid config: {}", e))
        })?;

        // Probe config (Primary) for now
        let spice_ratio = PrimeSqrtSlopesRotConfig::adaptive_spice_for_landscape(false);
        let p_config = PrimeSqrtSlopesRotConfig::with_spice(spice_ratio);
        // Note: PrimeSqrtSlopesRotProbe internally handles seeds via seed_rotation.
        // We pass seed to constructor.
        let probe = PrimeSqrtSlopesRotProbe::with_seed_and_config(seed, p_config);

        Ok(ArqonProbe {
            inner: probe,
            config,
        })
    }

    /// Generate a single pure LDS point at the given global index (Stateless)
    fn sample_at(&self, index: usize) -> HashMap<String, f64> {
        self.inner.sample_at(index, &self.config)
    }

    /// Generate a range of pure LDS points [start, start+count) (Stateless)
    /// This enables zero-coordination sharding.
    fn sample_range(&self, start: usize, count: usize) -> Vec<HashMap<String, f64>> {
        self.inner.sample_range(start, count, &self.config)
    }
}

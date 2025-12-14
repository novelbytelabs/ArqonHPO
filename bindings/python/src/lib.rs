#![allow(non_local_definitions)]
use pyo3::prelude::*;
use arqonhpo_core::machine::Solver;
use arqonhpo_core::config::SolverConfig;
use arqonhpo_core::artifact::EvalTrace;
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
        let config: SolverConfig = serde_json::from_str(&config_json)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid config: {}", e)))?;
        
        Ok(ArqonSolver {
            inner: Solver::new(config),
        })
    }

    fn ask(&mut self) -> PyResult<Option<Vec<HashMap<String, f64>>>> {
        let candidates = self.inner.ask();
        Ok(candidates)
    }

    fn tell(&mut self, results_json: String) -> PyResult<()> {
        let results: Vec<EvalTrace> = serde_json::from_str(&results_json)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid results: {}", e)))?;
        self.inner.tell(results);
        Ok(())
    }

    fn get_history_len(&self) -> usize {
        self.inner.history.len()
    }
}

#[pymodule]
fn _internal(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<ArqonSolver>()?;
    Ok(())
}

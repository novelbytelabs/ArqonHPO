//! Test module for ArqonHPO core algorithms.
//!
//! This module contains unit tests for:
//! - Classifier algorithms (residual decay, variance)
//! - TPE strategy (Scott's rule bandwidth)
//! - Nelder-Mead strategy (all 5 operations)
//! - Probe strategies (prime-index, uniform)

#[cfg(test)]
mod test_classify;

#[cfg(test)]
mod test_tpe;

#[cfg(test)]
mod test_nelder_mead;

#[cfg(test)]
mod test_probe;

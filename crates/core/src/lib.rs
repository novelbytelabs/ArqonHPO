//! ArqonHPO Core - Boundary code for Python interface.
//!
//! Constitution VIII.3: This crate is BOUNDARY CODE, not hot-path.
//! HashMap usage is ALLOWED here. Conversion to dense ParamVec happens
//! at the hotpath crate boundary (see `hotpath::config_atomic::ParamRegistry`).
#![allow(clippy::disallowed_types)] // Boundary code - HashMap allowed per VIII.3

pub mod artifact;
pub mod classify;
pub mod config;
pub mod machine;
pub mod probe;
pub mod rng;
pub mod strategies;
pub mod variant_catalog;

// Re-export hotpath as adaptive_engine for API compatibility
pub use hotpath as adaptive_engine;

#[cfg(test)]
mod tests;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod lib_tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(0, 0), 0);
        assert_eq!(add(100, 200), 300);
    }
}

#![cfg_attr(test, allow(clippy::disallowed_types))] // Allow in tests (for test logic only)
#![allow(dead_code)] // TODO: Remove this before phase completion
#![allow(unused_variables)] // TODO: Remove this before phase completion

pub mod artifact;
pub mod classify;
pub mod config;
pub mod machine;
pub mod probe;
pub mod rng;
pub mod strategies;
pub mod adaptive_engine;

#[cfg(test)]
mod tests;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

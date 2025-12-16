pub mod adaptive_engine;
pub mod artifact;
pub mod classify;
pub mod config;
pub mod governance;
pub mod machine;
pub mod omega;
pub mod probe;
pub mod rng;
pub mod strategies;
pub mod variant_catalog;

#[cfg(test)]
mod tests;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

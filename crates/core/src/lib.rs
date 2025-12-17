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

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Returns a deterministic RNG seeded from the given u64.
///
/// We use ChaCha8 as our standard CSPRNG for reproducibility across platforms.
pub fn get_rng(seed: u64) -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(seed)
}

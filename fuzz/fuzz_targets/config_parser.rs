#![no_main]
use libfuzzer_sys::fuzz_target;
use arqonhpo_core::config::SolverConfig;

fuzz_target!(|data: &[u8]| {
    // Try to parse arbitrary bytes as JSON config
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = serde_json::from_str::<SolverConfig>(s);
    }
});

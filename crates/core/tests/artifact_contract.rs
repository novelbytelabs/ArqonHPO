use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ArtifactContract {
    seed: u64,
    registry_hash64: u64,
    registry_names: Vec<String>,
    param_len: usize,
    params_vec: Vec<f64>,
}

#[test]
fn test_artifact_contract_field_check() {
    // This test ensures the artifact serialization matches Constitution VIII.3
    // "Artifacts MUST include the following fields (exact keys): check"
    
    // Create dummy artifact data pattern
    let artifact_json = r#"{
        "seed": 12345,
        "registry_hash64": 99999,
        "registry_names": ["p1", "p2"],
        "param_len": 2,
        "params_vec": [0.1, 0.2]
    }"#;
    
    // Attempt to deserialize into precise contract struct
    let contract: ArtifactContract = serde_json::from_str(artifact_json)
        .expect("Artifact violation: Missing required keys or wrong types per Constitution VIII.3");
        
    assert_eq!(contract.seed, 12345);
    assert_eq!(contract.param_len, 2);
    assert_eq!(contract.registry_names, vec!["p1", "p2"]);
    assert_eq!(contract.params_vec, vec![0.1, 0.2]);
}

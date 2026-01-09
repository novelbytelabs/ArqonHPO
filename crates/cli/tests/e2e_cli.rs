use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

/// Create a valid config JSON file
fn create_config() -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"{{
            "seed": 42,
            "budget": 10,
            "probe_ratio": 0.5,
            "bounds": {{"x": {{"min": 0.0, "max": 1.0}}}}
        }}"#
    )
    .unwrap();
    file
}

#[test]
fn test_validate_command_success() -> Result<(), Box<dyn std::error::Error>> {
    let config_file = create_config();

    let output = Command::new(assert_cmd::cargo::cargo_bin!("arqonhpo-cli"))
        .arg("validate")
        .arg("--config")
        .arg(config_file.path())
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Config OK"));

    Ok(())
}

#[test]
fn test_validate_command_invalid() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = NamedTempFile::new()?;
    // Invalid config: budget is 0
    writeln!(
        file,
        r#"{{
            "seed": 42,
            "budget": 0,
            "probe_ratio": 0.5,
            "bounds": {{"x": {{"min": 0.0, "max": 1.0}}}}
        }}"#
    )?;

    let output = Command::new(assert_cmd::cargo::cargo_bin!("arqonhpo-cli"))
        .arg("validate")
        .arg("--config")
        .arg(file.path())
        .output()?;

    assert!(!output.status.success());

    Ok(())
}

#[test]
fn test_ask_command_basic() -> Result<(), Box<dyn std::error::Error>> {
    let config_file = create_config();

    let output = Command::new(assert_cmd::cargo::cargo_bin!("arqonhpo-cli"))
        .arg("ask")
        .arg("--config")
        .arg(config_file.path())
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should return JSON array of candidates
    assert!(stdout.starts_with('[') || stdout.starts_with("null"));

    Ok(())
}

#[test]
fn test_ask_command_with_batch_limit() -> Result<(), Box<dyn std::error::Error>> {
    let config_file = create_config();

    let output = Command::new(assert_cmd::cargo::cargo_bin!("arqonhpo-cli"))
        .arg("ask")
        .arg("--config")
        .arg(config_file.path())
        .arg("--batch")
        .arg("2")
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Parse and verify batch limit
    let candidates: Vec<serde_json::Value> = serde_json::from_str(&stdout)?;
    assert!(candidates.len() <= 2);

    Ok(())
}

#[test]
fn test_tell_command_with_results_file() -> Result<(), Box<dyn std::error::Error>> {
    // Create state file
    let state_file = NamedTempFile::new()?;
    let state_content = r#"{
        "config": {
            "seed": 42,
            "budget": 10,
            "probe_ratio": 0.5,
            "bounds": {"x": {"min": 0.0, "max": 1.0}}
        },
        "history": [],
        "run_id": "test"
    }"#;
    std::fs::write(state_file.path(), state_content)?;

    // Create results file
    let results_file = NamedTempFile::new()?;
    let results_content = r#"[{"params": {"x": 0.5}, "value": 0.25, "cost": 1.0}]"#;
    std::fs::write(results_file.path(), results_content)?;

    let output = Command::new(assert_cmd::cargo::cargo_bin!("arqonhpo-cli"))
        .arg("tell")
        .arg("--state")
        .arg(state_file.path())
        .arg("--results")
        .arg(results_file.path())
        .output()?;

    assert!(output.status.success());

    // Verify state was updated
    let updated_state: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(state_file.path())?)?;
    assert_eq!(updated_state["history"].as_array().unwrap().len(), 1);

    Ok(())
}

#[test]
fn test_export_import_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    // Create initial state
    let state_file = NamedTempFile::new()?;
    let state_content = r#"{
        "config": {
            "seed": 42,
            "budget": 10,
            "probe_ratio": 0.5,
            "bounds": {"x": {"min": 0.0, "max": 1.0}}
        },
        "history": [{"params": {"x": 0.5}, "value": 0.25, "cost": 1.0}],
        "run_id": "export-test"
    }"#;
    std::fs::write(state_file.path(), state_content)?;

    // Export
    let artifact_file = NamedTempFile::new()?;
    let export_output = Command::new(assert_cmd::cargo::cargo_bin!("arqonhpo-cli"))
        .arg("export")
        .arg("--state")
        .arg(state_file.path())
        .arg("--output")
        .arg(artifact_file.path())
        .output()?;
    assert!(export_output.status.success());

    // Verify artifact
    let artifact: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(artifact_file.path())?)?;
    assert_eq!(artifact["run_id"], "export-test");
    assert_eq!(artifact["history"].as_array().unwrap().len(), 1);

    // Import to new state file
    let new_state_file = NamedTempFile::new()?;
    let import_output = Command::new(assert_cmd::cargo::cargo_bin!("arqonhpo-cli"))
        .arg("import")
        .arg("--artifact")
        .arg(artifact_file.path())
        .arg("--state")
        .arg(new_state_file.path())
        .output()?;
    assert!(import_output.status.success());

    // Verify imported state
    let imported_state: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(new_state_file.path())?)?;
    assert_eq!(imported_state["run_id"], "export-test");
    assert_eq!(imported_state["history"].as_array().unwrap().len(), 1);

    Ok(())
}

#[test]
fn test_interactive_ask_tell() -> Result<(), Box<dyn std::error::Error>> {
    let config_file = create_config();

    // Spawn interactive process
    let mut child = Command::new(assert_cmd::cargo::cargo_bin!("arqonhpo-cli"))
        .arg("interactive")
        .arg("--config")
        .arg(config_file.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.take().unwrap();

    // Send ASK command
    writeln!(stdin, r#"{{"cmd": "ask"}}"#)?;
    stdin.flush()?;

    // Read response
    use std::io::{BufRead, BufReader};
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    reader.read_line(&mut line)?;

    // Parse response
    let response: serde_json::Value = serde_json::from_str(&line)?;
    assert!(response.get("params").is_some());

    // Send TELL command
    let params = &response["params"].as_array().unwrap()[0];
    let tell_cmd = format!(
        r#"{{"cmd": "tell", "results": [{{"params": {}, "value": 0.5, "cost": 1.0}}]}}"#,
        serde_json::to_string(params)?
    );
    writeln!(child.stdin.as_mut().unwrap(), "{}", tell_cmd)?;

    // Close stdin to terminate
    drop(child.stdin.take());

    // Wait for process to exit
    let status = child.wait()?;
    assert!(status.success());

    Ok(())
}

use assert_cmd::cargo::CommandCargoExt;
use reqwest::blocking::Client;
use std::process::Command;
use tempfile::NamedTempFile;

#[test]
#[allow(deprecated)]
fn test_dashboard_e2e_server() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Prepare temp files for state, events, actions
    let state_file = NamedTempFile::new()?;
    let events_file = NamedTempFile::new()?;
    let actions_file = NamedTempFile::new()?;

    // Write some initial state
    let initial_state = r#"{
        "config": {
            "budget": 100,
            "bounds": {},
            "seed": 1
        },
        "history": [],
        "run_id": "test-run"
    }"#;
    std::fs::write(state_file.path(), initial_state)?;

    // 2. Spawn server on port 0 (OS assigns port)?
    // tiny_http doesn't return the port easily if we use 0 in the CLI arg unless we parse stdout.
    // Let's use a random high port or try port 0 and grep stdout.
    // Pro trick: Use port 0, capture stdout line "Dashboard running at http://127.0.0.1:PORT"

    let mut cmd = Command::cargo_bin("arqonhpo-cli")?;
    cmd.arg("dashboard")
        .arg("--state")
        .arg(state_file.path())
        .arg("--events")
        .arg(events_file.path())
        .arg("--actions")
        .arg(actions_file.path())
        .arg("--addr")
        .arg("127.0.0.1:0");

    // We need to spawn and read stdout efficiently.
    // Using `process::Command` directly.

    use std::io::{BufRead, BufReader};
    use std::process::Stdio;

    let mut child = cmd.stdout(Stdio::piped()).spawn()?;

    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    let mut port: u16 = 0;
    for line in reader.lines() {
        let line = line?;
        if line.contains("Dashboard running at http://") {
            // Parse port
            if let Some(addr_str) = line.split("http://").nth(1) {
                if let Some(port_str) = addr_str.split(':').nth(1) {
                    port = port_str.trim().parse::<u16>()?;
                    break;
                }
            }
        }
    }

    if port == 0 {
        // Failed to start or parse
        child.kill()?;
        return Err("Failed to extract port from dashboard stdout".into());
    }

    // 3. Make HTTP requests
    let client = Client::new();
    let base_url = format!("http://127.0.0.1:{}", port);

    // GET /api/summary
    let resp = client.get(format!("{}/api/summary", base_url)).send()?;
    assert!(resp.status().is_success());
    let body: serde_json::Value = resp.json()?;
    assert_eq!(body["run_id"], "test-run");

    // POST /api/actions
    let action_payload = serde_json::json!({
        "action": "tune",
        "knob": "timeout",
        "value": 300
    });
    let resp = client
        .post(format!("{}/api/actions", base_url))
        .json(&action_payload)
        .send()?;
    assert!(resp.status().is_success());

    // 4. Verify side effects (actions file)
    // Give it a moment to flush? append_line uses sync write so it should be immediate.
    let actions_content = std::fs::read_to_string(actions_file.path())?;
    assert!(actions_content.contains("timeout"));
    assert!(actions_content.contains("300"));

    // 5. Cleanup
    child.kill()?;

    Ok(())
}

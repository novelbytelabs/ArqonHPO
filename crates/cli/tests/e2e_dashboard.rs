use reqwest::blocking::Client;
use std::net::TcpListener;
use std::process::Command;
use tempfile::NamedTempFile;

#[test]
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

    // 2. Reserve a local port; skip if binding is not permitted (sandboxed env).
    let listener = match TcpListener::bind("127.0.0.1:0") {
        Ok(listener) => listener,
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => return Ok(()),
        Err(err) => return Err(err.into()),
    };
    let port = listener.local_addr()?.port();
    drop(listener);

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("arqonhpo-cli"));
    cmd.arg("dashboard")
        .arg("--state")
        .arg(state_file.path())
        .arg("--events")
        .arg(events_file.path())
        .arg("--actions")
        .arg(actions_file.path())
        .arg("--addr")
        .arg(format!("127.0.0.1:{}", port));

    let mut child = cmd.spawn()?;

    // 3. Make HTTP requests
    let client = Client::new();
    let base_url = format!("http://127.0.0.1:{}", port);

    // GET /api/summary (retry briefly for startup)
    let mut resp = None;
    for _ in 0..10 {
        match client.get(format!("{}/api/summary", base_url)).send() {
            Ok(r) => {
                resp = Some(r);
                break;
            }
            Err(_) => std::thread::sleep(std::time::Duration::from_millis(50)),
        }
    }
    let resp = resp.ok_or("Failed to connect to dashboard server")?;
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

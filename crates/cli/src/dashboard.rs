use crate::{load_state, Metrics};
use miette::{Context, IntoDiagnostic, Result};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const DASHBOARD_HTML: &str = include_str!("../assets/dashboard.html");
pub const DASHBOARD_CSS: &str = include_str!("../assets/dashboard.css");
pub const DASHBOARD_JS: &str = include_str!("../assets/dashboard.js");

pub fn json_response(
    result: Result<serde_json::Value>,
) -> tiny_http::Response<std::io::Cursor<Vec<u8>>> {
    match result {
        Ok(value) => {
            let data = serde_json::to_vec(&value).unwrap_or_default();
            tiny_http::Response::from_data(data).with_header(
                tiny_http::Header::from_bytes(&b"Content-Type"[..], "application/json").unwrap(),
            )
        }
        Err(err) => tiny_http::Response::from_string(err.to_string()).with_status_code(500),
    }
}

pub fn plain_response(
    body: &str,
    content_type: &str,
) -> tiny_http::Response<std::io::Cursor<Vec<u8>>> {
    tiny_http::Response::from_data(body.as_bytes().to_vec())
        .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], content_type).unwrap())
}

pub fn load_state_json(state_path: &Path, metrics: &Metrics) -> Result<serde_json::Value> {
    let state = load_state(state_path)?;
    metrics.set_history_len(state.history.len());
    serde_json::to_value(state).into_diagnostic()
}

pub fn load_summary_json(state_path: &Path) -> Result<serde_json::Value> {
    let state = load_state(state_path)?;
    let best = state
        .history
        .iter()
        .map(|entry| entry.value)
        .min_by(|left, right| left.partial_cmp(right).unwrap());
    let latest = state.history.last().map(|entry| entry.value);
    let summary = serde_json::json!({
        "run_id": state.run_id,
        "budget": state.config.budget,
        "history_len": state.history.len(),
        "best": best,
        "latest": latest,
    });
    Ok(summary)
}

pub fn load_events_json(
    events_path: Option<&PathBuf>,
    params: &HashMap<String, String>,
) -> Result<serde_json::Value> {
    let limit = params
        .get("limit")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(100);
    let filter = params.get("event").map(String::as_str);
    let search = params.get("q").map(String::as_str);
    let events = if let Some(path) = events_path {
        read_event_values(path, filter, search, limit)?
    } else {
        Vec::new()
    };
    Ok(serde_json::json!({ "events": events }))
}

pub fn load_actions_json(
    actions_path: Option<&PathBuf>,
    params: &HashMap<String, String>,
) -> Result<serde_json::Value> {
    let limit = params
        .get("limit")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(50);
    let actions = if let Some(path) = actions_path {
        read_jsonl_values(path, limit)?
    } else {
        Vec::new()
    };
    Ok(serde_json::json!({ "actions": actions }))
}

/// Store an action from a generic reader (body) to a file.
/// Accepted reader allows easy testing without mocking tiny_http::Request.
pub fn store_action<R: Read>(
    mut reader: R,
    actions_path: Option<&PathBuf>,
) -> Result<serde_json::Value> {
    let Some(path) = actions_path else {
        return Err(miette::miette!("Actions path not configured"));
    };
    let mut body = String::new();
    reader.read_to_string(&mut body).into_diagnostic()?;

    let mut value: serde_json::Value = serde_json::from_str(&body)
        .into_diagnostic()
        .with_context(|| "Invalid JSON body")?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64;

    if let serde_json::Value::Object(ref mut map) = value {
        map.entry("timestamp_us".to_string())
            .or_insert(serde_json::Value::Number(timestamp.into()));
    }

    let line = serde_json::to_string(&value).into_diagnostic()?;
    append_line(path, &line)?;
    Ok(serde_json::json!({ "ok": true }))
}

fn append_line(path: &Path, line: &str) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .into_diagnostic()
        .with_context(|| format!("Failed to open {}", path.display()))?;
    writeln!(file, "{}", line).into_diagnostic()?;
    Ok(())
}

fn read_event_values(
    path: &Path,
    filter: Option<&str>,
    search: Option<&str>,
    limit: usize,
) -> Result<Vec<serde_json::Value>> {
    let contents = fs::read_to_string(path)
        .into_diagnostic()
        .with_context(|| format!("Failed to read events file {}", path.display()))?;
    let mut values = Vec::new();
    for line in contents.lines().rev() {
        if line.trim().is_empty() {
            continue;
        }
        if let Some(search) = search {
            if !line.contains(search) {
                continue;
            }
        }
        let value: serde_json::Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if let Some(filter) = filter {
            let event_type = value
                .get("event")
                .or_else(|| value.get("event_type"))
                .and_then(|field| field.as_str())
                .unwrap_or("");
            if event_type != filter {
                continue;
            }
        }
        values.push(value);
        if values.len() >= limit {
            break;
        }
    }
    values.reverse();
    Ok(values)
}

fn read_jsonl_values(path: &Path, limit: usize) -> Result<Vec<serde_json::Value>> {
    let contents = fs::read_to_string(path)
        .into_diagnostic()
        .with_context(|| format!("Failed to read actions file {}", path.display()))?;
    let mut values = Vec::new();
    for line in contents.lines().rev() {
        if line.trim().is_empty() {
            continue;
        }
        let value: serde_json::Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(_) => continue,
        };
        values.push(value);
        if values.len() >= limit {
            break;
        }
    }
    values.reverse();
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SolverState;
    use arqonhpo_core::artifact::SeedPoint;
    use arqonhpo_core::config::{Domain, Scale, SolverConfig};
    use std::fs;
    use std::io::Cursor;
    use tempfile::NamedTempFile;

    fn create_test_state() -> SolverState {
        let mut bounds = HashMap::new();
        bounds.insert(
            "x".to_string(),
            Domain {
                min: 0.0,
                max: 1.0,
                scale: Scale::Linear,
            },
        );
        SolverState {
            config: SolverConfig {
                bounds,
                budget: 10,
                seed: 42,
                probe_ratio: 0.3,
                strategy_params: None,
            },
            history: vec![
                SeedPoint {
                    params: [("x".to_string(), 0.5)].into_iter().collect(),
                    value: 0.25,
                    cost: 1.0,
                },
                SeedPoint {
                    params: [("x".to_string(), 0.3)].into_iter().collect(),
                    value: 0.10,
                    cost: 1.0,
                },
            ],
            run_id: Some("test-run".to_string()),
        }
    }

    #[test]
    fn test_json_response_success() {
        let value = serde_json::json!({"ok": true});
        let response = json_response(Ok(value.clone()));
        // Check status and content type
        let status_code = response.status_code().0;
        assert_eq!(status_code, 200);
    }

    #[test]
    fn test_json_response_error() {
        let err = miette::miette!("Test error");
        let response = json_response(Err(err));
        let status_code = response.status_code().0;
        assert_eq!(status_code, 500);
    }

    #[test]
    fn test_plain_response_html() {
        let response = plain_response("<html></html>", "text/html");
        let status_code = response.status_code().0;
        assert_eq!(status_code, 200);
    }

    #[test]
    fn test_plain_response_css() {
        let response = plain_response("body { color: red; }", "text/css");
        let status_code = response.status_code().0;
        assert_eq!(status_code, 200);
    }

    #[test]
    fn test_plain_response_js() {
        let response = plain_response("console.log('hi');", "text/javascript");
        let status_code = response.status_code().0;
        assert_eq!(status_code, 200);
    }

    #[test]
    fn test_load_state_json_success() -> Result<()> {
        let file = NamedTempFile::new().into_diagnostic()?;
        let path = file.path().to_path_buf();
        let state = create_test_state();
        fs::write(&path, serde_json::to_string(&state).unwrap()).into_diagnostic()?;

        let metrics = Metrics::init(None)?;
        let result = load_state_json(&path, &metrics)?;

        assert!(result.get("config").is_some());
        assert!(result.get("history").is_some());
        assert_eq!(result["run_id"], "test-run");
        Ok(())
    }

    #[test]
    fn test_load_summary_json_success() -> Result<()> {
        let file = NamedTempFile::new().into_diagnostic()?;
        let path = file.path().to_path_buf();
        let state = create_test_state();
        fs::write(&path, serde_json::to_string(&state).unwrap()).into_diagnostic()?;

        let result = load_summary_json(&path)?;

        assert_eq!(result["run_id"], "test-run");
        assert_eq!(result["budget"], 10);
        assert_eq!(result["history_len"], 2);
        // Best value should be 0.10 (the minimum)
        assert!((result["best"].as_f64().unwrap() - 0.10).abs() < 0.001);
        // Latest value should be 0.10 (last entry)
        assert!((result["latest"].as_f64().unwrap() - 0.10).abs() < 0.001);
        Ok(())
    }

    #[test]
    fn test_load_events_filtering() -> Result<()> {
        let file = NamedTempFile::new().into_diagnostic()?;
        let path = file.path().to_path_buf();
        let events = [
            r#"{"event": "start", "timestamp_us": 100}"#,
            r#"{"event": "stop", "timestamp_us": 200}"#,
            r#"{"event": "error", "timestamp_us": 300, "details": "bad bad"}"#,
        ];
        fs::write(&path, events.join("\n")).into_diagnostic()?;

        let _metrics = Metrics::init(None)?; // Dummy metrics

        // Test Filter
        let mut params = HashMap::new();
        params.insert("event".to_string(), "start".to_string());
        let result = load_events_json(Some(&path), &params)?;
        let array = result["events"].as_array().unwrap();
        assert_eq!(array.len(), 1);
        assert_eq!(array[0]["event"], "start");

        // Test Search
        let mut params = HashMap::new();
        params.insert("q".to_string(), "bad".to_string());
        let result = load_events_json(Some(&path), &params)?;
        let array = result["events"].as_array().unwrap();
        assert_eq!(array.len(), 1);
        assert_eq!(array[0]["event"], "error");

        Ok(())
    }

    #[test]
    fn test_load_events_no_path() -> Result<()> {
        let params = HashMap::new();
        let result = load_events_json(None, &params)?;
        let array = result["events"].as_array().unwrap();
        assert!(array.is_empty());
        Ok(())
    }

    #[test]
    fn test_load_events_with_empty_lines() -> Result<()> {
        let file = NamedTempFile::new().into_diagnostic()?;
        let path = file.path().to_path_buf();
        let content = r#"{"event": "a"}

{"event": "b"}

"#;
        fs::write(&path, content).into_diagnostic()?;

        let params = HashMap::new();
        let result = load_events_json(Some(&path), &params)?;
        let array = result["events"].as_array().unwrap();
        assert_eq!(array.len(), 2);
        Ok(())
    }

    #[test]
    fn test_load_events_with_invalid_json() -> Result<()> {
        let file = NamedTempFile::new().into_diagnostic()?;
        let path = file.path().to_path_buf();
        let content = r#"{"event": "valid"}
not json
{"event": "also_valid"}"#;
        fs::write(&path, content).into_diagnostic()?;

        let params = HashMap::new();
        let result = load_events_json(Some(&path), &params)?;
        let array = result["events"].as_array().unwrap();
        // Should skip invalid line
        assert_eq!(array.len(), 2);
        Ok(())
    }

    #[test]
    fn test_load_events_with_event_type_field() -> Result<()> {
        let file = NamedTempFile::new().into_diagnostic()?;
        let path = file.path().to_path_buf();
        // Using "event_type" instead of "event"
        let content = r#"{"event_type": "custom", "data": 123}"#;
        fs::write(&path, content).into_diagnostic()?;

        let mut params = HashMap::new();
        params.insert("event".to_string(), "custom".to_string());
        let result = load_events_json(Some(&path), &params)?;
        let array = result["events"].as_array().unwrap();
        assert_eq!(array.len(), 1);
        Ok(())
    }

    #[test]
    fn test_load_actions_limit() -> Result<()> {
        let file = NamedTempFile::new().into_diagnostic()?;
        let path = file.path().to_path_buf();
        let mut lines = Vec::new();
        for i in 0..10 {
            lines.push(format!(r#"{{"i": {}}}"#, i));
        }
        fs::write(&path, lines.join("\n")).into_diagnostic()?;

        let mut params = HashMap::new();
        params.insert("limit".to_string(), "3".to_string());
        let result = load_actions_json(Some(&path), &params)?;
        let array = result["actions"].as_array().unwrap();
        // Since it reads in reverse, we get the last 3, but reversed back to chronological
        assert_eq!(array.len(), 3);
        assert_eq!(array[0]["i"], 7);
        assert_eq!(array[2]["i"], 9);

        Ok(())
    }

    #[test]
    fn test_load_actions_no_path() -> Result<()> {
        let params = HashMap::new();
        let result = load_actions_json(None, &params)?;
        let array = result["actions"].as_array().unwrap();
        assert!(array.is_empty());
        Ok(())
    }

    #[test]
    fn test_load_actions_with_invalid_json() -> Result<()> {
        let file = NamedTempFile::new().into_diagnostic()?;
        let path = file.path().to_path_buf();
        let content = r#"{"valid": 1}
broken json
{"valid": 2}"#;
        fs::write(&path, content).into_diagnostic()?;

        let params = HashMap::new();
        let result = load_actions_json(Some(&path), &params)?;
        let array = result["actions"].as_array().unwrap();
        // Should skip invalid line
        assert_eq!(array.len(), 2);
        Ok(())
    }

    #[test]
    fn test_store_action_valid() -> Result<()> {
        let file = NamedTempFile::new().into_diagnostic()?;
        let path = file.path().to_path_buf();
        let body = r#"{"action": "tune", "knob": "timeout"}"#;
        let reader = Cursor::new(body);

        let response = store_action(reader, Some(&path))?;
        assert_eq!(response, serde_json::json!({ "ok": true }));

        let content = fs::read_to_string(&path).into_diagnostic()?;
        assert!(content.contains(r#""action":"tune""#));
        assert!(content.contains(r#""timestamp_us":"#));
        Ok(())
    }

    #[test]
    fn test_store_action_with_existing_timestamp() -> Result<()> {
        let file = NamedTempFile::new().into_diagnostic()?;
        let path = file.path().to_path_buf();
        // Already has timestamp_us
        let body = r#"{"action": "test", "timestamp_us": 12345}"#;
        let reader = Cursor::new(body);

        let response = store_action(reader, Some(&path))?;
        assert_eq!(response, serde_json::json!({ "ok": true }));

        let content = fs::read_to_string(&path).into_diagnostic()?;
        // Should keep existing timestamp
        assert!(content.contains("12345"));
        Ok(())
    }

    #[test]
    fn test_store_action_invalid_json() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_path_buf();
        let body = r#"{"broken": json"#;
        let reader = Cursor::new(body);

        let result = store_action(reader, Some(&path));
        assert!(result.is_err());
    }

    #[test]
    fn test_store_action_no_path() {
        let body = r#"{}"#;
        let reader = Cursor::new(body);
        let result = store_action(reader, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_dashboard_assets_not_empty() {
        assert!(!DASHBOARD_HTML.trim().is_empty());
        assert!(!DASHBOARD_CSS.trim().is_empty());
        assert!(!DASHBOARD_JS.trim().is_empty());
    }
}

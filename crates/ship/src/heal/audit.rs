use anyhow::Result;
use rusqlite::{Connection, params};
use crate::heal::parser_rust::TestFailure;
use crate::heal::r#loop::HealOutcome;
use crate::oracle::hash::compute_hash;
use uuid::Uuid;
use chrono::Utc;

pub struct AuditLog {
    conn: Connection,
}

impl AuditLog {
    pub fn open(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(Self { conn })
    }
    
    pub fn log_attempt(
        &self,
        failure: &TestFailure,
        prompt: &str,
        fix: &str,
        outcome: &HealOutcome,
    ) -> Result<()> {
        let run_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now().to_rfc3339();
        let prompt_hash = compute_hash(prompt);
        let diff_hash = compute_hash(fix);
        let outcome_str = format!("{:?}", outcome);
        
        self.conn.execute(
            "INSERT INTO healing_attempts (run_id, timestamp, file_path, error_msg, prompt_hash, diff_hash, outcome)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                run_id,
                timestamp,
                failure.file_path,
                failure.error_message,
                prompt_hash,
                diff_hash,
                outcome_str
            ],
        )?;
        
        Ok(())
    }
}

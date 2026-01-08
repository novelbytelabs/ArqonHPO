use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Commit {
    pub hash: String,
    pub commit_type: String,
    pub scope: Option<String>,
    pub description: String,
    pub is_breaking: bool,
}

pub struct CommitParser {
    root: PathBuf,
}

impl CommitParser {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
    
    /// Get commits since last tag
    pub fn get_commits_since_last_tag(&self) -> Result<Vec<Commit>> {
        // Get last tag
        let tag_output = Command::new("git")
            .args(["describe", "--tags", "--abbrev=0"])
            .current_dir(&self.root)
            .output()?;
        
        let last_tag = if tag_output.status.success() {
            String::from_utf8_lossy(&tag_output.stdout).trim().to_string()
        } else {
            "".to_string()
        };
        
        // Get log since tag (or all if no tag)
        let range = if last_tag.is_empty() {
            "HEAD".to_string()
        } else {
            format!("{}..HEAD", last_tag)
        };
        
        let log_output = Command::new("git")
            .args(["log", "--format=%H %s", &range])
            .current_dir(&self.root)
            .output()?;
        
        let log_str = String::from_utf8_lossy(&log_output.stdout);
        let commits = log_str.lines()
            .filter_map(|line| self.parse_commit_line(line))
            .collect();
        
        Ok(commits)
    }
    
    fn parse_commit_line(&self, line: &str) -> Option<Commit> {
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.len() < 2 {
            return None;
        }
        
        let hash = parts[0].to_string();
        let message = parts[1];
        
        // Parse conventional commit: type(scope)!: description
        let is_breaking = message.contains("BREAKING CHANGE") || message.contains("!:");
        
        // Simple parser: just extract type
        let commit_type = if message.starts_with("feat") {
            "feat"
        } else if message.starts_with("fix") {
            "fix"
        } else if message.starts_with("chore") {
            "chore"
        } else if message.starts_with("docs") {
            "docs"
        } else if message.starts_with("refactor") {
            "refactor"
        } else if message.starts_with("test") {
            "test"
        } else {
            "other"
        };
        
        Some(Commit {
            hash,
            commit_type: commit_type.to_string(),
            scope: None, // TODO: Extract scope
            description: message.to_string(),
            is_breaking,
        })
    }
}

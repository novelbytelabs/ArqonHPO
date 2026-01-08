use anyhow::Result;
use crate::ship::commits::Commit;

#[derive(Debug, Clone)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemVer {
    pub fn parse(version_str: &str) -> Result<Self> {
        let clean = version_str.trim_start_matches('v');
        let parts: Vec<&str> = clean.split('.').collect();
        
        let major = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
        let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
        
        Ok(Self { major, minor, patch })
    }
    
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
    
    pub fn bump_major(&self) -> Self {
        Self { major: self.major + 1, minor: 0, patch: 0 }
    }
    
    pub fn bump_minor(&self) -> Self {
        Self { major: self.major, minor: self.minor + 1, patch: 0 }
    }
    
    pub fn bump_patch(&self) -> Self {
        Self { major: self.major, minor: self.minor, patch: self.patch + 1 }
    }
}

/// Calculate next version based on conventional commits
pub fn calculate_next_version(current: &SemVer, commits: &[Commit]) -> SemVer {
    let has_breaking = commits.iter().any(|c| c.is_breaking);
    let has_feat = commits.iter().any(|c| c.commit_type == "feat");
    
    if has_breaking {
        current.bump_major()
    } else if has_feat {
        current.bump_minor()
    } else {
        current.bump_patch()
    }
}

/// Generate changelog from commits
pub fn generate_changelog(version: &SemVer, commits: &[Commit]) -> String {
    let mut changelog = format!("## v{}\n\n", version.to_string());
    
    // Group by type
    let features: Vec<_> = commits.iter().filter(|c| c.commit_type == "feat").collect();
    let fixes: Vec<_> = commits.iter().filter(|c| c.commit_type == "fix").collect();
    let others: Vec<_> = commits.iter()
        .filter(|c| c.commit_type != "feat" && c.commit_type != "fix")
        .collect();
    
    if !features.is_empty() {
        changelog.push_str("### Features\n\n");
        for commit in features {
            changelog.push_str(&format!("- {}\n", commit.description));
        }
        changelog.push('\n');
    }
    
    if !fixes.is_empty() {
        changelog.push_str("### Bug Fixes\n\n");
        for commit in fixes {
            changelog.push_str(&format!("- {}\n", commit.description));
        }
        changelog.push('\n');
    }
    
    if !others.is_empty() {
        changelog.push_str("### Other Changes\n\n");
        for commit in others {
            changelog.push_str(&format!("- {}\n", commit.description));
        }
        changelog.push('\n');
    }
    
    changelog
}

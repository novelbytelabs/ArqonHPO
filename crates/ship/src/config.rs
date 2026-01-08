use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub meta: MetaConfig,
    pub oracle: OracleConfig,
    pub heal: HealConfig,
    pub ship: ShipConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaConfig {
    pub config_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleConfig {
    pub include_globs: Vec<String>,
    pub exclude_globs: Vec<String>,
    pub model_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealConfig {
    pub max_attempts: u32,
    pub model_id: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipConfig {
    pub require_branches: Vec<String>,
    pub version_scheme: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            meta: MetaConfig { config_version: 1 },
            oracle: OracleConfig {
                include_globs: vec!["src/**/*.rs".to_string(), "src/**/*.py".to_string()],
                exclude_globs: vec![
                    "target/".to_string(),
                    "venv/".to_string(),
                    ".git/".to_string(),
                ],
                model_path: "~/.arqon/models/".to_string(),
            },
            heal: HealConfig {
                max_attempts: 2,
                model_id: "deepseek-coder-1.3b-instruct".to_string(),
                enabled: true,
            },
            ship: ShipConfig {
                require_branches: vec!["main".to_string()],
                version_scheme: "semver".to_string(),
            },
        }
    }
}

impl Config {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            anyhow::bail!("Config file not found at {:?}", path);
        }
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file at {:?}", path))?;

        let config: Config =
            toml::from_str(&content).with_context(|| "Failed to parse config TOML")?;

        Ok(config)
    }

    pub fn load_default() -> Self {
        Self::default()
    }
}

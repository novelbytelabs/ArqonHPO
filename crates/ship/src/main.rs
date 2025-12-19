mod config;

use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Context, Result};
use config::Config;

use clap::{Parser, Subcommand, Args};
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Context, Result};
use config::Config;
mod oracle;

#[derive(Parser)]
#[command(name = "arqon")]
#[command(about = "ArqonShip: DevSecOps Automation System", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to config file
    #[arg(long, global = true, default_value = ".arqon/config.toml")]
    config: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize ArqonShip in the current repository
    Init,
    /// Build the Codebase Oracle (Graph + Vectors)
    Scan,
    /// Query the Codebase Oracle
    Chat(ChatArgs),
    /// Autonomous Self-Healing CI
    Heal,
    /// Governed Release Pipeline
    Ship,
}

#[derive(Args)]
struct ChatArgs {
    /// The query string
    #[arg(long, short)]
    query: String,
    
    /// Use CLI output mode instead of TUI (default for now)
    #[arg(long)]
    cli: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Load config (except for Init)
    let config = if let Commands::Init = &cli.command {
        Config::default() // Dummy
    } else {
        Config::load_from_file(&cli.config).unwrap_or_default()
    };

    match &cli.command {
        Commands::Init => handle_init(&cli.config)?,
        Commands::Scan => {
            let root = std::env::current_dir()?;
            oracle::scan_codebase(&config, &root).await?;
        },
        Commands::Chat(args) => {
            let root = std::env::current_dir()?;
            let db_path = root.join(".arqon/graph.db");
            let vector_path = root.join(".arqon/vectors.lance");
            
            let engine = oracle::query::QueryEngine::new(
                db_path.to_str().unwrap(),
                vector_path.to_str().unwrap()
            ).await?;
            
            let results = engine.query(&args.query).await?;
            for res in results {
                println!("[{}] {} (Score: {})", res.path, res.name, res.score);
            }
        }
        Commands::Heal => println!("TODO: Implement heal"),
        Commands::Ship => println!("TODO: Implement ship"),
    }

    Ok(())
}

fn handle_init(config_path: &Path) -> Result<()> {
    if config_path.exists() {
        println!("Config file already exists at {:?}", config_path);
        return Ok(());
    }

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
    }

    let default_config = Config::default();
    let toml_string = toml::to_string_pretty(&default_config)
        .context("Failed to serialize default config")?;

    fs::write(config_path, toml_string)
        .with_context(|| format!("Failed to write config file to {:?}", config_path))?;

    println!("Initialized ArqonShip configuration at {:?}", config_path);
    Ok(())
}

mod config;
mod oracle;
mod heal;
mod ship;

use clap::{Parser, Subcommand, Args};
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Context, Result};
use config::Config;

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
    Heal(HealArgs),
    /// Governed Release Pipeline
    Ship(ShipArgs),
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

#[derive(Args)]
struct HealArgs {
    /// Path to the test output file (cargo test --message-format=json)
    #[arg(long)]
    log_file: Option<PathBuf>,
    
    /// Maximum healing attempts (default: 2)
    #[arg(long, default_value = "2")]
    max_attempts: u32,
}

#[derive(Args)]
struct ShipArgs {
    /// Skip pre-flight checks
    #[arg(long)]
    skip_checks: bool,
    
    /// Dry run (don't create PR)
    #[arg(long)]
    dry_run: bool,
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
            oracle::scan_codebase(&root).await?;
        },
        Commands::Chat(args) => {
            let root = std::env::current_dir()?;
            let db_path = root.join(".arqon/graph.db");
            let vector_path = root.join(".arqon/vectors.lance");
            
            let mut engine = oracle::query::QueryEngine::new(
                db_path.to_str().unwrap(),
                vector_path.to_str().unwrap()
            ).await?;
            
            let results = engine.query(&args.query).await?;
            for res in results {
                println!("[{}] {} (Score: {})", res.path, res.name, res.score);
            }
        }
        Commands::Heal(args) => {
            println!("Starting self-healing pipeline...");
            println!("Max attempts: {}", args.max_attempts);
            println!("Heal command not yet fully implemented (LLM integration pending)");
        }
        Commands::Ship(args) => {
            let root = std::env::current_dir()?;
            println!("Starting release pipeline...");
            
            if !args.skip_checks {
                let checker = ship::ConstitutionCheck::new(root.clone());
                if !checker.run_all()? {
                    println!("Constitution checks failed. Use --skip-checks to override.");
                    std::process::exit(1);
                }
            }
            
            // Parse commits and calculate version
            let parser = ship::CommitParser::new(root.clone());
            let commits = parser.get_commits_since_last_tag()?;
            
            let current_version = ship::SemVer::parse("0.0.0")?; // TODO: Read from Cargo.toml
            let next_version = ship::calculate_next_version(&current_version, &commits);
            let changelog = ship::generate_changelog(&next_version, &commits);
            
            println!("Next version: v{}", next_version.to_string());
            println!("\nChangelog:\n{}", changelog);
            
            if args.dry_run {
                println!("\n[DRY RUN] Would create release PR");
            } else {
                println!("\n[INFO] To create PR, set GITHUB_TOKEN and run without --dry-run");
            }
        }
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

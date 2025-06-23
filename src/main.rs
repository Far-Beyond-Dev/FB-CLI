use clap::{Parser, Subcommand};
use anyhow::Result;
use colored::*;

mod commands;
mod utils;

use commands::{horizon, repo};

#[derive(Parser)]
#[command(
    name = "fbcli",
    about = "Far Beyond Development Kit - CLI tool for Horizon plugin development and repo management",
    version = "0.1.0",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Horizon game server related commands
    #[command(subcommand)]
    Horizon(horizon::HorizonCommand),
    
    /// Repository management commands
    #[command(subcommand)]
    Repo(repo::RepoCommand),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Print welcome banner
    println!("{}", "🚀 Far Beyond Development Kit".bright_cyan().bold());
    println!("{}", "════════════════════════════".bright_cyan());
    println!();

    match cli.command {
        Commands::Horizon(cmd) => horizon::handle_command(cmd).await,
        Commands::Repo(cmd) => repo::handle_command(cmd).await,
    }
}
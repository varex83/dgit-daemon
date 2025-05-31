use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod client;
mod commands;
mod config;

use commands::{account, daemon, repo};

#[derive(Parser)]
#[command(
    name = "dgit",
    about = "CLI for interacting with the decentralized git daemon",
    version
)]
struct Cli {
    /// Set the verbosity level
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Daemon URL (can also be set via DGIT_DAEMON_URL env var)
    #[arg(long, global = true, env = "DGIT_DAEMON_URL", default_value = "http://localhost:3000")]
    daemon_url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the daemon
    Daemon {
        /// Port to run the daemon on
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },

    /// Repository management commands
    #[command(subcommand)]
    Repo(repo::RepoCommands),

    /// Account management commands
    #[command(subcommand)]
    Account(account::AccountCommands),

    /// Check daemon health
    Health,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let cli = Cli::parse();

    let log_level = match cli.verbose {
        0 => Level::ERROR,
        1 => Level::WARN,
        2 => Level::INFO,
        3 => Level::DEBUG,
        _ => Level::TRACE,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    match cli.command {
        Commands::Daemon { port } => {
            daemon::start_daemon(port).await?;
        }
        Commands::Repo(cmd) => {
            let client = client::DaemonClient::new(cli.daemon_url);
            repo::handle_command(cmd, client).await?;
        }
        Commands::Account(cmd) => {
            account::handle_command(cmd).await?;
        }
        Commands::Health => {
            let client = client::DaemonClient::new(cli.daemon_url);
            match client.health_check().await {
                Ok(_) => println!("{}", "✓ Daemon is healthy".green()),
                Err(e) => {
                    eprintln!("{}", format!("✗ Daemon health check failed: {}", e).red());
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}

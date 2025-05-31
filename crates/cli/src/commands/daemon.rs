use anyhow::Result;
use colored::*;
use std::process::Command;
use tokio::signal;

pub async fn start_daemon(port: u16) -> Result<()> {
    println!("{}", format!("Starting daemon on port {}...", port).green());

    std::env::set_var("PORT", port.to_string());

    let mut child = Command::new("cargo")
        .args(&["run", "--package", "daemon"])
        .env("PORT", port.to_string())
        .spawn()?;

    println!("{}", "Daemon started. Press Ctrl+C to stop.".yellow());

    signal::ctrl_c().await?;

    println!("{}", "\nShutting down daemon...".yellow());
    child.kill()?;

    Ok(())
}
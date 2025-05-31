use anyhow::Result;
use clap::Subcommand;
use colored::*;
use dialoguer::{Input, Password, Select};

use crate::config::{Account, Config};

#[derive(Subcommand)]
pub enum AccountCommands {
    /// Add a new account
    Add {
        /// Account name
        #[arg(short, long)]
        name: Option<String>,

        /// Private key (will prompt if not provided)
        #[arg(short, long)]
        private_key: Option<String>,

        /// Ethereum address
        #[arg(short, long)]
        address: Option<String>,
    },

    /// Remove an account
    Remove {
        /// Account name to remove
        name: String,
    },

    /// List all accounts
    List,

    /// Switch to a different account
    Switch {
        /// Account name to switch to
        name: Option<String>,
    },

    /// Show the active account
    Current,
}

pub async fn handle_command(cmd: AccountCommands) -> Result<()> {
    let mut config = Config::load()?;

    match cmd {
        AccountCommands::Add { name, private_key, address } => {
            add_account(&mut config, name, private_key, address).await?;
        }
        AccountCommands::Remove { name } => {
            remove_account(&mut config, &name)?;
        }
        AccountCommands::List => {
            list_accounts(&config);
        }
        AccountCommands::Switch { name } => {
            switch_account(&mut config, name)?;
        }
        AccountCommands::Current => {
            show_current_account(&config);
        }
    }

    Ok(())
}

async fn add_account(
    config: &mut Config,
    name: Option<String>,
    private_key: Option<String>,
    address: Option<String>,
) -> Result<()> {
    let name = match name {
        Some(n) => n,
        None => Input::new()
            .with_prompt("Account name")
            .interact_text()?,
    };

    if config.accounts.contains_key(&name) {
        anyhow::bail!("Account '{}' already exists", name);
    }

    let private_key = match private_key {
        Some(pk) => pk,
        None => Password::new()
            .with_prompt("Private key")
            .interact()?,
    };

    let address = match address {
        Some(addr) => addr,
        None => Input::new()
            .with_prompt("Ethereum address")
            .interact_text()?,
    };

    let account = Account {
        name: name.clone(),
        private_key,
        address: address.clone(),
    };

    config.add_account(account)?;

    println!("{}", format!("✓ Account '{}' added successfully", name).green());
    println!("  Address: {}", address.cyan());

    if config.accounts.len() == 1 {
        println!("{}", format!("  Set as active account").yellow());
    }

    Ok(())
}

fn remove_account(config: &mut Config, name: &str) -> Result<()> {
    if !config.accounts.contains_key(name) {
        anyhow::bail!("Account '{}' not found", name);
    }

    config.remove_account(name)?;

    println!("{}", format!("✓ Account '{}' removed", name).green());

    if let Some(active) = &config.active_account {
        println!("{}", format!("  Active account is now '{}'", active).yellow());
    }

    Ok(())
}

fn list_accounts(config: &Config) {
    let accounts = config.list_accounts();

    if accounts.is_empty() {
        println!("{}", "No accounts configured".yellow());
        println!("Use 'dgit account add' to add an account");
        return;
    }

    println!("{}", "Configured accounts:".bold());
    for (name, account, is_active) in accounts {
        let status = if is_active { " (active)".green() } else { "".normal() };
        println!("  {} {}{}", "•".cyan(), name.bold(), status);
        println!("    Address: {}", account.address.dimmed());
    }
}

fn switch_account(config: &mut Config, name: Option<String>) -> Result<()> {
    let name = match name {
        Some(n) => n,
        None => {
            let accounts: Vec<String> = config.accounts.keys().cloned().collect();
            if accounts.is_empty() {
                anyhow::bail!("No accounts configured");
            }

            let selection = Select::new()
                .with_prompt("Select account")
                .items(&accounts)
                .interact()?;

            accounts[selection].clone()
        }
    };

    config.set_active_account(&name)?;

    if let Some(account) = config.accounts.get(&name) {
        println!("{}", format!("✓ Switched to account '{}'", name).green());
        println!("  Address: {}", account.address.cyan());
    }

    Ok(())
}

fn show_current_account(config: &Config) {
    match config.get_active_account() {
        Some(account) => {
            println!("{}", "Active account:".bold());
            println!("  Name: {}", account.name.cyan());
            println!("  Address: {}", account.address);
        }
        None => {
            println!("{}", "No active account".yellow());
            println!("Use 'dgit account add' to add an account");
        }
    }
} 
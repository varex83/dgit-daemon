use anyhow::Result;
use clap::Subcommand;
use colored::*;

use crate::client::DaemonClient;
use crate::config::Config;

#[derive(Subcommand)]
pub enum RepoCommands {
    /// Create a new repository
    Create {
        /// Repository name
        name: String,
    },

    /// Repository role management
    #[command(subcommand)]
    Role(RoleCommands),
}

#[derive(Subcommand)]
pub enum RoleCommands {
    /// Grant pusher role to an address
    GrantPusher {
        /// Repository name
        #[arg(short, long)]
        repo: String,

        /// Address to grant role to (uses active account if not specified)
        #[arg(short, long)]
        address: Option<String>,
    },

    /// Revoke pusher role from an address
    RevokePusher {
        /// Repository name
        #[arg(short, long)]
        repo: String,

        /// Address to revoke role from (uses active account if not specified)
        #[arg(short, long)]
        address: Option<String>,
    },

    /// Grant admin role to an address
    GrantAdmin {
        /// Repository name
        #[arg(short, long)]
        repo: String,

        /// Address to grant role to (uses active account if not specified)
        #[arg(short, long)]
        address: Option<String>,
    },

    /// Revoke admin role from an address
    RevokeAdmin {
        /// Repository name
        #[arg(short, long)]
        repo: String,

        /// Address to revoke role from (uses active account if not specified)
        #[arg(short, long)]
        address: Option<String>,
    },

    /// Check if an address has pusher role
    CheckPusher {
        /// Repository name
        #[arg(short, long)]
        repo: String,

        /// Address to check (uses active account if not specified)
        #[arg(short, long)]
        address: Option<String>,
    },

    /// Check if an address has admin role
    CheckAdmin {
        /// Repository name
        #[arg(short, long)]
        repo: String,

        /// Address to check (uses active account if not specified)
        #[arg(short, long)]
        address: Option<String>,
    },
}

pub async fn handle_command(cmd: RepoCommands, client: DaemonClient) -> Result<()> {
    match cmd {
        RepoCommands::Create { name } => {
            create_repo(client, &name).await?;
        }
        RepoCommands::Role(role_cmd) => {
            handle_role_command(role_cmd, client).await?;
        }
    }

    Ok(())
}

async fn create_repo(client: DaemonClient, name: &str) -> Result<()> {
    println!("{}", format!("Creating repository '{}'...", name).yellow());

    match client.create_repo(name).await {
        Ok(response) => {
            println!("{}", format!("✓ Repository '{}' created successfully", name).green());
            println!("  Contract address: {}", response.address.cyan());
        }
        Err(e) => {
            eprintln!("{}", format!("✗ Failed to create repository: {}", e).red());
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn handle_role_command(cmd: RoleCommands, client: DaemonClient) -> Result<()> {
    let config = Config::load()?;

    match cmd {
        RoleCommands::GrantPusher { repo, address } => {
            let address = get_address(address, &config)?;
            grant_pusher_role(client, &repo, &address).await?;
        }
        RoleCommands::RevokePusher { repo, address } => {
            let address = get_address(address, &config)?;
            revoke_pusher_role(client, &repo, &address).await?;
        }
        RoleCommands::GrantAdmin { repo, address } => {
            let address = get_address(address, &config)?;
            grant_admin_role(client, &repo, &address).await?;
        }
        RoleCommands::RevokeAdmin { repo, address } => {
            let address = get_address(address, &config)?;
            revoke_admin_role(client, &repo, &address).await?;
        }
        RoleCommands::CheckPusher { repo, address } => {
            let address = get_address(address, &config)?;
            check_pusher_role(client, &repo, &address).await?;
        }
        RoleCommands::CheckAdmin { repo, address } => {
            let address = get_address(address, &config)?;
            check_admin_role(client, &repo, &address).await?;
        }
    }

    Ok(())
}

fn get_address(address: Option<String>, config: &Config) -> Result<String> {
    match address {
        Some(addr) => Ok(addr),
        None => {
            config.get_active_account()
                .map(|account| account.address.clone())
                .ok_or_else(|| anyhow::anyhow!("No active account. Use 'dgit account add' to add one."))
        }
    }
}

async fn grant_pusher_role(client: DaemonClient, repo: &str, address: &str) -> Result<()> {
    println!("{}", format!("Granting pusher role to {} for repository '{}'...", address, repo).yellow());

    match client.grant_pusher_role(repo, address).await {
        Ok(_) => {
            println!("{}", format!("✓ Pusher role granted to {}", address).green());
        }
        Err(e) => {
            eprintln!("{}", format!("✗ Failed to grant pusher role: {}", e).red());
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn revoke_pusher_role(client: DaemonClient, repo: &str, address: &str) -> Result<()> {
    println!("{}", format!("Revoking pusher role from {} for repository '{}'...", address, repo).yellow());

    match client.revoke_pusher_role(repo, address).await {
        Ok(_) => {
            println!("{}", format!("✓ Pusher role revoked from {}", address).green());
        }
        Err(e) => {
            eprintln!("{}", format!("✗ Failed to revoke pusher role: {}", e).red());
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn grant_admin_role(client: DaemonClient, repo: &str, address: &str) -> Result<()> {
    println!("{}", format!("Granting admin role to {} for repository '{}'...", address, repo).yellow());

    match client.grant_admin_role(repo, address).await {
        Ok(_) => {
            println!("{}", format!("✓ Admin role granted to {}", address).green());
        }
        Err(e) => {
            eprintln!("{}", format!("✗ Failed to grant admin role: {}", e).red());
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn revoke_admin_role(client: DaemonClient, repo: &str, address: &str) -> Result<()> {
    println!("{}", format!("Revoking admin role from {} for repository '{}'...", address, repo).yellow());

    match client.revoke_admin_role(repo, address).await {
        Ok(_) => {
            println!("{}", format!("✓ Admin role revoked from {}", address).green());
        }
        Err(e) => {
            eprintln!("{}", format!("✗ Failed to revoke admin role: {}", e).red());
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn check_pusher_role(client: DaemonClient, repo: &str, address: &str) -> Result<()> {
    match client.check_pusher_role(repo, address).await {
        Ok(has_role) => {
            if has_role {
                println!("{}", format!("✓ {} has pusher role for repository '{}'", address, repo).green());
            } else {
                println!("{}", format!("✗ {} does not have pusher role for repository '{}'", address, repo).yellow());
            }
        }
        Err(e) => {
            eprintln!("{}", format!("✗ Failed to check pusher role: {}", e).red());
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn check_admin_role(client: DaemonClient, repo: &str, address: &str) -> Result<()> {
    match client.check_admin_role(repo, address).await {
        Ok(has_role) => {
            if has_role {
                println!("{}", format!("✓ {} has admin role for repository '{}'", address, repo).green());
            } else {
                println!("{}", format!("✗ {} does not have admin role for repository '{}'", address, repo).yellow());
            }
        }
        Err(e) => {
            eprintln!("{}", format!("✗ Failed to check admin role: {}", e).red());
            std::process::exit(1);
        }
    }

    Ok(())
}
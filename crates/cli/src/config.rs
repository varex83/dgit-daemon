use anyhow::{Context, Result};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub accounts: HashMap<String, Account>,
    pub active_account: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    pub name: String,
    pub private_key: String,
    pub address: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            let config = Config::default();
            config.save()?;
            return Ok(config);
        }

        let content = fs::read_to_string(&config_path)
            .context("Failed to read config file")?;

        toml::from_str(&content).context("Failed to parse config file")
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        fs::write(&config_path, content)
            .context("Failed to write config file")?;

        Ok(())
    }

    pub fn add_account(&mut self, account: Account) -> Result<()> {
        if self.accounts.is_empty() {
            self.active_account = Some(account.name.clone());
        }

        self.accounts.insert(account.name.clone(), account);
        self.save()
    }

    pub fn remove_account(&mut self, name: &str) -> Result<()> {
        self.accounts.remove(name);

        if self.active_account.as_ref() == Some(&name.to_string()) {
            self.active_account = self.accounts.keys().next().cloned();
        }

        self.save()
    }

    pub fn set_active_account(&mut self, name: &str) -> Result<()> {
        if !self.accounts.contains_key(name) {
            anyhow::bail!("Account '{}' not found", name);
        }

        self.active_account = Some(name.to_string());
        self.save()
    }

    pub fn get_active_account(&self) -> Option<&Account> {
        self.active_account
            .as_ref()
            .and_then(|name| self.accounts.get(name))
    }

    pub fn list_accounts(&self) -> Vec<(&String, &Account, bool)> {
        self.accounts
            .iter()
            .map(|(name, account)| {
                let is_active = self.active_account.as_ref() == Some(name);
                (name, account, is_active)
            })
            .collect()
    }

    fn config_path() -> Result<PathBuf> {
        let config_dir = config_dir()
            .context("Failed to determine config directory")?;

        Ok(config_dir.join("dgit").join("config.toml"))
    }
}
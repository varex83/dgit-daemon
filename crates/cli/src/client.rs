use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct DaemonClient {
    client: Client,
    base_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRepoResponse {
    pub repo: String,
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleResponse {
    pub has_role: bool,
}

impl DaemonClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    pub async fn health_check(&self) -> Result<()> {
        let url = format!("{}/health", self.base_url);
        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            anyhow::bail!("Health check failed with status: {}", response.status())
        }
    }

    pub async fn create_repo(&self, repo_name: &str) -> Result<CreateRepoResponse> {
        let url = format!("{}/create-repo/{}", self.base_url, repo_name);
        let response = self.client.post(&url).send().await?;

        if response.status().is_success() {
            response.json().await.context("Failed to parse create repo response")
        } else {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to create repository: {}", error_text)
        }
    }

    pub async fn grant_pusher_role(&self, repo: &str, address: &str) -> Result<()> {
        let url = format!("{}/repo/{}/grant-pusher/{}", self.base_url, repo, address);
        let response = self.client.post(&url).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to grant pusher role: {}", error_text)
        }
    }

    pub async fn revoke_pusher_role(&self, repo: &str, address: &str) -> Result<()> {
        let url = format!("{}/repo/{}/revoke-pusher/{}", self.base_url, repo, address);
        let response = self.client.post(&url).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to revoke pusher role: {}", error_text)
        }
    }

    pub async fn grant_admin_role(&self, repo: &str, address: &str) -> Result<()> {
        let url = format!("{}/repo/{}/grant-admin/{}", self.base_url, repo, address);
        let response = self.client.post(&url).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to grant admin role: {}", error_text)
        }
    }

    pub async fn revoke_admin_role(&self, repo: &str, address: &str) -> Result<()> {
        let url = format!("{}/repo/{}/revoke-admin/{}", self.base_url, repo, address);
        let response = self.client.post(&url).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to revoke admin role: {}", error_text)
        }
    }

    pub async fn check_pusher_role(&self, repo: &str, address: &str) -> Result<bool> {
        let url = format!("{}/repo/{}/check-pusher/{}", self.base_url, repo, address);
        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let role_resp: RoleResponse = response.json().await?;
            Ok(role_resp.has_role)
        } else {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to check pusher role: {}", error_text)
        }
    }

    pub async fn check_admin_role(&self, repo: &str, address: &str) -> Result<bool> {
        let url = format!("{}/repo/{}/check-admin/{}", self.base_url, repo, address);
        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let role_resp: RoleResponse = response.json().await?;
            Ok(role_resp.has_role)
        } else {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to check admin role: {}", error_text)
        }
    }
} 
use axum::{extract::{Path, State}, response::IntoResponse, Json};
use serde::Serialize;
use anyhow::Result;
use ethcontract::Address;
use std::str::FromStr;

use crate::state::ContractState;

#[derive(Debug, Serialize)]
pub struct RoleResponse {
    pub repo: String,
    pub address: String,
    pub role: String,
    pub granted: bool,
}

#[derive(Debug, Serialize)]
pub struct RoleCheckResponse {
    pub repo: String,
    pub address: String,
    pub role: String,
    pub has_role: bool,
}

pub async fn grant_pusher_role(
    State(contract_state): State<ContractState>,
    Path((repo, address)): Path<(String, String)>,
) -> impl IntoResponse {
    match handle_grant_pusher_role(contract_state, repo, address).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

async fn handle_grant_pusher_role(
    contract_state: ContractState,
    repo: String,
    address_str: String,
) -> Result<RoleResponse> {
    let contract = contract_state.get_contract(&repo).await
        .ok_or_else(|| anyhow::anyhow!("Repository not found"))?;

    let address = Address::from_str(&address_str)
        .map_err(|_| anyhow::anyhow!("Invalid address format"))?;

    contract.grant_pusher_role(address).await?;

    Ok(RoleResponse {
        repo,
        address: address_str,
        role: "pusher".to_string(),
        granted: true,
    })
}

pub async fn revoke_pusher_role(
    State(contract_state): State<ContractState>,
    Path((repo, address)): Path<(String, String)>,
) -> impl IntoResponse {
    match handle_revoke_pusher_role(contract_state, repo, address).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

async fn handle_revoke_pusher_role(
    contract_state: ContractState,
    repo: String,
    address_str: String,
) -> Result<RoleResponse> {
    let contract = contract_state.get_contract(&repo).await
        .ok_or_else(|| anyhow::anyhow!("Repository not found"))?;

    let address = Address::from_str(&address_str)
        .map_err(|_| anyhow::anyhow!("Invalid address format"))?;

    contract.revoke_pusher_role(address).await?;

    Ok(RoleResponse {
        repo,
        address: address_str,
        role: "pusher".to_string(),
        granted: false,
    })
}

pub async fn grant_admin_role(
    State(contract_state): State<ContractState>,
    Path((repo, address)): Path<(String, String)>,
) -> impl IntoResponse {
    match handle_grant_admin_role(contract_state, repo, address).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

async fn handle_grant_admin_role(
    contract_state: ContractState,
    repo: String,
    address_str: String,
) -> Result<RoleResponse> {
    let contract = contract_state.get_contract(&repo).await
        .ok_or_else(|| anyhow::anyhow!("Repository not found"))?;

    let address = Address::from_str(&address_str)
        .map_err(|_| anyhow::anyhow!("Invalid address format"))?;

    contract.grant_admin_role(address).await?;

    Ok(RoleResponse {
        repo,
        address: address_str,
        role: "admin".to_string(),
        granted: true,
    })
}

// Revoke admin role
pub async fn revoke_admin_role(
    State(contract_state): State<ContractState>,
    Path((repo, address)): Path<(String, String)>,
) -> impl IntoResponse {
    match handle_revoke_admin_role(contract_state, repo, address).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

async fn handle_revoke_admin_role(
    contract_state: ContractState,
    repo: String,
    address_str: String,
) -> Result<RoleResponse> {
    let contract = contract_state.get_contract(&repo).await
        .ok_or_else(|| anyhow::anyhow!("Repository not found"))?;

    let address = Address::from_str(&address_str)
        .map_err(|_| anyhow::anyhow!("Invalid address format"))?;

    contract.revoke_admin_role(address).await?;

    Ok(RoleResponse {
        repo,
        address: address_str,
        role: "admin".to_string(),
        granted: false,
    })
}

pub async fn check_pusher_role(
    State(contract_state): State<ContractState>,
    Path((repo, address)): Path<(String, String)>,
) -> impl IntoResponse {
    match handle_check_pusher_role(contract_state, repo, address).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

async fn handle_check_pusher_role(
    contract_state: ContractState,
    repo: String,
    address_str: String,
) -> Result<RoleCheckResponse> {
    let contract = contract_state.get_contract(&repo).await
        .ok_or_else(|| anyhow::anyhow!("Repository not found"))?;

    let address = Address::from_str(&address_str)
        .map_err(|_| anyhow::anyhow!("Invalid address format"))?;

    let has_role = contract.has_pusher_role(address).await?;

    Ok(RoleCheckResponse {
        repo,
        address: address_str,
        role: "pusher".to_string(),
        has_role,
    })
}

pub async fn check_admin_role(
    State(contract_state): State<ContractState>,
    Path((repo, address)): Path<(String, String)>,
) -> impl IntoResponse {
    match handle_check_admin_role(contract_state, repo, address).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

async fn handle_check_admin_role(
    contract_state: ContractState,
    repo: String,
    address_str: String,
) -> Result<RoleCheckResponse> {
    let contract = contract_state.get_contract(&repo).await
        .ok_or_else(|| anyhow::anyhow!("Repository not found"))?;

    let address = Address::from_str(&address_str)
        .map_err(|_| anyhow::anyhow!("Invalid address format"))?;

    let has_role = contract.has_admin_role(address).await?;

    Ok(RoleCheckResponse {
        repo,
        address: address_str,
        role: "admin".to_string(),
        has_role,
    })
}
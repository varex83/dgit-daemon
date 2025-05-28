use axum::{extract::{Path, State}, response::IntoResponse, Json};
use onchain::contract_interaction::ContractInteraction;
use serde::Serialize;
use anyhow::Result;

use crate::state::ContractState;

#[derive(Debug, Serialize)]
pub struct CreateRepoResponse {
    pub repo: String,
    pub address: String,
}

pub async fn create_repo(
    State(contract_state): State<ContractState>,
    Path(repo): Path<String>,
) -> impl IntoResponse {
    match handle_create_repo(contract_state, repo).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

async fn handle_create_repo(
    contract_state: ContractState,
    repo: String,
) -> Result<CreateRepoResponse> {
    let contract = contract_state.get_contract(&repo).await;
    if contract.is_some() {
        return Err(anyhow::anyhow!("Repository already exists"));
    }

    let contract = ContractInteraction::deploy().await?;
    contract_state.insert_contract(repo.clone(), contract.clone()).await;

    Ok(CreateRepoResponse { repo, address: contract.address() })
}

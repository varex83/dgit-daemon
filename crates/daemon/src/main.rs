use std::net::SocketAddr;

use axum::{
    routing::{get, post},
    Router,
};
use daemon::{handlers::{create_repo, health_check, receive_pack, upload_pack, info_refs}, state::ContractState};
use tracing::info;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let contract_state = ContractState::new();

    let app = Router::new()
        .route("/{repo}/git-upload-pack", post(upload_pack))
        .route("/{repo}/git-receive-pack", post(receive_pack))
        .route("/{repo}/info/refs", get(info_refs))
        .route("/create-repo/{repo}", post(create_repo))
        .route("/health", get(health_check))
        .with_state(contract_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

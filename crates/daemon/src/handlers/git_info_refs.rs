use axum::{extract::{Path, State, Query}, response::IntoResponse};
use anyhow::{anyhow, bail, Result};
use tracing::{debug, info, warn};
use serde::Deserialize;
use tokio::process::Command;
use tempfile;
use std::process::Stdio;
use crate::state::ContractState;

#[derive(Debug, Deserialize)]
pub struct InfoRefsQuery {
    service: Option<String>,
}

pub async fn info_refs(
    Query(query): Query<InfoRefsQuery>,
    State(contract_state): State<ContractState>,
    Path(repo): Path<String>,
) -> impl IntoResponse {
    let service = query.service.unwrap_or_default();
    info!("Git info_refs called for repo: {} with service: {}", repo, service);

    match handle_info_refs(contract_state, repo, &service).await {
        Ok(response) => {
            let content_type = if service == "git-upload-pack" {
                "application/x-git-upload-pack-advertisement"
            } else if service == "git-receive-pack" {
                "application/x-git-receive-pack-advertisement"
            } else {
                "text/plain"
            };

            let mut headers = axum::http::HeaderMap::new();
            headers.insert(axum::http::header::CONTENT_TYPE, content_type.parse().unwrap());
            headers.insert(axum::http::header::CACHE_CONTROL, "no-cache".parse().unwrap());
            headers.insert(axum::http::header::CONNECTION, "keep-alive".parse().unwrap());

            (headers, response).into_response()
        },
        Err(e) => {
            warn!("Error in info_refs: {:?}", e);
            (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response()
        },
    }
}

async fn handle_info_refs(
    contract_state: ContractState,
    repo: String,
    service: &str,
) -> Result<Vec<u8>> {
    // First, verify that the repository exists
    info!("Looking up contract for repo: {}", repo);
    let contract = contract_state.get_contract(&repo).await
        .ok_or_else(|| anyhow!("Repository not found"))?;

    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path();

    debug!("Created temporary directory: {:?}", temp_path);

    let output = Command::new("git")
        .args(["init", "--bare"])
        .current_dir(temp_path)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to initialize git repo: {}", stderr));
    }

    info!("Fetching refs from blockchain for repo: {}", repo);
    let refs = contract.get_refs().await?;

    info!("Found {} refs for repo {}", refs.len(), repo);
    debug!("Setting up {} refs in the repository", refs.len());

    let refs_dir = temp_path.join("refs");
    let heads_dir = refs_dir.join("heads");
    tokio::fs::create_dir_all(&heads_dir).await?;

    let tags_dir = refs_dir.join("tags");
    tokio::fs::create_dir_all(&tags_dir).await?;

    let objects_dir = temp_path.join("objects");
    tokio::fs::create_dir_all(&objects_dir).await?;

    for ref_data in &refs {
        if ref_data.is_active {
            let ref_name = &ref_data.name;
            let sha1 = match String::from_utf8(ref_data.data.clone()) {
                Ok(s) => s,
                Err(_) => {
                    bail!("Failed to convert ref data to string");
                },
            };

            if sha1.len() != 40 || !ref_name.starts_with("refs/") {
                bail!("Malformed ref {}: {}", ref_name, sha1);
            }

            debug!("Setting up ref {}: {}", ref_name, sha1);

            let ref_file_path = temp_path.join(ref_name);
            if let Some(parent) = ref_file_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }

            tokio::fs::write(&ref_file_path, format!("{}\n", sha1)).await?;
        }
    }

    let update_server_info = Command::new("git")
        .args(["update-server-info"])
        .current_dir(temp_path)
        .output()
        .await?;

    if !update_server_info.status.success() {
        let stderr = String::from_utf8_lossy(&update_server_info.stderr);
        warn!("Failed to update server info: {}", stderr);
    }

    match service {
        "git-upload-pack" | "git-receive-pack" => {
            // Use Git's built-in command to generate the advertisement
            let git_command = if service == "git-upload-pack" {
                "upload-pack"
            } else {
                "receive-pack"
            };

            let mut cmd = Command::new("git");
            cmd.args([git_command, "--advertise-refs", "."])
                .current_dir(temp_path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            let output = cmd.output().await?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow!("Failed to generate refs advertisement: {}", stderr));
            }

            let mut response = Vec::new();

            let service_announcement = format!("# service={}\n", service);
            let pkt_len = 4 + service_announcement.len();
            let pkt_header = format!("{:04x}", pkt_len);

            response.extend_from_slice(pkt_header.as_bytes());
            response.extend_from_slice(service_announcement.as_bytes());

            response.extend_from_slice(b"0000");
            response.extend_from_slice(&output.stdout);

            debug!("Generated refs advertisement of size {} bytes", response.len());

            Ok(response)
        },
        _ => {
            Err(anyhow!("Unknown service: {}", service))
        }
    }
}
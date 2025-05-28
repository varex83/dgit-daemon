use axum::{extract::{Path, State}, response::IntoResponse};
use anyhow::{anyhow, Result};
use tokio::process::Command;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info, error, debug};
use tempfile::tempdir;
use crate::state::ContractState;
use std::path::PathBuf;
use std::process::Stdio;
use onchain::ipfs;

pub async fn upload_pack(
    State(contract_state): State<ContractState>,
    Path(repo): Path<String>,
    req_body: axum::body::Body,
) -> impl IntoResponse {
    info!("Git upload-pack called for repo: {}", repo);
    match handle_upload_pack(contract_state, repo, req_body).await {
        Ok(response) => {
            info!("Successfully processed upload-pack request, response size: {} bytes", response.len());

            let mut headers = axum::http::HeaderMap::new();
            headers.insert(axum::http::header::CONTENT_TYPE, "application/x-git-upload-pack-result".parse().unwrap());
            headers.insert(axum::http::header::CACHE_CONTROL, "no-cache".parse().unwrap());
            headers.insert(axum::http::header::CONNECTION, "keep-alive".parse().unwrap());

            (headers, response).into_response()
        },
        Err(e) => {
            error!("Error in upload_pack: {:?}", e);
            (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response()
        }
    }
}

async fn handle_upload_pack(
    contract_state: ContractState,
    repo: String,
    req_body: axum::body::Body,
) -> Result<Vec<u8>> {
    info!("Looking up contract for repo: {}", repo);
    let contract = contract_state.get_contract(&repo).await
        .ok_or_else(|| anyhow!("Repository not found"))?;

    let temp_dir = tempdir()?;
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

    if refs.is_empty() {
        return Err(anyhow!("Repository has no refs"));
    }

    let refs_dir = temp_path.join("refs");
    let heads_dir = refs_dir.join("heads");
    tokio::fs::create_dir_all(&heads_dir).await?;

    let tags_dir = refs_dir.join("tags");
    tokio::fs::create_dir_all(&tags_dir).await?;

    let objects_dir = temp_path.join("objects");
    let objects_info_dir = objects_dir.join("info");
    let objects_pack_dir = objects_dir.join("pack");
    tokio::fs::create_dir_all(&objects_info_dir).await?;
    tokio::fs::create_dir_all(&objects_pack_dir).await?;

    for ref_data in &refs {
        if ref_data.is_active {
            let ref_name = &ref_data.name;
            let sha1 = String::from_utf8(ref_data.data.clone())?;

            debug!("Setting up ref {}: {}", ref_name, sha1);

            let ref_file_path = temp_path.join(ref_name);
            if let Some(parent) = ref_file_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }

            tokio::fs::write(&ref_file_path, format!("{}\n", sha1)).await?;
        }
    }

    let body_bytes = axum::body::to_bytes(req_body, usize::MAX).await?;
    debug!("Client request size: {} bytes", body_bytes.len());

    let wanted_commits = parse_wanted_objects(&body_bytes)?;
    info!("Client wants {} commits", wanted_commits.len());

    if !wanted_commits.is_empty() {
        for commit_hash in &wanted_commits {
            debug!("Checking if commit {} exists in contract", commit_hash);
            match contract.is_object_exist(commit_hash.clone()).await {
                Ok(true) => {
                    debug!("Commit {} verified in the blockchain", commit_hash);
                },
                Ok(false) => {
                    error!("Commit {} not found in blockchain", commit_hash);
                    return Err(anyhow!("upload-pack: not our ref {}", commit_hash));
                },
                Err(e) => {
                    error!("Error checking commit {} existence: {}", commit_hash, e);
                    return Err(anyhow!("Error checking commit existence: {}", e));
                }
            }
        }
    }

    let objects = contract.get_objects().await?;
    info!("Fetched {} objects from blockchain", objects.len());

    for object in objects {
        let object_hash = object.hash;
        let ipfs_url = String::from_utf8(object.ipfs_url)?;
        let object_path = get_object_path(temp_path, &object_hash);

        let local_path = objects_dir.join(object_path);
        let local_path_str = local_path.to_string_lossy();

        ipfs::download_from_ipfs(&ipfs_url, &local_path_str).await?;
    }

    debug!("Running git upload-pack command");
    let mut cmd = Command::new("git");
    cmd.args(["upload-pack", "--stateless-rpc", "."])
        .current_dir(temp_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(&body_bytes).await?;
    }

    let mut response = Vec::new();
    if let Some(mut stdout) = child.stdout.take() {
        stdout.read_to_end(&mut response).await?;
    }

    let status = child.wait().await?;
    if !status.success() {
        let mut err_msg = Vec::new();
        if let Some(mut stderr) = child.stderr.take() {
            stderr.read_to_end(&mut err_msg).await?;
        }
        let err_str = String::from_utf8_lossy(&err_msg);
        error!("git upload-pack stderr: {}", err_str);

        if response.is_empty() {
            return Err(anyhow!("git upload-pack failed: {}", err_str));
        }
    }

    debug!("Generated response of size {} bytes", response.len());
    Ok(response)
}

fn parse_wanted_objects(body: &[u8]) -> Result<Vec<String>> {
    let body_str = std::str::from_utf8(body)?;
    let mut wanted = Vec::new();

    for line in body_str.lines() {
        if line.starts_with("want ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                wanted.push(parts[1].to_string());
            }
        }
    }

    Ok(wanted)
}

pub fn get_object_path(repo_path: &std::path::Path, hash: &str) -> PathBuf {
    if hash.len() < 2 {
        return repo_path.join("objects").join(hash);
    }

    let dir = &hash[0..2];
    let file = &hash[2..];
    repo_path.join("objects").join(dir).join(file)
}

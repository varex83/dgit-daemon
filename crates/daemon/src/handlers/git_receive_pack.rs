use axum::{extract::{Path, State}, response::IntoResponse};
use anyhow::{anyhow, Result};
use tokio::process::Command;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::fs;
use tracing::{info, error, debug};
use tempfile::tempdir;
use walkdir::WalkDir;
use std::process::Stdio;
use onchain::ipfs;
use crate::{handlers::get_object_path, state::ContractState};

pub async fn receive_pack(
    State(contract_state): State<ContractState>,
    Path(repo): Path<String>,
    req_body: axum::body::Body,
) -> impl IntoResponse {
    info!("Git receive-pack called for repo: {}", repo);
    match handle_receive_pack(contract_state, repo, req_body).await {
        Ok(response) => {
            info!("Successfully processed receive-pack request, response size: {} bytes", response.len());

            let mut headers = axum::http::HeaderMap::new();
            headers.insert(axum::http::header::CONTENT_TYPE, "application/x-git-receive-pack-result".parse().unwrap());
            headers.insert(axum::http::header::CACHE_CONTROL, "no-cache".parse().unwrap());
            headers.insert(axum::http::header::CONNECTION, "keep-alive".parse().unwrap());

            (headers, response).into_response()
        },
        Err(e) => {
            error!("Error in receive_pack: {:?}", e);
            (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response()
        }
    }
}

async fn handle_receive_pack(
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

    info!("Fetching existing refs from blockchain for repo: {}", repo);
    let existing_refs = contract.get_refs().await?;
    info!("Found {} existing refs for repo {}", existing_refs.len(), repo);

    let refs_dir = temp_path.join("refs");
    let heads_dir = refs_dir.join("heads");
    tokio::fs::create_dir_all(&heads_dir).await?;

    let tags_dir = refs_dir.join("tags");
    tokio::fs::create_dir_all(&tags_dir).await?;

    for ref_data in &existing_refs {
        let ref_name = &ref_data.name;
        let sha1 = String::from_utf8(ref_data.data.clone())?;

        debug!("Setting up ref {}: {}", ref_name, sha1);

        let ref_file_path = temp_path.join(ref_name);
        if let Some(parent) = ref_file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&ref_file_path, format!("{}\n", sha1)).await?;
    }

    let objects_dir = temp_path.join("objects");
    tokio::fs::create_dir_all(&objects_dir).await?;

    let objects = contract.get_objects().await?;
    for object in objects {
        let object_hash = object.hash;
        let ipfs_url = String::from_utf8(object.ipfs_url)?;
        let object_path = get_object_path(temp_path, &object_hash);
        let local_path = objects_dir.join(object_path);
        let local_path_str = local_path.to_string_lossy();
        ipfs::download_from_ipfs(&ipfs_url, &local_path_str).await?;
    }

    let body_bytes = axum::body::to_bytes(req_body, usize::MAX).await?;
    debug!("Client request size: {} bytes", body_bytes.len());

    debug!("Running git receive-pack command");
    let mut cmd = Command::new("git");
    cmd.args(["receive-pack", "--stateless-rpc", "."])
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
        error!("git receive-pack failed: {}", err_str);
        return Err(anyhow!("git receive-pack failed: {}", err_str));
    }

    let objects_dir = temp_path.join("objects");

    info!("Scanning for new objects to upload to IPFS");
    let mut objects_to_upload = Vec::new();
    for entry in WalkDir::new(&objects_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file()) {

        let object_path = entry.path();
        let obj_dir_name = object_path.parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("");

        let obj_file_name = entry.file_name().to_str().unwrap_or("");
        let obj_hash = format!("{}{}", obj_dir_name, obj_file_name);

        debug!("Checking if object {} exists in blockchain", obj_hash);

        match contract.is_object_exist(obj_hash.clone()).await {
            Ok(true) => {
                debug!("Object {} already exists in blockchain, skipping", obj_hash);
            },
            _ => {
                debug!("Found new object to upload: {}", obj_hash);
                objects_to_upload.push((obj_hash, object_path.to_path_buf()));
            }
        }
    }

    info!("Found {} new objects to upload", objects_to_upload.len());

    let mut object_hashes = Vec::new();
    let mut ipfs_urls = Vec::new();

    for (obj_hash, obj_path) in objects_to_upload {
        let path_str = obj_path.to_string_lossy();

        debug!("Uploading object {} to IPFS", obj_hash);
        match ipfs::load_to_ipfs(&path_str).await {
            Ok(ipfs_hash) => {
                debug!("Object {} uploaded to IPFS with hash {}", obj_hash, ipfs_hash);
                object_hashes.push(obj_hash);
                ipfs_urls.push(ipfs_hash.as_bytes().to_vec());
            },
            Err(e) => {
                error!("Failed to upload object {} to IPFS: {}", obj_hash, e);
                return Err(anyhow!("Failed to upload object to IPFS: {}", e));
            }
        }
    }

    if !object_hashes.is_empty() {
        info!("Storing {} object hashes in blockchain", object_hashes.len());
        match contract.add_objects(object_hashes.clone(), ipfs_urls).await {
            Ok(_) => debug!("Successfully stored object hashes in blockchain"),
            Err(e) => {
                error!("Failed to store object hashes in blockchain: {}", e);
                return Err(anyhow!("Failed to store object hashes in blockchain: {}", e));
            }
        }
    }

    info!("Collecting updated refs");
    let mut updated_refs = Vec::new();
    let mut ref_data = Vec::new();

    for entry in WalkDir::new(heads_dir)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file()) {

        let ref_path = entry.path();
        let ref_content = fs::read_to_string(ref_path).await?;
        let ref_content = ref_content.trim();

        let heads_rel_path = ref_path.strip_prefix(temp_path)?;
        let ref_name = heads_rel_path.to_string_lossy().to_string();

        debug!("Found updated ref: {} -> {}", ref_name, ref_content);
        updated_refs.push(ref_name);
        ref_data.push(ref_content.as_bytes().to_vec());
    }

    for entry in WalkDir::new(tags_dir)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file()) {

        let ref_path = entry.path();
        let ref_content = fs::read_to_string(ref_path).await?;
        let ref_content = ref_content.trim();

        let tags_rel_path = ref_path.strip_prefix(temp_path)?;
        let ref_name = tags_rel_path.to_string_lossy().to_string();

        debug!("Found updated tag: {} -> {}", ref_name, ref_content);
        updated_refs.push(ref_name);
        ref_data.push(ref_content.as_bytes().to_vec());
    }

    if !updated_refs.is_empty() {
        info!("Storing {} updated refs in blockchain", updated_refs.len());
        match contract.add_refs(updated_refs.clone(), ref_data).await {
            Ok(_) => debug!("Successfully stored updated refs in blockchain"),
            Err(e) => {
                error!("Failed to store refs in blockchain: {}", e);
                return Err(anyhow!("Failed to store refs in blockchain: {}", e));
            }
        }

        for ref_name in updated_refs.iter() {
            debug!("Verifying ref {} was properly stored", ref_name);
            let mut found = false;

            for blockchain_ref in contract.get_refs().await? {
                if blockchain_ref.name == *ref_name && blockchain_ref.is_active {
                    found = true;
                    break;
                }
            }

            if !found {
                error!("Failed to verify ref {} was stored in blockchain", ref_name);
                return Err(anyhow!("Failed to verify ref was stored in blockchain: {}", ref_name));
            }
        }
    }

    info!("Push operation completed successfully");
    Ok(response)
}

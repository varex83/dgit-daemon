use crate::config::Config;
use anyhow::{bail, Result};
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde::Deserialize;
use std::path::Path;
use tokio::fs::{create_dir_all, File, read};
use tokio::io::AsyncWriteExt;
use tracing::{debug, info, error, instrument, warn};

#[derive(Debug, Deserialize)]
struct IPFSAddResponse {
    #[allow(dead_code)]
    #[serde(default, rename = "Name")]
    name: String,

    #[allow(dead_code)]
    #[serde(default, rename = "Hash")]
    hash: String,

    #[allow(dead_code)]
    #[serde(default, rename = "Size")]
    size: String,
}


fn extract_git_object(content: &[u8]) -> Result<(String, Vec<u8>)> {
    if let Some(null_pos) = content.iter().position(|&b| b == 0) {
        let header = std::str::from_utf8(&content[..null_pos])?;
        let parts: Vec<&str> = header.split(' ').collect();
        if parts.len() != 2 {
            bail!("Invalid Git object header format");
        }

        let obj_type = parts[0].to_string();
        let data = content[null_pos+1..].to_vec();

        Ok((obj_type, data))
    } else {
        bail!("Invalid Git object format: missing null byte separator");
    }
}

#[instrument(skip_all, fields(file_path = file_path), err)]
pub async fn load_to_ipfs(file_path: &str) -> Result<String> {
    info!("Loading file to local IPFS daemon: {}", file_path);

    let ipfs_api = Config::ipfs_api_url().unwrap_or_else(|| "http://127.0.0.1:5001".to_string());
    debug!("Using IPFS API URL: {}", ipfs_api);

    let content = match read(file_path).await {
        Ok(content) => content,
        Err(e) => {
            error!("Failed to read file {}: {}", file_path, e);
            bail!("Failed to read file: {}", e);
        }
    };
    debug!("Read file content, size: {} bytes", content.len());

    let filename = Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("git_object");

    debug!("Using filename for upload: {}", filename);

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(5))
        .build()?;

    for attempt in 1..=3 {
        info!("Uploading to local IPFS daemon (attempt {}/3)", attempt);

        match upload_to_ipfs(&client, &ipfs_api, &content, filename).await {
            Ok(cid) => {
                info!("Successfully uploaded file to IPFS, CID: {}", cid);

                let gateway = Config::ipfs_prefix();
                if !gateway.is_empty() {
                    debug!("Verifying content is retrievable from gateway: {}", gateway);
                    let verification_url = format!("{}{}", gateway, cid);

                    match client.head(&verification_url).send().await {
                        Ok(resp) => {
                            if resp.status().is_success() {
                                info!("CID {} verified as retrievable from gateway", cid);
                            } else {
                                warn!("CID {} returned status code {} from gateway", cid, resp.status());
                                warn!("Content may not be immediately retrievable, may need time to propagate");
                            }
                        },
                        Err(e) => {
                            warn!("Failed to verify CID availability: {}", e);
                            warn!("Content may not be immediately retrievable, may need time to propagate");
                        }
                    }
                }

                return Ok(cid);
            },
            Err(e) => {
                if attempt == 3 {
                    error!("All upload attempts failed. Last error: {}", e);
                    bail!("Failed to upload file to IPFS after 3 attempts: {}", e);
                }

                warn!("Upload attempt {} failed: {}. Retrying...", attempt, e);
                let backoff_ms = 1000 * (1 << (attempt - 1));
                warn!("Waiting {}ms before next attempt", backoff_ms);
                tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
            }
        }
    }

    bail!("Failed to upload to IPFS after maximum retries");
}

async fn upload_to_ipfs(client: &Client, ipfs_api: &str, content: &[u8], filename: &str) -> Result<String> {
    debug!("Uploading to IPFS daemon with filename: {}", filename);

    let upload_content = if content.len() > 10 {
        if let Ok((obj_type, _)) = extract_git_object(content) {
            debug!("Detected Git object of type: {}", obj_type);
            content.to_vec()
        } else {
            content.to_vec()
        }
    } else {
        content.to_vec()
    };

    // Important: Don't modify Git object binary format
    let file_part = Part::bytes(upload_content)
        .file_name(filename.to_owned())
        .mime_str("application/octet-stream")?;

    let upload_url = format!("{}/api/v0/add?pin=true&raw-leaves=true", ipfs_api);
    debug!("Sending POST request to IPFS API: {}", upload_url);

    let form = Form::new().part("file", file_part);

    let resp = match client
        .post(&upload_url)
        .multipart(form)
        .send()
        .await 
    {
        Ok(response) => response,
        Err(e) => {
            error!("Failed to send request to IPFS: {}", e);
            if e.is_timeout() {
                bail!("Request to IPFS timed out. Is your IPFS daemon running?");
            } else if e.is_connect() {
                bail!("Connection error to IPFS API. Make sure your IPFS daemon is running at {}", ipfs_api);
            } else {
                bail!("Failed to send request to IPFS: {}", e);
            }
        }
    };

    let status = resp.status();
    let resp_text = match resp.text().await {
        Ok(text) => text,
        Err(e) => {
            error!("Failed to get response text: {}", e);
            bail!("Failed to parse IPFS response: {}", e);
        }
    };

    debug!("IPFS response status: {}, body: {}", status, resp_text);

    if !status.is_success() {
        error!("IPFS upload failed with status: {}", status);
        bail!("Failed to upload to IPFS: {}", resp_text);
    }

    match serde_json::from_str::<IPFSAddResponse>(&resp_text) {
        Ok(response) => {
            if !response.hash.is_empty() {
                debug!("Successfully extracted CID from response: {}", response.hash);
                return Ok(response.hash);
            }

            error!("Empty hash received from IPFS");
            bail!("Invalid response from IPFS: no hash returned");
        },
        Err(e) => {
            error!("Failed to parse IPFS response as JSON: {}", e);
            error!("Response body: {}", resp_text);
            bail!("Failed to parse IPFS response: {}", e);
        }
    }
}

#[instrument(skip_all, fields(ipfs_hash = ipfs_hash, file_path = file_path), err)]
pub async fn download_from_ipfs(ipfs_hash: &str, file_path: &str) -> Result<()> {
    info!("Downloading from IPFS: {} -> {}", ipfs_hash, file_path);

    if let Some(parent) = Path::new(file_path).parent() {
        debug!("Creating parent directories: {:?}", parent);
        match create_dir_all(parent).await {
            Ok(_) => debug!("Parent directories created successfully"),
            Err(e) => {
                error!("Failed to create parent directories: {}", e);
                return Err(anyhow::anyhow!("Failed to create directories: {}", e));
            }
        }
    }

    let gateway_prefix = Config::ipfs_prefix();
    let ipfs_api = Config::ipfs_api_url().unwrap_or_else(|| "http://127.0.0.1:5001".to_string());

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    for attempt in 1..=3 {
        info!("Attempting to download from IPFS (attempt {}/3)", attempt);

        if attempt > 1 {
            let backoff_ms = 1000 * (1 << (attempt - 2));
            debug!("Backing off for {}ms before retry", backoff_ms);
            tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
        }

        let block_url = format!("{}/api/v0/block/get?arg={}", ipfs_api, ipfs_hash);
        debug!("Trying to download raw block from IPFS API: {}", block_url);

        let downloaded_content = match client.post(&block_url).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.bytes().await {
                        Ok(bytes) => {
                            debug!("Downloaded {} bytes from IPFS block API", bytes.len());
                            Some(bytes.to_vec())
                        },
                        Err(e) => {
                            warn!("Failed to read response body from IPFS block API: {}", e);
                            None
                        }
                    }
                } else {
                    warn!("IPFS block/get API returned status {}, trying alternative", resp.status());
                    None
                }
            },
            Err(e) => {
                warn!("Failed to download via IPFS block API: {}", e);
                None
            }
        };

        if let Some(content) = downloaded_content {
            let mut dest = match File::create(file_path).await {
                Ok(file) => file,
                Err(e) => {
                    error!("Failed to create output file {}: {}", file_path, e);
                    return Err(anyhow::anyhow!("Failed to create file: {}", e));
                }
            };

            if let Err(e) = dest.write_all(&content).await {
                error!("Failed to write data to file: {}", e);
                return Err(anyhow::anyhow!("Failed to write file: {}", e));
            }

            info!("Successfully downloaded IPFS content ({} bytes) to {}", content.len(), file_path);
            return Ok(());
        }

        let cat_url = format!("{}/api/v0/cat?arg={}", ipfs_api, ipfs_hash);
        debug!("Trying to download from IPFS cat API: {}", cat_url);

        let downloaded_content = match client.post(&cat_url).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.bytes().await {
                        Ok(bytes) => {
                            debug!("Downloaded {} bytes from IPFS cat API", bytes.len());
                            Some(bytes.to_vec())
                        },
                        Err(e) => {
                            warn!("Failed to read response body from IPFS cat API: {}", e);
                            None
                        }
                    }
                } else {
                    warn!("IPFS cat API returned status {}", resp.status());
                    None
                }
            },
            Err(e) => {
                warn!("Failed to download via IPFS cat API: {}", e);
                None
            }
        };

        if let Some(content) = downloaded_content {
            let mut dest = match File::create(file_path).await {
                Ok(file) => file,
                Err(e) => {
                    error!("Failed to create output file {}: {}", file_path, e);
                    return Err(anyhow::anyhow!("Failed to create file: {}", e));
                }
            };

            if let Err(e) = dest.write_all(&content).await {
                error!("Failed to write data to file: {}", e);
                return Err(anyhow::anyhow!("Failed to write file: {}", e));
            }

            info!("Successfully downloaded IPFS content ({} bytes) to {}", content.len(), file_path);
            return Ok(());
        }

        if !gateway_prefix.is_empty() {
            let gateway_url = format!("{}{}", gateway_prefix, ipfs_hash);
            debug!("Trying to download from IPFS gateway: {}", gateway_url);

            let downloaded_content = match client.get(&gateway_url).send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        match resp.bytes().await {
                            Ok(bytes) => {
                                debug!("Downloaded {} bytes from IPFS gateway", bytes.len());
                                Some(bytes.to_vec())
                            },
                            Err(e) => {
                                warn!("Failed to read response body from gateway: {}", e);
                                None
                            }
                        }
                    } else {
                        warn!("Gateway returned status {}", resp.status());
                        None
                    }
                },
                Err(e) => {
                    warn!("Failed to connect to gateway: {}", e);
                    None
                }
            };

            if let Some(content) = downloaded_content {
                let mut dest = match File::create(file_path).await {
                    Ok(file) => file,
                    Err(e) => {
                        error!("Failed to create output file {}: {}", file_path, e);
                        return Err(anyhow::anyhow!("Failed to create file: {}", e));
                    }
                };

                if let Err(e) = dest.write_all(&content).await {
                    error!("Failed to write data to file: {}", e);
                    return Err(anyhow::anyhow!("Failed to write file: {}", e));
                }

                info!("Successfully downloaded IPFS content ({} bytes) to {}", content.len(), file_path);
                return Ok(());
            }
        }
    }

    error!("Failed to download from IPFS after maximum retries");
    Err(anyhow::anyhow!("Failed to download from IPFS after all attempts"))
}

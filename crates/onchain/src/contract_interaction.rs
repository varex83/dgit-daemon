use crate::config::Config;
use anyhow::Result;
use ethcontract::prelude::*;
use std::str::FromStr;
use tracing::{debug, info, error, trace, instrument, warn};

ethcontract::contract!("crates/onchain/artifacts/contracts/RepositoryContract.sol/RepositoryContract.json");

#[derive(Debug, Clone)]
pub struct ContractInteraction {
    pub contract: RepositoryContract,
    pub client: Web3<Http>,
}

#[derive(Debug, Clone)]
pub struct Object {
    pub hash: String,
    pub ipfs_url: Vec<u8>,
    pub pusher: Address,
}

#[derive(Debug, Clone)]
pub struct Ref {
    pub name: String,
    pub data: Vec<u8>,
    pub is_active: bool,
    pub pusher: Address,
}

impl Default for ContractInteraction {
    fn default() -> Self {
        let rpc_url = Config::rpc_url();
        debug!("Initializing ContractInteraction with RPC URL: {}", rpc_url);
        
        let http = Http::new(&rpc_url).unwrap();
        let client = Web3::new(http);

        let contract = RepositoryContract::at(
            &client,
            Address::from_str("0x0000000000000000000000000000000000000000").unwrap(),
        );

        info!("ContractInteraction initialized with default zero address");
        ContractInteraction { contract, client }
    }
}

impl ContractInteraction {
    pub fn new() -> Self {
        debug!("Creating new ContractInteraction using default implementation");
        Self::default()
    }

    #[instrument(err)]
    pub async fn deploy() -> Result<Self> {
        let rpc_url = dotenv::var("RPC_URL").unwrap_or("http://localhost:8545".to_string());
        info!("Deploying new contract to RPC endpoint: {}", rpc_url);

        let http = Http::new(&rpc_url).unwrap();
        let client = Web3::new(http);

        debug!("Initiating contract deployment");
        let contract = RepositoryContract::builder(&client)
            .gas(4_000_000.into())
            .deploy()
            .await?;

        let address = contract.address();
        info!("Contract successfully deployed at address: {:?}", address);

        Ok(ContractInteraction { contract, client })
    }

    pub fn address(&self) -> String {
        let bytes = self.contract.address().to_fixed_bytes();
        let mut address = "0x".to_string();
        for byte in bytes {
            address.push_str(&format!("{:02x}", byte));
        }
        trace!("Contract address: {}", address);
        address
    }

    #[instrument(skip(self, ipfs_url), fields(hash_len = hash.len(), ipfs_url_len = ipfs_url.len()), err)]
    pub async fn save_object(&self, hash: String, ipfs_url: Vec<u8>) -> Result<()> {
        info!("Saving object with hash: {}", hash);
        trace!("IPFS URL length: {} bytes", ipfs_url.len());

        match self.contract
            .save_object(hash.clone(), Bytes(ipfs_url))
            .send()
            .await {
                Ok(tx) => {
                    info!("Object saved successfully, tx hash: {:?}", tx.hash());
                    debug!("Transaction details: {:?}", tx);
                    Ok(())
                },
                Err(e) => {
                    error!("Failed to save object with hash {}: {}", hash, e);
                    Err(anyhow::Error::from(e))
                }
            }
    }

    #[instrument(skip(self, data), fields(ref_name = reference, data_len = data.len()), err)]
    pub async fn add_ref(&self, reference: String, data: Vec<u8>) -> Result<()> {
        info!("Adding ref: {}, data length: {} bytes", reference, data.len());

        match self.contract
            .add_ref(reference.clone(), Bytes(data))
            .send()
            .await {
                Ok(tx) => {
                    info!("Ref added successfully, tx hash: {:?}", tx.hash());
                    debug!("Transaction details: {:?}", tx);
                    Ok(())
                },
                Err(e) => {
                    error!("Failed to add ref {}: {}", reference, e);
                    Err(anyhow::Error::from(e))
                }
            }
    }

    #[instrument(skip(self, config), fields(config_len = config.len()), err)]
    pub async fn update_config(&self, config: Vec<u8>) -> Result<()> {
        info!("Updating contract config, data size: {} bytes", config.len());

        match self.contract
            .update_config(Bytes(config))
            .send()
            .await {
                Ok(tx) => {
                    info!("Config updated successfully, tx hash: {:?}", tx.hash());
                    debug!("Transaction details: {:?}", tx);
                    Ok(())
                },
                Err(e) => {
                    error!("Failed to update config: {}", e);
                    Err(anyhow::Error::from(e))
                }
            }
    }

    #[instrument(skip(self), err)]
    pub async fn get_config(&self) -> Result<Vec<u8>> {
        debug!("Retrieving contract config");

        match self.contract
            .get_config()
            .call()
            .await {
                Ok(Bytes(data)) => {
                    info!("Retrieved config, size: {} bytes", data.len());
                    trace!("Config data: {:?}", data);
                    Ok(data.to_vec())
                },
                Err(e) => {
                    error!("Failed to get config: {}", e);
                    Err(anyhow::Error::from(e))
                }
            }
    }

    #[instrument(skip(self), err)]
    pub async fn get_object_by_id(&self, id: U256) -> Result<Object> {
        info!("Retrieving object by ID: {}", id);

        match self.contract
            .get_object_by_id(id)
            .call()
            .await {
                Ok((hash, ipfs_url, pusher)) => {
                    info!("Retrieved object {} with hash: {}", id, hash);
                    debug!("Object details - IPFS URL length: {} bytes, pusher: {:?}", ipfs_url.0.len(), pusher);

                    Ok(Object {
                        hash,
                        ipfs_url: ipfs_url.0,
                        pusher,
                    })
                },
                Err(e) => {
                    error!("Failed to retrieve object by ID {}: {}", id, e);
                    Err(anyhow::Error::from(e))
                }
            }
    }

    #[instrument(skip(self), err)]
    pub async fn get_object(&self, hash: String) -> Result<Object> {
        info!("Retrieving object with hash: {}", hash);

        match self.contract
            .get_object(hash.clone())
            .call()
            .await {
                Ok((hash, ipfs_url, pusher)) => {
                    info!("Retrieved object with hash: {}", hash);
                    debug!("Object details - IPFS URL length: {} bytes, pusher: {:?}", ipfs_url.0.len(), pusher);

                    Ok(Object {
                        hash,
                        ipfs_url: ipfs_url.0,
                        pusher,
                    })
                },
                Err(e) => {
                    error!("Failed to retrieve object with hash {}: {}", hash, e);
                    Err(anyhow::Error::from(e))
                }
            }
    }

    #[instrument(skip(self), err)]
    pub async fn is_object_exist(&self, hash: String) -> Result<bool> {
        debug!("Checking if object exists with hash: {}", hash);

        match self.contract
            .is_object_exist(hash.clone())
            .call()
            .await {
                Ok(exists) => {
                    info!("Object check for hash {}: exists = {}", hash, exists);
                    Ok(exists)
                },
                Err(e) => {
                    error!("Failed to check if object exists with hash {}: {}", hash, e);
                    Err(anyhow::Error::from(e))
                }
            }
    }

    #[instrument(skip(self), fields(hashes_count = hashes.len()), err)]
    pub async fn check_objects(&self, hashes: Vec<String>) -> Result<Vec<bool>> {
        info!("Checking existence of {} objects", hashes.len());
        trace!("Object hashes: {:?}", hashes);

        match self.contract
            .check_objects(hashes.clone())
            .call()
            .await {
                Ok(results) => {
                    let exist_count = results.iter().filter(|&exists| *exists).count();
                    info!("Object check results: {}/{} objects exist", exist_count, results.len());
                    debug!("Detailed results: {:?}", results);
                    Ok(results)
                },
                Err(e) => {
                    error!("Failed to check objects: {}", e);
                    Err(anyhow::Error::from(e))
                }
            }
    }

    #[instrument(skip(self, hashes, ipfs_urls), fields(count = hashes.len()), err)]
    pub async fn add_objects(&self, hashes: Vec<String>, ipfs_urls: Vec<Vec<u8>>) -> Result<()> {
        info!("Adding batch of {} objects", hashes.len());
        trace!("Object hashes: {:?}", hashes);

        if hashes.is_empty() || hashes.len() != ipfs_urls.len() {
            error!("Invalid objects data: hashes.len={}, ipfs_urls.len={}", hashes.len(), ipfs_urls.len());
            return Err(anyhow::anyhow!("Invalid objects data: mismatched lengths"));
        }

        let bytes_ipfs_urls = ipfs_urls
            .iter()
            .map(|e| Bytes(e.clone()))
            .collect::<Vec<Bytes<Vec<u8>>>>();

        let max_retries = 3;

        for retry in 0..max_retries {
            if retry > 0 {
                let backoff_ms = 500 * (1 << (retry - 1));
                debug!("Retrying add_objects (attempt {}/{}), waiting {}ms...", retry + 1, max_retries, backoff_ms);
                tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
            }

            let tx_result = self.contract
                .add_objects(hashes.clone(), bytes_ipfs_urls.clone())
                .send()
                .await;

            match tx_result {
                Ok(tx) => {
                    info!("Successfully added {} objects, tx hash: {:?}", hashes.len(), tx.hash());
                    debug!("Transaction details: {:?}", tx);

                    let receipt_result = self.client.eth().transaction_receipt(tx.hash()).await;

                    match receipt_result {
                        Ok(Some(receipt)) => {
                            if receipt.status == Some(1.into()) {
                                info!("Transaction confirmed with success status");
                                return Ok(());
                            } else {
                                error!("Transaction failed with status: {:?}", receipt.status);
                                // Continue to retry
                            }
                        },
                        Ok(None) => {
                            warn!("Transaction receipt not available yet, assuming success");
                            return Ok(());
                        },
                        Err(e) => {
                            error!("Failed to check transaction receipt: {}", e);
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to add objects batch (attempt {}/{}): {}", retry + 1, max_retries, e);

                    let error_msg = e.to_string();
                    let is_recoverable = error_msg.contains("nonce too low") || 
                                         error_msg.contains("gas price too low") ||
                                         error_msg.contains("replacement transaction underpriced");

                    if is_recoverable {
                        debug!("Encountered recoverable error, will retry");
                    } else if retry == max_retries - 1 {
                        return Err(anyhow::anyhow!("Failed to add objects: {}", e));
                    }
                }
            }
        }

        Err(anyhow::anyhow!("Failed to add objects after {} retries", max_retries))
    }

    #[instrument(skip(self, references, data), fields(count = references.len()), err)]
    pub async fn add_refs(&self, references: Vec<String>, data: Vec<Vec<u8>>) -> Result<()> {
        info!("Adding batch of {} refs", references.len());
        trace!("Ref names: {:?}", references);

        if references.is_empty() || references.len() != data.len() {
            error!("Invalid refs data: references.len={}, data.len={}", references.len(), data.len());
            return Err(anyhow::anyhow!("Invalid refs data: mismatched lengths"));
        }

        let bytes_data = data
            .iter()
            .map(|e| Bytes(e.clone()))
            .collect::<Vec<Bytes<Vec<u8>>>>();

        let max_retries = 3;

        for retry in 0..max_retries {
            if retry > 0 {
                let backoff_ms = 500 * (1 << (retry - 1));
                debug!("Retrying add_refs (attempt {}/{}), waiting {}ms...", retry + 1, max_retries, backoff_ms);
                tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
            }

            let tx_result = self.contract
                .add_refs(references.clone(), bytes_data.clone())
                .gas(4_000_000.into())
                .send()
                .await;

            match tx_result {
                Ok(tx) => {
                    info!("Successfully added {} refs, tx hash: {:?}", references.len(), tx.hash());
                    debug!("Transaction details: {:?}", tx);

                    let receipt_result = self.client.eth().transaction_receipt(tx.hash()).await;

                    match receipt_result {
                        Ok(Some(receipt)) => {
                            if receipt.status == Some(1.into()) {
                                info!("Transaction confirmed with success status");
                                return Ok(());
                            } else {
                                error!("Transaction failed with status: {:?}", receipt.status);
                                // Continue to retry
                            }
                        },
                        Ok(None) => {
                            warn!("Transaction receipt not available yet, assuming success");
                            return Ok(());
                        },
                        Err(e) => {
                            error!("Failed to check transaction receipt: {}", e);
                            // Continue to retry
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to add refs batch (attempt {}/{}): {}", retry + 1, max_retries, e);

                    let error_msg = e.to_string();
                    let is_recoverable = error_msg.contains("nonce too low") || 
                                        error_msg.contains("gas price too low") ||
                                        error_msg.contains("replacement transaction underpriced");

                    if is_recoverable {
                        debug!("Encountered recoverable error, will retry");
                    } else if retry == max_retries - 1 {
                        return Err(anyhow::anyhow!("Failed to add refs: {}", e));
                    }
                }
            }
        }

        Err(anyhow::anyhow!("Failed to add refs after {} retries", max_retries))
    }

    #[instrument(skip(self), err)]
    pub async fn get_objects(&self) -> Result<Vec<Object>> {
        info!("Retrieving all objects");

        match self.contract.get_objects().call().await {
            Ok(objects) => {
                info!("Retrieved {} objects", objects.len());

                let mut result = Vec::new();
                for object in objects {
                    result.push(Object {
                        hash: object.0,
                        ipfs_url: object.1.0,
                        pusher: object.2,
                    });
                }

                debug!("Object count: {}", result.len());
                trace!("Object hashes: {:?}", result.iter().map(|o| &o.hash).collect::<Vec<_>>());
                Ok(result)
            },
            Err(e) => {
                error!("Failed to retrieve objects: {}", e);
                Err(anyhow::Error::from(e))
            }
        }
    }

    #[instrument(skip(self), err)]
    pub async fn get_refs(&self) -> Result<Vec<Ref>> {
        info!("Retrieving all refs");

        match self.contract.get_refs().call().await {
            Ok(objects) => {
                info!("Retrieved {} refs", objects.len());

                let mut result = Vec::new();
                for object in objects {
                    result.push(Ref {
                        name: object.0,
                        data: object.1.0,
                        is_active: object.2,
                        pusher: object.3,
                    });
                }

                debug!("Ref count: {}", result.len());
                trace!("Ref names: {:?}", result.iter().map(|r| &r.name).collect::<Vec<_>>());
                Ok(result)
            },
            Err(e) => {
                error!("Failed to retrieve refs: {}", e);
                Err(anyhow::Error::from(e))
            }
        }
    }

    #[instrument(skip(self), err)]
    pub async fn get_objects_length(&self) -> Result<U256> {
        debug!("Retrieving object count");

        match self.contract
            .get_objects_length()
            .call()
            .await {
                Ok(length) => {
                    info!("Total objects in contract: {}", length);
                    Ok(length)
                },
                Err(e) => {
                    error!("Failed to get objects length: {}", e);
                    Err(anyhow::Error::from(e))
                }
            }
    }

    #[instrument(skip(self), err)]
    pub async fn get_refs_length(&self) -> Result<U256> {
        debug!("Retrieving ref count");

        match self.contract
            .get_refs_length()
            .call()
            .await {
                Ok(length) => {
                    info!("Total refs in contract: {}", length);
                    Ok(length)
                },
                Err(e) => {
                    error!("Failed to get refs length: {}", e);
                    Err(anyhow::Error::from(e))
                }
            }
    }

    #[instrument(skip(self), err)]
    pub async fn get_ref_by_id(&self, id: U256) -> Result<Ref> {
        info!("Retrieving ref by ID: {}", id);

        match self.contract
            .get_ref_by_id(id)
            .call()
            .await {
                Ok((name, data, is_active, pusher)) => {
                    info!("Retrieved ref {} with name: {}", id, name);
                    debug!("Ref details - data length: {} bytes, active: {}, pusher: {:?}", 
                           data.0.len(), is_active, pusher);

                    Ok(Ref {
                        name,
                        data: data.0,
                        is_active,
                        pusher,
                    })
                },
                Err(e) => {
                    error!("Failed to retrieve ref by ID {}: {}", id, e);
                    Err(anyhow::Error::from(e))
                }
            }
    }
}

use tracing::{debug, warn};

pub struct Config;

impl Config {
    pub fn pk() -> String {
        match dotenv::var("PK") {
            Ok(key) => {
                debug!("Loaded private key, length: {}", key.len());
                key
            },
            Err(_) => {
                warn!("PK environment variable not found, using empty string");
                "".to_string()
            }
        }
    }

    pub fn rpc_url() -> String {
        match dotenv::var("RPC_URL") {
            Ok(url) => {
                debug!("Loaded RPC URL: {}", url);
                url
            },
            Err(_) => {
                let default = "http://localhost:8545".to_string();
                warn!("RPC_URL environment variable not found, using default: {}", default);
                default
            }
        }
    }

    pub fn ipfs_prefix() -> String {
        match dotenv::var("IPFS_PREFIX") {
            Ok(prefix) => {
                debug!("Loaded IPFS prefix: {}", prefix);
                prefix
            },
            Err(_) => {
                warn!("IPFS_PREFIX environment variable not found, using empty string");
                "".to_string()
            }
        }
    }

    pub fn ipfs_api_url() -> Option<String> {
        std::env::var("IPFS_API_URL").ok()
    }
}

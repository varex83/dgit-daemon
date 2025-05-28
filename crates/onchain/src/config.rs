use tracing::{debug, warn};

pub struct Config;

impl Config {
    pub fn get_pinata_secret_api_key() -> String {
        match dotenv::var("PINATA_SECRET_API_KEY") {
            Ok(key) => {
                debug!("Loaded Pinata secret API key, length: {}", key.len());
                key
            },
            Err(_) => {
                warn!("PINATA_SECRET_API_KEY environment variable not found, using empty string");
                "".to_string()
            }
        }
    }

    pub fn get_pinata_api_key() -> String {
        match dotenv::var("PINATA_API_KEY") {
            Ok(key) => {
                debug!("Loaded Pinata API key, length: {}", key.len());
                key
            },
            Err(_) => {
                warn!("PINATA_API_KEY environment variable not found, using empty string");
                "".to_string()
            }
        }
    }

    pub fn get_pinata_jwt() -> String {
        match dotenv::var("PINATA_JWT") {
            Ok(jwt) => {
                debug!("Loaded Pinata JWT, length: {}", jwt.len());
                jwt
            },
            Err(_) => {
                warn!("PINATA_JWT environment variable not found, using empty string");
                "".to_string()
            }
        }
    }

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

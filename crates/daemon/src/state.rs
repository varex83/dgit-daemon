use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use onchain::contract_interaction::ContractInteraction;

#[derive(Debug, Clone)]
pub struct ContractState {
    inner: Arc<Mutex<ContractStateInner>>,
}

#[derive(Debug)]
pub struct ContractStateInner {
    contracts: HashMap<String, ContractInteraction>,
}

impl Default for ContractState {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ContractStateInner {
                contracts: HashMap::new(),
            })),
        }
    }
}

impl ContractState {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get_contract(&self, repo: &str) -> Option<ContractInteraction> {
        let inner = self.inner.lock().await;
        inner.contracts.get(repo).cloned()
    }

    pub async fn insert_contract(&self, repo: String, contract: ContractInteraction) {
        let mut inner = self.inner.lock().await;
        inner.contracts.insert(repo, contract);
    }
}

impl Clone for ContractStateInner {
    fn clone(&self) -> Self {
        Self {
            contracts: self.contracts.clone(),
        }
    }
}
// File: icn_api/src/lib.rs

use icn_core::IcnNode;
use icn_common::{Transaction, Proposal, IcnResult};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ApiLayer {
    node: Arc<RwLock<IcnNode>>,
}

impl ApiLayer {
    pub fn new(node: Arc<RwLock<IcnNode>>) -> Self {
        ApiLayer { node }
    }

    pub async fn submit_transaction(&self, tx: Transaction) -> IcnResult<()> {
        let node = self.node.read().await;
        node.process_transaction(tx).await
    }

    pub async fn create_proposal(&self, proposal: Proposal) -> IcnResult<String> {
        let node = self.node.read().await;
        node.create_proposal(proposal)
    }

    pub async fn create_identity(&self, attributes: std::collections::HashMap<String, String>) -> IcnResult<DecentralizedIdentity> {
        let node = self.node.read().await;
        node.create_identity(attributes)
    }
}

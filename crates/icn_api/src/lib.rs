// File: crates/icn_api/src/lib.rs

use icn_core::IcnNode;
use icn_common::{Transaction, Proposal, IcnResult};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ApiLayer {
    node: Arc<Mutex<IcnNode>>,
}

impl ApiLayer {
    pub fn new(node: Arc<Mutex<IcnNode>>) -> Self {
        ApiLayer { node }
    }

    pub async fn submit_transaction(&self, tx: Transaction) -> IcnResult<()> {
        let node = self.node.lock().await;
        node.process_transaction(tx).await
    }

    pub async fn create_proposal(&self, proposal: Proposal) -> IcnResult<String> {
        let node = self.node.lock().await;
        node.create_proposal(proposal)
    }

    pub async fn create_identity(&self, attributes: std::collections::HashMap<String, String>) -> IcnResult<DecentralizedIdentity> {
        let node = self.node.lock().await;
        node.create_identity(attributes)
    }
}

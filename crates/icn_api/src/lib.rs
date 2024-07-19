use crate::blockchain::{Blockchain, Transaction};
use crate::governance::DemocraticSystem;
use crate::governance::democracy::ProposalStatus as DemocracyProposalStatus;
// Remove this line
// use crate::error::Error;

use serde::{Deserialize, Serialize, Serializer, Deserializer};
use tokio::sync::RwLock;
use std::sync::Arc;
use chrono::{DateTime, Utc, Duration};

// The rest of the file remains the same...

#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

pub struct ApiLayer {
    blockchain: Arc<RwLock<Blockchain>>,
    governance: Arc<RwLock<DemocraticSystem>>,
}

impl ApiLayer {
    pub fn new(
        blockchain: Arc<RwLock<Blockchain>>,
        governance: Arc<RwLock<DemocraticSystem>>,
    ) -> Self {
        Self {
            blockchain,
            governance,
        }
    }

    pub async fn get_blockchain_info(&self) -> ApiResponse<BlockchainInfo> {
        let blockchain = self.blockchain.read().await;
        let info = BlockchainInfo {
            block_count: blockchain.chain.len(),
            last_block_hash: blockchain.chain.last().map(|b| b.hash.clone()),
        };
        ApiResponse {
            success: true,
            data: Some(info),
            error: None,
        }
    }

    pub async fn submit_transaction(&self, transaction: Transaction) -> ApiResponse<String> {
        let mut blockchain = self.blockchain.write().await;
        match blockchain.add_transaction(transaction) {
            Ok(()) => ApiResponse {
                success: true,
                data: Some("Transaction submitted successfully".to_string()),
                error: None,
            },
            Err(e) => ApiResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            },
        }
    }

    pub async fn get_balance(&self, address: &str) -> ApiResponse<f64> {
        let blockchain = self.blockchain.read().await;
        let balance = blockchain.get_balance(address);
        ApiResponse {
            success: true,
            data: Some(balance),
            error: None,
        }
    }

    pub async fn create_proposal(&self, proposal: Proposal) -> ApiResponse<String> {
        let mut governance = self.governance.write().await;
        match governance.create_proposal(
            proposal.title,
            proposal.description,
            proposal.proposer,
            proposal.voting_period,
            proposal.proposal_type,
            proposal.category,
            proposal.required_quorum,
            proposal.execution_timestamp,
        ) {
            Ok(id) => ApiResponse {
                success: true,
                data: Some(id),
                error: None,
            },
            Err(e) => ApiResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            },
        }
    }

    pub async fn vote_on_proposal(&self, vote: Vote) -> ApiResponse<String> {
        let mut governance = self.governance.write().await;
        match governance.vote(vote.voter, vote.proposal_id, vote.in_favor, vote.weight) {
            Ok(()) => ApiResponse {
                success: true,
                data: Some("Vote recorded successfully".to_string()),
                error: None,
            },
            Err(e) => ApiResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            },
        }
    }

    pub async fn get_proposal_status(&self, proposal_id: &str) -> ApiResponse<ProposalStatus> {
        let governance = self.governance.read().await;
        match governance.get_proposal(proposal_id) {
            Some(proposal) => ApiResponse {
                success: true,
                data: Some(ProposalStatus::from(proposal.status.clone())),
                error: None,
            },
            None => ApiResponse {
                success: false,
                data: None,
                error: Some("Proposal not found".to_string()),
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct BlockchainInfo {
    pub block_count: usize,
    pub last_block_hash: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Proposal {
    pub title: String,
    pub description: String,
    pub proposer: String,
    #[serde(serialize_with = "serialize_duration", deserialize_with = "deserialize_duration")]
    pub voting_period: Duration,
    pub proposal_type: crate::governance::democracy::ProposalType,
    pub category: crate::governance::democracy::ProposalCategory,
    pub required_quorum: f64,
    pub execution_timestamp: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize)]
pub struct Vote {
    pub voter: String,
    pub proposal_id: String,
    pub in_favor: bool,
    pub weight: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Implemented,
}

impl From<DemocracyProposalStatus> for ProposalStatus {
    fn from(status: DemocracyProposalStatus) -> Self {
        match status {
            DemocracyProposalStatus::Active => ProposalStatus::Active,
            DemocracyProposalStatus::Passed => ProposalStatus::Passed,
            DemocracyProposalStatus::Rejected => ProposalStatus::Rejected,
            DemocracyProposalStatus::Implemented => ProposalStatus::Implemented,
        }
    }
}

fn serialize_duration<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_i64(duration.num_seconds())
}

fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let seconds = i64::deserialize(deserializer)?;
    Ok(Duration::seconds(seconds))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::currency::CurrencyType;

    // Helper function to create a mock ApiLayer for testing
    async fn create_mock_api_layer() -> ApiLayer {
        let blockchain = Arc::new(RwLock::new(Blockchain::new()));
        let governance = Arc::new(RwLock::new(DemocraticSystem::new()));

        ApiLayer::new(blockchain, governance)
    }

    #[tokio::test]
    async fn test_get_blockchain_info() {
        let api = create_mock_api_layer().await;
        let info = api.get_blockchain_info().await;
        assert!(info.success);
        assert_eq!(info.data.unwrap().block_count, 1); // Genesis block
    }

    #[tokio::test]
    async fn test_submit_transaction() {
        let api = create_mock_api_layer().await;
        let transaction = Transaction::new("Alice".to_string(), "Bob".to_string(), 100.0, CurrencyType::BasicNeeds, 1000);
        let result = api.submit_transaction(transaction).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_get_balance() {
        let api = create_mock_api_layer().await;
        let balance = api.get_balance("Alice").await;
        assert!(balance.success);
        assert_eq!(balance.data.unwrap(), 0.0); // Initial balance
    }
}

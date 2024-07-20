use icn_types::{Block, Transaction, Proposal, ProposalStatus, CurrencyType};
use icn_common::{CommonError, CommonResult};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

pub struct ApiLayer {
    blockchain: Arc<RwLock<BlockchainInterface>>,
    consensus: Arc<RwLock<ConsensusInterface>>,
    currency_system: Arc<RwLock<CurrencySystemInterface>>,
    governance: Arc<RwLock<GovernanceInterface>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockchainInfo {
    pub block_count: usize,
    pub last_block_hash: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Vote {
    pub voter: String,
    pub proposal_id: String,
    pub in_favor: bool,
    pub weight: f64,
}

impl ApiLayer {
    pub fn new(
        blockchain: Arc<RwLock<BlockchainInterface>>,
        consensus: Arc<RwLock<ConsensusInterface>>,
        currency_system: Arc<RwLock<CurrencySystemInterface>>,
        governance: Arc<RwLock<GovernanceInterface>>,
    ) -> Self {
        ApiLayer {
            blockchain,
            consensus,
            currency_system,
            governance,
        }
    }

    pub async fn get_blockchain_info(&self) -> CommonResult<ApiResponse<BlockchainInfo>> {
        let blockchain = self.blockchain.read().await;
        let info = blockchain.get_info().await?;
        Ok(ApiResponse {
            success: true,
            data: Some(info),
            error: None,
        })
    }

    pub async fn submit_transaction(&self, transaction: Transaction) -> CommonResult<ApiResponse<String>> {
        let mut blockchain = self.blockchain.write().await;
        blockchain.add_transaction(transaction).await?;
        Ok(ApiResponse {
            success: true,
            data: Some("Transaction submitted successfully".to_string()),
            error: None,
        })
    }

    pub async fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> CommonResult<ApiResponse<f64>> {
        let currency_system = self.currency_system.read().await;
        let balance = currency_system.get_balance(address, currency_type).await?;
        Ok(ApiResponse {
            success: true,
            data: Some(balance),
            error: None,
        })
    }

    pub async fn create_proposal(&self, proposal: Proposal) -> CommonResult<ApiResponse<String>> {
        let mut governance = self.governance.write().await;
        let proposal_id = governance.create_proposal(proposal).await?;
        Ok(ApiResponse {
            success: true,
            data: Some(proposal_id),
            error: None,
        })
    }

    pub async fn vote_on_proposal(&self, vote: Vote) -> CommonResult<ApiResponse<String>> {
        let mut governance = self.governance.write().await;
        governance.vote_on_proposal(vote).await?;
        Ok(ApiResponse {
            success: true,
            data: Some("Vote recorded successfully".to_string()),
            error: None,
        })
    }

    pub async fn get_proposal_status(&self, proposal_id: &str) -> CommonResult<ApiResponse<ProposalStatus>> {
        let governance = self.governance.read().await;
        let status = governance.get_proposal_status(proposal_id).await?;
        Ok(ApiResponse {
            success: true,
            data: Some(status),
            error: None,
        })
    }
}

#[async_trait::async_trait]
pub trait BlockchainInterface {
    async fn get_info(&self) -> CommonResult<BlockchainInfo>;
    async fn add_transaction(&mut self, transaction: Transaction) -> CommonResult<()>;
}

#[async_trait::async_trait]
pub trait ConsensusInterface {
    // Add consensus-related methods here
}

#[async_trait::async_trait]
pub trait CurrencySystemInterface {
    async fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> CommonResult<f64>;
}

#[async_trait::async_trait]
pub trait GovernanceInterface {
    async fn create_proposal(&mut self, proposal: Proposal) -> CommonResult<String>;
    async fn vote_on_proposal(&mut self, vote: Vote) -> CommonResult<()>;
    async fn get_proposal_status(&self, proposal_id: &str) -> CommonResult<ProposalStatus>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    // Implement mock structures for testing
    struct MockBlockchain;
    struct MockConsensus;
    struct MockCurrencySystem;
    struct MockGovernance;

    #[async_trait::async_trait]
    impl BlockchainInterface for MockBlockchain {
        async fn get_info(&self) -> CommonResult<BlockchainInfo> {
            Ok(BlockchainInfo {
                block_count: 1,
                last_block_hash: Some("0000000000000000000000000000000000000000000000000000000000000000".to_string()),
            })
        }

        async fn add_transaction(&mut self, _transaction: Transaction) -> CommonResult<()> {
            Ok(())
        }
    }

    #[async_trait::async_trait]
    impl ConsensusInterface for MockConsensus {}

    #[async_trait::async_trait]
    impl CurrencySystemInterface for MockCurrencySystem {
        async fn get_balance(&self, _address: &str, _currency_type: &CurrencyType) -> CommonResult<f64> {
            Ok(100.0)
        }
    }

    #[async_trait::async_trait]
    impl GovernanceInterface for MockGovernance {
        async fn create_proposal(&mut self, _proposal: Proposal) -> CommonResult<String> {
            Ok("new_proposal_id".to_string())
        }

        async fn vote_on_proposal(&mut self, _vote: Vote) -> CommonResult<()> {
            Ok(())
        }

        async fn get_proposal_status(&self, _proposal_id: &str) -> CommonResult<ProposalStatus> {
            Ok(ProposalStatus::Active)
        }
    }

    #[tokio::test]
    async fn test_api_layer() {
        let api = ApiLayer::new(
            Arc::new(RwLock::new(MockBlockchain)),
            Arc::new(RwLock::new(MockConsensus)),
            Arc::new(RwLock::new(MockCurrencySystem)),
            Arc::new(RwLock::new(MockGovernance)),
        );

        // Test get_blockchain_info
        let blockchain_info = api.get_blockchain_info().await.unwrap();
        assert!(blockchain_info.success);
        assert_eq!(blockchain_info.data.unwrap().block_count, 1);

        // Test submit_transaction
        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };
        let submit_result = api.submit_transaction(transaction).await.unwrap();
        assert!(submit_result.success);

        // Test get_balance
        let balance_result = api.get_balance("Alice", &CurrencyType::BasicNeeds).await.unwrap();
        assert!(balance_result.success);
        assert_eq!(balance_result.data.unwrap(), 100.0);

        // Test create_proposal
        let proposal = Proposal {
            id: "".to_string(),
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            proposer: "Alice".to_string(),
            created_at: Utc::now(),
            voting_ends_at: Utc::now() + chrono::Duration::days(7),
            status: ProposalStatus::Active,
            proposal_type: ProposalType::Constitutional,
            category: ProposalCategory::Economic,
            required_quorum: 0.66,
            execution_timestamp: None,
        };
        let create_proposal_result = api.create_proposal(proposal).await.unwrap();
        assert!(create_proposal_result.success);

        // Test vote_on_proposal
        let vote = Vote {
            voter: "Bob".to_string(),
            proposal_id: "new_proposal_id".to_string(),
            in_favor: true,
            weight: 1.0,
        };
        let vote_result = api.vote_on_proposal(vote).await.unwrap();
        assert!(vote_result.success);

        // Test get_proposal_status
        let status_result = api.get_proposal_status("new_proposal_id").await.unwrap();
        assert!(status_result.success);
        assert_eq!(status_result.data.unwrap(), ProposalStatus::Active);
    }
}
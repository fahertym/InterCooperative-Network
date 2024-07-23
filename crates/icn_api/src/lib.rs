use icn_common::{Block, Transaction, Proposal, ProposalStatus, CurrencyType};
use icn_common::{CommonError, CommonResult};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, error};

/// A generic API response structure.
#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Indicates whether the API call was successful.
    pub success: bool,
    /// Contains the data returned by the API call, if any.
    pub data: Option<T>,
    /// Contains the error message, if any.
    pub error: Option<String>,
}

/// API Layer struct to manage different modules.
pub struct ApiLayer {
    blockchain: Arc<RwLock<dyn BlockchainInterface>>,
    consensus: Arc<RwLock<dyn ConsensusInterface>>,
    currency_system: Arc<RwLock<dyn CurrencySystemInterface>>,
    governance: Arc<RwLock<dyn GovernanceInterface>>,
}

/// Struct containing information about the blockchain.
#[derive(Clone, Serialize, Deserialize)]
pub struct BlockchainInfo {
    pub block_count: usize,
    pub last_block_hash: Option<String>,
}

/// Struct representing a vote in the governance system.
#[derive(Serialize, Deserialize)]
pub struct Vote {
    pub voter: String,
    pub proposal_id: String,
    pub in_favor: bool,
    pub weight: f64,
}

impl ApiLayer {
    /// Creates a new instance of ApiLayer.
    ///
    /// # Arguments
    ///
    /// * `blockchain` - A reference to the blockchain module.
    /// * `consensus` - A reference to the consensus module.
    /// * `currency_system` - A reference to the currency system module.
    /// * `governance` - A reference to the governance module.
    pub fn new(
        blockchain: Arc<RwLock<dyn BlockchainInterface>>,
        consensus: Arc<RwLock<dyn ConsensusInterface>>,
        currency_system: Arc<RwLock<dyn CurrencySystemInterface>>,
        governance: Arc<RwLock<dyn GovernanceInterface>>,
    ) -> Self {
        ApiLayer {
            blockchain,
            consensus,
            currency_system,
            governance,
        }
    }

    /// Fetches information about the blockchain.
    ///
    /// # Returns
    ///
    /// * `CommonResult<ApiResponse<BlockchainInfo>>` - The result of the API call containing blockchain information.
    pub async fn get_blockchain_info(&self) -> CommonResult<ApiResponse<BlockchainInfo>> {
        info!("Fetching blockchain info");
        let blockchain = self.blockchain.read().await;
        match blockchain.get_info().await {
            Ok(info) => Ok(ApiResponse {
                success: true,
                data: Some(info),
                error: None,
            }),
            Err(e) => {
                error!("Failed to fetch blockchain info: {:?}", e);
                Ok(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                })
            }
        }
    }

    /// Submits a transaction to the blockchain.
    ///
    /// # Arguments
    ///
    /// * `transaction` - The transaction to be submitted.
    ///
    /// # Returns
    ///
    /// * `CommonResult<ApiResponse<String>>` - The result of the API call.
    pub async fn submit_transaction(&self, transaction: Transaction) -> CommonResult<ApiResponse<String>> {
        info!("Submitting transaction: {:?}", transaction);
        let mut blockchain = self.blockchain.write().await;
        match blockchain.add_transaction(transaction).await {
            Ok(_) => Ok(ApiResponse {
                success: true,
                data: Some("Transaction submitted successfully".to_string()),
                error: None,
            }),
            Err(e) => {
                error!("Failed to submit transaction: {:?}", e);
                Ok(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                })
            }
        }
    }

    /// Fetches the balance for a given address and currency type.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to fetch the balance for.
    /// * `currency_type` - The type of currency to fetch the balance of.
    ///
    /// # Returns
    ///
    /// * `CommonResult<ApiResponse<f64>>` - The result of the API call.
    pub async fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> CommonResult<ApiResponse<f64>> {
        info!("Fetching balance for address: {}, currency type: {:?}", address, currency_type);
        let currency_system = self.currency_system.read().await;
        match currency_system.get_balance(address, currency_type).await {
            Ok(balance) => Ok(ApiResponse {
                success: true,
                data: Some(balance),
                error: None,
            }),
            Err(e) => {
                error!("Failed to fetch balance: {:?}", e);
                Ok(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                })
            }
        }
    }

    /// Creates a new proposal in the governance system.
    ///
    /// # Arguments
    ///
    /// * `proposal` - The proposal to be created.
    ///
    /// # Returns
    ///
    /// * `CommonResult<ApiResponse<String>>` - The result of the API call.
    pub async fn create_proposal(&self, proposal: Proposal) -> CommonResult<ApiResponse<String>> {
        info!("Creating proposal: {:?}", proposal);
        let mut governance = self.governance.write().await;
        match governance.create_proposal(proposal).await {
            Ok(proposal_id) => Ok(ApiResponse {
                success: true,
                data: Some(proposal_id),
                error: None,
            }),
            Err(e) => {
                error!("Failed to create proposal: {:?}", e);
                Ok(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                })
            }
        }
    }

    /// Votes on an existing proposal.
    ///
    /// # Arguments
    ///
    /// * `vote` - The vote to be cast.
    ///
    /// # Returns
    ///
    /// * `CommonResult<ApiResponse<String>>` - The result of the API call.
    pub async fn vote_on_proposal(&self, vote: Vote) -> CommonResult<ApiResponse<String>> {
        info!("Voting on proposal: {:?}", vote);
        let mut governance = self.governance.write().await;
        match governance.vote_on_proposal(vote).await {
            Ok(_) => Ok(ApiResponse {
                success: true,
                data: Some("Vote recorded successfully".to_string()),
                error: None,
            }),
            Err(e) => {
                error!("Failed to vote on proposal: {:?}", e);
                Ok(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                })
            }
        }
    }

    /// Fetches the status of a given proposal.
    ///
    /// # Arguments
    ///
    /// * `proposal_id` - The ID of the proposal to fetch the status of.
    ///
    /// # Returns
    ///
    /// * `CommonResult<ApiResponse<ProposalStatus>>` - The result of the API call.
    pub async fn get_proposal_status(&self, proposal_id: &str) -> CommonResult<ApiResponse<ProposalStatus>> {
        info!("Fetching proposal status for ID: {}", proposal_id);
        let governance = self.governance.read().await;
        match governance.get_proposal_status(proposal_id).await {
            Ok(status) => Ok(ApiResponse {
                success: true,
                data: Some(status),
                error: None,
            }),
            Err(e) => {
                error!("Failed to fetch proposal status: {:?}", e);
                Ok(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                })
            }
        }
    }
}

#[async_trait::async_trait]
pub trait BlockchainInterface {
    async fn get_info(&self) -> CommonResult<BlockchainInfo>;
    async fn add_transaction(&mut self, transaction: Transaction) -> CommonResult<()>;
}

#[async_trait::async_trait]
pub trait ConsensusInterface {
    async fn validate_block(&self, block: &Block) -> CommonResult<()>;
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
    impl ConsensusInterface for MockConsensus {
        async fn validate_block(&self, _block: &Block) -> CommonResult<()> {
            Ok(())
        }
    }

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

        let blockchain_info = api.get_blockchain_info().await.unwrap();
        assert!(blockchain_info.success);
        assert_eq!(blockchain_info.data.unwrap().block_count, 1);

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

        let balance_result = api.get_balance("Alice", &CurrencyType::BasicNeeds).await.unwrap();
        assert!(balance_result.success);
        assert_eq!(balance_result.data.unwrap(), 100.0);

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

        let vote = Vote {
            voter: "Bob".to_string(),
            proposal_id: "new_proposal_id".to_string(),
            in_favor: true,
            weight: 1.0,
        };
        let vote_result = api.vote_on_proposal(vote).await.unwrap();
        assert!(vote_result.success);

        let status_result = api.get_proposal_status("new_proposal_id").await.unwrap();
        assert!(status_result.success);
        assert_eq!(status_result.data.unwrap(), ProposalStatus::Active);
    }
}

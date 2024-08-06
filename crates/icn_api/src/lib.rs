// File: icn_api/src/lib.rs

use std::sync::Arc;
use tokio::sync::RwLock;
use warp::{Filter, Rejection, Reply};
use serde::{Deserialize, Serialize};
use icn_common::{IcnResult, IcnError, Transaction, Proposal, CurrencyType, ProposalType, ProposalCategory, ProposalStatus};
use serde_json::json;
use chrono::{Duration, Utc};

// ApiLayer struct remains unchanged
pub struct ApiLayer {
    node: Arc<RwLock<icn_core::IcnNode>>,
}

impl ApiLayer {
    pub fn new(node: Arc<RwLock<icn_core::IcnNode>>) -> Self {
        ApiLayer { node }
    }

    // Existing methods remain unchanged
    pub async fn submit_transaction(&self, transaction: Transaction) -> IcnResult<()> {
        let node = self.node.read().await;
        node.process_transaction(transaction).await
    }

    pub async fn create_proposal(&self, proposal: Proposal) -> IcnResult<String> {
        let node = self.node.read().await;
        node.create_proposal(proposal).await
    }

    pub async fn vote_on_proposal(&self, proposal_id: &str, voter: String, in_favor: bool, weight: f64) -> IcnResult<()> {
        let node = self.node.read().await;
        node.vote_on_proposal(proposal_id, voter, in_favor, weight).await
    }

    pub async fn finalize_proposal(&self, proposal_id: &str) -> IcnResult<ProposalStatus> {
        let node = self.node.read().await;
        node.finalize_proposal(proposal_id).await
    }

    pub async fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        let node = self.node.read().await;
        node.get_balance(address, currency_type).await
    }

    pub async fn mint_currency(&self, address: &str, currency_type: &CurrencyType, amount: f64) -> IcnResult<()> {
        let node = self.node.write().await;
        node.mint_currency(address, currency_type, amount).await
    }

    pub async fn create_identity(&self, attributes: std::collections::HashMap<String, String>) -> IcnResult<String> {
        let node = self.node.write().await;
        node.create_identity(attributes).await
    }

    pub async fn allocate_resource(&self, resource_type: &str, amount: u64) -> IcnResult<()> {
        let node = self.node.write().await;
        node.allocate_resource(resource_type, amount).await
    }

    pub async fn get_network_stats(&self) -> IcnResult<icn_common::NetworkStats> {
        let node = self.node.read().await;
        node.get_network_stats().await
    }

    // New method to get proposal status
    pub async fn get_proposal_status(&self, proposal_id: &str) -> IcnResult<ProposalStatus> {
        let node = self.node.read().await;
        node.get_proposal_status(proposal_id).await
    }
}

// Request and response structs
#[derive(Deserialize)]
struct CreateProposalRequest {
    title: String,
    description: String,
    proposer: String,
    proposal_type: ProposalType,
    category: ProposalCategory,
}

#[derive(Serialize)]
struct CreateProposalResponse {
    proposal_id: String,
}

#[derive(Deserialize)]
struct GetProposalStatusRequest {
    proposal_id: String,
}

#[derive(Serialize)]
struct GetProposalStatusResponse {
    status: ProposalStatus,
}

// Helper function to convert IcnError to warp::Rejection
fn icn_error_to_rejection(error: IcnError) -> warp::Rejection {
    warp::reject::custom(error)
}

// API routes
pub fn api_routes(
    api_layer: Arc<RwLock<ApiLayer>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let api_layer = warp::any().map(move || api_layer.clone());

    let submit_transaction = warp::post()
        .and(warp::path("transaction"))
        .and(warp::body::json())
        .and(api_layer.clone())
        .and_then(handle_submit_transaction);

    let create_proposal = warp::post()
        .and(warp::path("proposal"))
        .and(warp::body::json())
        .and(api_layer.clone())
        .and_then(handle_create_proposal);

    let vote_on_proposal = warp::post()
        .and(warp::path("vote"))
        .and(warp::body::json())
        .and(api_layer.clone())
        .and_then(handle_vote_on_proposal);

    let get_balance = warp::get()
        .and(warp::path("balance"))
        .and(warp::query())
        .and(api_layer.clone())
        .and_then(handle_get_balance);

    let mint_currency = warp::post()
        .and(warp::path("mint"))
        .and(warp::body::json())
        .and(api_layer.clone())
        .and_then(handle_mint_currency);

    let create_identity = warp::post()
        .and(warp::path("identity"))
        .and(warp::body::json())
        .and(api_layer.clone())
        .and_then(handle_create_identity);

    let allocate_resource = warp::post()
        .and(warp::path("allocate"))
        .and(warp::body::json())
        .and(api_layer.clone())
        .and_then(handle_allocate_resource);

    let get_network_stats = warp::get()
        .and(warp::path("stats"))
        .and(api_layer.clone())
        .and_then(handle_get_network_stats);

    let get_proposal_status = warp::get()
        .and(warp::path("proposal"))
        .and(warp::path("status"))
        .and(warp::query())
        .and(api_layer.clone())
        .and_then(handle_get_proposal_status);

    submit_transaction
        .or(create_proposal)
        .or(vote_on_proposal)
        .or(get_balance)
        .or(mint_currency)
        .or(create_identity)
        .or(allocate_resource)
        .or(get_network_stats)
        .or(get_proposal_status)
}

// Handler functions
async fn handle_submit_transaction(
    transaction: Transaction,
    api_layer: Arc<RwLock<ApiLayer>>,
) -> Result<impl Reply, Rejection> {
    let api_layer = api_layer.read().await;
    api_layer
        .submit_transaction(transaction)
        .await
        .map(|_| warp::reply::json(&json!({"status": "success"})))
        .map_err(icn_error_to_rejection)
}

async fn handle_create_proposal(
    proposal_request: CreateProposalRequest,
    api_layer: Arc<RwLock<ApiLayer>>,
) -> Result<impl Reply, Rejection> {
    let api_layer = api_layer.read().await;
    let proposal = Proposal {
        id: Uuid::new_v4().to_string(),
        title: proposal_request.title,
        description: proposal_request.description,
        proposer: proposal_request.proposer,
        created_at: Utc::now(),
        voting_ends_at: Utc::now() + Duration::days(7), // Set voting period to 7 days
        status: ProposalStatus::Active,
        proposal_type: proposal_request.proposal_type,
        category: proposal_request.category,
        required_quorum: 0.51, // Set a default quorum, can be made configurable
        execution_timestamp: None,
    };
    api_layer
        .create_proposal(proposal)
        .await
        .map(|proposal_id| warp::reply::json(&CreateProposalResponse { proposal_id }))
        .map_err(icn_error_to_rejection)
}

async fn handle_vote_on_proposal(
    vote: Vote,
    api_layer: Arc<RwLock<ApiLayer>>,
) -> Result<impl Reply, Rejection> {
    let api_layer = api_layer.read().await;
    api_layer
        .vote_on_proposal(&vote.proposal_id, vote.voter, vote.in_favor, vote.weight)
        .await
        .map(|_| warp::reply::json(&json!({"status": "success"})))
        .map_err(icn_error_to_rejection)
}

async fn handle_get_balance(
    query: GetBalanceQuery,
    api_layer: Arc<RwLock<ApiLayer>>,
) -> Result<impl Reply, Rejection> {
    let api_layer = api_layer.read().await;
    api_layer
        .get_balance(&query.address, &query.currency_type)
        .await
        .map(|balance| warp::reply::json(&json!({"balance": balance})))
        .map_err(icn_error_to_rejection)
}

async fn handle_mint_currency(
    request: MintCurrencyRequest,
    api_layer: Arc<RwLock<ApiLayer>>,
) -> Result<impl Reply, Rejection> {
    let api_layer = api_layer.read().await;
    api_layer
        .mint_currency(&request.address, &request.currency_type, request.amount)
        .await
        .map(|_| warp::reply::json(&json!({"status": "success"})))
        .map_err(icn_error_to_rejection)
}

async fn handle_create_identity(
    attributes: HashMap<String, String>,
    api_layer: Arc<RwLock<ApiLayer>>,
) -> Result<impl Reply, Rejection> {
    let api_layer = api_layer.read().await;
    api_layer
        .create_identity(attributes)
        .await
        .map(|id| warp::reply::json(&json!({"identity_id": id})))
        .map_err(icn_error_to_rejection)
}

async fn handle_allocate_resource(
    request: AllocateResourceRequest,
    api_layer: Arc<RwLock<ApiLayer>>,
) -> Result<impl Reply, Rejection> {
    let api_layer = api_layer.read().await;
    api_layer
        .allocate_resource(&request.resource_type, request.amount)
        .await
        .map(|_| warp::reply::json(&json!({"status": "success"})))
        .map_err(icn_error_to_rejection)
}

async fn handle_get_network_stats(
    api_layer: Arc<RwLock<ApiLayer>>,
) -> Result<impl Reply, Rejection> {
    let api_layer = api_layer.read().await;
    api_layer
        .get_network_stats()
        .await
        .map(|stats| warp::reply::json(&stats))
        .map_err(icn_error_to_rejection)
}

async fn handle_get_proposal_status(
    query: GetProposalStatusRequest,
    api_layer: Arc<RwLock<ApiLayer>>,
) -> Result<impl Reply, Rejection> {
    let api_layer = api_layer.read().await;
    api_layer
        .get_proposal_status(&query.proposal_id)
        .await
        .map(|status| warp::reply::json(&GetProposalStatusResponse { status }))
        .map_err(icn_error_to_rejection)
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_core::Config;
    use std::net::SocketAddr;

    async fn setup_test_env() -> (Arc<RwLock<ApiLayer>>, Arc<RwLock<icn_core::IcnNode>>) {
        let config = Config {
            shard_count: 1,
            consensus_threshold: 0.66,
            consensus_quorum: 0.51,
            network_port: 8080,
        };
        let node = Arc::new(RwLock::new(icn_core::IcnNode::new(config).await.unwrap()));
        let api_layer = Arc::new(RwLock::new(ApiLayer::new(Arc::clone(&node))));
        (api_layer, node)
    }

    #[tokio::test]
    async fn test_submit_transaction() {
        let (api_layer, _) = setup_test_env().await;
        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 100.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: chrono::Utc::now().timestamp(),
            signature: None,
        };

        let result = handle_submit_transaction(transaction, api_layer).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_proposal() {
        let (api_layer, _) = setup_test_env().await;
        let proposal_request = CreateProposalRequest {
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            proposer: "Alice".to_string(),
            proposal_type: ProposalType::Constitutional,
            category: ProposalCategory::Economic,
        };

        let result = handle_create_proposal(proposal_request, api_layer).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_proposal_status() {
        let (api_layer, node) = setup_test_env().await;
        
        // First, create a proposal
        let proposal = Proposal {
            id: "test_proposal".to_string(),
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            proposer: "Alice".to_string(),
            created_at: chrono::Utc::now(),
            voting_ends_at: chrono::Utc::now() + chrono::Duration::days(7),
            status: ProposalStatus::Active,
            proposal_type: ProposalType::Constitutional,
            category: ProposalCategory::Economic,
            required_quorum: 0.51,
            execution_timestamp: None,
        };
        node.write().await.create_proposal(proposal).await.unwrap();

        // Now, test getting the proposal status
        let query = GetProposalStatusRequest {
            proposal_id: "test_proposal".to_string(),
        };

        let result = handle_get_proposal_status(query, api_layer).await;
        assert!(result.is_ok());

        if let Ok(reply) = result {
            let response: GetProposalStatusResponse = serde_json::from_slice(reply.into_response().body().as_ref()).unwrap();
            assert_eq!(response.status, ProposalStatus::Active);
        }
    }

    #[tokio::test]
    async fn test_vote_on_proposal() {
        let (api_layer, node) = setup_test_env().await;
        
        // Create a proposal
        let proposal = Proposal {
            id: "test_proposal".to_string(),
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            proposer: "Alice".to_string(),
            created_at: chrono::Utc::now(),
            voting_ends_at: chrono::Utc::now() + chrono::Duration::days(7),
            status: ProposalStatus::Active,
            proposal_type: ProposalType::Constitutional,
            category: ProposalCategory::Economic,
            required_quorum: 0.51,
            execution_timestamp: None,
        };
        node.write().await.create_proposal(proposal).await.unwrap();

        // Vote on the proposal
        let vote = Vote {
            proposal_id: "test_proposal".to_string(),
            voter: "Bob".to_string(),
            in_favor: true,
            weight: 1.0,
        };

        let result = handle_vote_on_proposal(vote, api_layer).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_balance() {
        let (api_layer, node) = setup_test_env().await;
        
        // Initialize balance for Alice
        node.write().await.mint_currency("Alice", &CurrencyType::BasicNeeds, 100.0).await.unwrap();

        let query = GetBalanceQuery {
            address: "Alice".to_string(),
            currency_type: CurrencyType::BasicNeeds,
        };

        let result = handle_get_balance(query, api_layer).await;
        assert!(result.is_ok());

        if let Ok(reply) = result {
            let response: serde_json::Value = serde_json::from_slice(reply.into_response().body().as_ref()).unwrap();
            assert_eq!(response["balance"], 100.0);
        }
    }

    #[tokio::test]
    async fn test_mint_currency() {
        let (api_layer, _) = setup_test_env().await;
        
        let request = MintCurrencyRequest {
            address: "Alice".to_string(),
            currency_type: CurrencyType::BasicNeeds,
            amount: 100.0,
        };

        let result = handle_mint_currency(request, api_layer).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_identity() {
        let (api_layer, _) = setup_test_env().await;
        
        let mut attributes = HashMap::new();
        attributes.insert("name".to_string(), "Alice".to_string());
        attributes.insert("email".to_string(), "alice@example.com".to_string());

        let result = handle_create_identity(attributes, api_layer).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_allocate_resource() {
        let (api_layer, _) = setup_test_env().await;
        
        let request = AllocateResourceRequest {
            resource_type: "computing_power".to_string(),
            amount: 100,
        };

        let result = handle_allocate_resource(request, api_layer).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_network_stats() {
        let (api_layer, _) = setup_test_env().await;

        let result = handle_get_network_stats(api_layer).await;
        assert!(result.is_ok());
    }
}
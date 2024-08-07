// File: crates/icn_api/src/lib.rs

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

    // New method to get block information
    pub async fn get_block_info(&self, identifier: &str) -> IcnResult<icn_blockchain::Block> {
        let node = self.node.read().await;
        if let Ok(height) = identifier.parse::<u64>() {
            node.get_block_by_height(height).await
        } else {
            node.get_block_by_hash(identifier).await
        }
    }

    // New method to get current network difficulty
    pub async fn get_network_difficulty(&self) -> IcnResult<f64> {
        let node = self.node.read().await;
        node.get_network_difficulty().await
    }

    // New method to submit a new smart contract
    pub async fn submit_smart_contract(&self, code: String) -> IcnResult<String> {
        let node = self.node.write().await;
        node.deploy_smart_contract(code).await
    }

    // New method to execute a smart contract
    pub async fn execute_smart_contract(&self, contract_id: &str, function: &str, args: Vec<icn_vm::Value>) -> IcnResult<Option<icn_vm::Value>> {
        let node = self.node.write().await;
        node.execute_smart_contract(contract_id, function, args).await
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

#[derive(Deserialize)]
struct GetBlockInfoRequest {
    identifier: String,
}

#[derive(Serialize)]
struct GetBlockInfoResponse {
    block: icn_blockchain::Block,
}

#[derive(Serialize)]
struct GetNetworkDifficultyResponse {
    difficulty: f64,
}

#[derive(Deserialize)]
struct SubmitSmartContractRequest {
    code: String,
}

#[derive(Serialize)]
struct SubmitSmartContractResponse {
    contract_id: String,
}

#[derive(Deserialize)]
struct ExecuteSmartContractRequest {
    contract_id: String,
    function: String,
    args: Vec<icn_vm::Value>,
}

#[derive(Serialize)]
struct ExecuteSmartContractResponse {
    result: Option<icn_vm::Value>,
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

    let get_block_info = warp::get()
        .and(warp::path("block"))
        .and(warp::query())
        .and(api_layer.clone())
        .and_then(handle_get_block_info);

    let get_network_difficulty = warp::get()
        .and(warp::path("difficulty"))
        .and(api_layer.clone())
        .and_then(handle_get_network_difficulty);

    let submit_smart_contract = warp::post()
        .and(warp::path("contract"))
        .and(warp::body::json())
        .and(api_layer.clone())
        .and_then(handle_submit_smart_contract);

    let execute_smart_contract = warp::post()
        .and(warp::path("contract"))
        .and(warp::path("execute"))
        .and(warp::body::json())
        .and(api_layer.clone())
        .and_then(handle_execute_smart_contract);

    submit_transaction
        .or(create_proposal)
        .or(vote_on_proposal)
        .or(get_balance)
        .or(mint_currency)
        .or(create_identity)
        .or(allocate_resource)
        .or(get_network_stats)
        .or(get_proposal_status)
        .or(get_block_info)
        .or(get_network_difficulty)
        .or(submit_smart_contract)
        .or(execute_smart_contract)
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

async fn handle_get_block_info(
    query: GetBlockInfoRequest,
    api_layer: Arc<RwLock<ApiLayer>>,
) -> Result<impl Reply, Rejection> {
    let api_layer = api_layer.read().await;
    api_layer
        .get_block_info(&query.identifier)
        .await
        .map(|block| warp::reply::json(&GetBlockInfoResponse { block }))
        .map_err(icn_error_to_rejection)
}

async fn handle_get_network_difficulty(
    api_layer: Arc<RwLock<ApiLayer>>,
) -> Result<impl Reply, Rejection> {
    let api_layer = api_layer.read().await;
    api_layer
        .get_network_difficulty()
        .await
        .map(|difficulty| warp::reply::json(&GetNetworkDifficultyResponse { difficulty }))
        .map_err(icn_error_to_rejection)
}

async fn handle_submit_smart_contract(
    request: SubmitSmartContractRequest,
    api_layer: Arc<RwLock<ApiLayer>>,
) -> Result<impl Reply, Rejection> {
    let api_layer = api_layer.read().await;
    api_layer
        .submit_smart_contract(request.code)
        .await
        .map(|contract_id| warp::reply::json(&SubmitSmartContractResponse { contract_id }))
        .map_err(icn_error_to_rejection)
}

async fn handle_execute_smart_contract(
    request: ExecuteSmartContractRequest,
    api_layer: Arc<RwLock<ApiLayer>>,
) -> Result<impl Reply, Rejection> {
    let api_layer = api_layer.read().await;
    api_layer
        .execute_smart_contract(&request.contract_id, &request.function, request.args)
        .await
        .map(|result| warp::reply::json(&ExecuteSmartContractResponse { result }))
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
    async fn test_get_block_info() {
        let (api_layer, node) = setup_test_env().await;

        // Create a test block
        let block = icn_blockchain::Block::new(
            1,
            vec![],
            "previous_hash".to_string(),
            1,
        );

        // Add the block to the blockchain
        {
            let mut node = node.write().await;
            node.add_block(block.clone()).await.unwrap();
        }

        let query = GetBlockInfoRequest {
            identifier: block.hash.clone(),
        };

        let result = handle_get_block_info(query, api_layer).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_network_difficulty() {
        let (api_layer, _) = setup_test_env().await;

        let result = handle_get_network_difficulty(api_layer).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_submit_smart_contract() {
        let (api_layer, _) = setup_test_env().await;
        let request = SubmitSmartContractRequest {
            code: "contract TestContract { }".to_string(),
        };

        let result = handle_submit_smart_contract(request, api_layer).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_smart_contract() {
        let (api_layer, node) = setup_test_env().await;

        // First, deploy a test contract
        let contract_id = {
            let mut node = node.write().await;
            node.deploy_smart_contract("contract TestContract { function test() -> int { return 42; } }".to_string()).await.unwrap()
        };

        let request = ExecuteSmartContractRequest {
            contract_id: contract_id.clone(),
            function: "test".to_string(),
            args: vec![],
        };

        let result = handle_execute_smart_contract(request, api_layer).await;
        assert!(result.is_ok());

        if let Ok(warp::reply::Json(response)) = result {
            let response: ExecuteSmartContractResponse = serde_json::from_value(response.into_inner()).unwrap();
            assert_eq!(response.result, Some(icn_vm::Value::Int(42)));
        } else {
            panic!("Unexpected response type");
        }
    }
}
// File: crates/icn_api/src/web.rs

use warp::Filter;
use crate::ApiLayer;
use std::sync::Arc;
use icn_common::{Transaction, Proposal, DecentralizedIdentity};
use warp::http::StatusCode;
use tokio::sync::RwLock;

pub async fn start_web_server(api: Arc<RwLock<ApiLayer>>) {
    let api = warp::any().map(move || api.clone());

    let hello = warp::path!("hello" / String)
        .map(|name| format!("Hello, {}!", name));

    let transaction = warp::post()
        .and(warp::path("transaction"))
        .and(warp::body::json())
        .and(api.clone())
        .and_then(handle_transaction);

    let proposal = warp::post()
        .and(warp::path("proposal"))
        .and(warp::body::json())
        .and(api.clone())
        .and_then(handle_proposal);

    let identity = warp::post()
        .and(warp::path("identity"))
        .and(warp::body::json())
        .and(api.clone())
        .and_then(handle_identity);

    let block_info = warp::get()
        .and(warp::path("block"))
        .and(warp::query())
        .and(api.clone())
        .and_then(handle_block_info);

    let network_difficulty = warp::get()
        .and(warp::path("difficulty"))
        .and(api.clone())
        .and_then(handle_network_difficulty);

    let submit_contract = warp::post()
        .and(warp::path("contract"))
        .and(warp::body::json())
        .and(api.clone())
        .and_then(handle_submit_contract);

    let execute_contract = warp::post()
        .and(warp::path("contract"))
        .and(warp::path("execute"))
        .and(warp::body::json())
        .and(api.clone())
        .and_then(handle_execute_contract);

    let routes = hello
        .or(transaction)
        .or(proposal)
        .or(identity)
        .or(block_info)
        .or(network_difficulty)
        .or(submit_contract)
        .or(execute_contract);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

async fn handle_transaction(tx: Transaction, api: Arc<RwLock<ApiLayer>>) -> Result<impl warp::Reply, warp::Rejection> {
    let api = api.read().await;
    match api.submit_transaction(tx).await {
        Ok(_) => Ok(warp::reply::with_status("Transaction submitted", StatusCode::OK)),
        Err(e) => {
            eprintln!("Error submitting transaction: {}", e);
            Ok(warp::reply::with_status(format!("Transaction submission failed: {}", e), StatusCode::BAD_REQUEST))
        }
    }
}

async fn handle_proposal(proposal: Proposal, api: Arc<RwLock<ApiLayer>>) -> Result<impl warp::Reply, warp::Rejection> {
    let api = api.read().await;
    match api.create_proposal(proposal).await {
        Ok(response) => Ok(warp::reply::json(&response)),
        Err(e) => {
            eprintln!("Error creating proposal: {}", e);
            Ok(warp::reply::with_status(format!("Proposal creation failed: {}", e), StatusCode::BAD_REQUEST))
        }
    }
}

async fn handle_identity(identity: std::collections::HashMap<String, String>, api: Arc<RwLock<ApiLayer>>) -> Result<impl warp::Reply, warp::Rejection> {
    let api = api.read().await;
    match api.create_identity(identity).await {
        Ok(response) => Ok(warp::reply::json(&response)),
        Err(e) => {
            eprintln!("Error creating identity: {}", e);
            Ok(warp::reply::with_status(format!("Identity creation failed: {}", e), StatusCode::BAD_REQUEST))
        }
    }
}

async fn handle_block_info(query: crate::GetBlockInfoRequest, api: Arc<RwLock<ApiLayer>>) -> Result<impl warp::Reply, warp::Rejection> {
    let api = api.read().await;
    match api.get_block_info(&query.identifier).await {
        Ok(block) => Ok(warp::reply::json(&block)),
        Err(e) => {
            eprintln!("Error getting block info: {}", e);
            Ok(warp::reply::with_status(format!("Failed to get block info: {}", e), StatusCode::NOT_FOUND))
        }
    }
}

async fn handle_network_difficulty(api: Arc<RwLock<ApiLayer>>) -> Result<impl warp::Reply, warp::Rejection> {
    let api = api.read().await;
    match api.get_network_difficulty().await {
        Ok(difficulty) => Ok(warp::reply::json(&difficulty)),
        Err(e) => {
            eprintln!("Error getting network difficulty: {}", e);
            Ok(warp::reply::with_status(format!("Failed to get network difficulty: {}", e), StatusCode::INTERNAL_SERVER_ERROR))
        }
    }
}

async fn handle_submit_contract(contract: crate::SubmitSmartContractRequest, api: Arc<RwLock<ApiLayer>>) -> Result<impl warp::Reply, warp::Rejection> {
    let api = api.read().await;
    match api.submit_smart_contract(contract.code).await {
        Ok(contract_id) => Ok(warp::reply::json(&contract_id)),
        Err(e) => {
            eprintln!("Error submitting smart contract: {}", e);
            Ok(warp::reply::with_status(format!("Smart contract submission failed: {}", e), StatusCode::BAD_REQUEST))
        }
    }
}

async fn handle_execute_contract(request: crate::ExecuteSmartContractRequest, api: Arc<RwLock<ApiLayer>>) -> Result<impl warp::Reply, warp::Rejection> {
    let api = api.read().await;
    match api.execute_smart_contract(&request.contract_id, &request.function, request.args).await {
        Ok(result) => Ok(warp::reply::json(&result)),
        Err(e) => {
            eprintln!("Error executing smart contract: {}", e);
            Ok(warp::reply::with_status(format!("Smart contract execution failed: {}", e), StatusCode::BAD_REQUEST))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_core::Config;
    use std::net::SocketAddr;
    use warp::test::request;
    use serde_json::json;

    async fn setup_test_env() -> Arc<RwLock<ApiLayer>> {
        let config = Config {
            shard_count: 1,
            consensus_threshold: 0.66,
            consensus_quorum: 0.51,
            network_port: 8080,
        };
        let node = Arc::new(RwLock::new(icn_core::IcnNode::new(config).await.unwrap()));
        Arc::new(RwLock::new(ApiLayer::new(node)))
    }

    #[tokio::test]
    async fn test_handle_transaction() {
        let api = setup_test_env().await;
        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 100.0,
            currency_type: icn_common::CurrencyType::BasicNeeds,
            timestamp: chrono::Utc::now().timestamp(),
            signature: None,
        };

        let response = request()
            .method("POST")
            .path("/transaction")
            .json(&transaction)
            .reply(&api_routes(api))
            .await;

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_handle_block_info() {
        let api = setup_test_env().await;
        let block_hash = "test_hash";

        let response = request()
            .method("GET")
            .path(&format!("/block?identifier={}", block_hash))
            .reply(&api_routes(api))
            .await;

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_handle_network_difficulty() {
        let api = setup_test_env().await;

        let response = request()
            .method("GET")
            .path("/difficulty")
            .reply(&api_routes(api))
            .await;

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_handle_submit_contract() {
        let api = setup_test_env().await;
        let contract = json!({
            "code": "contract TestContract { function test() -> int { return 42; } }"
        });

        let response = request()
            .method("POST")
            .path("/contract")
            .json(&contract)
            .reply(&api_routes(api))
            .await;

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_handle_execute_contract() {
        let api = setup_test_env().await;
        let contract_id = "test_contract_id";
        let execute_request = json!({
            "contract_id": contract_id,
            "function": "test",
            "args": []
        });

        let response = request()
            .method("POST")
            .path("/contract/execute")
            .json(&execute_request)
            .reply(&api_routes(api))
            .await;

        assert_eq!(response.status(), StatusCode::OK);
    }
}
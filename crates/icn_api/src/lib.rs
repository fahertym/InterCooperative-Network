async fn handle_create_proposal(proposal: Proposal, api_layer: Arc<RwLock<ApiLayer>>) -> Result<impl warp::Reply, warp::Rejection> {
    let api_layer = api_layer.read().await;
    match api_layer.create_proposal(proposal).await {
        Ok(id) => Ok(warp::reply::json(&json!({"status": "success", "proposal_id": id}))),
        Err(e) => Ok(warp::reply::json(&json!({"status": "error", "message": e.to_string()}))),
    }
}

async fn handle_vote_on_proposal(vote: VoteRequest, api_layer: Arc<RwLock<ApiLayer>>) -> Result<impl warp::Reply, warp::Rejection> {
    let api_layer = api_layer.read().await;
    match api_layer.vote_on_proposal(&vote.proposal_id, vote.voter, vote.in_favor, vote.weight).await {
        Ok(_) => Ok(warp::reply::json(&json!({"status": "success"}))),
        Err(e) => Ok(warp::reply::json(&json!({"status": "error", "message": e.to_string()}))),
    }
}

async fn handle_finalize_proposal(request: FinalizeProposalRequest, api_layer: Arc<RwLock<ApiLayer>>) -> Result<impl warp::Reply, warp::Rejection> {
    let api_layer = api_layer.read().await;
    match api_layer.finalize_proposal(&request.proposal_id).await {
        Ok(status) => Ok(warp::reply::json(&json!({"status": "success", "proposal_status": status}))),
        Err(e) => Ok(warp::reply::json(&json!({"status": "error", "message": e.to_string()}))),
    }
}

async fn handle_get_balance(query: GetBalanceQuery, api_layer: Arc<RwLock<ApiLayer>>) -> Result<impl warp::Reply, warp::Rejection> {
    let api_layer = api_layer.read().await;
    match api_layer.get_balance(&query.address, &query.currency_type).await {
        Ok(balance) => Ok(warp::reply::json(&json!({"status": "success", "balance": balance}))),
        Err(e) => Ok(warp::reply::json(&json!({"status": "error", "message": e.to_string()}))),
    }
}

async fn handle_mint_currency(request: MintCurrencyRequest, api_layer: Arc<RwLock<ApiLayer>>) -> Result<impl warp::Reply, warp::Rejection> {
    let api_layer = api_layer.read().await;
    match api_layer.mint_currency(&request.address, &request.currency_type, request.amount).await {
        Ok(_) => Ok(warp::reply::json(&json!({"status": "success"}))),
        Err(e) => Ok(warp::reply::json(&json!({"status": "error", "message": e.to_string()}))),
    }
}

async fn handle_create_identity(attributes: HashMap<String, String>, api_layer: Arc<RwLock<ApiLayer>>) -> Result<impl warp::Reply, warp::Rejection> {
    let api_layer = api_layer.read().await;
    match api_layer.create_identity(attributes).await {
        Ok(id) => Ok(warp::reply::json(&json!({"status": "success", "identity_id": id}))),
        Err(e) => Ok(warp::reply::json(&json!({"status": "error", "message": e.to_string()}))),
    }
}

async fn handle_allocate_resource(request: AllocateResourceRequest, api_layer: Arc<RwLock<ApiLayer>>) -> Result<impl warp::Reply, warp::Rejection> {
    let api_layer = api_layer.read().await;
    match api_layer.allocate_resource(&request.resource_type, request.amount).await {
        Ok(_) => Ok(warp::reply::json(&json!({"status": "success"}))),
        Err(e) => Ok(warp::reply::json(&json!({"status": "error", "message": e.to_string()}))),
    }
}

async fn handle_get_network_stats(api_layer: Arc<RwLock<ApiLayer>>) -> Result<impl warp::Reply, warp::Rejection> {
    let api_layer = api_layer.read().await;
    match api_layer.get_network_stats().await {
        Ok(stats) => Ok(warp::reply::json(&json!({"status": "success", "stats": stats}))),
        Err(e) => Ok(warp::reply::json(&json!({"status": "error", "message": e.to_string()}))),
    }
}

#[derive(Deserialize)]
struct VoteRequest {
    proposal_id: String,
    voter: String,
    in_favor: bool,
    weight: f64,
}

#[derive(Deserialize)]
struct FinalizeProposalRequest {
    proposal_id: String,
}

#[derive(Deserialize)]
struct GetBalanceQuery {
    address: String,
    currency_type: CurrencyType,
}

#[derive(Deserialize)]
struct MintCurrencyRequest {
    address: String,
    currency_type: CurrencyType,
    amount: f64,
}

#[derive(Deserialize)]
struct AllocateResourceRequest {
    resource_type: String,
    amount: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_core::Config;
    use std::net::SocketAddr;

    async fn setup_test_env() -> (Arc<RwLock<ApiLayer>>, Arc<RwLock<IcnNode>>) {
        let config = Config {
            shard_count: 1,
            consensus_threshold: 0.66,
            consensus_quorum: 0.51,
            network_port: 8080,
        };
        let node = Arc::new(RwLock::new(IcnNode::new(config).await.unwrap()));
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

        let result = handle_create_proposal(proposal, api_layer).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_vote_on_proposal() {
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

        // Now, vote on the proposal
        let vote_request = VoteRequest {
            proposal_id: "test_proposal".to_string(),
            voter: "Bob".to_string(),
            in_favor: true,
            weight: 1.0,
        };

        let result = handle_vote_on_proposal(vote_request, api_layer).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_finalize_proposal() {
        let (api_layer, node) = setup_test_env().await;
        
        // First, create and vote on a proposal
        let proposal = Proposal {
            id: "test_proposal".to_string(),
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            proposer: "Alice".to_string(),
            created_at: chrono::Utc::now(),
            voting_ends_at: chrono::Utc::now() - chrono::Duration::hours(1), // Set voting period to have ended
            status: ProposalStatus::Active,
            proposal_type: ProposalType::Constitutional,
            category: ProposalCategory::Economic,
            required_quorum: 0.51,
            execution_timestamp: None,
        };
        node.write().await.create_proposal(proposal).await.unwrap();
        node.write().await.vote_on_proposal("test_proposal", "Bob".to_string(), true, 1.0).await.unwrap();

        // Now, finalize the proposal
        let finalize_request = FinalizeProposalRequest {
            proposal_id: "test_proposal".to_string(),
        };

        let result = handle_finalize_proposal(finalize_request, api_layer).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_balance() {
        let (api_layer, node) = setup_test_env().await;
        
        // First, mint some currency for an address
        node.write().await.mint_currency("Alice", &CurrencyType::BasicNeeds, 100.0).await.unwrap();

        // Now, get the balance
        let query = GetBalanceQuery {
            address: "Alice".to_string(),
            currency_type: CurrencyType::BasicNeeds,
        };

        let result = handle_get_balance(query, api_layer).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mint_currency() {
        let (api_layer, _) = setup_test_env().await;
        
        let mint_request = MintCurrencyRequest {
            address: "Alice".to_string(),
            currency_type: CurrencyType::BasicNeeds,
            amount: 100.0,
        };

        let result = handle_mint_currency(mint_request, api_layer).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_identity() {
        let (api_layer, _) = setup_test_env().await;
        
        let mut attributes = HashMap::new();
        attributes.insert("name".to_string(), "Alice".to_string());
        attributes.insert("age".to_string(), "30".to_string());

        let result = handle_create_identity(attributes, api_layer).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_allocate_resource() {
        let (api_layer, _) = setup_test_env().await;
        
        let allocate_request = AllocateResourceRequest {
            resource_type: "computing_power".to_string(),
            amount: 100,
        };

        let result = handle_allocate_resource(allocate_request, api_layer).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_network_stats() {
        let (api_layer, _) = setup_test_env().await;

        let result = handle_get_network_stats(api_layer).await;
        assert!(result.is_ok());
    }
}
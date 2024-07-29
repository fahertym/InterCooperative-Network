use icn_core::IcnNode;
use icn_common::{Transaction, Proposal, IcnResult, DecentralizedIdentity};
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

use warp::Filter;

pub fn routes(api_layer: Arc<RwLock<ApiLayer>>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let api = warp::path("api");

    let submit_transaction = warp::path("submit_transaction")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_api_layer(api_layer.clone()))
        .and_then(|tx: Transaction, api_layer: Arc<RwLock<ApiLayer>>| async move {
            match api_layer.read().await.submit_transaction(tx).await {
                Ok(_) => Ok(warp::reply::json(&"Transaction submitted successfully")),
                Err(err) => Err(warp::reject::custom(err)),
            }
        });

    let create_proposal = warp::path("create_proposal")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_api_layer(api_layer.clone()))
        .and_then(|proposal: Proposal, api_layer: Arc<RwLock<ApiLayer>>| async move {
            match api_layer.read().await.create_proposal(proposal).await {
                Ok(id) => Ok(warp::reply::json(&id)),
                Err(err) => Err(warp::reject::custom(err)),
            }
        });

    let create_identity = warp::path("create_identity")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_api_layer(api_layer))
        .and_then(|attributes: std::collections::HashMap<String, String>, api_layer: Arc<RwLock<ApiLayer>>| async move {
            match api_layer.read().await.create_identity(attributes).await {
                Ok(identity) => Ok(warp::reply::json(&identity)),
                Err(err) => Err(warp::reject::custom(err)),
            }
        });

    api.and(submit_transaction.or(create_proposal).or(create_identity))
}

fn with_api_layer(api_layer: Arc<RwLock<ApiLayer>>) -> impl Filter<Extract = (Arc<RwLock<ApiLayer>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || api_layer.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::{CurrencyType, Transaction, Proposal};
    use warp::Filter;
    use tokio::runtime::Runtime;

    #[test]
    fn test_api_routes() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let node = Arc::new(RwLock::new(IcnNode::new()));
            let api_layer = Arc::new(RwLock::new(ApiLayer::new(node)));

            let api = routes(api_layer);

            let transaction = Transaction {
                from: "Alice".to_string(),
                to: "Bob".to_string(),
                amount: 100.0,
                currency_type: CurrencyType::BasicNeeds,
                timestamp: chrono::Utc::now().timestamp(),
                signature: None,
            };

            let proposal = Proposal {
                id: "1".to_string(),
                title: "Proposal Title".to_string(),
                description: "Proposal Description".to_string(),
                proposer: "Alice".to_string(),
                created_at: chrono::Utc::now(),
                voting_ends_at: chrono::Utc::now(),
                status: icn_common::ProposalStatus::Active,
                proposal_type: icn_common::ProposalType::EconomicAdjustment,
                category: icn_common::ProposalCategory::Economic,
                required_quorum: 0.6,
                execution_timestamp: None,
            };

            let res = warp::test::request()
                .method("POST")
                .path("/api/submit_transaction")
                .json(&transaction)
                .reply(&api)
                .await;

            assert_eq!(res.status(), 200);
            assert_eq!(res.body(), "Transaction submitted successfully");

            let res = warp::test::request()
                .method("POST")
                .path("/api/create_proposal")
                .json(&proposal)
                .reply(&api)
                .await;

            assert_eq!(res.status(), 200);
            assert!(serde_json::from_slice::<String>(res.body()).is_ok());
        });
    }
}

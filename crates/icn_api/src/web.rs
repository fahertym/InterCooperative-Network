// File: crates/icn_api/src/web.rs

use warp::Filter;
use crate::ApiLayer;
use std::sync::Arc;

pub async fn start_web_server(api: Arc<ApiLayer>) {
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

    let routes = hello
        .or(transaction)
        .or(proposal);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

async fn handle_transaction(tx: Transaction, api: Arc<ApiLayer>) -> Result<impl warp::Reply, warp::Rejection> {
    match api.submit_transaction(tx).await {
        Ok(response) => Ok(warp::reply::json(&response)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

async fn handle_proposal(proposal: Proposal, api: Arc<ApiLayer>) -> Result<impl warp::Reply, warp::Rejection> {
    match api.create_proposal(proposal).await {
        Ok(response) => Ok(warp::reply::json(&response)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}
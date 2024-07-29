use icn_core::{IcnNode, Config};
use icn_common::{Transaction, Proposal, CurrencyType, ProposalStatus, ProposalType, ProposalCategory};
use icn_identity::DecentralizedIdentity;
use chrono::{Duration, Utc};
use log::{info, error};
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config = Config {
        shard_count: 4,
        consensus_threshold: 0.66,
        consensus_quorum: 0.51,
        network_port: 8080,
    };

    info!("Starting InterCooperative Network demo...");
    let node = IcnNode::new(config).await?;
    node.start().await?;

    // Demo 1: Create identities
    info!("Demo 1: Creating identities");
    let alice_identity = create_identity(&node, "Alice").await?;
    let bob_identity = create_identity(&node, "Bob").await?;

    // Demo 2: Mint currency
    info!("Demo 2: Minting currency");
    node.mint_currency(&alice_identity.id, CurrencyType::BasicNeeds, 1000.0).await?;
    node.mint_currency(&bob_identity.id, CurrencyType::BasicNeeds, 500.0).await?;

    // Demo 3: Process a transaction
    info!("Demo 3: Processing a transaction");
    let transaction = Transaction {
        from: alice_identity.id.clone(),
        to: bob_identity.id.clone(),
        amount: 100.0,
        currency_type: CurrencyType::BasicNeeds,
        timestamp: Utc::now().timestamp(),
        signature: None, // In a real scenario, this should be signed
    };
    node.process_transaction(transaction).await?;

    // Demo 4: Check balances
    info!("Demo 4: Checking balances");
    let alice_balance = node.get_balance(&alice_identity.id, &CurrencyType::BasicNeeds).await?;
    let bob_balance = node.get_balance(&bob_identity.id, &CurrencyType::BasicNeeds).await?;
    info!("Alice's balance: {}", alice_balance);
    info!("Bob's balance: {}", bob_balance);

    // Demo 5: Create and vote on a proposal
    info!("Demo 5: Creating and voting on a proposal");
    let proposal = Proposal {
        id: Uuid::new_v4().to_string(),
        title: "Increase node count".to_string(),
        description: "Proposal to increase the number of nodes in the network".to_string(),
        proposer: alice_identity.id.clone(),
        created_at: Utc::now(),
        voting_ends_at: Utc::now() + Duration::days(7),
        status: ProposalStatus::Active,
        proposal_type: ProposalType::NetworkUpgrade,
        category: ProposalCategory::Technical,
        required_quorum: 0.51,
        execution_timestamp: None,
    };
    let proposal_id = node.create_proposal(proposal).await?;
    
    node.vote_on_proposal(&proposal_id, alice_identity.id.clone(), true).await?;
    node.vote_on_proposal(&proposal_id, bob_identity.id.clone(), false).await?;

    // For demo purposes, we'll finalize the proposal immediately
    let proposal_status = node.finalize_proposal(&proposal_id).await?;
    info!("Proposal status: {:?}", proposal_status);

    // Demo 6: Allocate a resource
    info!("Demo 6: Allocating a resource");
    node.allocate_resource("computing_power", 100).await?;

    // Demo 7: Get network stats
    info!("Demo 7: Getting network stats");
    let network_stats = node.get_network_stats().await?;
    info!("Network stats: {:?}", network_stats);

    info!("Demo completed successfully!");
    node.stop().await?;

    Ok(())
}

async fn create_identity(node: &IcnNode, name: &str) -> Result<DecentralizedIdentity, Box<dyn std::error::Error>> {
    let mut attributes = HashMap::new();
    attributes.insert("name".to_string(), name.to_string());
    Ok(node.create_identity(attributes).await?)
}
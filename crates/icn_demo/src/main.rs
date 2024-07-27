// File: /home/matt/InterCooperative-Network/crates/icn_demo/src/main.rs

use icn_core::IcnNode;
use icn_common::{Transaction, Proposal, CurrencyType, ProposalType, ProposalCategory, ProposalStatus};
use chrono::{Utc, Duration};
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting InterCooperative Network Demo");

    let node = IcnNode::new().await?;

    // Create identities
    println!("\nCreating identities...");
    let alice_identity = node.create_identity(HashMap::new()).await?;
    let bob_identity = node.create_identity(HashMap::new()).await?;
    println!("Created identities for Alice and Bob");

    // Initialize currency system
    println!("\nInitializing currency system...");
    node.currency_system.write().await.mint(&alice_identity.id, &CurrencyType::BasicNeeds, 1000.0)?;
    println!("Initialized Alice's account with 1000 BasicNeeds");

    // Check balances
    let alice_balance = node.get_balance(&alice_identity.id, &CurrencyType::BasicNeeds).await?;
    let bob_balance = node.get_balance(&bob_identity.id, &CurrencyType::BasicNeeds).await?;
    println!("Alice's balance: {} BasicNeeds", alice_balance);
    println!("Bob's balance: {} BasicNeeds", bob_balance);

    // Process a transaction
    println!("\nProcessing transaction...");
    let transaction = Transaction {
        from: alice_identity.id.clone(),
        to: bob_identity.id.clone(),
        amount: 100.0,
        currency_type: CurrencyType::BasicNeeds,
        timestamp: Utc::now().timestamp(),
        signature: None,
    };
    node.process_transaction(transaction).await?;
    println!("Processed transaction from Alice to Bob");

    // Check updated balances
    let alice_balance = node.get_balance(&alice_identity.id, &CurrencyType::BasicNeeds).await?;
    let bob_balance = node.get_balance(&bob_identity.id, &CurrencyType::BasicNeeds).await?;
    println!("Alice's updated balance: {} BasicNeeds", alice_balance);
    println!("Bob's updated balance: {} BasicNeeds", bob_balance);

    // Create a proposal
    println!("\nCreating a proposal...");
    let proposal = Proposal {
        id: Uuid::new_v4().to_string(),
        title: "Increase node count".to_string(),
        description: "Proposal to increase the number of nodes in the network".to_string(),
        proposer: alice_identity.id.clone(),
        created_at: Utc::now(),
        voting_ends_at: Utc::now() + Duration::days(7),
        status: ProposalStatus::Active,
        proposal_type: ProposalType::Constitutional,
        category: ProposalCategory::Technical,
        required_quorum: 0.51,
        execution_timestamp: None,
    };
    let proposal_id = node.create_proposal(proposal).await?;
    println!("Created proposal: {}", proposal_id);

    // Vote on the proposal
    println!("\nVoting on the proposal...");
    node.vote_on_proposal(&proposal_id, alice_identity.id.clone(), true).await?;
    node.vote_on_proposal(&proposal_id, bob_identity.id.clone(), false).await?;
    println!("Alice voted in favor, Bob voted against");

    // Finalize the proposal
    println!("\nFinalizing the proposal...");
    let proposal_status = node.finalize_proposal(&proposal_id).await?;
    println!("Proposal status after finalization: {:?}", proposal_status);

    // Create a new block
    println!("\nCreating a new block...");
    let new_block = node.create_block().await?;
    println!("Created new block: {:?}", new_block);

    // Display final state
    println!("\nFinal state:");
    let alice_balance = node.get_balance(&alice_identity.id, &CurrencyType::BasicNeeds).await?;
    let bob_balance = node.get_balance(&bob_identity.id, &CurrencyType::BasicNeeds).await?;
    println!("Alice's final balance: {} BasicNeeds", alice_balance);
    println!("Bob's final balance: {} BasicNeeds", bob_balance);

    let blockchain_height = node.get_blockchain_height().await;
    println!("Blockchain height: {}", blockchain_height);

    let is_valid = node.validate_blockchain().await;
    println!("Is blockchain valid: {}", is_valid);

    println!("\nDemo completed successfully!");
    Ok(())
}
use icn_core::IcnNode;
use icn_common::{Transaction, Proposal, CurrencyType, ProposalType, ProposalCategory, ProposalStatus};
use chrono::{Utc, Duration};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting InterCooperative Network Demo");

    let node = IcnNode::new()?;

    // Create identities
    println!("\nCreating identities...");
    let alice_identity = node.create_identity(HashMap::new())?;
    let bob_identity = node.create_identity(HashMap::new())?;
    println!("Created identities for Alice and Bob");

    // Initialize currency system
    println!("\nInitializing currency system...");
    node.process_transaction(Transaction {
        from: "genesis".to_string(),
        to: alice_identity.id.clone(),
        amount: 1000.0,
        currency_type: CurrencyType::BasicNeeds,
        timestamp: Utc::now().timestamp(),
        signature: None,
    })?;
    println!("Initialized Alice's account with 1000 BasicNeeds");

    // Check balances
    let alice_balance = node.get_balance(&alice_identity.id, &CurrencyType::BasicNeeds)?;
    let bob_balance = node.get_balance(&bob_identity.id, &CurrencyType::BasicNeeds)?;
    println!("Alice's balance: {} BasicNeeds", alice_balance);
    println!("Bob's balance: {} BasicNeeds", bob_balance);

    // Process a transaction
    println!("\nProcessing transaction...");
    node.process_transaction(Transaction {
        from: alice_identity.id.clone(),
        to: bob_identity.id.clone(),
        amount: 100.0,
        currency_type: CurrencyType::BasicNeeds,
        timestamp: Utc::now().timestamp(),
        signature: None,
    })?;
    println!("Processed transaction from Alice to Bob");

    // Check updated balances
    let alice_balance = node.get_balance(&alice_identity.id, &CurrencyType::BasicNeeds)?;
    let bob_balance = node.get_balance(&bob_identity.id, &CurrencyType::BasicNeeds)?;
    println!("Alice's updated balance: {} BasicNeeds", alice_balance);
    println!("Bob's updated balance: {} BasicNeeds", bob_balance);

    // Create a proposal
    println!("\nCreating a proposal...");
    let proposal = Proposal {
        id: "proposal1".to_string(),
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
    let proposal_id = node.create_proposal(proposal)?;
    println!("Created proposal: {}", proposal_id);

    // Vote on the proposal
    println!("\nVoting on the proposal...");
    node.vote_on_proposal(&proposal_id, alice_identity.id.clone(), true)?;
    node.vote_on_proposal(&proposal_id, bob_identity.id.clone(), false)?;
    println!("Alice voted in favor, Bob voted against");

    // Finalize the proposal
    println!("\nFinalizing the proposal...");
    let proposal_status = node.finalize_proposal(&proposal_id)?;
    println!("Proposal status after finalization: {:?}", proposal_status);

    // Create a new block
    println!("\nCreating a new block...");
    let new_block = node.create_block()?;
    println!("Created new block: {:?}", new_block);

    // Display final state
    println!("\nFinal state:");
    let alice_balance = node.get_balance(&alice_identity.id, &CurrencyType::BasicNeeds)?;
    let bob_balance = node.get_balance(&bob_identity.id, &CurrencyType::BasicNeeds)?;
    println!("Alice's final balance: {} BasicNeeds", alice_balance);
    println!("Bob's final balance: {} BasicNeeds", bob_balance);

    println!("\nDemo completed successfully!");
    Ok(())
}
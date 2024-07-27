// crates/icn_demo/src/main.rs

use icn_common::{Transaction, Proposal, CurrencyType, ProposalStatus, ProposalType, ProposalCategory};
use icn_blockchain::Blockchain;
use icn_consensus::PoCConsensus;
use icn_governance::GovernanceSystem;
use icn_currency::CurrencySystem;
use chrono::{Duration, Utc};
use uuid::Uuid;

fn main() {
    println!("Starting InterCooperative Network Demo");

    // Initialize blockchain
    let mut blockchain = Blockchain::new();

    // Initialize consensus mechanism
    let mut consensus = PoCConsensus::new(0.66, 0.51).expect("Failed to create consensus mechanism");

    // Initialize governance system
    let mut governance = GovernanceSystem::new();

    // Initialize currency system
    let mut currency_system = CurrencySystem::new();
    currency_system.add_currency(CurrencyType::BasicNeeds, 1000000.0, 0.01);

    // Simulate network activity
    simulate_transactions(&mut blockchain);
    simulate_proposal(&mut governance);
    simulate_block_creation(&mut blockchain, &mut consensus);

    println!("Demo completed successfully!");
}

fn simulate_transactions(blockchain: &mut Blockchain) {
    println!("Simulating transactions...");

    let transaction1 = Transaction {
        from: "Alice".to_string(),
        to: "Bob".to_string(),
        amount: 100.0,
        currency_type: CurrencyType::BasicNeeds,
        timestamp: Utc::now().timestamp(),
        signature: None,
    };

    let transaction2 = Transaction {
        from: "Bob".to_string(),
        to: "Charlie".to_string(),
        amount: 50.0,
        currency_type: CurrencyType::BasicNeeds,
        timestamp: Utc::now().timestamp(),
        signature: None,
    };

    blockchain.add_transaction(transaction1).expect("Failed to add transaction 1");
    blockchain.add_transaction(transaction2).expect("Failed to add transaction 2");

    println!("Transactions added to the blockchain");
}

fn simulate_proposal(governance: &mut GovernanceSystem) {
    println!("Simulating proposal creation and voting...");

    let proposal = Proposal {
        id: Uuid::new_v4().to_string(),
        title: "Increase node count".to_string(),
        description: "Proposal to increase the number of nodes in the network".to_string(),
        proposer: "Alice".to_string(),
        created_at: Utc::now(),
        voting_ends_at: Utc::now() + Duration::days(7),
        status: ProposalStatus::Active,
        proposal_type: ProposalType::NetworkUpgrade,
        category: ProposalCategory::Technical,
        required_quorum: 0.51,
        execution_timestamp: None,
    };

    let proposal_id = governance.create_proposal(proposal).expect("Failed to create proposal");
    governance.vote_on_proposal(&proposal_id, "Alice".to_string(), true).expect("Failed to vote on proposal");
    governance.vote_on_proposal(&proposal_id, "Bob".to_string(), true).expect("Failed to vote on proposal");
    governance.vote_on_proposal(&proposal_id, "Charlie".to_string(), false).expect("Failed to vote on proposal");

    // For demonstration purposes, we'll finalize the proposal immediately
    // In a real system, this would happen after the voting period ends
    let result = governance.finalize_proposal(&proposal_id).expect("Failed to finalize proposal");
    println!("Proposal finalized with result: {:?}", result);
}

fn simulate_block_creation(blockchain: &mut Blockchain, consensus: &mut PoCConsensus) {
    println!("Simulating block creation...");

    let new_block = blockchain.create_block().expect("Failed to create block");
    consensus.process_new_block(new_block.clone()).expect("Failed to process new block");

    println!("New block created and processed: {:?}", new_block);
}
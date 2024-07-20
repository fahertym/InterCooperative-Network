// icn_core/tests/integration_tests.rs

use icn_core::{IcnNode, Config};
use icn_types::{Transaction, Proposal, ProposalType, ProposalCategory, CurrencyType, ProposalStatus, IcnResult};
use chrono::{Utc, Duration};

fn setup() -> IcnResult<IcnNode> {
    let config = Config::default();
    let node = IcnNode::new(config)?;
    node.start()?;
    Ok(node)
}

fn teardown(node: &IcnNode) -> IcnResult<()> {
    node.stop()
}

#[test]
fn test_node_lifecycle() -> IcnResult<()> {
    let node = setup()?;
    teardown(&node)
}

#[test]
fn test_transaction_processing() -> IcnResult<()> {
    let node = setup()?;

    let transaction = Transaction {
        from: "Alice".to_string(),
        to: "Bob".to_string(),
        amount: 100.0,
        currency_type: CurrencyType::BasicNeeds,
        timestamp: Utc::now().timestamp(),
        signature: None, // In a real scenario, this should be signed
    };

    node.process_transaction(transaction)?;

    // TODO: Add assertions to verify the transaction was processed correctly
    // This might involve querying the blockchain or checking account balances

    teardown(&node)
}

#[test]
fn test_proposal_creation() -> IcnResult<()> {
    let node = setup()?;

    let proposal = Proposal {
        id: String::new(), // This will be set by the system
        title: "Test Proposal".to_string(),
        description: "This is a test proposal".to_string(),
        proposer: "Alice".to_string(),
        created_at: Utc::now(),
        voting_ends_at: Utc::now() + Duration::days(7),
        status: ProposalStatus::Active,
        proposal_type: ProposalType::Constitutional,
        category: ProposalCategory::Economic,
        required_quorum: 0.51,
        execution_timestamp: None,
    };

    let proposal_id = node.create_proposal(proposal)?;
    assert!(!proposal_id.is_empty(), "Proposal ID should not be empty");

    // TODO: Add more assertions to verify the proposal was created correctly
    // This might involve querying the governance system for the proposal details

    teardown(&node)
}

#[test]
fn test_get_balance() -> IcnResult<()> {
    let node = setup()?;

    // First, we need to add some balance to an account
    let deposit_transaction = Transaction {
        from: "Genesis".to_string(),
        to: "Alice".to_string(),
        amount: 1000.0,
        currency_type: CurrencyType::BasicNeeds,
        timestamp: Utc::now().timestamp(),
        signature: None,
    };

    node.process_transaction(deposit_transaction)?;

    // Now, let's check the balance
    let balance = node.get_balance("Alice", &CurrencyType::BasicNeeds)?;
    assert_eq!(balance, 1000.0, "Balance should be 1000.0 after deposit");

    teardown(&node)
}

#[test]
fn test_cross_shard_transaction() -> IcnResult<()> {
    let node = setup()?;

    // Assume Alice and Bob are on different shards
    // First, add balance to Alice's account
    let deposit_transaction = Transaction {
        from: "Genesis".to_string(),
        to: "Alice".to_string(),
        amount: 1000.0,
        currency_type: CurrencyType::BasicNeeds,
        timestamp: Utc::now().timestamp(),
        signature: None,
    };

    node.process_transaction(deposit_transaction)?;

    // Now, perform a cross-shard transaction from Alice to Bob
    let cross_shard_transaction = Transaction {
        from: "Alice".to_string(),
        to: "Bob".to_string(),
        amount: 500.0,
        currency_type: CurrencyType::BasicNeeds,
        timestamp: Utc::now().timestamp(),
        signature: None,
    };

    node.process_transaction(cross_shard_transaction)?;

    // Check balances after the transaction
    let alice_balance = node.get_balance("Alice", &CurrencyType::BasicNeeds)?;
    let bob_balance = node.get_balance("Bob", &CurrencyType::BasicNeeds)?;

    assert_eq!(alice_balance, 500.0, "Alice's balance should be 500.0 after transfer");
    assert_eq!(bob_balance, 500.0, "Bob's balance should be 500.0 after receiving transfer");

    teardown(&node)
}

// TODO: Add more integration tests as needed
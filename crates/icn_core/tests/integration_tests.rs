// File: crates/icn_core/tests/integration_tests.rs

use icn_core::{IcnNode, Config};
use icn_common::{Transaction, Proposal, CurrencyType, ProposalStatus, ProposalType, ProposalCategory};
use tokio::test;
use std::collections::HashMap;
use chrono::Utc;
use uuid::Uuid;

#[tokio::test]
async fn test_node_creation_and_basic_operations() {
    let config = Config {
        shard_count: 4,
        consensus_threshold: 0.66,
        consensus_quorum: 0.51,
        network_port: 8080,
    };

    let node = IcnNode::new(config).unwrap();
    node.start().await.unwrap();

    // Test create identity
    let mut attributes = HashMap::new();
    attributes.insert("name".to_string(), "Alice".to_string());
    attributes.insert("email".to_string(), "alice@example.com".to_string());
    let identity = node.create_identity(attributes).unwrap();
    assert_eq!(identity.attributes.get("name"), Some(&"Alice".to_string()));

    // Test process transaction
    let transaction = Transaction {
        from: "Alice".to_string(),
        to: "Bob".to_string(),
        amount: 50.0,
        currency_type: CurrencyType::BasicNeeds,
        timestamp: chrono::Utc::now().timestamp(),
        signature: None,
    };
    assert!(node.process_transaction(transaction).await.is_ok());

    // Test create proposal
    let proposal = Proposal {
        id: Uuid::new_v4().to_string(),
        title: "Test Proposal".to_string(),
        description: "This is a test proposal".to_string(),
        proposer: "Alice".to_string(),
        created_at: Utc::now(),
        voting_ends_at: Utc::now() + chrono::Duration::days(7),
        status: ProposalStatus::Active,
        proposal_type: ProposalType::Constitutional,
        category: ProposalCategory::Economic,
        required_quorum: 0.66,
        execution_timestamp: None,
    };
    assert!(node.create_proposal(proposal).is_ok());

    // Test get network stats
    let stats = node.get_network_stats().await.unwrap();
    assert!(stats.connected_peers >= 0);

    // Test allocate resource
    assert!(node.allocate_resource("computing_power", 100).is_ok());

    // Test get balance
    let balance = node.get_balance("Alice", &CurrencyType::BasicNeeds).unwrap();
    assert!(balance >= 0.0);

    node.stop().await.unwrap();
}

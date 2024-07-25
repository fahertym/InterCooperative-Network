// File: crates/icn_core/tests/integration_tests.rs

use icn_core::{IcnNode, Config};
use icn_common::{Transaction, Proposal, ProposalType, ProposalCategory, CurrencyType, ProposalStatus};
use chrono::Duration;

#[tokio::test]
async fn test_node_creation_and_basic_operations() {
    let config = Config {
        shard_count: 4,
        consensus_threshold: 0.66,
        consensus_quorum: 0.51,
        network_port: 8080,
    };

    let node = IcnNode::new(config).unwrap();
    assert!(node.start().is_ok());

     // Test create identity
     let mut attributes = std::collections::HashMap::new();
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
         timestamp: Utc::now().timestamp(),
         signature: None,
     };
     assert!(node.process_transaction(transaction).is_ok());
 
     // Test create proposal
     let proposal = Proposal {
         id: Uuid::new_v4().to_string(),
         title: "Test Proposal".to_string(),
         description: "This is a test proposal".to_string(),
         proposer: "Alice".to_string(),
         created_at: Utc::now(),
         voting_ends_at: Utc::now() + Duration::days(7),
         status: ProposalStatus::Active,
         proposal_type: ProposalType::Constitutional,
         category: ProposalCategory::Economic,
         required_quorum: 0.66,
         execution_timestamp: None,
     };
     assert!(node.create_proposal(proposal).is_ok());
 
     // Test get network stats
     let stats = node.get_network_stats().unwrap();
     assert!(stats.connected_peers >= 0);
 
     // Test allocate resource
     assert!(node.allocate_resource("computing_power", 100).is_ok());
 
     // Test get balance
     let balance = node.get_balance("Alice", &CurrencyType::BasicNeeds).unwrap();
     assert!(balance >= 0.0);
 
     assert!(node.stop().is_ok());
 }
 
 #[tokio::test]
 async fn test_concurrent_transactions() {
     let config = Config {
         shard_count: 4,
         consensus_threshold: 0.66,
         consensus_quorum: 0.51,
         network_port: 8081,
     };
 
     let node = Arc::new(IcnNode::new(config).unwrap());
     node.start().await.unwrap();
 
     let mut handles = vec![];
 
     for i in 0..100 {
         let node_clone = Arc::clone(&node);
         let handle = tokio::spawn(async move {
             let tx = Transaction {
                 from: format!("User{}", i),
                 to: format!("User{}", (i + 1) % 100),
                 amount: 1.0,
                 currency_type: CurrencyType::BasicNeeds,
                 timestamp: chrono::Utc::now().timestamp(),
                 signature: None,
             };
             node_clone.process_transaction(tx).await.unwrap();
         });
         handles.push(handle);
     }
 
     for handle in handles {
         handle.await.unwrap();
     }
 
     let total_transactions = node.blockchain.read().unwrap().get_total_transactions();
     assert_eq!(total_transactions, 100);
 
     node.stop().await.unwrap();
 }
 
 #[tokio::test]
 async fn test_governance_workflow() {
     let config = Config {
         shard_count: 4,
         consensus_threshold: 0.66,
         consensus_quorum: 0.51,
         network_port: 8082,
     };
 
     let node = IcnNode::new(config).unwrap();
     node.start().await.unwrap();
 
     // Create a proposal
     let proposal = Proposal {
         id: "gov_proposal1".to_string(),
         title: "Increase Education Currency Supply".to_string(),
         description: "Proposal to increase the supply of Education currency by 10%".to_string(),
         proposer: "Alice".to_string(),
         created_at: chrono::Utc::now(),
         voting_ends_at: chrono::Utc::now() + chrono::Duration::days(7),
         status: ProposalStatus::Active,
         proposal_type: ProposalType::EconomicAdjustment,
         category: ProposalCategory::Economic,
         required_quorum: 0.75,
         execution_timestamp: None,
     };
 
     let proposal_id = node.create_proposal(proposal).unwrap();
 
     // Simulate voting
     for i in 0..10 {
         let voter = format!("Voter{}", i);
         let vote = i % 2 == 0; // Alternating yes/no votes
         node.vote_on_proposal(&proposal_id, &voter, vote).unwrap();
     }
 
     // Fast-forward time to end of voting period
     // In a real scenario, you'd use a time mocking library
     node.governance.write().unwrap().update_proposal_status(&proposal_id).unwrap();
 
     // Check proposal status
     let final_status = node.governance.read().unwrap().get_proposal_status(&proposal_id).unwrap();
     assert_eq!(final_status, ProposalStatus::Passed);
 
     // Execute the proposal
     node.governance.write().unwrap().execute_proposal(&proposal_id).unwrap();
 
     // Verify the outcome (in this case, check if Education currency supply increased)
     let education_supply = node.currency_system.read().unwrap().get_total_supply(&CurrencyType::Education).unwrap();
     assert!(education_supply > 0.0); // Assuming initial supply was 0
 
     node.stop().await.unwrap();
 }
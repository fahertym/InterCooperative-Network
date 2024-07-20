// icn_core/tests/integration_tests.rs

use icn_core::{IcnNode, Config};
use icn_types::{Transaction, Proposal, ProposalType, ProposalCategory, CurrencyType, ProposalStatus};
use chrono::Duration;
use log::info;
use uuid::Uuid;

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_node_initialization() {
        let config = Config::default();
        let node = IcnNode::new(config);
        assert!(node.is_ok());
    }

    #[test]
    fn test_node_start_stop() {
        let config = Config::default();
        let node = IcnNode::new(config).unwrap();
        assert!(node.start().is_ok());
        assert!(node.stop().is_ok());
    }

    #[test]
    fn test_process_transaction() {
        let config = Config::default();
        let node = IcnNode::new(config).unwrap();
        node.start().unwrap();

        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 100.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: chrono::Utc::now().timestamp(),
            signature: None,
        };

        assert!(node.process_transaction(transaction).is_ok());
        node.stop().unwrap();
    }

    #[test]
    fn test_create_proposal() {
        let config = Config::default();
        let node = IcnNode::new(config).unwrap();
        node.start().unwrap();

        let proposal = Proposal {
            id: Uuid::new_v4().to_string(),
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            proposer: "Alice".to_string(),
            created_at: chrono::Utc::now(),
            voting_ends_at: chrono::Utc::now() + Duration::days(7),
            status: ProposalStatus::Active,
            proposal_type: ProposalType::Constitutional,
            category: ProposalCategory::Economic,
            required_quorum: 0.66,
            execution_timestamp: None,
        };

        assert!(node.create_proposal(proposal).is_ok());
        node.stop().unwrap();
    }

    #[test]
    fn test_get_balance() {
        let node = setup().unwrap();

        // First, we need to add some balance to an account
        let deposit_transaction = Transaction {
            from: "Genesis".to_string(),
            to: "Alice".to_string(),
            amount: 1000.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: chrono::Utc::now().timestamp(),
            signature: None,
        };

        node.process_transaction(deposit_transaction).unwrap();

        // Now, let's check the balance
        let balance = node.get_balance("Alice", &CurrencyType::BasicNeeds).unwrap();
        assert_eq!(balance, 1000.0, "Balance should be 1000.0 after deposit");

        teardown(&node).unwrap();
    }

    #[test]
    fn test_cross_shard_transaction() {
        let node = setup().unwrap();

        // Assume Alice and Bob are on different shards
        // First, add balance to Alice's account
        let deposit_transaction = Transaction {
            from: "Genesis".to_string(),
            to: "Alice".to_string(),
            amount: 1000.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: chrono::Utc::now().timestamp(),
            signature: None,
        };

        node.process_transaction(deposit_transaction).unwrap();

        // Now, perform a cross-shard transaction from Alice to Bob
        let cross_shard_transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 500.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: chrono::Utc::now().timestamp(),
            signature: None,
        };

        node.process_transaction(cross_shard_transaction).unwrap();

        // Check balances after the transaction
        let alice_balance = node.get_balance("Alice", &CurrencyType::BasicNeeds).unwrap();
        let bob_balance = node.get_balance("Bob", &CurrencyType::BasicNeeds).unwrap();

        assert_eq!(alice_balance, 500.0, "Alice's balance should be 500.0 after transfer");
        assert_eq!(bob_balance, 500.0, "Bob's balance should be 500.0 after receiving transfer");

        teardown(&node).unwrap();
    }
}

// Filename: src/tests/blockchain_and_consensus_tests.rs

use crate::blockchain::{Blockchain, Transaction};
use crate::currency::CurrencyType;
use crate::consensus::Consensus;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockchain_creation() {
        let blockchain = Blockchain::new();
        assert_eq!(blockchain.chain.len(), 1, "Blockchain should be initialized with one genesis block");
        assert_eq!(blockchain.chain[0].index, 0, "Genesis block should have index 0");
    }

    #[test]
    fn test_block_creation() {
        let mut blockchain = Blockchain::new();
        blockchain.consensus.add_member("Alice".to_string());
        let transactions = vec![
            Transaction::new("Alice".to_string(), "Bob".to_string(), 100.0, CurrencyType::BasicNeeds, 1000),
            Transaction::new("Bob".to_string(), "Charlie".to_string(), 50.0, CurrencyType::Education, 1000),
        ];
        let result = blockchain.create_block("Alice".to_string());
        assert!(result.is_ok(), "Block creation failed: {:?}", result.err());
        assert_eq!(blockchain.chain.len(), 2, "Blockchain should have two blocks after creation");
    }

    #[test]
    fn test_reputation_update() {
        let mut blockchain = Blockchain::new();
        blockchain.consensus.add_member("Alice".to_string());
        blockchain.consensus.update_reputation("Alice", 0.5);
        assert_eq!(blockchain.consensus.get_reputation("Alice"), Some(1.5), 
                   "Alice's reputation should be updated to 1.5");
    }

    #[test]
    fn test_voting() {
        let mut blockchain = Blockchain::new();
        blockchain.consensus.add_member("Alice".to_string());
        blockchain.consensus.add_member("Bob".to_string());
        blockchain.consensus.add_member("Charlie".to_string());
        
        let transactions = vec![
            Transaction::new("Alice".to_string(), "Bob".to_string(), 100.0, CurrencyType::BasicNeeds, 1000),
        ];
        blockchain.create_block("Alice".to_string()).expect("Block creation should succeed");
        
        assert!(blockchain.vote_on_block("Alice", 1, true).is_ok(), "Alice should be able to vote");
        assert!(blockchain.vote_on_block("Bob", 1, true).is_ok(), "Bob should be able to vote");
        assert!(blockchain.vote_on_block("Charlie", 1, true).is_ok(), "Charlie should be able to vote");
        
        assert!(blockchain.consensus.is_block_valid(1), "Block should be valid after voting");
    }
}

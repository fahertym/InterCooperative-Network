// Filename: src/tests/blockchain_tests.rs

use crate::blockchain::{Blockchain, Transaction};
use crate::currency::CurrencyType;

#[test]
fn test_blockchain_creation() {
    let blockchain = Blockchain::new();
    assert_eq!(blockchain.chain.len(), 1);
    assert_eq!(blockchain.chain[0].index, 0);
}

#[test]
fn test_add_block() {
    let mut blockchain = Blockchain::new();
    let transaction = Transaction::new(
        "Alice".to_string(),
        "Bob".to_string(),
        100.0,
        CurrencyType::BasicNeeds,
        1000,
    );
    blockchain.add_transaction(transaction);
    assert!(blockchain.create_block("Node1".to_string()).is_ok());
    assert_eq!(blockchain.chain.len(), 2);
}

#[test]
fn test_blockchain_validity() {
    let mut blockchain = Blockchain::new();
    let transaction = Transaction::new(
        "Alice".to_string(),
        "Bob".to_string(),
        100.0,
        CurrencyType::BasicNeeds,
        1000,
    );
    blockchain.add_transaction(transaction);
    assert!(blockchain.create_block("Node1".to_string()).is_ok());
    assert!(blockchain.is_chain_valid());
}

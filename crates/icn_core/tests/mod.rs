// ===============================================
// Tests Module
// ===============================================
// This module re-exports the contents of the tests submodules.
// The tests submodules contain various test cases to ensure
// the correctness and reliability of the blockchain implementation.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::currency::CurrencyType;
    use ed25519_dalek::Keypair;
    use rand::rngs::OsRng;

    #[test]
    fn test_cross_shard_transaction() {
        let node = IcnNode::new();

        // Initialize balances
        {
            let mut sharding_manager = node.sharding_manager.write().unwrap();
            sharding_manager.add_address_to_shard("Alice".to_string(), 0);
            sharding_manager.add_address_to_shard("Bob".to_string(), 1);
            sharding_manager.initialize_balance("Alice".to_string(), CurrencyType::BasicNeeds, 1000.0);
        }

        let mut csprng = OsRng{};
        let keypair: Keypair = Keypair::generate(&mut csprng);

        let mut transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            500.0,
            CurrencyType::BasicNeeds,
            1000,
        );
        if let Err(e) = transaction.sign(&keypair) {
            panic!("Failed to sign transaction: {}", e);
        }

        println!("Signed transaction: {:?}", transaction);

        assert!(node.process_cross_shard_transaction(&transaction).is_ok(), "Cross-shard transaction failed");

        // Check balances after transaction
        let sharding_manager = node.sharding_manager.read().unwrap();
        assert_eq!(sharding_manager.get_balance("Alice".to_string(), CurrencyType::BasicNeeds), 500.0);
        assert_eq!(sharding_manager.get_balance("Bob".to_string(), CurrencyType::BasicNeeds), 500.0);
    }
}

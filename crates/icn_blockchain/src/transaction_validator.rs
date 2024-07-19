use crate::{Transaction, Blockchain};
use icn_core::error::{Error, Result};

pub struct TransactionValidator;

impl TransactionValidator {
    pub fn validate_transaction(transaction: &Transaction, blockchain: &Blockchain) -> bool {
        if !Self::is_double_spend(transaction, blockchain) &&
           Self::validate_currency_and_amount(transaction) &&
           Self::check_sufficient_balance(transaction, blockchain) &&
           Self::validate_signature(transaction)
        {
            true
        } else {
            false
        }
    }

    fn is_double_spend(transaction: &Transaction, blockchain: &Blockchain) -> bool {
        for block in &blockchain.chain {
            for tx in &block.transactions {
                if tx == transaction {
                    return true;
                }
            }
        }
        false
    }

    fn validate_currency_and_amount(transaction: &Transaction) -> bool {
        transaction.amount > 0.0
    }

    fn check_sufficient_balance(transaction: &Transaction, blockchain: &Blockchain) -> bool {
        let balance = blockchain.get_balance(&transaction.from);
        balance >= transaction.amount + transaction.get_fee()
    }

    fn validate_signature(transaction: &Transaction) -> bool {
        if let Ok(is_valid) = transaction.verify() {
            is_valid
        } else {
            false
        }
    }

    pub fn can_process_transaction(transaction: &Transaction, blockchain: &Blockchain) -> Result<()> {
        if !Self::validate_transaction(transaction, blockchain) {
            return Err(Error::BlockchainError("Invalid transaction".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::currency::CurrencyType;
    use ed25519_dalek::Keypair;
    use rand::rngs::OsRng;

    fn create_signed_transaction(from: &str, to: &str, amount: f64) -> Transaction {
        let mut tx = Transaction::new(
            from.to_string(),
            to.to_string(),
            amount,
            CurrencyType::BasicNeeds,
            1000,
        );
        let mut csprng = OsRng{};
        let keypair: Keypair = Keypair::generate(&mut csprng);
        tx.sign(&keypair).unwrap();
        tx
    }

    #[test]
    fn test_validate_transaction() {
        let mut blockchain = Blockchain::new();
        let tx = create_signed_transaction("Alice", "Bob", 50.0);

        // Add some balance to Alice's account
        blockchain.add_transaction(create_signed_transaction("Genesis", "Alice", 100.0)).unwrap();
        blockchain.create_block("Miner".to_string()).unwrap();

        assert!(TransactionValidator::validate_transaction(&tx, &blockchain));
    }

    #[test]
    fn test_insufficient_balance() {
        let mut blockchain = Blockchain::new();
        let tx = create_signed_transaction("Alice", "Bob", 150.0);

        // Add some balance to Alice's account, but not enough
        blockchain.add_transaction(create_signed_transaction("Genesis", "Alice", 100.0)).unwrap();
        blockchain.create_block("Miner".to_string()).unwrap();

        assert!(!TransactionValidator::validate_transaction(&tx, &blockchain));
    }

    #[test]
    fn test_double_spend() {
        let mut blockchain = Blockchain::new();
        let tx = create_signed_transaction("Alice", "Bob", 50.0);

        // Add some balance to Alice's account
        blockchain.add_transaction(create_signed_transaction("Genesis", "Alice", 100.0)).unwrap();
        blockchain.create_block("Miner".to_string()).unwrap();

        // Add the transaction to the blockchain
        blockchain.add_transaction(tx.clone()).unwrap();
        blockchain.create_block("Miner".to_string()).unwrap();

        // Try to validate the same transaction again
        assert!(!TransactionValidator::validate_transaction(&tx, &blockchain));
    }
}
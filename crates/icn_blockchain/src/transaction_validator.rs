use crate::{Transaction, Blockchain};
use icn_common::error::{IcnError, IcnResult};

pub struct TransactionValidator;

impl TransactionValidator {
    pub fn validate_transaction(transaction: &Transaction, blockchain: &Blockchain) -> IcnResult<()> {
        if Self::is_double_spend(transaction, blockchain)? {
            return Err(IcnError::Blockchain("Double spend detected".to_string()));
        }

        if !Self::validate_currency_and_amount(transaction) {
            return Err(IcnError::Currency("Invalid currency or amount".to_string()));
        }

        if !Self::check_sufficient_balance(transaction, blockchain)? {
            return Err(IcnError::Currency("Insufficient balance".to_string()));
        }

        if !Self::validate_signature(transaction)? {
            return Err(IcnError::Identity("Invalid signature".to_string()));
        }

        Ok(())
    }

    fn is_double_spend(transaction: &Transaction, blockchain: &Blockchain) -> IcnResult<bool> {
        for block in &blockchain.chain {
            for tx in &block.transactions {
                if tx == transaction {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn validate_currency_and_amount(transaction: &Transaction) -> bool {
        transaction.amount > 0.0
    }

    fn check_sufficient_balance(transaction: &Transaction, blockchain: &Blockchain) -> IcnResult<bool> {
        let balance = blockchain.get_balance(&transaction.from, &transaction.currency_type)?;
        Ok(balance >= transaction.amount + transaction.get_fee())
    }

    fn validate_signature(transaction: &Transaction) -> IcnResult<bool> {
        transaction.verify().map_err(|e| IcnError::Identity(format!("Signature verification failed: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::{Transaction, Blockchain};
    use ed25519_dalek::Keypair;
    use rand::rngs::OsRng;
    use std::sync::Arc;

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
        let mut blockchain = Blockchain::new(Arc::new(TransactionValidator));
        let tx = create_signed_transaction("Alice", "Bob", 50.0);

        // Add some balance to Alice's account
        blockchain.add_transaction(create_signed_transaction("Genesis", "Alice", 100.0)).unwrap();
        blockchain.create_block().unwrap();

        assert!(TransactionValidator::validate_transaction(&tx, &blockchain).is_ok());
    }

    #[test]
    fn test_insufficient_balance() {
        let mut blockchain = Blockchain::new(Arc::new(TransactionValidator));
        let tx = create_signed_transaction("Alice", "Bob", 150.0);

        // Add some balance to Alice's account, but not enough
        blockchain.add_transaction(create_signed_transaction("Genesis", "Alice", 100.0)).unwrap();
        blockchain.create_block().unwrap();

        assert!(TransactionValidator::validate_transaction(&tx, &blockchain).is_err());
    }

    #[test]
    fn test_double_spend() {
        let mut blockchain = Blockchain::new(Arc::new(TransactionValidator));
        let tx = create_signed_transaction("Alice", "Bob", 50.0);

        // Add some balance to Alice's account
        blockchain.add_transaction(create_signed_transaction("Genesis", "Alice", 100.0)).unwrap();
        blockchain.create_block().unwrap();

        // Add the transaction to the blockchain
        blockchain.add_transaction(tx.clone()).unwrap();
        blockchain.create_block().unwrap();

        // Try to validate the same transaction again
        assert!(TransactionValidator::validate_transaction(&tx, &blockchain).is_err());
    }
}

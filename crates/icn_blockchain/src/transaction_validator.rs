// File: icn_blockchain/src/transaction_validator.rs

use crate::{Transaction, Blockchain};
use icn_common::error::{IcnError, IcnResult};
use chrono::Utc;
use ed25519_dalek::{PublicKey, Signature, Verifier};

/// Trait for validating transactions within a blockchain context.
pub trait TransactionValidator {
    /// Validates a transaction.
    ///
    /// # Arguments
    ///
    /// * `transaction` - The transaction to validate.
    /// * `blockchain` - The blockchain context.
    ///
    /// # Returns
    ///
    /// `IcnResult<()>` indicating whether the transaction is valid.
    fn validate_transaction(&self, transaction: &Transaction, blockchain: &Blockchain) -> IcnResult<()>;
}

/// Default implementation of the `TransactionValidator` trait.
pub struct DefaultTransactionValidator;

impl TransactionValidator for DefaultTransactionValidator {
    fn validate_transaction(&self, transaction: &Transaction, blockchain: &Blockchain) -> IcnResult<()> {
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

        if !Self::validate_timestamp(transaction) {
            return Err(IcnError::Blockchain("Transaction timestamp is not valid".to_string()));
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
        if let (Some(signature), Some(public_key_bytes)) = (&transaction.signature, &transaction.from_public_key) {
            let public_key = PublicKey::from_bytes(public_key_bytes)?;
            let signature = Signature::from_bytes(signature)?;
            public_key.verify(transaction.to_bytes().as_slice(), &signature)
                .map_err(|e| IcnError::Identity(format!("Signature verification failed: {}", e)))?;
            Ok(true)
        } else {
            Err(IcnError::Identity("Missing signature or public key".to_string()))
        }
    }

    fn validate_timestamp(transaction: &Transaction) -> bool {
        let current_time = Utc::now().timestamp();
        transaction.timestamp <= current_time && transaction.timestamp >= (current_time - 60 * 60) // within the last hour
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::currency::CurrencyType;
    use icn_common::{Transaction, Blockchain};
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
        let mut blockchain = Blockchain::new(Box::new(DefaultTransactionValidator));
        let tx = create_signed_transaction("Alice", "Bob", 50.0);

        blockchain.add_transaction(create_signed_transaction("Genesis", "Alice", 100.0)).unwrap();
        blockchain.create_block().unwrap();

        assert!(DefaultTransactionValidator::validate_transaction(&tx, &blockchain).is_ok());
    }

    #[test]
    fn test_insufficient_balance() {
        let mut blockchain = Blockchain::new(Box::new(DefaultTransactionValidator));
        let tx = create_signed_transaction("Alice", "Bob", 150.0);

        blockchain.add_transaction(create_signed_transaction("Genesis", "Alice", 100.0)).unwrap();
        blockchain.create_block().unwrap();

        assert!(DefaultTransactionValidator::validate_transaction(&tx, &blockchain).is_err());
    }

    #[test]
    fn test_double_spend() {
        let mut blockchain = Blockchain::new(Box::new(DefaultTransactionValidator));
        let tx = create_signed_transaction("Alice", "Bob", 50.0);

        blockchain.add_transaction(create_signed_transaction("Genesis", "Alice", 100.0)).unwrap();
        blockchain.create_block().unwrap();

        blockchain.add_transaction(tx.clone()).unwrap();
        blockchain.create_block().unwrap();

        assert!(DefaultTransactionValidator::validate_transaction(&tx, &blockchain).is_err());
    }
}

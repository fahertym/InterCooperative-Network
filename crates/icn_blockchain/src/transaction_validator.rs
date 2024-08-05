// File: icn_blockchain/src/asset_tokenization.rs

use icn_common::bit_utils::{BitVec, set_bit, clear_bit, toggle_bit, rotate_left, rotate_right};
use crate::{Transaction, Blockchain};
use icn_common::error::{IcnError, IcnResult};

/// A struct for validating transactions.
pub struct TransactionValidator;

impl TransactionValidator {
    /// Validates a transaction within the context of a blockchain.
    ///
    /// # Arguments
    ///
    /// * `transaction` - The transaction to validate.
    /// * `blockchain` - The blockchain context.
    ///
    /// # Returns
    ///
    /// `IcnResult<()>` indicating whether the transaction is valid.
    pub fn validate_transaction(transaction: &Transaction, blockchain: &Blockchain) -> IcnResult<()> {
        Self::is_double_spend(transaction, blockchain)?;
        Self::validate_currency_and_amount(transaction)?;
        Self::check_sufficient_balance(transaction, blockchain)?;
        Self::validate_signature(transaction)?;
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

    fn validate_currency_and_amount(transaction: &Transaction) -> IcnResult<()> {
        if transaction.amount <= 0.0 {
            return Err(IcnError::Currency("Invalid currency or amount".to_string()));
        }
        Ok(())
    }

    fn check_sufficient_balance(transaction: &Transaction, blockchain: &Blockchain) -> IcnResult<()> {
        let balance = blockchain.get_balance(&transaction.from, &transaction.currency_type)?;
        if balance < transaction.amount + transaction.get_fee() {
            return Err(IcnError::Currency("Insufficient balance".to_string()));
        }
        Ok(())
    }

    fn validate_signature(transaction: &Transaction) -> IcnResult<()> {
        if !transaction.verify().map_err(|e| IcnError::Identity(format!("Signature verification failed: {}", e)))? {
            return Err(IcnError::Identity("Invalid signature".to_string()));
        }
        Ok(())
    }

    /// Checks if a transaction can be processed by the blockchain.
    ///
    /// # Arguments
    ///
    /// * `transaction` - The transaction to check.
    /// * `blockchain` - The blockchain context.
    ///
    /// # Returns
    ///
    /// `IcnResult<()>` indicating whether the transaction can be processed.
    pub fn can_process_transaction(transaction: &Transaction, blockchain: &Blockchain) -> IcnResult<()> {
        Self::validate_transaction(transaction, blockchain)
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
        let mut blockchain = Blockchain::new(Box::new(MockTransactionValidator));
        let tx = create_signed_transaction("Alice", "Bob", 50.0);

        blockchain.add_transaction(create_signed_transaction("Genesis", "Alice", 100.0)).unwrap();
        blockchain.create_block().unwrap();

        assert!(TransactionValidator::validate_transaction(&tx, &blockchain).is_ok());
    }

    #[test]
    fn test_insufficient_balance() {
        let mut blockchain = Blockchain::new(Box::new(MockTransactionValidator));
        let tx = create_signed_transaction("Alice", "Bob", 150.0);

        blockchain.add_transaction(create_signed_transaction("Genesis", "Alice", 100.0)).unwrap();
        blockchain.create_block().unwrap();

        assert!(TransactionValidator::validate_transaction(&tx, &blockchain).is_err());
    }

    #[test]
    fn test_double_spend() {
        let mut blockchain = Blockchain::new(Box::new(MockTransactionValidator));
        let tx = create_signed_transaction("Alice", "Bob", 50.0);

        blockchain.add_transaction(create_signed_transaction("Genesis", "Alice", 100.0)).unwrap();
        blockchain.create_block().unwrap();

        blockchain.add_transaction(tx.clone()).unwrap();
        blockchain.create_block().unwrap();

        assert!(TransactionValidator::validate_transaction(&tx, &blockchain).is_err());
    }
}

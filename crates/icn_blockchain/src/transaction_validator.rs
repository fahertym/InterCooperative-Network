// File: icn_blockchain/src/transaction_validator.rs

use crate::{Transaction, Blockchain};
use icn_common::error::{IcnError, IcnResult};
use chrono::Utc;

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
        transaction.verify().map_err(|e| IcnError::Identity(format!("Signature verification failed: {}", e)))
    }

    fn validate_timestamp(transaction: &Transaction) -> bool {
        let current_time = Utc::now().timestamp();
        transaction.timestamp <= current_time && transaction.timestamp >= (current_time - 60 * 60) // within the last hour
    }
}

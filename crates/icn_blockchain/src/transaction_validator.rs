// crates/icn_blockchain/src/transaction_validator.rs
use crate::{Transaction, Blockchain};

pub struct TransactionValidator;

impl TransactionValidator {
    pub fn validate_transaction(transaction: &Transaction, blockchain: &Blockchain) -> bool {
        if !Self::is_double_spend(transaction, blockchain) &&
           Self::validate_currency_and_amount(transaction) &&
           Self::check_sufficient_balance(transaction, blockchain) {
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
        balance >= transaction.amount
    }
}
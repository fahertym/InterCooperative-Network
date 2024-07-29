use icn_common::{Block, Transaction, IcnResult, IcnError, CurrencyType};
use chrono::Utc;

pub struct Blockchain {
    pub chain: Vec<Block>,
    pending_transactions: Vec<Transaction>,
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis_block = Block {
            index: 0,
            timestamp: Utc::now().timestamp(),
            transactions: Vec::new(),
            previous_hash: String::from("0"),
            hash: String::from("genesis_hash"),
        };

        Blockchain {
            chain: vec![genesis_block],
            pending_transactions: Vec::new(),
        }
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> IcnResult<()> {
        self.pending_transactions.push(transaction);
        Ok(())
    }

    pub fn create_block(&mut self) -> IcnResult<Block> {
        let previous_block = self.chain.last()
            .ok_or_else(|| IcnError::Blockchain("Empty blockchain".into()))?;

        let mut new_block = Block {
            index: self.chain.len() as u64,
            timestamp: Utc::now().timestamp(),
            transactions: std::mem::take(&mut self.pending_transactions),
            previous_hash: previous_block.hash.clone(),
            hash: String::new(),
        };

        new_block.hash = Self::calculate_hash(&new_block);
        self.chain.push(new_block.clone());
        Ok(new_block)
    }

    fn calculate_hash(block: &Block) -> String {
        // This is a placeholder hash function. In a real implementation,
        // you would use a cryptographic hash function.
        format!("hash_of_block_{}", block.index)
    }

    pub fn validate_chain(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            if current_block.previous_hash != previous_block.hash {
                return false;
            }

            if current_block.hash != Self::calculate_hash(current_block) {
                return false;
            }
        }
        true
    }

    pub fn get_latest_block(&self) -> Option<&Block> {
        self.chain.last()
    }

    pub fn get_block_by_index(&self, index: u64) -> Option<&Block> {
        self.chain.get(index as usize)
    }

    pub fn get_block_by_hash(&self, hash: &str) -> Option<&Block> {
        self.chain.iter().find(|block| block.hash == hash)
    }

    pub fn get_pending_transactions(&self) -> &[Transaction] {
        &self.pending_transactions
    }

    pub fn clear_pending_transactions(&mut self) {
        self.pending_transactions.clear();
    }

    pub fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        let mut balance = 0.0;
        for block in &self.chain {
            for transaction in &block.transactions {
                if transaction.currency_type == *currency_type {
                    if transaction.from == address {
                        balance -= transaction.amount;
                    }
                    if transaction.to == address {
                        balance += transaction.amount;
                    }
                }
            }
        }
        Ok(balance)
    }

    pub fn get_transaction_count(&self) -> usize {
        self.chain.iter().map(|block| block.transactions.len()).sum()
    }

    pub fn get_chain(&self) -> &Vec<Block> {
        &self.chain
    }
}

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
                if tx.from == transaction.from && 
                   tx.to == transaction.to && 
                   tx.amount == transaction.amount && 
                   tx.currency_type == transaction.currency_type &&
                   tx.timestamp == transaction.timestamp {
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
        Ok(balance >= transaction.amount)
    }

    fn validate_signature(_transaction: &Transaction) -> IcnResult<bool> {
        // In a real implementation, this would verify the transaction signature
        // For now, we'll assume all transactions are valid
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockchain_creation() {
        let blockchain = Blockchain::new();
        assert_eq!(blockchain.chain.len(), 1);
        assert_eq!(blockchain.chain[0].index, 0);
    }

    #[test]
    fn test_add_block() {
        let mut blockchain = Blockchain::new();
        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 0,
            signature: None,
        };

        blockchain.add_transaction(transaction).unwrap();
        assert!(blockchain.create_block().is_ok());
        assert_eq!(blockchain.chain.len(), 2);
    }

    #[test]
    fn test_blockchain_validity() {
        let mut blockchain = Blockchain::new();
        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 0,
            signature: None,
        };

        blockchain.add_transaction(transaction).unwrap();
        blockchain.create_block().unwrap();
        assert!(blockchain.validate_chain());

        // Tamper with a block to test invalid chain
        blockchain.chain[1].transactions[0].amount = 100.0;
        assert!(!blockchain.validate_chain());
    }
}
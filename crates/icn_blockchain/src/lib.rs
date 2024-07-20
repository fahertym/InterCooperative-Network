use icn_types::{IcnResult, IcnError, Block, Transaction, CurrencyType};
use std::collections::HashMap;
use chrono::Utc;
use sha2::{Sha256, Digest};

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub balances: HashMap<String, HashMap<CurrencyType, f64>>,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut chain = Vec::new();
        chain.push(Block::genesis());
        Blockchain {
            chain,
            pending_transactions: Vec::new(),
            balances: HashMap::new(),
        }
    }

    pub fn add_block(&mut self, block: Block) -> IcnResult<()> {
        if self.is_valid_block(&block) {
            self.chain.push(block);
            Ok(())
        } else {
            Err(IcnError::Blockchain("Invalid block".to_string()))
        }
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> IcnResult<()> {
        if self.verify_transaction(&transaction) {
            self.pending_transactions.push(transaction);
            Ok(())
        } else {
            Err(IcnError::Blockchain("Invalid transaction".to_string()))
        }
    }

    pub fn create_block(&mut self, miner: String) -> IcnResult<Block> {
        let previous_block = self.chain.last().ok_or_else(|| IcnError::Blockchain("No previous block found".to_string()))?;
        
        let new_block = Block {
            index: self.chain.len() as u64,
            timestamp: Utc::now().timestamp(),
            transactions: self.pending_transactions.clone(),
            previous_hash: previous_block.hash.clone(),
            hash: String::new(), // Will be set in calculate_hash
        };

        let new_block = self.calculate_hash(new_block);
        self.pending_transactions.clear();
        Ok(new_block)
    }

    pub fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> f64 {
        self.balances
            .get(address)
            .and_then(|balances| balances.get(currency_type))
            .cloned()
            .unwrap_or(0.0)
    }

    pub fn update_balance(&mut self, address: &str, currency_type: &CurrencyType, amount: f64) -> IcnResult<()> {
        let balance = self.balances
            .entry(address.to_string())
            .or_insert_with(HashMap::new)
            .entry(currency_type.clone())
            .or_insert(0.0);
        *balance += amount;
        if *balance < 0.0 {
            return Err(IcnError::Blockchain("Insufficient balance".to_string()));
        }
        Ok(())
    }

    fn is_valid_block(&self, block: &Block) -> bool {
        if block.index != self.chain.len() as u64 {
            return false;
        }
        if let Some(last_block) = self.chain.last() {
            if block.previous_hash != last_block.hash {
                return false;
            }
        } else if block.index != 0 {
            return false;
        }

        let calculated_hash = self.calculate_hash(block.clone()).hash;
        calculated_hash == block.hash
    }

    fn calculate_hash(&self, mut block: Block) -> Block {
        let mut hasher = Sha256::new();
        hasher.update(block.index.to_string());
        hasher.update(block.timestamp.to_string());
        for transaction in &block.transactions {
            hasher.update(transaction.from.as_bytes());
            hasher.update(transaction.to.as_bytes());
            hasher.update(transaction.amount.to_string());
            hasher.update(format!("{:?}", transaction.currency_type));
        }
        hasher.update(&block.previous_hash);
        
        let hash = format!("{:x}", hasher.finalize());
        block.hash = hash;
        block
    }

    fn verify_transaction(&self, transaction: &Transaction) -> bool {
        // Verify sender has sufficient balance
        let sender_balance = self.get_balance(&transaction.from, &transaction.currency_type);
        if sender_balance < transaction.amount {
            return false;
        }

        // Verify transaction signature
        // Note: This is a placeholder. In a real implementation, you would use cryptographic verification.
        transaction.signature.is_some()
    }
}

impl Block {
    pub fn genesis() -> Self {
        Block {
            index: 0,
            timestamp: Utc::now().timestamp(),
            transactions: Vec::new(),
            previous_hash: String::from("0"),
            hash: String::from("genesis_hash"),
        }
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
    fn test_add_transaction_and_create_block() {
        let mut blockchain = Blockchain::new();
        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 100.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: Some(vec![]),
        };

        assert!(blockchain.add_transaction(transaction).is_ok());
        assert_eq!(blockchain.pending_transactions.len(), 1);

        let new_block = blockchain.create_block("Miner1".to_string()).unwrap();
        assert_eq!(new_block.index, 1);
        assert_eq!(new_block.transactions.len(), 1);

        assert!(blockchain.add_block(new_block).is_ok());
        assert_eq!(blockchain.chain.len(), 2);
        assert!(blockchain.pending_transactions.is_empty());
    }

    #[test]
    fn test_get_and_update_balance() {
        let mut blockchain = Blockchain::new();
        let address = "Alice";
        let currency_type = CurrencyType::BasicNeeds;

        assert_eq!(blockchain.get_balance(address, &currency_type), 0.0);

        assert!(blockchain.update_balance(address, &currency_type, 100.0).is_ok());
        assert_eq!(blockchain.get_balance(address, &currency_type), 100.0);

        assert!(blockchain.update_balance(address, &currency_type, -50.0).is_ok());
        assert_eq!(blockchain.get_balance(address, &currency_type), 50.0);

        assert!(blockchain.update_balance(address, &currency_type, -100.0).is_err());
    }
}
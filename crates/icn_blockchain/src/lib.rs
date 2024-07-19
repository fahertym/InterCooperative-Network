mod block;
mod transaction;
mod transaction_validator;
mod asset_tokenization;

pub use block::Block;
pub use transaction::Transaction;
pub use transaction_validator::TransactionValidator;
pub use asset_tokenization::{AssetToken, AssetRegistry};

use icn_common::{Error, Result, CurrencyType};
use icn_consensus::PoCConsensus;

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub asset_tokens: HashMap<String, CurrencyType>,
    pub bonds: HashMap<String, CurrencyType>,
    pub consensus: PoCConsensus,
    pub asset_registry: AssetRegistry,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut blockchain = Blockchain {
            chain: vec![],
            pending_transactions: vec![],
            asset_tokens: HashMap::new(),
            bonds: HashMap::new(),
            consensus: PoCConsensus::new(0.5, 0.66),
            asset_registry: AssetRegistry::new(),
        };
        
        let genesis_block = Block::new(0, vec![], String::new());
        blockchain.chain.push(genesis_block);
        
        blockchain
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<()> {
        if TransactionValidator::validate_transaction(&transaction, self) {
            self.pending_transactions.push(transaction);
            Ok(())
        } else {
            Err(Error {
                message: "Invalid transaction".to_string(),
            })
        }
    }

    pub fn create_block(&mut self, author: String) -> Result<()> {
        let previous_block = self.chain.last().ok_or_else(|| Error {
            message: "No previous block found".to_string(),
        })?;
        let new_block = Block::new(
            self.chain.len() as u64,
            self.pending_transactions.clone(),
            previous_block.hash.clone(),
        );
        
        self.chain.push(new_block);
        self.pending_transactions.clear();
        Ok(())
    }

    pub fn get_latest_block(&self) -> Option<&Block> {
        self.chain.last()
    }

    pub fn get_balance(&self, address: &str) -> f64 {
        let mut balance = 0.0;
        for block in &self.chain {
            for transaction in &block.transactions {
                if transaction.from == address {
                    balance -= transaction.amount;
                }
                if transaction.to == address {
                    balance += transaction.amount;
                }
            }
        }
        balance
    }

    pub fn validate_chain(&self) -> Result<()> {
        for i in 1..self.chain.len() {
            let previous_block = &self.chain[i - 1];
            let current_block = &self.chain[i];

            if current_block.previous_hash != previous_block.hash {
                return Err(Error {
                    message: "Invalid previous hash".to_string(),
                });
            }

            if current_block.hash != current_block.calculate_hash() {
                return Err(Error {
                    message: "Invalid block hash".to_string(),
                });
            }
        }
        Ok(())
    }

    pub fn get_asset_token(&self, asset_id: &str) -> Option<&CurrencyType> {
        self.asset_tokens.get(asset_id)
    }

    pub fn get_bond(&self, bond_id: &str) -> Option<&CurrencyType> {
        self.bonds.get(bond_id)
    }

    pub fn add_asset_token(&mut self, asset_id: String, currency_type: CurrencyType) -> Result<()> {
        if self.asset_tokens.contains_key(&asset_id) {
            return Err(Error {
                message: "Asset token already exists".to_string(),
            });
        }
        self.asset_tokens.insert(asset_id, currency_type);
        Ok(())
    }

    pub fn add_bond(&mut self, bond_id: String, currency_type: CurrencyType) -> Result<()> {
        if self.bonds.contains_key(&bond_id) {
            return Err(Error {
                message: "Bond already exists".to_string(),
            });
        }
        self.bonds.insert(bond_id, currency_type);
        Ok(())
    }

    pub fn execute_smart_contracts(&mut self) -> Result<()> {
        // Implement smart contract execution logic here
        Ok(())
    }

    pub fn create_asset_token(&mut self, name: String, description: String, owner: String, metadata: serde_json::Value) -> Result<AssetToken> {
        let token = self.asset_registry.create_token(name, description, owner, metadata);
        self.asset_tokens.insert(token.id.clone(), CurrencyType::AssetToken(token.id.clone()));
        Ok(token)
    }

    pub fn transfer_asset_token(&mut self, token_id: &str, new_owner: &str) -> Result<()> {
        self.asset_registry.transfer_token(token_id, new_owner.to_string())?;
        Ok(())
    }

    pub fn get_asset_token_details(&self, token_id: &str) -> Option<&AssetToken> {
        self.asset_registry.get_token(token_id)
    }

    pub fn transfer_bond(&mut self, bond_id: &str, new_owner: &str) -> Result<()> {
        // Implement bond transfer logic here
        Ok(())
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
        let transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1000,
        );

        assert!(blockchain.add_transaction(transaction).is_ok());
        assert_eq!(blockchain.pending_transactions.len(), 1);

        assert!(blockchain.create_block("Miner1".to_string()).is_ok());
        assert_eq!(blockchain.chain.len(), 2);
        assert!(blockchain.pending_transactions.is_empty());
    }

    #[test]
    fn test_get_balance() {
        let mut blockchain = Blockchain::new();
        let transaction1 = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1000,
        );
        let transaction2 = Transaction::new(
            "Bob".to_string(),
            "Alice".to_string(),
            50.0,
            CurrencyType::BasicNeeds,
            1000,
        );

        blockchain.add_transaction(transaction1).unwrap();
        blockchain.add_transaction(transaction2).unwrap();
        blockchain.create_block("Miner1".to_string()).unwrap();

        assert_eq!(blockchain.get_balance("Alice"), -50.0);
        assert_eq!(blockchain.get_balance("Bob"), 50.0);
    }

    #[test]
    fn test_validate_chain() {
        let mut blockchain = Blockchain::new();
        let transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1000,
        );

        blockchain.add_transaction(transaction).unwrap();
        blockchain.create_block("Miner1".to_string()).unwrap();

        assert!(blockchain.validate_chain().is_ok());

        // Tamper with a block
        blockchain.chain[1].hash = "tampered_hash".to_string();
        assert!(blockchain.validate_chain().is_err());
    }

    #[test]
    fn test_asset_tokens_and_bonds() {
        let mut blockchain = Blockchain::new();
        
        assert!(blockchain.add_asset_token("ASSET1".to_string(), CurrencyType::AssetToken("ASSET1".to_string())).is_ok());
        assert!(blockchain.add_bond("BOND1".to_string(), CurrencyType::Bond("BOND1".to_string())).is_ok());

        assert!(blockchain.get_asset_token("ASSET1").is_some());
        assert!(blockchain.get_bond("BOND1").is_some());
        assert!(blockchain.get_asset_token("NONEXISTENT").is_none());
        assert!(blockchain.get_bond("NONEXISTENT").is_none());
    }

    #[test]
    fn test_create_and_transfer_asset_token() {
        let mut blockchain = Blockchain::new();
        
        let metadata = serde_json::json!({
            "type": "Real Estate",
            "location": "123 Main St, Anytown, USA",
            "square_feet": 2000
        });

        let token = blockchain.create_asset_token(
            "Main Street Property".to_string(),
            "A beautiful property on Main Street".to_string(),
            "Alice".to_string(),
            metadata
        ).unwrap();

        assert_eq!(token.owner, "Alice");
        
        blockchain.transfer_asset_token(&token.id, "Bob").unwrap();
        
        let updated_token = blockchain.get_asset_token_details(&token.id).unwrap();
        assert_eq!(updated_token.owner, "Bob");
    }

    #[test]
    fn test_double_spend_prevention() {
        let mut blockchain = Blockchain::new();
        let transaction1 = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1000,
        );

        assert!(blockchain.add_transaction(transaction1.clone()).is_ok());
        blockchain.create_block("Miner1".to_string()).unwrap();

        // Attempt to add the same transaction again
        assert!(blockchain.add_transaction(transaction1).is_err());
    }

    #[test]
    fn test_insufficient_balance_prevention() {
        let mut blockchain = Blockchain::new();
        let transaction1 = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1000,
        );

        blockchain.add_transaction(transaction1).unwrap();
        blockchain.create_block("Miner1".to_string()).unwrap();

        // Attempt to spend more than Alice's balance
        let transaction2 = Transaction::new(
            "Alice".to_string(),
            "Charlie".to_string(),
            200.0,
            CurrencyType::BasicNeeds,
            1000,
        );

        assert!(blockchain.add_transaction(transaction2).is_err());
    }
}

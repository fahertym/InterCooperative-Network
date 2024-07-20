use icn_utils::error::{IcnError, IcnResult};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use sha2::{Sha256, Digest};

mod asset_tokenization;
mod transaction;
mod block;

pub use self::asset_tokenization::{AssetToken, AssetRegistry};
pub use self::transaction::Transaction;
pub use self::block::Block;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum CurrencyType {
    BasicNeeds,
    Education,
    Environmental,
    Community,
    Volunteer,
    Storage,
    Processing,
    Energy,
    Luxury,
    Service,
    Custom(String),
    AssetToken(String),
    Bond(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub asset_registry: AssetRegistry,
    pub currency_balances: HashMap<String, HashMap<CurrencyType, f64>>,
}

impl Blockchain {
    pub fn new() -> Self {
        Blockchain {
            chain: vec![Block::genesis()],
            pending_transactions: Vec::new(),
            asset_registry: AssetRegistry::new(),
            currency_balances: HashMap::new(),
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
        let new_block = Block::new(
            self.chain.len() as u64,
            self.pending_transactions.clone(),
            previous_block.hash.clone(),
            miner,
        );
        self.pending_transactions.clear();
        Ok(new_block)
    }

    pub fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> f64 {
        self.currency_balances
            .get(address)
            .and_then(|balances| balances.get(currency_type))
            .cloned()
            .unwrap_or(0.0)
    }

    pub fn update_balance(&mut self, address: &str, currency_type: &CurrencyType, amount: f64) -> IcnResult<()> {
        let balance = self.currency_balances
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

    pub fn create_asset_token(&mut self, name: String, description: String, owner: String) -> IcnResult<AssetToken> {
        self.asset_registry.create_token(name, description, owner, serde_json::json!({}))
            .map_err(|e| IcnError::Blockchain(e.to_string()))
    }

    pub fn transfer_asset_token(&mut self, token_id: &str, new_owner: &str) -> IcnResult<()> {
        self.asset_registry.transfer_token(token_id, new_owner.to_string())
            .map_err(|e| IcnError::Blockchain(e.to_string()))
    }

    fn is_valid_block(&self, block: &Block) -> bool {
        if block.index != self.chain.len() as u64 {
            return false;
        }
        if let Some(last_block) = self.chain.last() {
            if block.previous_hash != last_block.hash {
                return false;
            }
        }
        block.hash == block.calculate_hash()
    }

    fn verify_transaction(&self, transaction: &Transaction) -> bool {
        // Verify sender has sufficient balance
        let sender_balance = self.get_balance(&transaction.from, &transaction.currency_type);
        if sender_balance < transaction.amount {
            return false;
        }

        // Verify transaction signature
        transaction.verify()
    }

    pub fn get_latest_block(&self) -> Option<&Block> {
        self.chain.last()
    }

    pub fn get_block_by_index(&self, index: u64) -> Option<&Block> {
        self.chain.get(index as usize)
    }

    pub fn get_transaction_history(&self, address: &str) -> Vec<&Transaction> {
        self.chain.iter().flat_map(|block| {
            block.transactions.iter().filter(|tx| tx.from == address || tx.to == address)
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Keypair, Signer};
    use rand::rngs::OsRng;

    fn create_signed_transaction(from: &str, to: &str, amount: f64, currency_type: CurrencyType) -> Transaction {
        let mut csprng = OsRng{};
        let keypair: Keypair = Keypair::generate(&mut csprng);
        let mut transaction = Transaction::new(from.to_string(), to.to_string(), amount, currency_type);
        transaction.sign(&keypair).unwrap();
        transaction
    }

    #[test]
    fn test_blockchain_creation() {
        let blockchain = Blockchain::new();
        assert_eq!(blockchain.chain.len(), 1);
        assert_eq!(blockchain.chain[0].index, 0);
        assert!(blockchain.pending_transactions.is_empty());
    }

    #[test]
    fn test_add_transaction() {
        let mut blockchain = Blockchain::new();
        let transaction = create_signed_transaction("Alice", "Bob", 100.0, CurrencyType::BasicNeeds);
        blockchain.update_balance("Alice", &CurrencyType::BasicNeeds, 1000.0).unwrap();
        assert!(blockchain.add_transaction(transaction).is_ok());
        assert_eq!(blockchain.pending_transactions.len(), 1);
    }

    #[test]
    fn test_create_and_add_block() {
        let mut blockchain = Blockchain::new();
        let transaction = create_signed_transaction("Alice", "Bob", 100.0, CurrencyType::BasicNeeds);
        blockchain.update_balance("Alice", &CurrencyType::BasicNeeds, 1000.0).unwrap();
        blockchain.add_transaction(transaction).unwrap();
        let new_block = blockchain.create_block("Miner1".to_string()).unwrap();
        assert!(blockchain.add_block(new_block).is_ok());
        assert_eq!(blockchain.chain.len(), 2);
        assert!(blockchain.pending_transactions.is_empty());
    }

    #[test]
    fn test_get_and_update_balance() {
        let mut blockchain = Blockchain::new();
        assert_eq!(blockchain.get_balance("Alice", &CurrencyType::BasicNeeds), 0.0);
        
        assert!(blockchain.update_balance("Alice", &CurrencyType::BasicNeeds, 100.0).is_ok());
        assert_eq!(blockchain.get_balance("Alice", &CurrencyType::BasicNeeds), 100.0);
        
        assert!(blockchain.update_balance("Alice", &CurrencyType::BasicNeeds, -50.0).is_ok());
        assert_eq!(blockchain.get_balance("Alice", &CurrencyType::BasicNeeds), 50.0);
        
        assert!(blockchain.update_balance("Alice", &CurrencyType::BasicNeeds, -100.0).is_err());
    }

    #[test]
    fn test_create_and_transfer_asset_token() {
        let mut blockchain = Blockchain::new();
        let token = blockchain.create_asset_token("Token1".to_string(), "Test Token".to_string(), "Alice".to_string()).unwrap();
        assert_eq!(token.owner, "Alice");
        
        assert!(blockchain.transfer_asset_token(&token.id, "Bob").is_ok());
        let updated_token = blockchain.asset_registry.get_token(&token.id).unwrap();
        assert_eq!(updated_token.owner, "Bob");
        
        assert!(blockchain.transfer_asset_token("non_existent_token", "Charlie").is_err());
    }

    #[test]
    fn test_invalid_block() {
        let mut blockchain = Blockchain::new();
        let invalid_block = Block::new(
            999, // Invalid index
            vec![],
            "invalid_previous_hash".to_string(),
            "Miner1".to_string(),
        );
        assert!(blockchain.add_block(invalid_block).is_err());
    }

    #[test]
    fn test_multiple_currency_types() {
        let mut blockchain = Blockchain::new();
        assert!(blockchain.update_balance("Alice", &CurrencyType::BasicNeeds, 100.0).is_ok());
        assert!(blockchain.update_balance("Alice", &CurrencyType::Education, 50.0).is_ok());
        
        assert_eq!(blockchain.get_balance("Alice", &CurrencyType::BasicNeeds), 100.0);
        assert_eq!(blockchain.get_balance("Alice", &CurrencyType::Education), 50.0);
        assert_eq!(blockchain.get_balance("Alice", &CurrencyType::Environmental), 0.0);
    }

    #[test]
    fn test_transaction_verification() {
        let mut blockchain = Blockchain::new();
        blockchain.update_balance("Alice", &CurrencyType::BasicNeeds, 1000.0).unwrap();
        
        let valid_transaction = create_signed_transaction("Alice", "Bob", 100.0, CurrencyType::BasicNeeds);
        assert!(blockchain.add_transaction(valid_transaction).is_ok());

        let invalid_transaction = create_signed_transaction("Alice", "Bob", 2000.0, CurrencyType::BasicNeeds);
        assert!(blockchain.add_transaction(invalid_transaction).is_err());
    }

    #[test]
    fn test_get_latest_block() {
        let mut blockchain = Blockchain::new();
        let new_block = blockchain.create_block("Miner1".to_string()).unwrap();
        blockchain.add_block(new_block).unwrap();

        let latest_block = blockchain.get_latest_block().unwrap();
        assert_eq!(latest_block.index, 1);
    }

    #[test]
    fn test_get_block_by_index() {
        let mut blockchain = Blockchain::new();
        let new_block = blockchain.create_block("Miner1".to_string()).unwrap();
        blockchain.add_block(new_block).unwrap();

        let genesis_block = blockchain.get_block_by_index(0).unwrap();
        assert_eq!(genesis_block.index, 0);

        let second_block = blockchain.get_block_by_index(1).unwrap();
        assert_eq!(second_block.index, 1);

        assert!(blockchain.get_block_by_index(2).is_none());
    }

    #[test]
    fn test_get_transaction_history() {
        let mut blockchain = Blockchain::new();
        blockchain.update_balance("Alice", &CurrencyType::BasicNeeds, 1000.0).unwrap();
        
        let transaction1 = create_signed_transaction("Alice", "Bob", 100.0, CurrencyType::BasicNeeds);
        let transaction2 = create_signed_transaction("Bob", "Alice", 50.0, CurrencyType::BasicNeeds);
        
        blockchain.add_transaction(transaction1).unwrap();
        blockchain.add_transaction(transaction2).unwrap();
        
        let new_block = blockchain.create_block("Miner1".to_string()).unwrap();
        blockchain.add_block(new_block).unwrap();

        let alice_history = blockchain.get_transaction_history("Alice");
        assert_eq!(alice_history.len(), 2);

        let charlie_history = blockchain.get_transaction_history("Charlie");
        assert!(charlie_history.is_empty());
    }
}

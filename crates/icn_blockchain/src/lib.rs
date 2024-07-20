//! InterCooperative Network Blockchain Module
//!
//! This module implements the core blockchain functionality for the InterCooperative Network.
//! It includes structures and methods for managing blocks, transactions, and the overall
//! blockchain state, as well as basic smart contract support.

use icn_common::{CommonError, CommonResult, Block, Transaction, CurrencyType};
use icn_utils::{hash_data, calculate_merkle_root};
use std::collections::HashMap;
use chrono::Utc;
use ed25519_dalek::PublicKey;
use bincode;
use sha2::Sha256;
use hex;



/// Represents the entire blockchain and its current state.
pub struct Blockchain {
    /// The chain of blocks, starting with the genesis block.
    pub chain: Vec<Block>,
    /// Transactions that have not yet been included in a block.
    pub pending_transactions: Vec<Transaction>,
    /// Current balance state for all accounts across all currency types.
    pub balances: HashMap<String, HashMap<CurrencyType, f64>>,
    /// Smart contracts deployed on the blockchain.
    pub contracts: HashMap<String, SmartContract>,
}

impl Blockchain {
    /// Creates a new blockchain with a genesis block.
    pub fn new() -> Self {
        let mut chain = Vec::new();
        chain.push(Block::genesis());
        Blockchain {
            chain,
            pending_transactions: Vec::new(),
            balances: HashMap::new(),
            contracts: HashMap::new(),
        }
    }

    /// Adds a new block to the blockchain.
    ///
    /// This method validates the block before adding it to the chain.
    ///
    /// # Arguments
    ///
    /// * `block` - The block to be added to the blockchain.
    ///
    /// # Returns
    ///
    /// A `CommonResult` indicating success or containing an error if the block is invalid.
    pub fn add_block(&mut self, block: Block) -> CommonResult<()> {
        if self.is_valid_block(&block) {
            // Process all transactions in the block
            for transaction in &block.transactions {
                self.process_transaction(transaction)?;
            }
            // Add the block to the chain
            self.chain.push(block);
            // Clear pending transactions that are now in the block
            self.pending_transactions.clear();
            Ok(())
        } else {
            Err(CommonError::Blockchain("Invalid block".to_string()))
        }
    }

    /// Adds a new transaction to the pending transactions pool.
    ///
    /// # Arguments
    ///
    /// * `transaction` - The transaction to be added to the pending pool.
    ///
    /// # Returns
    ///
    /// A `CommonResult` indicating success or containing an error if the transaction is invalid.
    pub fn add_transaction(&mut self, transaction: Transaction) -> CommonResult<()> {
        if self.is_valid_transaction(&transaction) {
            self.pending_transactions.push(transaction);
            Ok(())
        } else {
            Err(CommonError::Blockchain("Invalid transaction".to_string()))
        }
    }

    /// Creates a new block with the current pending transactions.
    ///
    /// # Arguments
    ///
    /// * `miner` - The address of the miner creating this block.
    ///
    /// # Returns
    ///
    /// A `CommonResult` containing the newly created block or an error if block creation fails.
    pub fn create_block(&self, miner: String) -> CommonResult<Block> {
        let previous_block = self.chain.last()
            .ok_or_else(|| CommonError::Blockchain("No previous block found".to_string()))?;
        
        let new_block = Block {
            index: self.chain.len() as u64,
            timestamp: Utc::now().timestamp(),
            transactions: self.pending_transactions.clone(),
            previous_hash: previous_block.hash.clone(),
            hash: String::new(), // Will be set in calculate_block_hash
            merkle_root: self.calculate_merkle_root(&self.pending_transactions),
            difficulty: self.adjust_difficulty(),
            nonce: 0, // Should be set by the miner
        };

        let new_block = self.calculate_block_hash(new_block);
        Ok(new_block)
    }

    /// Retrieves the current balance for a given address and currency type.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to check the balance for.
    /// * `currency_type` - The type of currency to check the balance of.
    ///
    /// # Returns
    ///
    /// The current balance as a `f64`.
    pub fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> f64 {
        self.balances
            .get(address)
            .and_then(|balances| balances.get(currency_type))
            .cloned()
            .unwrap_or(0.0)
    }

    /// Updates the balance for a given address and currency type.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to update the balance for.
    /// * `currency_type` - The type of currency to update.
    /// * `amount` - The amount to add (or subtract if negative) from the balance.
    ///
    /// # Returns
    ///
    /// A `CommonResult` indicating success or containing an error if the update fails.
    pub fn update_balance(&mut self, address: &str, currency_type: &CurrencyType, amount: f64) -> CommonResult<()> {
        let balance = self.balances
            .entry(address.to_string())
            .or_insert_with(HashMap::new)
            .entry(currency_type.clone())
            .or_insert(0.0);
        *balance += amount;
        if *balance < 0.0 {
            return Err(CommonError::Blockchain("Insufficient balance".to_string()));
        }
        Ok(())
    }

    /// Processes a transaction, updating relevant balances.
    ///
    /// # Arguments
    ///
    /// * `transaction` - The transaction to process.
    ///
    /// # Returns
    ///
    /// A `CommonResult` indicating success or containing an error if processing fails.
    fn process_transaction(&mut self, transaction: &Transaction) -> CommonResult<()> {
        self.update_balance(&transaction.from, &transaction.currency_type, -transaction.amount)?;
        self.update_balance(&transaction.to, &transaction.currency_type, transaction.amount)?;
        Ok(())
    }

    /// Validates a block before adding it to the chain.
    ///
    /// # Arguments
    ///
    /// * `block` - The block to validate.
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the block is valid.
    fn is_valid_block(&self, block: &Block) -> bool {
        // Check if the block index is correct
        if block.index != self.chain.len() as u64 {
            return false;
        }

        // Check if the previous hash matches the hash of the last block in the chain
        if let Some(last_block) = self.chain.last() {
            if block.previous_hash != last_block.hash {
                return false;
            }
        } else if block.index != 0 {
            return false;
        }

        // Validate the block's hash
        let calculated_hash = self.calculate_block_hash(block.clone()).hash;
        if calculated_hash != block.hash {
            return false;
        }

        // Validate the merkle root
        let calculated_merkle_root = self.calculate_merkle_root(&block.transactions);
        if calculated_merkle_root != block.merkle_root {
            return false;
        }

        // Validate proof of work
        if !self.is_valid_proof_of_work(&block.hash, block.difficulty) {
            return false;
        }

        // Validate all transactions in the block
        block.transactions.iter().all(|tx| self.is_valid_transaction(tx))
    }

    /// Validates a transaction before adding it to the pending transactions.
    ///
    /// # Arguments
    ///
    /// * `transaction` - The transaction to validate.
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the transaction is valid.
    fn is_valid_transaction(&self, transaction: &Transaction) -> bool {
        // Check if the sender has sufficient balance
        let sender_balance = self.get_balance(&transaction.from, &transaction.currency_type);
        if sender_balance < transaction.amount {
            return false;
        }

        // Verify the transaction signature
        self.verify_transaction_signature(transaction)
    }

    /// Calculates the hash of a block.
    ///
    /// # Arguments
    ///
    /// * `block` - The block to calculate the hash for.
    ///
    /// # Returns
    ///
    /// A new `Block` with the calculated hash set.
    fn calculate_block_hash(&self, mut block: Block) -> Block {
        let mut hasher = sha2::Sha256::new();
        hasher.update(block.index.to_string().as_bytes());
        hasher.update(block.timestamp.to_string().as_bytes());
        hasher.update(&block.merkle_root);
        hasher.update(block.previous_hash.as_bytes());
        hasher.update(block.difficulty.to_string().as_bytes());
        hasher.update(block.nonce.to_string().as_bytes());
        
        let hash = format!("{:x}", hasher.finalize());
        block.hash = hash;
        block
    }

    /// Calculates and returns the merkle root of the transactions in a block.
    ///
    /// # Arguments
    ///
    /// * `transactions` - A slice of transactions to calculate the merkle root for.
    ///
    /// # Returns
    ///
    /// A vector of bytes representing the merkle root.
    pub fn calculate_merkle_root(&self, transactions: &[Transaction]) -> Vec<u8> {
        let transaction_hashes: Vec<Vec<u8>> = transactions
            .iter()
            .map(|tx| hash_data(&bincode::serialize(tx).unwrap()))
            .collect();
        calculate_merkle_root(&transaction_hashes)
    }

    /// Adjusts the mining difficulty based on the time taken to mine recent blocks.
    ///
    /// # Returns
    ///
    /// The new difficulty as a `u64`.
    pub fn adjust_difficulty(&self) -> u64 {
        let current_difficulty = self.get_current_difficulty();
        let target_block_time = 600; // 10 minutes in seconds
        let actual_block_time = self.get_average_block_time();

        if actual_block_time < target_block_time {
            current_difficulty + 1
        } else if actual_block_time > target_block_time && current_difficulty > 1 {
            current_difficulty - 1
        } else {
            current_difficulty
        }
    }

    /// Gets the current mining difficulty.
    ///
    /// # Returns
    ///
    /// The current difficulty as a `u64`.
    fn get_current_difficulty(&self) -> u64 {
        self.chain.last().map_or(1, |block| block.difficulty)
    }

    /// Calculates the average time between recent blocks.
    ///
    /// # Returns
    ///
    /// The average block time in seconds as a `f64`.
    fn get_average_block_time(&self) -> f64 {
        if self.chain.len() < 2 {
            return 600.0; // Default to target time if not enough blocks
        }

        let num_blocks_to_consider = std::cmp::min(10, self.chain.len() - 1);
        let recent_blocks = &self.chain[self.chain.len() - num_blocks_to_consider - 1..];

        let total_time: i64 = recent_blocks.windows(2)
            .map(|pair| pair[1].timestamp - pair[0].timestamp)
            .sum();

        total_time as f64 / num_blocks_to_consider as f64
    }

    /// Verifies the signature of a transaction.
    ///
    /// # Arguments
    ///
    /// * `transaction` - The transaction to verify.
    ///
    /// # Returns
    ///
    /// A `bool` indicating whether the signature is valid.
    pub fn verify_transaction_signature(&self, transaction: &Transaction) -> bool {
        if let Some(signature) = &transaction.signature {
            let public_key = self.get_public_key(&transaction.from);
            let message = self.create_transaction_message(transaction);
            crypto::verify(&public_key, &message, signature)
        } else {
            false
        }
    }

    /// Creates a message to be signed for a transaction.
    ///
    /// # Arguments
    ///
    /// * `transaction` - The transaction to create a message for.
    ///
    /// # Returns
    ///
    /// A vector of bytes representing the message to be signed.
    fn create_transaction_message(&self, transaction: &Transaction) -> Vec<u8> {
        let mut message = Vec::new();
        message.extend_from_slice(transaction.from.as_bytes());
        message.extend_from_slice(transaction.to.as_bytes());
        message.extend_from_slice(&transaction.amount.to_le_bytes());
        message.extend_from_slice(&bincode::serialize(&transaction.currency_type).unwrap());
        message.extend_from_slice(&transaction.timestamp.to_le_bytes());
        message
    }

    /// Gets the public key for a given address.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to get the public key for.
    ///
    /// # Returns
    ///
    /// The public key as an `ed25519_dalek::PublicKey`.
    fn get_public_key(&self, address: &str) -> PublicKey {
        // In a real implementation, you would fetch this from a key storage system
        // For now, we'll create a dummy public key
        let dummy_bytes = [0u8; 32];
        PublicKey::from_bytes(&dummy_bytes).unwrap()
    }

    /// Checks if the proof of work for a block is valid.
    ///
    /// # Arguments
    ///
    /// * `hash` - The hash to check.
    /// * `difficulty` - The current mining difficulty.
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the proof of work is valid.
    fn is_valid_proof_of_work(&self, hash: &str, difficulty: u64) -> bool {
        let prefix = "0".repeat(difficulty as usize);
        hash.starts_with(&prefix)
    }

    /// Deploys a new smart contract to the blockchain.
    ///
    /// # Arguments
    ///
    /// * `code` - The bytecode of the smart contract.
    ///
    /// # Returns
    ///
    /// A `CommonResult` containing the address of the deployed contract,
    /// or an error if deployment fails.
    pub fn deploy_contract(&mut self, code: Vec<u8>) -> CommonResult<String> {
        let address = format!("contract_{}", self.chain.len());
        let contract = SmartContract::new(address.clone(), code);
        self.contracts.insert(address.clone(), contract);
        Ok(address)
    }

    /// Calls a smart contract function.
    ///
    /// # Arguments
    ///
    /// * `contract_address` - The address of the contract to call.
    /// * `input` - The input data for the contract call.
    ///
    /// # Returns
    ///
    /// A `CommonResult` containing the output of the contract call,
    /// or an error if the call fails.
    pub fn call_contract(&mut self, contract_address: &str, input: &[u8]) -> CommonResult<Vec<u8>> {
        let contract = self.contracts.get_mut(contract_address)
            .ok_or_else(|| CommonError::Blockchain(format!("Contract not found: {}", contract_address)))?;
        contract.execute(input)
    }

    /// Gets the latest block in the blockchain.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the latest `Block`, or `None` if the chain is empty.
    pub fn get_latest_block(&self) -> Option<&Block> {
        self.chain.last()
    }

    /// Gets the current height of the blockchain.
    ///
    /// # Returns
    ///
    /// The number of blocks in the chain as a `u64`.
    pub fn get_height(&self) -> u64 {
        self.chain.len() as u64
    }

    /// Validates the entire blockchain.
    ///
    /// # Returns
    ///
    /// A `bool` indicating whether the entire blockchain is valid.
    pub fn is_chain_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            if !self.is_valid_block(current_block) {
                return false;
            }

            if current_block.previous_hash != previous_block.hash {
                return false;
            }
        }
        true
    }

    /// Gets all transactions for a specific address.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to get transactions for.
    ///
    /// # Returns
    ///
    /// A vector of references to `Transaction`s involving the given address.
    pub fn get_transactions_for_address(&self, address: &str) -> Vec<&Transaction> {
        self.chain
            .iter()
            .flat_map(|block| block.transactions.iter())
            .filter(|tx| tx.from == address || tx.to == address)
            .collect()
    }

    /// Gets the total number of transactions in the blockchain.
    ///
    /// # Returns
    ///
    /// The total number of transactions as a `usize`.
    pub fn get_total_transactions(&self) -> usize {
        self.chain
            .iter()
            .map(|block| block.transactions.len())
            .sum()
    }
}

impl Block {
    /// Creates a genesis block (the first block in the blockchain).
    ///
    /// # Returns
    ///
    /// A new `Block` instance representing the genesis block.
    pub fn genesis() -> Self {
        let mut block = Block {
            index: 0,
            timestamp: Utc::now().timestamp(),
            transactions: Vec::new(),
            previous_hash: String::from("0"),
            hash: String::new(),
            merkle_root: Vec::new(),
            difficulty: 1,
            nonce: 0,
        };
        block.hash = hex::encode(hash_data(block.timestamp.to_string().as_bytes()));
        block
    }
}

/// Represents a smart contract in the blockchain.
pub struct SmartContract {
    pub address: String,
    pub code: Vec<u8>,
    pub state: HashMap<String, Vec<u8>>,
}

impl SmartContract {
    pub fn new(address: String, code: Vec<u8>) -> Self {
        SmartContract {
            address,
            code,
            state: HashMap::new(),
        }
    }

    /// Executes the smart contract with the given input.
    ///
    /// # Arguments
    ///
    /// * `input` - The input data for the contract execution.
    ///
    /// # Returns
    ///
    /// A `CommonResult` containing the output of the contract execution,
    /// or an error if the execution fails.
    pub fn execute(&mut self, input: &[u8]) -> CommonResult<Vec<u8>> {
        // This is a placeholder for contract execution logic
        // In a real implementation, this would interpret the contract code
        // and modify the contract state based on the input
        Ok(vec![])
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
            amount: 100.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: Some(vec![]),
        };

        blockchain.add_transaction(transaction).unwrap();
        let new_block = blockchain.create_block("Miner1".to_string()).unwrap();
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

    #[test]
    fn test_chain_validation() {
        let mut blockchain = Blockchain::new();
        assert!(blockchain.is_chain_valid());

        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 100.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: Some(vec![1, 2, 3]), // Dummy signature
        };

        blockchain.add_transaction(transaction).unwrap();
        let new_block = blockchain.create_block("Miner1".to_string()).unwrap();
        blockchain.add_block(new_block).unwrap();

        assert!(blockchain.is_chain_valid());

        // Tamper with a block
        blockchain.chain[1].transactions[0].amount = 200.0;
        assert!(!blockchain.is_chain_valid());
    }

    #[test]
    fn test_transaction_validation() {
        let mut blockchain = Blockchain::new();
        
        // Valid transaction
        let valid_transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: Some(vec![1, 2, 3]), // Dummy signature
        };

        // Add some balance for Alice
        blockchain.update_balance("Alice", &CurrencyType::BasicNeeds, 100.0).unwrap();

        assert!(blockchain.is_valid_transaction(&valid_transaction));

        // Invalid transaction (insufficient balance)
        let invalid_transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 150.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: Some(vec![1, 2, 3]), // Dummy signature
        };

        assert!(!blockchain.is_valid_transaction(&invalid_transaction));
    }

    #[test]
    fn test_merkle_root_calculation() {
        let blockchain = Blockchain::new();
        
        let transactions = vec![
            Transaction {
                from: "Alice".to_string(),
                to: "Bob".to_string(),
                amount: 50.0,
                currency_type: CurrencyType::BasicNeeds,
                timestamp: Utc::now().timestamp(),
                signature: Some(vec![1, 2, 3]), // Dummy signature
            },
            Transaction {
                from: "Bob".to_string(),
                to: "Charlie".to_string(),
                amount: 30.0,
                currency_type: CurrencyType::BasicNeeds,
                timestamp: Utc::now().timestamp(),
                signature: Some(vec![4, 5, 6]), // Dummy signature
            },
        ];

        let merkle_root = blockchain.calculate_merkle_root(&transactions[..]);
        assert_eq!(merkle_root, merkle_root2);
        assert_ne!(merkle_root, merkle_root3);

        // Calculate merkle root again with the same transactions
        let merkle_root2 = blockchain.calculate_merkle_root(&transactions);
        assert_eq!(merkle_root, merkle_root2);

        // Calculate merkle root with different transactions
        let transactions2 = vec![
            Transaction {
                from: "David".to_string(),
                to: "Eve".to_string(),
                amount: 20.0,
                currency_type: CurrencyType::BasicNeeds,
                timestamp: Utc::now().timestamp(),
                signature: Some(vec![7, 8, 9]), // Dummy signature
            },
        ];

        let merkle_root3 = blockchain.calculate_merkle_root(&transactions2);
        assert_ne!(merkle_root, merkle_root3);
    }

    #[test]
    fn test_difficulty_adjustment() {
        let mut blockchain = Blockchain::new();
        
        // Add some blocks with different timestamps
        for i in 1..11 {
            let mut block = Block {
                index: i,
                timestamp: Utc::now().timestamp() + i * 600, // 10 minutes apart
                transactions: vec![],
                previous_hash: "dummy_hash".to_string(),
                hash: "dummy_hash".to_string(),
                merkle_root: vec![],
                difficulty: 1,
                nonce: 0,
            };
            block.hash = blockchain.calculate_block_hash(block.clone()).hash;
            blockchain.chain.push(block);
        }

        let initial_difficulty = blockchain.get_current_difficulty();
        let new_difficulty = blockchain.adjust_difficulty();

        // The difficulty should remain the same as blocks are 10 minutes apart
        assert_eq!(initial_difficulty, new_difficulty);

        // Now let's add some blocks with shorter intervals
        for i in 11..21 {
            let mut block = Block {
                index: i,
                timestamp: Utc::now().timestamp() + i * 300, // 5 minutes apart
                transactions: vec![],
                previous_hash: "dummy_hash".to_string(),
                hash: "dummy_hash".to_string(),
                merkle_root: vec![],
                difficulty: 1,
                nonce: 0,
            };
            block.hash = blockchain.calculate_block_hash(block.clone()).hash;
            blockchain.chain.push(block);
        }

        let new_difficulty = blockchain.adjust_difficulty();

        // The difficulty should increase as blocks are being mined faster than target time
        assert!(new_difficulty > initial_difficulty);
    }

    #[test]
    fn test_smart_contract_deployment_and_execution() {
        let mut blockchain = Blockchain::new();
        
        // Deploy a simple smart contract
        let contract_code = vec![1, 2, 3, 4, 5]; // Dummy bytecode
        let contract_address = blockchain.deploy_contract(contract_code.clone()).unwrap();

        // Check if the contract was deployed successfully
        assert!(blockchain.contracts.contains_key(&contract_address));

        // Execute the smart contract
        let input = vec![6, 7, 8]; // Dummy input
        let result = blockchain.call_contract(&contract_address, &input);

        // Our placeholder implementation always returns an empty vector
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![]);
    }

    #[test]
    fn test_get_transactions_for_address() {
        let mut blockchain = Blockchain::new();
        
        let transaction1 = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: Some(vec![1, 2, 3]), // Dummy signature
        };

        let transaction2 = Transaction {
            from: "Bob".to_string(),
            to: "Alice".to_string(),
            amount: 30.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: Some(vec![4, 5, 6]), // Dummy signature
        };

        blockchain.add_transaction(transaction1.clone()).unwrap();
        blockchain.add_transaction(transaction2.clone()).unwrap();
        let new_block = blockchain.create_block("Miner1".to_string()).unwrap();
        blockchain.add_block(new_block).unwrap();

        let alice_transactions = blockchain.get_transactions_for_address("Alice");
        assert_eq!(alice_transactions.len(), 2);
        assert!(alice_transactions.contains(&&transaction1));
        assert!(alice_transactions.contains(&&transaction2));

        let bob_transactions = blockchain.get_transactions_for_address("Bob");
        assert_eq!(bob_transactions.len(), 2);
        assert!(bob_transactions.contains(&&transaction1));
        assert!(bob_transactions.contains(&&transaction2));

        let charlie_transactions = blockchain.get_transactions_for_address("Charlie");
        assert_eq!(charlie_transactions.len(), 0);
    }

    #[test]
    fn test_get_total_transactions() {
        let mut blockchain = Blockchain::new();
        
        assert_eq!(blockchain.get_total_transactions(), 0);

        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: Some(vec![1, 2, 3]), // Dummy signature
        };

        blockchain.add_transaction(transaction).unwrap();
        let new_block = blockchain.create_block("Miner1".to_string()).unwrap();
        blockchain.add_block(new_block).unwrap();

        assert_eq!(blockchain.get_total_transactions(), 1);

        // Add another block with a transaction
        let transaction2 = Transaction {
            from: "Bob".to_string(),
            to: "Charlie".to_string(),
            amount: 30.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: Some(vec![4, 5, 6]), // Dummy signature
        };

        blockchain.add_transaction(transaction2).unwrap();
        let new_block = blockchain.create_block("Miner2".to_string()).unwrap();
        blockchain.add_block(new_block).unwrap();

        assert_eq!(blockchain.get_total_transactions(), 2);
    }

    #[test]
    fn test_proof_of_work() {
        let blockchain = Blockchain::new();
        
        let valid_hash = "0000abcdefghijklmnopqrstuvwxyz";
        let invalid_hash = "abcdefghijklmnopqrstuvwxyz";
        let difficulty = 4;

        assert!(blockchain.is_valid_proof_of_work(valid_hash, difficulty));
        assert!(!blockchain.is_valid_proof_of_work(invalid_hash, difficulty));
    }

    #[test]
    fn test_block_creation_with_pending_transactions() {
        let mut blockchain = Blockchain::new();
        
        let transaction1 = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: Some(vec![1, 2, 3]), // Dummy signature
        };

        let transaction2 = Transaction {
            from: "Bob".to_string(),
            to: "Charlie".to_string(),
            amount: 30.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: Some(vec![4, 5, 6]), // Dummy signature
        };

        blockchain.add_transaction(transaction1.clone()).unwrap();
        blockchain.add_transaction(transaction2.clone()).unwrap();

        let new_block = blockchain.create_block("Miner1".to_string()).unwrap();
        
        assert_eq!(new_block.transactions.len(), 2);
        assert!(new_block.transactions.contains(&transaction1));
        assert!(new_block.transactions.contains(&transaction2));

        blockchain.add_block(new_block).unwrap();

        assert!(blockchain.pending_transactions.is_empty());
    }
}
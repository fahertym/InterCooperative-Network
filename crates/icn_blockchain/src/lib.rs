use icn_common::{Block, Transaction, IcnResult, IcnError, Hashable, CurrencyType, ZKProof};
use icn_zkp::{ZKPManager, TransactionCircuit, VoteCircuit};
use std::collections::{VecDeque, HashMap};
use chrono::Utc;
use log::{info, error, debug, warn};
use bellman::groth16;
use bls12_381::Bls12;

const MAX_TRANSACTIONS_PER_BLOCK: usize = 100;

pub struct Blockchain {
    chain: Vec<Block>,
    pending_transactions: VecDeque<Transaction>,
    zkp_manager: ZKPManager,
    utxo_set: HashMap<String, f64>,
}

impl Blockchain {
    pub fn new() -> IcnResult<Self> {
        Ok(Blockchain {
            chain: vec![Block::genesis()],
            pending_transactions: VecDeque::new(),
            zkp_manager: ZKPManager::new()?,
            utxo_set: HashMap::new(),
        })
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> IcnResult<()> {
        if self.verify_transaction(&transaction)? {
            self.pending_transactions.push_back(transaction);
            Ok(())
        } else {
            Err(IcnError::Blockchain("Invalid transaction".into()))
        }
    }

    pub fn create_confidential_transaction(&self, from: &str, to: &str, amount: f64, currency_type: CurrencyType) -> IcnResult<Transaction> {
        let mut transaction = Transaction::new(
            from.to_string(),
            to.to_string(),
            amount,
            currency_type,
            Utc::now().timestamp(),
        );

        let circuit = TransactionCircuit::new(&transaction);
        let proof = self.zkp_manager.create_proof(circuit)?;
        transaction.add_zkp(proof.to_vec(), TransactionCircuit::public_inputs(&transaction));

        Ok(transaction)
    }

    pub fn create_block(&mut self) -> IcnResult<Block> {
        let previous_block = self.chain.last()
            .ok_or_else(|| IcnError::Blockchain("No previous block found".into()))?;
        
        let transactions: Vec<Transaction> = self.pending_transactions
            .drain(..std::cmp::min(self.pending_transactions.len(), MAX_TRANSACTIONS_PER_BLOCK))
            .collect();

        let mut new_block = Block {
            index: self.chain.len() as u64,
            timestamp: Utc::now().timestamp(),
            transactions,
            previous_hash: previous_block.hash.clone(),
            hash: String::new(),
        };

        new_block.hash = new_block.hash();
        self.chain.push(new_block.clone());

        // Update UTXO set
        self.update_utxo_set(&new_block)?;

        Ok(new_block)
    }

    pub fn verify_transaction(&self, transaction: &Transaction) -> IcnResult<bool> {
        if let Some(zkp) = &transaction.zkp {
            // Verify the ZKP
            let proof = groth16::Proof::read(&zkp.proof[..])
                .map_err(|e| IcnError::ZKP(format!("Invalid proof: {}", e)))?;
            self.zkp_manager.verify_transaction_proof(&proof, transaction)
        } else {
            // If no ZKP, fall back to regular verification
            self.verify_utxo(transaction)
        }
    }

    fn verify_utxo(&self, transaction: &Transaction) -> IcnResult<bool> {
        let sender_balance = self.utxo_set.get(&transaction.from).cloned().unwrap_or(0.0);
        Ok(sender_balance >= transaction.amount)
    }

    fn update_utxo_set(&mut self, block: &Block) -> IcnResult<()> {
        for transaction in &block.transactions {
            let sender_balance = self.utxo_set.entry(transaction.from.clone()).or_insert(0.0);
            *sender_balance -= transaction.amount;

            let receiver_balance = self.utxo_set.entry(transaction.to.clone()).or_insert(0.0);
            *receiver_balance += transaction.amount;
        }
        Ok(())
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

    pub fn validate_chain(&self) -> IcnResult<bool> {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            if current_block.hash != current_block.hash() {
                return Ok(false);
            }

            if current_block.previous_hash != previous_block.hash {
                return Ok(false);
            }

            for transaction in &current_block.transactions {
                if !self.verify_transaction(transaction)? {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    pub fn get_balance(&self, address: &str) -> f64 {
        self.utxo_set.get(address).cloned().unwrap_or(0.0)
    }

    pub fn start(&self) -> IcnResult<()> {
        info!("Blockchain started");
        Ok(())
    }

    pub fn stop(&self) -> IcnResult<()> {
        info!("Blockchain stopped");
        Ok(())
    }
}

pub struct AnonymousVotingSystem {
    zkp_manager: ZKPManager,
    voters: HashMap<String, bool>, // Address -> Has voted
    votes: Vec<bool>,
}

impl AnonymousVotingSystem {
    pub fn new() -> IcnResult<Self> {
        Ok(AnonymousVotingSystem {
            zkp_manager: ZKPManager::new()?,
            voters: HashMap::new(),
            votes: Vec::new(),
        })
    }

    pub fn register_voter(&mut self, voter_address: String) {
        self.voters.insert(voter_address, false);
    }

    pub fn cast_vote(&mut self, voter_address: &str, vote: bool, proof: &[u8]) -> IcnResult<()> {
        if let Some(has_voted) = self.voters.get_mut(voter_address) {
            if *has_voted {
                return Err(IcnError::Governance("Voter has already cast a vote".into()));
            }

            // Verify the ZKP
            let zkp_proof = groth16::Proof::read(proof)
                .map_err(|e| IcnError::ZKP(format!("Invalid proof: {}", e)))?;
            
            if self.zkp_manager.verify_vote_proof(&zkp_proof, voter_address, vote)? {
                self.votes.push(vote);
                *has_voted = true;
                Ok(())
            } else {
                Err(IcnError::Governance("Invalid vote proof".into()))
            }
        } else {
            Err(IcnError::Governance("Voter
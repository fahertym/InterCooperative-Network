use icn_utils::{error::IcnError, IcnResult};
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProofOfCooperation {
    pub blocks: Vec<Block>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub previous_hash: String,
    pub timestamp: u64,
    pub data: String,
    pub hash: String,
}

impl Block {
    pub fn new(index: u64, previous_hash: String, timestamp: u64, data: String) -> Self {
        let mut block = Block {
            index,
            previous_hash,
            timestamp,
            data,
            hash: String::new(),
        };
        block.hash = block.calculate_hash();
        block
    }

    pub fn calculate_hash(&self) -> String {
        let input = format!("{}{}{}{}", self.index, self.previous_hash, self.timestamp, self.data);
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

impl ProofOfCooperation {
    pub fn new() -> Self {
        ProofOfCooperation {
            blocks: Vec::new(),
        }
    }

    pub fn add_block(&mut self, data: String) -> IcnResult<()> {
        let previous_hash = if let Some(last_block) = self.blocks.last() {
            last_block.hash.clone()
        } else {
            String::new()
        };

        let new_block = Block::new(
            self.blocks.len() as u64,
            previous_hash,
            Utc::now().timestamp_millis() as u64,
            data,
        );

        self.blocks.push(new_block);
        Ok(())
    }

    pub fn validate_chain(&self) -> IcnResult<()> {
        for i in 1..self.blocks.len() {
            let previous_block = &self.blocks[i - 1];
            let current_block = &self.blocks[i];

            if current_block.previous_hash != previous_block.hash {
                return Err(IcnError::Blockchain("Invalid previous hash".to_string()));
            }

            if current_block.hash != current_block.calculate_hash() {
                return Err(IcnError::Blockchain("Invalid block hash".to_string()));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_validate_blocks() {
        let mut poc = ProofOfCooperation::new();
        
        poc.add_block("Block 1 Data".to_string()).unwrap();
        poc.add_block("Block 2 Data".to_string()).unwrap();

        assert_eq!(poc.blocks.len(), 2);
        assert!(poc.validate_chain().is_ok());

        // Tamper with a block
        poc.blocks[1].data = "Tampered Data".to_string();
        assert!(poc.validate_chain().is_err());
    }
}

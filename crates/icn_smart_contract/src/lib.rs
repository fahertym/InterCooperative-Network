use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SmartContract {
    pub id: String,
    pub code: String,
    pub owner: String,
}

impl SmartContract {
    pub fn new(id: String, code: String, owner: String) -> Self {
        SmartContract { id, code, owner }
    }

    pub fn execute(&self, _context: &str) -> Result<String, String> {
        // Placeholder for smart contract execution logic
        Ok("Smart contract executed successfully".to_string())
    }
}

pub trait SmartContractEngine {
    fn deploy(&mut self, contract: SmartContract) -> Result<(), String>;
    fn execute(&self, contract_id: &str, context: &str) -> Result<String, String>;
}

pub struct SimpleSmartContractEngine {
    contracts: Vec<SmartContract>,
}

impl SimpleSmartContractEngine {
    pub fn new() -> Self {
        SimpleSmartContractEngine {
            contracts: Vec::new(),
        }
    }
}

impl SmartContractEngine for SimpleSmartContractEngine {
    fn deploy(&mut self, contract: SmartContract) -> Result<(), String> {
        self.contracts.push(contract);
        Ok(())
    }

    fn execute(&self, contract_id: &str, context: &str) -> Result<String, String> {
        self.contracts
            .iter()
            .find(|c| c.id == contract_id)
            .ok_or_else(|| "Contract not found".to_string())?
            .execute(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_contract_execution() {
        let contract = SmartContract::new(
            "test_contract".to_string(),
            "contract code".to_string(),
            "owner".to_string(),
        );
        let result = contract.execute("test context");
        assert!(result.is_ok());
    }

    #[test]
    fn test_smart_contract_engine() {
        let mut engine = SimpleSmartContractEngine::new();
        let contract = SmartContract::new(
            "test_contract".to_string(),
            "contract code".to_string(),
            "owner".to_string(),
        );
        assert!(engine.deploy(contract).is_ok());
        let result = engine.execute("test_contract", "test context");
        assert!(result.is_ok());
    }
}
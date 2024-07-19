use icn_common::{Error, Result};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetTokenContract {
    pub tokens: Vec<AssetToken>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetToken {
    pub id: String,
    pub owner: String,
    pub metadata: serde_json::Value,
}

impl AssetTokenContract {
    pub fn new() -> Self {
        AssetTokenContract {
            tokens: Vec::new(),
        }
    }

    pub fn create_token(&mut self, id: String, owner: String, metadata: serde_json::Value) -> Result<AssetToken> {
        if self.tokens.iter().any(|t| t.id == id) {
            return Err(Error {
                message: "Token already exists".to_string(),
            });
        }

        let token = AssetToken {
            id: id.clone(),
            owner: owner.clone(),
            metadata,
        };

        self.tokens.push(token.clone());
        Ok(token)
    }

    pub fn transfer_token(&mut self, id: &str, new_owner: String) -> Result<()> {
        let token = self.tokens.iter_mut().find(|t| t.id == id).ok_or_else(|| Error {
            message: "Token not found".to_string(),
        })?;

        token.owner = new_owner;
        Ok(())
    }

    pub fn get_token(&self, id: &str) -> Option<&AssetToken> {
        self.tokens.iter().find(|t| t.id == id)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BondContract {
    pub bonds: Vec<Bond>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Bond {
    pub id: String,
    pub owner: String,
    pub terms: String,
}

impl BondContract {
    pub fn new() -> Self {
        BondContract {
            bonds: Vec::new(),
        }
    }

    pub fn create_bond(&mut self, id: String, owner: String, terms: String) -> Result<Bond> {
        if self.bonds.iter().any(|b| b.id == id) {
            return Err(Error {
                message: "Bond already exists".to_string(),
            });
        }

        let bond = Bond {
            id: id.clone(),
            owner: owner.clone(),
            terms,
        };

        self.bonds.push(bond.clone());
        Ok(bond)
    }

    pub fn transfer_bond(&mut self, id: &str, new_owner: String) -> Result<()> {
        let bond = self.bonds.iter_mut().find(|b| b.id == id).ok_or_else(|| Error {
            message: "Bond not found".to_string(),
        })?;

        bond.owner = new_owner;
        Ok(())
    }

    pub fn get_bond(&self, id: &str) -> Option<&Bond> {
        self.bonds.iter().find(|b| b.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_create_and_transfer_token() {
        let mut contract = AssetTokenContract::new();
        let metadata = json!({ "type": "Real Estate", "location": "123 Main St" });

        let token = contract.create_token("token1".to_string(), "Alice".to_string(), metadata).unwrap();
        assert_eq!(token.owner, "Alice");

        contract.transfer_token("token1", "Bob".to_string()).unwrap();
        let updated_token = contract.get_token("token1").unwrap();
        assert_eq!(updated_token.owner, "Bob");
    }

    #[test]
    fn test_create_and_transfer_bond() {
        let mut contract = BondContract::new();
        let bond = contract.create_bond("bond1".to_string(), "Alice".to_string(), "Terms of the bond".to_string()).unwrap();
        assert_eq!(bond.owner, "Alice");

        contract.transfer_bond("bond1", "Bob".to_string()).unwrap();
        let updated_bond = contract.get_bond("bond1").unwrap();
        assert_eq!(updated_bond.owner, "Bob");
    }
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::currency::{AssetToken, Bond};

pub trait SmartContract: erased_serde::Serialize + Send + Sync {
    fn execute(&self, env: &mut ExecutionEnvironment) -> Result<String, String>;
    fn id(&self) -> String;
}

erased_serde::serialize_trait_object!(SmartContract);

pub struct ExecutionEnvironment {
    pub state: String,
}

impl ExecutionEnvironment {
    pub fn new() -> Self {
        ExecutionEnvironment {
            state: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AssetTokenContract {
    pub asset_token: AssetToken,
}

impl SmartContract for AssetTokenContract {
    fn execute(&self, _env: &mut ExecutionEnvironment) -> Result<String, String> {
        // Implementation would go here
        Ok("Asset token created".to_string())
    }

    fn id(&self) -> String {
        self.asset_token.asset_id.clone()
    }
}

#[derive(Serialize, Deserialize)]
pub struct BondContract {
    pub bond: Bond,
}

impl SmartContract for BondContract {
    fn execute(&self, _env: &mut ExecutionEnvironment) -> Result<String, String> {
        // Implementation would go here
        Ok("Bond created".to_string())
    }

    fn id(&self) -> String {
        self.bond.bond_id.clone()
    }
}

impl AssetTokenContract {
    pub fn new(asset_id: String, name: String, description: String, owner: String, value: f64) -> Self {
        Self {
            asset_token: AssetToken {
                asset_id,
                name,
                description,
                owner,
                value,
            }
        }
    }
}

impl BondContract {
    pub fn new(bond_id: String, name: String, description: String, issuer: String, face_value: f64, maturity_date: DateTime<Utc>, interest_rate: f64, owner: String) -> Self {
        Self {
            bond: Bond {
                bond_id,
                name,
                description,
                issuer,
                face_value,
                maturity_date,
                interest_rate,
                owner,
            }
        }
    }
}
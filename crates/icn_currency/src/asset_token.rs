use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetToken {
    pub asset_id: String,
    pub name: String,
    pub description: String,
    pub owner: String,
    pub value: f64,
}

impl AssetToken {
    pub fn new(asset_id: String, name: String, description: String, owner: String, value: f64) -> Self {
        AssetToken {
            asset_id,
            name,
            description,
            owner,
            value,
        }
    }

    pub fn transfer(&mut self, new_owner: String) {
        self.owner = new_owner;
    }
}

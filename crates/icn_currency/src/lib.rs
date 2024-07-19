mod currency;
mod wallet;
mod asset_token;
mod bond;

pub use currency::{Currency, CurrencyType, CurrencySystem};
pub use wallet::Wallet;
pub use asset_token::AssetToken;
pub use bond::Bond;

use icn_core::error::{Error, Result};

pub struct CurrencyManager {
    pub currency_system: CurrencySystem,
}

impl CurrencyManager {
    pub fn new() -> Self {
        CurrencyManager {
            currency_system: CurrencySystem::new(),
        }
    }

    pub fn create_wallet(&self) -> Wallet {
        Wallet::new()
    }

    pub fn create_asset_token(&self, asset_id: String, name: String, description: String, owner: String, value: f64) -> AssetToken {
        AssetToken::new(asset_id, name, description, owner, value)
    }

    pub fn create_bond(&self, bond_id: String, name: String, description: String, issuer: String, face_value: f64, maturity_date: chrono::DateTime<chrono::Utc>, interest_rate: f64, owner: String) -> Bond {
        Bond::new(bond_id, name, description, issuer, face_value, maturity_date, interest_rate, owner)
    }

    pub fn perform_adaptive_issuance(&mut self) -> Result<()> {
        self.currency_system.adaptive_issuance()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_currency_manager() {
        let mut manager = CurrencyManager::new();
        
        // Test wallet creation and operations
        let mut wallet = manager.create_wallet();
        assert!(wallet.deposit(CurrencyType::BasicNeeds, 100.0).is_ok());
        assert_eq!(wallet.get_balance(&CurrencyType::BasicNeeds), 100.0);

        // Test asset token creation
        let asset = manager.create_asset_token(
            "ASSET1".to_string(),
            "Test Asset".to_string(),
            "A test asset".to_string(),
            "Alice".to_string(),
            1000.0
        );
        assert_eq!(asset.owner, "Alice");

        // Test bond creation
        let maturity_date = Utc::now() + chrono::Duration::days(365);
        let bond = manager.create_bond(
            "BOND1".to_string(),
            "Test Bond".to_string(),
            "A test bond".to_string(),
            "ICN".to_string(),
            1000.0,
            maturity_date,
            0.05,
            "Bob".to_string()
        );
        assert_eq!(bond.owner, "Bob");

        // Test adaptive issuance
        assert!(manager.perform_adaptive_issuance().is_ok());
    }
}
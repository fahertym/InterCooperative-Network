pub mod block;
pub mod transaction;

pub use self::block::Block;
pub use self::transaction::Transaction;

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Error {
    pub message: String,
}

pub type Result<T> = std::result::Result<T, Error>;

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
pub struct Currency {
    pub currency_type: CurrencyType,
    pub total_supply: f64,
    pub creation_date: DateTime<Utc>,
    pub last_issuance: DateTime<Utc>,
    pub issuance_rate: f64,
}

impl Currency {
    pub fn new(currency_type: CurrencyType, initial_supply: f64, issuance_rate: f64) -> Self {
        let now = Utc::now();
        Currency {
            currency_type,
            total_supply: initial_supply,
            creation_date: now,
            last_issuance: now,
            issuance_rate,
        }
    }

    pub fn mint(&mut self, amount: f64) -> Result<()> {
        self.total_supply += amount;
        self.last_issuance = Utc::now();
        Ok(())
    }

    pub fn burn(&mut self, amount: f64) -> Result<()> {
        if amount > self.total_supply {
            return Err(Error {
                message: "Insufficient supply to burn".to_string(),
            });
        }
        self.total_supply -= amount;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurrencySystem {
    pub currencies: HashMap<CurrencyType, Currency>,
}

impl CurrencySystem {
    pub fn new() -> Self {
        CurrencySystem {
            currencies: HashMap::new(),
        }
    }

    pub fn add_currency(&mut self, currency_type: CurrencyType, initial_supply: f64, issuance_rate: f64) {
        let currency = Currency::new(currency_type.clone(), initial_supply, issuance_rate);
        self.currencies.insert(currency_type, currency);
    }

    pub fn get_currency(&self, currency_type: &CurrencyType) -> Option<&Currency> {
        self.currencies.get(currency_type)
    }

    pub fn get_currency_mut(&mut self, currency_type: &CurrencyType) -> Option<&mut Currency> {
        self.currencies.get_mut(currency_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_mint_burn() {
        let mut currency = Currency::new(CurrencyType::BasicNeeds, 1000.0, 0.01);
        assert!(currency.mint(100.0).is_ok());
        assert_eq!(currency.total_supply, 1100.0);
        assert!(currency.burn(200.0).is_ok());
        assert_eq!(currency.total_supply, 900.0);
        assert!(currency.burn(1000.0).is_err());
    }

    #[test]
    fn test_currency_system() {
        let mut system = CurrencySystem::new();
        system.add_currency(CurrencyType::BasicNeeds, 1000.0, 0.01);
        system.add_currency(CurrencyType::Education, 500.0, 0.005);

        assert!(system.get_currency(&CurrencyType::BasicNeeds).is_some());
        assert!(system.get_currency(&CurrencyType::Education).is_some());
        assert!(system.get_currency(&CurrencyType::Environmental).is_none());
    }
}

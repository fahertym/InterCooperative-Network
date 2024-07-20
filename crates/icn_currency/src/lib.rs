// File: icn_currency/src/lib.rs

use icn_types::{IcnResult, IcnError, CurrencyType};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
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

    pub fn mint(&mut self, amount: f64) -> IcnResult<()> {
        self.total_supply += amount;
        self.last_issuance = Utc::now();
        Ok(())
    }

    pub fn burn(&mut self, amount: f64) -> IcnResult<()> {
        if amount > self.total_supply {
            return Err(IcnError::Currency("Insufficient supply to burn".to_string()));
        }
        self.total_supply -= amount;
        Ok(())
    }
}

pub struct CurrencySystem {
    pub currencies: HashMap<CurrencyType, Currency>,
    balances: HashMap<String, HashMap<CurrencyType, f64>>,
}

impl CurrencySystem {
    pub fn new() -> Self {
        let mut system = CurrencySystem {
            currencies: HashMap::new(),
            balances: HashMap::new(),
        };
        
        system.add_currency(CurrencyType::BasicNeeds, 1_000_000.0, 0.01);
        system.add_currency(CurrencyType::Education, 500_000.0, 0.005);
        system.add_currency(CurrencyType::Environmental, 750_000.0, 0.008);
        system.add_currency(CurrencyType::Community, 250_000.0, 0.003);
        system.add_currency(CurrencyType::Volunteer, 100_000.0, 0.002);

        system
    }

    pub fn add_currency(&mut self, currency_type: CurrencyType, initial_supply: f64, issuance_rate: f64) {
        let currency = Currency::new(currency_type.clone(), initial_supply, issuance_rate);
        self.currencies.insert(currency_type, currency);
    }

    pub fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> f64 {
        self.balances
            .get(address)
            .and_then(|balances| balances.get(currency_type))
            .cloned()
            .unwrap_or(0.0)
    }

    pub fn update_balance(&mut self, address: &str, currency_type: &CurrencyType, amount: f64) -> IcnResult<()> {
        let balance = self.balances
            .entry(address.to_string())
            .or_insert_with(HashMap::new)
            .entry(currency_type.clone())
            .or_insert(0.0);
        *balance += amount;
        if *balance < 0.0 {
            return Err(IcnError::Currency("Insufficient balance".to_string()));
        }
        Ok(())
    }

    pub fn transfer(&mut self, from: &str, to: &str, currency_type: &CurrencyType, amount: f64) -> IcnResult<()> {
        self.update_balance(from, currency_type, -amount)?;
        self.update_balance(to, currency_type, amount)?;
        Ok(())
    }

    pub fn create_custom_currency(&mut self, name: String, initial_supply: f64, issuance_rate: f64) -> IcnResult<()> {
        let currency_type = CurrencyType::Custom(name.clone());
        if self.currencies.contains_key(&currency_type) {
            return Err(IcnError::Currency(format!("Currency '{}' already exists", name)));
        }
        self.add_currency(currency_type, initial_supply, issuance_rate);
        Ok(())
    }

    pub fn adaptive_issuance(&mut self) -> IcnResult<()> {
        let now = Utc::now();
        for currency in self.currencies.values_mut() {
            let time_since_last_issuance = now.signed_duration_since(currency.last_issuance);
            let issuance_amount = currency.total_supply * currency.issuance_rate * time_since_last_issuance.num_milliseconds() as f64 / 86_400_000.0; // Daily rate
            currency.mint(issuance_amount)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_system() {
        let mut system = CurrencySystem::new();
        
        // Test balance operations
        assert_eq!(system.get_balance("Alice", &CurrencyType::BasicNeeds), 0.0);
        system.update_balance("Alice", &CurrencyType::BasicNeeds, 100.0).unwrap();
        assert_eq!(system.get_balance("Alice", &CurrencyType::BasicNeeds), 100.0);

        // Test transfer
        system.transfer("Alice", "Bob", &CurrencyType::BasicNeeds, 50.0).unwrap();
        assert_eq!(system.get_balance("Alice", &CurrencyType::BasicNeeds), 50.0);
        assert_eq!(system.get_balance("Bob", &CurrencyType::BasicNeeds), 50.0);

        // Test custom currency creation
        system.create_custom_currency("LocalCoin".to_string(), 10_000.0, 0.005).unwrap();
        let local_coin = CurrencyType::Custom("LocalCoin".to_string());
        assert!(system.currencies.contains_key(&local_coin));

        // Test adaptive issuance
        let initial_supply = system.currencies[&CurrencyType::BasicNeeds].total_supply;
        system.adaptive_issuance().unwrap();
        assert!(system.currencies[&CurrencyType::BasicNeeds].total_supply > initial_supply);
    }
}
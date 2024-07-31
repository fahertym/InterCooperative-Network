// File: icn_currency/src/lib.rs

use icn_common::{IcnResult, IcnError, Transaction, CurrencyType};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
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
        if amount < 0.0 {
            return Err(IcnError::Currency("Cannot mint negative amount".into()));
        }
        self.total_supply += amount;
        self.last_issuance = Utc::now();
        Ok(())
    }

    pub fn burn(&mut self, amount: f64) -> IcnResult<()> {
        if amount < 0.0 {
            return Err(IcnError::Currency("Cannot burn negative amount".into()));
        }
        if amount > self.total_supply {
            return Err(IcnError::Currency("Insufficient supply to burn".into()));
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
        CurrencySystem {
            currencies: HashMap::new(),
            balances: HashMap::new(),
        }
    }

    pub fn add_currency(&mut self, currency_type: CurrencyType, initial_supply: f64, issuance_rate: f64) -> IcnResult<()> {
        if self.currencies.contains_key(&currency_type) {
            return Err(IcnError::Currency("Currency already exists".into()));
        }
        let currency = Currency::new(currency_type.clone(), initial_supply, issuance_rate);
        self.currencies.insert(currency_type, currency);
        Ok(())
    }

    pub fn mint(&mut self, currency_type: &CurrencyType, amount: f64) -> IcnResult<()> {
        let currency = self.currencies.get_mut(currency_type)
            .ok_or_else(|| IcnError::Currency("Currency not found".into()))?;
        currency.mint(amount)
    }

    pub fn burn(&mut self, currency_type: &CurrencyType, amount: f64) -> IcnResult<()> {
        let currency = self.currencies.get_mut(currency_type)
            .ok_or_else(|| IcnError::Currency("Currency not found".into()))?;
        currency.burn(amount)
    }

    pub fn process_transaction(&mut self, transaction: &Transaction) -> IcnResult<()> {
        self.transfer(
            &transaction.from,
            &transaction.to,
            &transaction.currency_type,
            transaction.amount,
        )
    }

    pub fn transfer(&mut self, from: &str, to: &str, currency_type: &CurrencyType, amount: f64) -> IcnResult<()> {
        if amount < 0.0 {
            return Err(IcnError::Currency("Cannot transfer negative amount".into()));
        }

        let from_balance = self.get_balance(from, currency_type)?;
        if from_balance < amount {
            return Err(IcnError::Currency("Insufficient balance".into()));
        }

        self.update_balance(from, currency_type, -amount)?;
        self.update_balance(to, currency_type, amount)?;

        Ok(())
    }

    pub fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        Ok(*self.balances
            .get(address)
            .and_then(|balances| balances.get(currency_type))
            .unwrap_or(&0.0))
    }

    fn update_balance(&mut self, address: &str, currency_type: &CurrencyType, amount: f64) -> IcnResult<()> {
        let balance = self.balances
            .entry(address.to_string())
            .or_insert_with(HashMap::new)
            .entry(currency_type.clone())
            .or_insert(0.0);
        *balance += amount;
        Ok(())
    }

    pub fn get_total_supply(&self, currency_type: &CurrencyType) -> IcnResult<f64> {
        self.currencies.get(currency_type)
            .map(|currency| currency.total_supply)
            .ok_or_else(|| IcnError::Currency("Currency not found".into()))
    }

    pub fn get_issuance_rate(&self, currency_type: &CurrencyType) -> IcnResult<f64> {
        self.currencies.get(currency_type)
            .map(|currency| currency.issuance_rate)
            .ok_or_else(|| IcnError::Currency("Currency not found".into()))
    }

    pub fn update_issuance_rate(&mut self, currency_type: &CurrencyType, new_rate: f64) -> IcnResult<()> {
        let currency = self.currencies.get_mut(currency_type)
            .ok_or_else(|| IcnError::Currency("Currency not found".into()))?;
        currency.issuance_rate = new_rate;
        Ok(())
    }

    pub fn list_currencies(&self) -> Vec<CurrencyType> {
        self.currencies.keys().cloned().collect()
    }

    pub fn get_currency_info(&self, currency_type: &CurrencyType) -> IcnResult<Currency> {
        self.currencies.get(currency_type)
            .cloned()
            .ok_or_else(|| IcnError::Currency("Currency not found".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_system() {
        let mut system = CurrencySystem::new();

        // Add currencies
        assert!(system.add_currency(CurrencyType::BasicNeeds, 1000.0, 0.01).is_ok());
        assert!(system.add_currency(CurrencyType::Education, 500.0, 0.005).is_ok());

        // Test minting
        assert!(system.mint(&CurrencyType::BasicNeeds, 100.0).is_ok());
        assert_eq!(system.get_total_supply(&CurrencyType::BasicNeeds).unwrap(), 1100.0);

        // Test burning
        assert!(system.burn(&CurrencyType::BasicNeeds, 50.0).is_ok());
        assert_eq!(system.get_total_supply(&CurrencyType::BasicNeeds).unwrap(), 1050.0);

        // Test transfer
        system.update_balance("Alice", &CurrencyType::BasicNeeds, 100.0).unwrap();
        assert!(system.transfer("Alice", "Bob", &CurrencyType::BasicNeeds, 50.0).is_ok());
        assert_eq!(system.get_balance("Alice", &CurrencyType::BasicNeeds).unwrap(), 50.0);
        assert_eq!(system.get_balance("Bob", &CurrencyType::BasicNeeds).unwrap(), 50.0);

        // Test insufficient balance
        assert!(system.transfer("Alice", "Bob", &CurrencyType::BasicNeeds, 100.0).is_err());

        // Test get issuance rate
        assert_eq!(system.get_issuance_rate(&CurrencyType::BasicNeeds).unwrap(), 0.01);

        // Test update issuance rate
        assert!(system.update_issuance_rate(&CurrencyType::BasicNeeds, 0.02).is_ok());
        assert_eq!(system.get_issuance_rate(&CurrencyType::BasicNeeds).unwrap(), 0.02);

        // Test list currencies
        let currencies = system.list_currencies();
        assert_eq!(currencies.len(), 2);
        assert!(currencies.contains(&CurrencyType::BasicNeeds));
        assert!(currencies.contains(&CurrencyType::Education));

        // Test get currency info
        let basic_needs_info = system.get_currency_info(&CurrencyType::BasicNeeds).unwrap();
        assert_eq!(basic_needs_info.total_supply, 1050.0);
        assert_eq!(basic_needs_info.issuance_rate, 0.02);
    }
}
use icn_types::{IcnResult, IcnError, CurrencyType, Currency, Transaction};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

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
        let currency = Currency {
            currency_type: currency_type.clone(),
            total_supply: initial_supply,
            creation_date: Utc::now(),
            last_issuance: Utc::now(),
            issuance_rate,
        };
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

    pub fn process_transaction(&mut self, transaction: &Transaction) -> IcnResult<()> {
        self.transfer(&transaction.from, &transaction.to, &transaction.currency_type, transaction.amount)
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
            currency.total_supply += issuance_amount;
            currency.last_issuance = now;
        }
        Ok(())
    }

    pub fn mint(&mut self, currency_type: &CurrencyType, amount: f64) -> IcnResult<()> {
        let currency = self.currencies.get_mut(currency_type)
            .ok_or_else(|| IcnError::Currency(format!("Currency {:?} not found", currency_type)))?;
        currency.total_supply += amount;
        Ok(())
    }

    pub fn burn(&mut self, currency_type: &CurrencyType, amount: f64) -> IcnResult<()> {
        let currency = self.currencies.get_mut(currency_type)
            .ok_or_else(|| IcnError::Currency(format!("Currency {:?} not found", currency_type)))?;
        if currency.total_supply < amount {
            return Err(IcnError::Currency("Insufficient supply to burn".to_string()));
        }
        currency.total_supply -= amount;
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

        // Test mint and burn
        let mint_amount = 1000.0;
        system.mint(&CurrencyType::BasicNeeds, mint_amount).unwrap();
        let new_supply = system.currencies[&CurrencyType::BasicNeeds].total_supply;
        assert_eq!(new_supply, initial_supply + mint_amount);

        system.burn(&CurrencyType::BasicNeeds, 500.0).unwrap();
        let final_supply = system.currencies[&CurrencyType::BasicNeeds].total_supply;
        assert_eq!(final_supply, new_supply - 500.0);

        // Test insufficient balance error
        assert!(system.transfer("Alice", "Bob", &CurrencyType::BasicNeeds, 1000.0).is_err());

        // Test non-existent currency error
        assert!(system.mint(&CurrencyType::Custom("NonExistent".to_string()), 100.0).is_err());
    }
}
// File: crates/icn_currency/src/lib.rs

use icn_common::{IcnResult, IcnError, CurrencyType, Transaction};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use log::{info, warn, error};

pub struct CurrencySystem {
    balances: Arc<RwLock<HashMap<String, HashMap<CurrencyType, f64>>>>,
    total_supply: Arc<RwLock<HashMap<CurrencyType, f64>>>,
}

impl CurrencySystem {
    pub fn new() -> Self {
        CurrencySystem {
            balances: Arc::new(RwLock::new(HashMap::new())),
            total_supply: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        let balances = self.balances.read().map_err(|_| IcnError::Currency("Failed to read balances".into()))?;
        balances.get(address)
            .and_then(|account_balances| account_balances.get(currency_type))
            .cloned()
            .ok_or_else(|| IcnError::Currency("Balance not found".into()))
    }

    pub fn transfer(&self, transaction: &Transaction) -> IcnResult<()> {
        let mut balances = self.balances.write().map_err(|_| IcnError::Currency("Failed to write balances".into()))?;
        
        let from_balance = balances.entry(transaction.from.clone()).or_insert_with(HashMap::new)
            .entry(transaction.currency_type.clone()).or_insert(0.0);
        
        if *from_balance < transaction.amount {
            return Err(IcnError::Currency("Insufficient balance".into()));
        }
        
        *from_balance -= transaction.amount;
        
        *balances.entry(transaction.to.clone()).or_insert_with(HashMap::new)
            .entry(transaction.currency_type.clone()).or_insert(0.0) += transaction.amount;
        
        Ok(())
    }

    pub fn mint(&self, address: &str, currency_type: CurrencyType, amount: f64) -> IcnResult<()> {
        let mut balances = self.balances.write().map_err(|_| IcnError::Currency("Failed to write balances".into()))?;
        let mut total_supply = self.total_supply.write().map_err(|_| IcnError::Currency("Failed to write total supply".into()))?;
        
        *balances.entry(address.to_string()).or_insert_with(HashMap::new)
            .entry(currency_type.clone()).or_insert(0.0) += amount;
        
        *total_supply.entry(currency_type).or_insert(0.0) += amount;
        
        Ok(())
    }

    pub fn burn(&self, address: &str, currency_type: CurrencyType, amount: f64) -> IcnResult<()> {
        let mut balances = self.balances.write().map_err(|_| IcnError::Currency("Failed to write balances".into()))?;
        let mut total_supply = self.total_supply.write().map_err(|_| IcnError::Currency("Failed to write total supply".into()))?;
        
        let balance = balances.get_mut(address)
            .and_then(|account_balances| account_balances.get_mut(&currency_type))
            .ok_or_else(|| IcnError::Currency("Balance not found".into()))?;
        
        if *balance < amount {
            return Err(IcnError::Currency("Insufficient balance to burn".into()));
        }
        
        *balance -= amount;
        *total_supply.get_mut(&currency_type).unwrap() -= amount;
        
        Ok(())
    }

    pub fn get_total_supply(&self, currency_type: &CurrencyType) -> IcnResult<f64> {
        let total_supply = self.total_supply.read().map_err(|_| IcnError::Currency("Failed to read total supply".into()))?;
        total_supply.get(currency_type)
            .cloned()
            .ok_or_else(|| IcnError::Currency("Currency not found".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer() {
        let currency_system = CurrencySystem::new();
        currency_system.mint("Alice", CurrencyType::BasicNeeds, 100.0).unwrap();
        
        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 0,
            signature: None,
        };
        
        assert!(currency_system.transfer(&transaction).is_ok());
        assert_eq!(currency_system.get_balance("Alice", &CurrencyType::BasicNeeds).unwrap(), 50.0);
        assert_eq!(currency_system.get_balance("Bob", &CurrencyType::BasicNeeds).unwrap(), 50.0);
    }

    #[test]
    fn test_mint_and_burn() {
        let currency_system = CurrencySystem::new();
        currency_system.mint("Alice", CurrencyType::BasicNeeds, 100.0).unwrap();
        assert_eq!(currency_system.get_total_supply(&CurrencyType::BasicNeeds).unwrap(), 100.0);
        
        currency_system.burn("Alice", CurrencyType::BasicNeeds, 30.0).unwrap();
        assert_eq!(currency_system.get_balance("Alice", &CurrencyType::BasicNeeds).unwrap(), 70.0);
        assert_eq!(currency_system.get_total_supply(&CurrencyType::BasicNeeds).unwrap(), 70.0);
    }
}
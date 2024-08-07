// File: crates/icn_currency/src/lib.rs

use icn_common::{IcnResult, IcnError, Transaction, CurrencyType};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Represents a currency in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Currency {
    pub currency_type: CurrencyType,
    pub total_supply: f64,
    pub creation_date: DateTime<Utc>,
    pub last_issuance: DateTime<Utc>,
    pub issuance_rate: f64,
}

impl Currency {
    /// Creates a new currency with the specified initial supply and issuance rate.
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

    /// Mints new currency, increasing the total supply.
    pub fn mint(&mut self, amount: f64) -> IcnResult<()> {
        if amount < 0.0 {
            return Err(IcnError::Currency("Cannot mint negative amount".into()));
        }
        self.total_supply += amount;
        self.last_issuance = Utc::now();
        Ok(())
    }

    /// Burns currency, decreasing the total supply.
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

/// Manages multiple currencies and their associated balances.
pub struct CurrencySystem {
    pub currencies: HashMap<CurrencyType, Currency>,
    balances: HashMap<String, HashMap<CurrencyType, f64>>,
}

impl CurrencySystem {
    /// Creates a new, empty currency system.
    pub fn new() -> Self {
        CurrencySystem {
            currencies: HashMap::new(),
            balances: HashMap::new(),
        }
    }

    /// Adds a new currency to the system with the specified initial supply and issuance rate.
    pub fn add_currency(&mut self, currency_type: CurrencyType, initial_supply: f64, issuance_rate: f64) -> IcnResult<()> {
        if self.currencies.contains_key(&currency_type) {
            return Err(IcnError::Currency("Currency already exists".into()));
        }
        let currency = Currency::new(currency_type.clone(), initial_supply, issuance_rate);
        self.currencies.insert(currency_type, currency);
        Ok(())
    }

    /// Mints new units of the specified currency.
    pub fn mint(&mut self, currency_type: &CurrencyType, amount: f64) -> IcnResult<()> {
        let currency = self.currencies.get_mut(currency_type)
            .ok_or_else(|| IcnError::Currency("Currency not found".into()))?;
        currency.mint(amount)
    }

    /// Burns units of the specified currency.
    pub fn burn(&mut self, currency_type: &CurrencyType, amount: f64) -> IcnResult<()> {
        let currency = self.currencies.get_mut(currency_type)
            .ok_or_else(|| IcnError::Currency("Currency not found".into()))?;
        currency.burn(amount)
    }

    /// Processes a transaction by transferring currency between two accounts.
    pub fn process_transaction(&mut self, transaction: &Transaction) -> IcnResult<()> {
        self.transfer(
            &transaction.from,
            &transaction.to,
            &transaction.currency_type,
            transaction.amount,
        )
    }

    /// Transfers a specified amount of currency from one account to another.
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

    /// Retrieves the balance of an account for a specified currency type.
    pub fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        Ok(*self.balances
            .get(address)
            .and_then(|balances| balances.get(currency_type))
            .unwrap_or(&0.0))
    }

    /// Updates the balance of an account by a specified amount.
    fn update_balance(&mut self, address: &str, currency_type: &CurrencyType, amount: f64) -> IcnResult<()> {
        let balance = self.balances
            .entry(address.to_string())
            .or_insert_with(HashMap::new)
            .entry(currency_type.clone())
            .or_insert(0.0);
        *balance += amount;
        Ok(())
    }

    /// Retrieves the total supply of a specified currency.
    pub fn get_total_supply(&self, currency_type: &CurrencyType) -> IcnResult<f64> {
        self.currencies.get(currency_type)
            .map(|currency| currency.total_supply)
            .ok_or_else(|| IcnError::Currency("Currency not found".into()))
    }

    /// Retrieves the issuance rate of a specified currency.
    pub fn get_issuance_rate(&self, currency_type: &CurrencyType) -> IcnResult<f64> {
        self.currencies.get(currency_type)
            .map(|currency| currency.issuance_rate)
            .ok_or_else(|| IcnError::Currency("Currency not found".into()))
    }

    /// Updates the issuance rate of a specified currency.
    pub fn update_issuance_rate(&mut self, currency_type: &CurrencyType, new_rate: f64) -> IcnResult<()> {
        let currency = self.currencies.get_mut(currency_type)
            .ok_or_else(|| IcnError::Currency("Currency not found".into()))?;
        currency.issuance_rate = new_rate;
        Ok(())
    }

    /// Lists all available currencies in the system.
    pub fn list_currencies(&self) -> Vec<CurrencyType> {
        self.currencies.keys().cloned().collect()
    }

    /// Retrieves information about a specific currency.
    pub fn get_currency_info(&self, currency_type: &CurrencyType) -> IcnResult<Currency> {
        self.currencies.get(currency_type)
            .cloned()
            .ok_or_else(|| IcnError::Currency("Currency not found".into()))
    }

    /// Exchanges currency from one type to another.
    pub fn exchange_currency(&mut self, from: &str, source_currency: &CurrencyType, target_currency: &CurrencyType, amount: f64) -> IcnResult<()> {
        // Check if both currencies exist
        if !self.currencies.contains_key(source_currency) || !self.currencies.contains_key(target_currency) {
            return Err(IcnError::Currency("Invalid currency type".into()));
        }

        // Check if the user has sufficient balance
        let source_balance = self.get_balance(from, source_currency)?;
        if source_balance < amount {
            return Err(IcnError::Currency("Insufficient balance for exchange".into()));
        }

        // Calculate exchange rate (simplified for demonstration)
        let exchange_rate = match (source_currency, target_currency) {
            (CurrencyType::BasicNeeds, CurrencyType::Education) => 1.2,
            (CurrencyType::Education, CurrencyType::BasicNeeds) => 0.8,
            (CurrencyType::BasicNeeds, CurrencyType::Environmental) => 1.5,
            (CurrencyType::Environmental, CurrencyType::BasicNeeds) => 0.6,
            (CurrencyType::Education, CurrencyType::Environmental) => 1.3,
            (CurrencyType::Environmental, CurrencyType::Education) => 0.7,
            _ => 1.0, // Default to 1:1 for other combinations or same currency
        };

        let target_amount = amount * exchange_rate;

        // Perform the exchange
        self.update_balance(from, source_currency, -amount)?;
        self.update_balance(from, target_currency, target_amount)?;

        // Update currency supplies
        let source_currency = self.currencies.get_mut(source_currency).unwrap();
        source_currency.burn(amount)?;

        let target_currency = self.currencies.get_mut(target_currency).unwrap();
        target_currency.mint(target_amount)?;

        Ok(())
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

    #[test]
    fn test_currency_operations() {
        let mut currency = Currency::new(CurrencyType::BasicNeeds, 1000.0, 0.01);

        // Test minting
        assert!(currency.mint(100.0).is_ok());
        assert_eq!(currency.total_supply, 1100.0);

        // Test burning
        assert!(currency.burn(50.0).is_ok());
        assert_eq!(currency.total_supply, 1050.0);

        // Test minting negative amount
        assert!(currency.mint(-100.0).is_err());

        // Test burning negative amount
        assert!(currency.burn(-50.0).is_err());

        // Test burning more than available
        assert!(currency.burn(2000.0).is_err());
    }

    #[test]
    fn test_currency_system_edge_cases() {
        let mut system = CurrencySystem::new();

        // Test adding duplicate currency
        assert!(system.add_currency(CurrencyType::BasicNeeds, 1000.0, 0.01).is_ok());
        assert!(system.add_currency(CurrencyType::BasicNeeds, 2000.0, 0.02).is_err());

        // Test operations on non-existent currency
        assert!(system.mint(&CurrencyType::Education, 100.0).is_err());
        assert!(system.burn(&CurrencyType::Education, 50.0).is_err());
        assert!(system.get_total_supply(&CurrencyType::Education).is_err());
        assert!(system.get_issuance_rate(&CurrencyType::Education).is_err());
        assert!(system.update_issuance_rate(&CurrencyType::Education, 0.03).is_err());
        assert!(system.get_currency_info(&CurrencyType::Education).is_err());

        // Test transfer with non-existent currency
        assert!(system.transfer("Alice", "Bob", &CurrencyType::Education, 50.0).is_err());

        // Test transfer with negative amount
        assert!(system.transfer("Alice", "Bob", &CurrencyType::BasicNeeds, -50.0).is_err());
    }

    #[test]
    fn test_currency_system_process_transaction() {
        let mut system = CurrencySystem::new();
        system.add_currency(CurrencyType::BasicNeeds, 1000.0, 0.01).unwrap();

        // Initialize balance for Alice
        system.update_balance("Alice", &CurrencyType::BasicNeeds, 100.0).unwrap();

        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };

        assert!(system.process_transaction(&transaction).is_ok());
        assert_eq!(system.get_balance("Alice", &CurrencyType::BasicNeeds).unwrap(), 50.0);
        assert_eq!(system.get_balance("Bob", &CurrencyType::BasicNeeds).unwrap(), 50.0);

        // Test processing invalid transaction
        let invalid_transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 100.0, // More than Alice's balance
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };

        assert!(system.process_transaction(&invalid_transaction).is_err());
    }

    #[test]
    fn test_exchange_currency() {
        let mut system = CurrencySystem::new();
        system.add_currency(CurrencyType::BasicNeeds, 1000.0, 0.01).unwrap();
        system.add_currency(CurrencyType::Education, 1000.0, 0.01).unwrap();
        system.update_balance("Alice", &CurrencyType::BasicNeeds, 100.0).unwrap();

        assert!(system.exchange_currency("Alice", &CurrencyType::BasicNeeds, &CurrencyType::Education, 50.0).is_ok());
        assert_eq!(system.get_balance("Alice", &CurrencyType::BasicNeeds).unwrap(), 50.0);
        assert_eq!(system.get_balance("Alice", &CurrencyType::Education).unwrap(), 60.0); // 50 * 1.2

        // Test insufficient balance
        assert!(system.exchange_currency("Alice", &CurrencyType::BasicNeeds, &CurrencyType::Education, 100.0).is_err());

        // Test invalid currency
        assert!(system.exchange_currency("Alice", &CurrencyType::BasicNeeds, &CurrencyType::Environmental, 10.0).is_err());
    }
}

use icn_common::{IcnResult, IcnError, CurrencyType, Transaction};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use log::{info, warn};

/// Represents the balance of a particular currency for a given account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub currency: CurrencyType,
    pub amount: f64,
}

/// Represents a currency system managing multiple currencies.
pub struct CurrencySystem {
    pub accounts: HashMap<String, Vec<Balance>>,
}

impl CurrencySystem {
    /// Creates a new CurrencySystem.
    pub fn new() -> Self {
        CurrencySystem {
            accounts: HashMap::new(),
        }
    }

    /// Mints a specified amount of a currency to a given account.
    ///
    /// # Arguments
    ///
    /// * `account` - The account to mint the currency to.
    /// * `currency` - The type of currency to mint.
    /// * `amount` - The amount of currency to mint.
    ///
    /// # Errors
    ///
    /// Returns `IcnError::Currency` if the operation fails.
    pub fn mint(&mut self, account: &str, currency: CurrencyType, amount: f64) -> IcnResult<()> {
        let balances = self.accounts.entry(account.to_string()).or_insert_with(Vec::new);
        if let Some(balance) = balances.iter_mut().find(|b| b.currency == currency) {
            balance.amount += amount;
        } else {
            balances.push(Balance { currency, amount });
        }
        info!("Minted {} of {:?} to {}", amount, currency, account);
        Ok(())
    }

    /// Burns a specified amount of a currency from a given account.
    ///
    /// # Arguments
    ///
    /// * `account` - The account to burn the currency from.
    /// * `currency` - The type of currency to burn.
    /// * `amount` - The amount of currency to burn.
    ///
    /// # Errors
    ///
    /// Returns `IcnError::Currency` if the account has insufficient balance.
    pub fn burn(&mut self, account: &str, currency: CurrencyType, amount: f64) -> IcnResult<()> {
        let balances = self.accounts.entry(account.to_string()).or_insert_with(Vec::new);
        if let Some(balance) = balances.iter_mut().find(|b| b.currency == currency) {
            if balance.amount < amount {
                return Err(IcnError::Currency("Insufficient balance".into()));
            }
            balance.amount -= amount;
            info!("Burned {} of {:?} from {}", amount, currency, account);
            Ok(())
        } else {
            Err(IcnError::Currency("Currency not found in account".into()))
        }
    }

    /// Transfers a specified amount of a currency from one account to another.
    ///
    /// # Arguments
    ///
    /// * `from` - The account to transfer the currency from.
    /// * `to` - The account to transfer the currency to.
    /// * `currency` - The type of currency to transfer.
    /// * `amount` - The amount of currency to transfer.
    ///
    /// # Errors
    ///
    /// Returns `IcnError::Currency` if the sender has insufficient balance or if the operation fails.
    pub fn transfer(&mut self, from: &str, to: &str, currency: CurrencyType, amount: f64) -> IcnResult<()> {
        self.burn(from, currency.clone(), amount)?;
        self.mint(to, currency, amount)?;
        info!("Transferred {} of {:?} from {} to {}", amount, currency, from, to);
        Ok(())
    }

    /// Gets the balance of a particular currency for a given account.
    ///
    /// # Arguments
    ///
    /// * `account` - The account to get the balance for.
    /// * `currency` - The type of currency to get the balance of.
    ///
    /// # Returns
    ///
    /// The balance of the specified currency for the given account.
    ///
    /// # Errors
    ///
    /// Returns `IcnError::Currency` if the currency is not found in the account.
    pub fn get_balance(&self, account: &str, currency: &CurrencyType) -> IcnResult<f64> {
        let balances = self.accounts.get(account)
            .ok_or_else(|| IcnError::Currency("Account not found".into()))?;
        
        let balance = balances.iter()
            .find(|b| &b.currency == currency)
            .ok_or_else(|| IcnError::Currency("Currency not found in account".into()))?;
        
        Ok(balance.amount)
    }

    /// Lists all balances for a given account.
    ///
    /// # Arguments
    ///
    /// * `account` - The account to list the balances for.
    ///
    /// # Returns
    ///
    /// A list of balances for the specified account.
    ///
    /// # Errors
    ///
    /// Returns `IcnError::Currency` if the account is not found.
    pub fn list_balances(&self, account: &str) -> IcnResult<Vec<Balance>> {
        let balances = self.accounts.get(account)
            .ok_or_else(|| IcnError::Currency("Account not found".into()))?;
        Ok(balances.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mint() {
        let mut currency_system = CurrencySystem::new();
        assert!(currency_system.mint("Alice", CurrencyType::BasicNeeds, 100.0).is_ok());
        assert_eq!(currency_system.get_balance("Alice", &CurrencyType::BasicNeeds).unwrap(), 100.0);
    }

    #[test]
    fn test_burn() {
        let mut currency_system = CurrencySystem::new();
        currency_system.mint("Alice", CurrencyType::BasicNeeds, 100.0).unwrap();
        assert!(currency_system.burn("Alice", CurrencyType::BasicNeeds, 50.0).is_ok());
        assert_eq!(currency_system.get_balance("Alice", &CurrencyType::BasicNeeds).unwrap(), 50.0);
        assert!(currency_system.burn("Alice", CurrencyType::BasicNeeds, 60.0).is_err());
    }

    #[test]
    fn test_transfer() {
        let mut currency_system = CurrencySystem::new();
        currency_system.mint("Alice", CurrencyType::BasicNeeds, 100.0).unwrap();
        assert!(currency_system.transfer("Alice", "Bob", CurrencyType::BasicNeeds, 50.0).is_ok());
        assert_eq!(currency_system.get_balance("Alice", &CurrencyType::BasicNeeds).unwrap(), 50.0);
        assert_eq!(currency_system.get_balance("Bob", &CurrencyType::BasicNeeds).unwrap(), 50.0);
        assert!(currency_system.transfer("Alice", "Bob", CurrencyType::BasicNeeds, 60.0).is_err());
    }

    #[test]
    fn test_get_balance() {
        let mut currency_system = CurrencySystem::new();
        currency_system.mint("Alice", CurrencyType::BasicNeeds, 100.0).unwrap();
        assert_eq!(currency_system.get_balance("Alice", &CurrencyType::BasicNeeds).unwrap(), 100.0);
        assert!(currency_system.get_balance("Alice", &CurrencyType::Education).is_err());
        assert!(currency_system.get_balance("Bob", &CurrencyType::BasicNeeds).is_err());
    }

    #[test]
    fn test_list_balances() {
        let mut currency_system = CurrencySystem::new();
        currency_system.mint("Alice", CurrencyType::BasicNeeds, 100.0).unwrap();
        currency_system.mint("Alice", CurrencyType::Education, 50.0).unwrap();
        let balances = currency_system.list_balances("Alice").unwrap();
        assert_eq!(balances.len(), 2);
        assert_eq!(balances[0].amount, 100.0);
        assert_eq!(balances[1].amount, 50.0);
        assert!(currency_system.list_balances("Bob").is_err());
    }
}

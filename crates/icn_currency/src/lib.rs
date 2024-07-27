use icn_common::{Transaction, CurrencyType, IcnResult, IcnError};
use std::collections::HashMap;

pub struct Account {
    balances: HashMap<CurrencyType, f64>,
}

impl Account {
    pub fn new() -> Self {
        Account {
            balances: HashMap::new(),
        }
    }

    pub fn get_balance(&self, currency_type: &CurrencyType) -> f64 {
        *self.balances.get(currency_type).unwrap_or(&0.0)
    }

    pub fn update_balance(&mut self, currency_type: &CurrencyType, amount: f64) -> IcnResult<()> {
        let balance = self.balances.entry(currency_type.clone()).or_insert(0.0);
        *balance += amount;
        if *balance < 0.0 {
            return Err(IcnError::Currency("Insufficient funds".into()));
        }
        Ok(())
    }
}

pub struct CurrencySystem {
    accounts: HashMap<String, Account>,
}

impl CurrencySystem {
    pub fn new() -> Self {
        CurrencySystem {
            accounts: HashMap::new(),
        }
    }

    pub fn create_account(&mut self, account_id: String) -> IcnResult<()> {
        if self.accounts.contains_key(&account_id) {
            return Err(IcnError::Currency("Account already exists".into()));
        }
        self.accounts.insert(account_id, Account::new());
        Ok(())
    }

    pub fn get_balance(&self, account_id: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        let account = self.accounts.get(account_id)
            .ok_or_else(|| IcnError::Currency("Account not found".into()))?;
        Ok(account.get_balance(currency_type))
    }

    pub fn process_transaction(&mut self, transaction: &Transaction) -> IcnResult<()> {
        let from_account = self.accounts.get_mut(&transaction.from)
            .ok_or_else(|| IcnError::Currency("Sender account not found".into()))?;
        
        from_account.update_balance(&transaction.currency_type, -transaction.amount)?;

        let to_account = self.accounts.get_mut(&transaction.to)
            .ok_or_else(|| IcnError::Currency("Recipient account not found".into()))?;
        
        to_account.update_balance(&transaction.currency_type, transaction.amount)?;

        Ok(())
    }

    pub fn mint(&mut self, account_id: &str, currency_type: &CurrencyType, amount: f64) -> IcnResult<()> {
        let account = self.accounts.get_mut(account_id)
            .ok_or_else(|| IcnError::Currency("Account not found".into()))?;
        account.update_balance(currency_type, amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_operations() {
        let mut account = Account::new();
        assert_eq!(account.get_balance(&CurrencyType::BasicNeeds), 0.0);

        account.update_balance(&CurrencyType::BasicNeeds, 100.0).unwrap();
        assert_eq!(account.get_balance(&CurrencyType::BasicNeeds), 100.0);

        account.update_balance(&CurrencyType::BasicNeeds, -50.0).unwrap();
        assert_eq!(account.get_balance(&CurrencyType::BasicNeeds), 50.0);

        assert!(account.update_balance(&CurrencyType::BasicNeeds, -100.0).is_err());
    }

    #[test]
    fn test_currency_system() {
        let mut system = CurrencySystem::new();

        system.create_account("Alice".to_string()).unwrap();
        system.create_account("Bob".to_string()).unwrap();

        system.mint("Alice", &CurrencyType::BasicNeeds, 100.0).unwrap();
        assert_eq!(system.get_balance("Alice", &CurrencyType::BasicNeeds).unwrap(), 100.0);

        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 12345,
            signature: None,
        };

        system.process_transaction(&transaction).unwrap();

        assert_eq!(system.get_balance("Alice", &CurrencyType::BasicNeeds).unwrap(), 50.0);
        assert_eq!(system.get_balance("Bob", &CurrencyType::BasicNeeds).unwrap(), 50.0);

        // Test insufficient funds
        let invalid_transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 100.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 12346,
            signature: None,
        };

        assert!(system.process_transaction(&invalid_transaction).is_err());

        // Test non-existent account
        assert!(system.get_balance("Charlie", &CurrencyType::BasicNeeds).is_err());

        // Test multiple currency types
        system.mint("Alice", &CurrencyType::Education, 200.0).unwrap();
        assert_eq!(system.get_balance("Alice", &CurrencyType::Education).unwrap(), 200.0);

        let education_transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 75.0,
            currency_type: CurrencyType::Education,
            timestamp: 12347,
            signature: None,
        };

        system.process_transaction(&education_transaction).unwrap();

        assert_eq!(system.get_balance("Alice", &CurrencyType::Education).unwrap(), 125.0);
        assert_eq!(system.get_balance("Bob", &CurrencyType::Education).unwrap(), 75.0);

        // Ensure BasicNeeds balances are unchanged
        assert_eq!(system.get_balance("Alice", &CurrencyType::BasicNeeds).unwrap(), 50.0);
        assert_eq!(system.get_balance("Bob", &CurrencyType::BasicNeeds).unwrap(), 50.0);
    }

    #[test]
    fn test_create_duplicate_account() {
        let mut system = CurrencySystem::new();

        system.create_account("Alice".to_string()).unwrap();
        assert!(system.create_account("Alice".to_string()).is_err());
    }

    #[test]
    fn test_mint_to_nonexistent_account() {
        let mut system = CurrencySystem::new();

        assert!(system.mint("Alice", &CurrencyType::BasicNeeds, 100.0).is_err());
    }
}
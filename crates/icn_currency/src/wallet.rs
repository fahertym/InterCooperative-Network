use super::CurrencyType;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use log::{info, error, debug};
use icn_core::error::{Error, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct Wallet {
    balances: HashMap<CurrencyType, f64>,
}

impl Wallet {
    pub fn new() -> Self {
        debug!("Creating new Wallet");
        Wallet {
            balances: HashMap::new(),
        }
    }

    pub fn deposit(&mut self, currency_type: CurrencyType, amount: f64) -> Result<()> {
        if amount < 0.0 {
            return Err(Error::CurrencyError("Cannot deposit negative amount".to_string()));
        }
        *self.balances.entry(currency_type.clone()).or_insert(0.0) += amount;
        info!("Deposited {} of {:?} into wallet", amount, currency_type);
        Ok(())
    }

    pub fn withdraw(&mut self, currency_type: CurrencyType, amount: f64) -> Result<()> {
        if amount < 0.0 {
            return Err(Error::CurrencyError("Cannot withdraw negative amount".to_string()));
        }
        let balance = self.balances.entry(currency_type.clone()).or_insert(0.0);
        if *balance < amount {
            error!("Insufficient balance for withdrawal. Requested: {}, Available: {}", amount, balance);
            return Err(Error::CurrencyError(format!("Insufficient balance for {:?}", currency_type)));
        }
        *balance -= amount;
        info!("Withdrawn {} of {:?} from wallet", amount, currency_type);
        Ok(())
    }

    pub fn get_balance(&self, currency_type: &CurrencyType) -> f64 {
        *self.balances.get(currency_type).unwrap_or(&0.0)
    }

    pub fn print_balances(&self) {
        info!("Wallet Balances:");
        for (currency_type, balance) in &self.balances {
            info!("{:?}: {}", currency_type, balance);
        }
    }

    pub fn transfer(&mut self, to: &mut Wallet, currency_type: CurrencyType, amount: f64) -> Result<()> {
        self.withdraw(currency_type.clone(), amount)?;
        to.deposit(currency_type, amount)?;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_operations() {
        let mut wallet = Wallet::new();
        
        // Test deposit
        assert!(wallet.deposit(CurrencyType::BasicNeeds, 100.0).is_ok());
        assert_eq!(wallet.get_balance(&CurrencyType::BasicNeeds), 100.0);

        // Test withdraw
        assert!(wallet.withdraw(CurrencyType::BasicNeeds, 50.0).is_ok());
        assert_eq!(wallet.get_balance(&CurrencyType::BasicNeeds), 50.0);

        // Test insufficient balance
        assert!(wallet.withdraw(CurrencyType::BasicNeeds, 100.0).is_err());

        // Test transfer
        let mut wallet2 = Wallet::new();
        assert!(wallet.transfer(&mut wallet2, CurrencyType::BasicNeeds, 25.0).is_ok());
        assert_eq!(wallet.get_balance(&CurrencyType::BasicNeeds), 25.0);
        assert_eq!(wallet2.get_balance(&CurrencyType::BasicNeeds), 25.0);
    }
}
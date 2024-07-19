use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
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

impl std::fmt::Display for CurrencyType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CurrencyType::Custom(name) => write!(f, "Custom({})", name),
            _ => write!(f, "{:?}", self),
        }
    }
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

    pub fn mint(&mut self, amount: f64) {
        self.total_supply += amount;
        self.last_issuance = Utc::now();
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
    pub currencies: std::collections::HashMap<CurrencyType, Currency>,
}

impl CurrencySystem {
    pub fn new() -> Self {
        let mut system = CurrencySystem {
            currencies: std::collections::HashMap::new(),
        };
        
        system.add_currency(CurrencyType::BasicNeeds, 1_000_000.0, 0.01);
        system.add_currency(CurrencyType::Education, 500_000.0, 0.005);
        system.add_currency(CurrencyType::Environmental, 750_000.0, 0.008);
        system.add_currency(CurrencyType::Community, 250_000.0, 0.003);
        system.add_currency(CurrencyType::Volunteer, 100_000.0, 0.002);
        system.add_currency(CurrencyType::Storage, 1_000_000.0, 0.01);
        system.add_currency(CurrencyType::Processing, 500_000.0, 0.005);
        system.add_currency(CurrencyType::Energy, 750_000.0, 0.008);
        system.add_currency(CurrencyType::Luxury, 100_000.0, 0.001);
        system.add_currency(CurrencyType::Service, 200_000.0, 0.004);

        system
    }

    pub fn add_currency(&mut self, currency_type: CurrencyType, initial_supply: f64, issuance_rate: f64) {
        let currency = Currency::new(currency_type.clone(), initial_supply, issuance_rate);
        self.currencies.insert(currency_type.clone(), currency);
    }

    pub fn get_currency(&self, currency_type: &CurrencyType) -> Option<&Currency> {
        self.currencies.get(currency_type)
    }

    pub fn get_currency_mut(&mut self, currency_type: &CurrencyType) -> Option<&mut Currency> {
        self.currencies.get_mut(currency_type)
    }

    pub fn create_custom_currency(&mut self, name: String, initial_supply: f64, issuance_rate: f64) -> Result<()> {
        let currency_type = CurrencyType::Custom(name.clone());
        if self.currencies.contains_key(&currency_type) {
            return Err(Error {
                message: format!("Currency '{}' already exists", name),
            });
        }
        self.add_currency(currency_type, initial_supply, issuance_rate);
        Ok(())
    }

    pub fn adaptive_issuance(&mut self) {
        let now = Utc::now();
        for currency in self.currencies.values_mut() {
            let time_since_last_issuance = now.signed_duration_since(currency.last_issuance);
            let issuance_amount = currency.total_supply * currency.issuance_rate * time_since_last_issuance.num_milliseconds() as f64 / 86_400_000.0; // Daily rate
            currency.mint(issuance_amount);
        }
    }

    pub fn print_currency_supplies(&self) {
        for (currency_type, currency) in &self.currencies {
            println!("{:?}: {}", currency_type, currency.total_supply);
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Wallet {
    balances: std::collections::HashMap<CurrencyType, f64>,
}

impl Wallet {
    pub fn new() -> Self {
        Wallet {
            balances: std::collections::HashMap::new(),
        }
    }

    pub fn deposit(&mut self, currency_type: CurrencyType, amount: f64) {
        *self.balances.entry(currency_type.clone()).or_insert(0.0) += amount;
    }

    pub fn withdraw(&mut self, currency_type: CurrencyType, amount: f64) -> Result<()> {
        let balance = self.balances.entry(currency_type.clone()).or_insert(0.0);
        if *balance < amount {
            return Err(Error {
                message: format!("Insufficient balance for {:?}", currency_type),
            });
        }
        *balance -= amount;
        Ok(())
    }

    pub fn get_balance(&self, currency_type: &CurrencyType) -> f64 {
        *self.balances.get(currency_type).unwrap_or(&0.0)
    }

    pub fn print_balances(&self) {
        for (currency_type, balance) in &self.balances {
            println!("{:?}: {}", currency_type, balance);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_mint_burn() {
        let mut currency = Currency::new(CurrencyType::BasicNeeds, 1000.0, 0.01);
        currency.mint(100.0);
        assert_eq!(currency.total_supply, 1100.0);
        assert!(currency.burn(200.0).is_ok());
        assert_eq!(currency.total_supply, 900.0);
        assert!(currency.burn(1000.0).is_err());
    }

    #[test]
    fn test_currency_system() {
        let mut system = CurrencySystem::new();
        system.add_currency(CurrencyType::Custom("TestCoin".to_string()), 500.0, 0.02);
        let currency = system.get_currency(&CurrencyType::Custom("TestCoin".to_string())).unwrap();
        assert_eq!(currency.total_supply, 500.0);
    }

    #[test]
    fn test_wallet_deposit_withdraw() {
        let mut wallet = Wallet::new();
        wallet.deposit(CurrencyType::BasicNeeds, 100.0);
        assert_eq!(wallet.get_balance(&CurrencyType::BasicNeeds), 100.0);
        assert!(wallet.withdraw(CurrencyType::BasicNeeds, 50.0).is_ok());
        assert_eq!(wallet.get_balance(&CurrencyType::BasicNeeds), 50.0);
        assert!(wallet.withdraw(CurrencyType::BasicNeeds, 100.0).is_err());
    }

    #[test]
    fn test_wallet_balances() {
        let mut wallet = Wallet::new();
        wallet.deposit(CurrencyType::BasicNeeds, 200.0);
        wallet.deposit(CurrencyType::Education, 150.0);
        assert_eq!(wallet.get_balance(&CurrencyType::BasicNeeds), 200.0);
        assert_eq!(wallet.get_balance(&CurrencyType::Education), 150.0);
    }
}

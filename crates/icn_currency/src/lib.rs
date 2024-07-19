use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt;

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

impl fmt::Display for CurrencyType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CurrencyType::Custom(name) => write!(f, "Custom({})", name),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Currency {
    pub currency_type: CurrencyType,
    pub total_supply: f64,
    pub creation_date: DateTime<Utc>,
    pub last_issuance: DateTime<Utc>,
    pub issuance_rate: f64,
}

impl Currency {
    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn mint(&mut self, amount: f64) {
        self.total_supply += amount;
        self.last_issuance = Utc::now();
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurrencySystem {
    pub currencies: HashMap<CurrencyType, Currency>,
}

impl CurrencySystem {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let mut system = CurrencySystem {
            currencies: HashMap::new(),
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

    #[allow(dead_code)]
    pub fn add_currency(&mut self, currency_type: CurrencyType, initial_supply: f64, issuance_rate: f64) {
        let currency = Currency::new(currency_type.clone(), initial_supply, issuance_rate);
        self.currencies.insert(currency_type, currency);
    }

    #[allow(dead_code)]
    pub fn get_currency(&self, currency_type: &CurrencyType) -> Option<&Currency> {
        self.currencies.get(currency_type)
    }

    #[allow(dead_code)]
    pub fn get_currency_mut(&mut self, currency_type: &CurrencyType) -> Option<&mut Currency> {
        self.currencies.get_mut(currency_type)
    }

    #[allow(dead_code)]
    pub fn create_custom_currency(&mut self, name: String, initial_supply: f64, issuance_rate: f64) -> Result<(), String> {
        let currency_type = CurrencyType::Custom(name.clone());
        if self.currencies.contains_key(&currency_type) {
            return Err(format!("Currency '{}' already exists", name));
        }
        self.add_currency(currency_type, initial_supply, issuance_rate);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn adaptive_issuance(&mut self) {
        let now = Utc::now();
        for currency in self.currencies.values_mut() {
            let time_since_last_issuance = now.signed_duration_since(currency.last_issuance);
            let issuance_amount = currency.total_supply * currency.issuance_rate * time_since_last_issuance.num_milliseconds() as f64 / 86_400_000.0; // Daily rate
            currency.mint(issuance_amount);
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Wallet {
    balances: HashMap<CurrencyType, f64>,
}

impl Wallet {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Wallet {
            balances: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn deposit(&mut self, currency_type: CurrencyType, amount: f64) {
        *self.balances.entry(currency_type).or_insert(0.0) += amount;
    }

    #[allow(dead_code)]
    pub fn withdraw(&mut self, currency_type: CurrencyType, amount: f64) -> Result<(), String> {
        let balance = self.balances.entry(currency_type.clone()).or_insert(0.0);
        if *balance < amount {
            return Err(format!("Insufficient balance for {:?}", currency_type));
        }
        *balance -= amount;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_balance(&self, currency_type: &CurrencyType) -> f64 {
        *self.balances.get(currency_type).unwrap_or(&0.0)
    }
}
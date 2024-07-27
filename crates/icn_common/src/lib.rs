use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub currency_type: CurrencyType,
    pub timestamp: i64,
    pub signature: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
}

impl Block {
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.index.to_be_bytes());
        hasher.update(self.timestamp.to_be_bytes());
        for transaction in &self.transactions {
            hasher.update(transaction.from.as_bytes());
            hasher.update(transaction.to.as_bytes());
            hasher.update(transaction.amount.to_be_bytes());
            hasher.update(format!("{:?}", transaction.currency_type).as_bytes());
            hasher.update(transaction.timestamp.to_be_bytes());
        }
        hasher.update(self.previous_hash.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub proposer: String,
    pub created_at: DateTime<Utc>,
    pub voting_ends_at: DateTime<Utc>,
    pub status: ProposalStatus,
    pub proposal_type: ProposalType,
    pub category: ProposalCategory,
    pub required_quorum: f64,
    pub execution_timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalType {
    Constitutional,
    EconomicAdjustment,
    NetworkUpgrade,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalCategory {
    Economic,
    Technical,
    Social,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CurrencyType {
    BasicNeeds,
    Education,
    Environmental,
    Community,
    Custom(String),
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

    pub fn mint(&mut self, amount: f64) -> IcnResult<()> {
        self.total_supply += amount;
        self.last_issuance = Utc::now();
        Ok(())
    }

    pub fn burn(&mut self, amount: f64) -> IcnResult<()> {
        if amount > self.total_supply {
            return Err(IcnError::Currency("Insufficient supply to burn".into()));
        }
        self.total_supply -= amount;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurrencySystem {
    pub currencies: HashMap<CurrencyType, Currency>,
}

impl CurrencySystem {
    pub fn new() -> Self {
        CurrencySystem {
            currencies: HashMap::new(),
        }
    }

    pub fn add_currency(&mut self, currency_type: CurrencyType, initial_supply: f64, issuance_rate: f64) {
        let currency = Currency::new(currency_type.clone(), initial_supply, issuance_rate);
        self.currencies.insert(currency_type, currency);
    }

    pub fn get_currency(&self, currency_type: &CurrencyType) -> Option<&Currency> {
        self.currencies.get(currency_type)
    }

    pub fn get_currency_mut(&mut self, currency_type: &CurrencyType) -> Option<&mut Currency> {
        self.currencies.get_mut(currency_type)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum IcnError {
    #[error("Blockchain error: {0}")]
    Blockchain(String),
    #[error("Consensus error: {0}")]
    Consensus(String),
    #[error("Currency error: {0}")]
    Currency(String),
    #[error("Governance error: {0}")]
    Governance(String),
    #[error("Identity error: {0}")]
    Identity(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Smart contract error: {0}")]
    SmartContract(String),
    #[error("ZKP error: {0}")]
    ZKP(String),
    #[error("VM error: {0}")]
    VM(String),
}

pub type IcnResult<T> = std::result::Result<T, IcnError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_mint_burn() {
        let mut currency = Currency::new(CurrencyType::BasicNeeds, 1000.0, 0.01);
        assert!(currency.mint(100.0).is_ok());
        assert_eq!(currency.total_supply, 1100.0);
        assert!(currency.burn(200.0).is_ok());
        assert_eq!(currency.total_supply, 900.0);
        assert!(currency.burn(1000.0).is_err());
    }

    #[test]
    fn test_currency_system() {
        let mut system = CurrencySystem::new();
        system.add_currency(CurrencyType::BasicNeeds, 1000.0, 0.01);
        system.add_currency(CurrencyType::Education, 500.0, 0.005);

        assert!(system.get_currency(&CurrencyType::BasicNeeds).is_some());
        assert!(system.get_currency(&CurrencyType::Education).is_some());
        assert!(system.get_currency(&CurrencyType::Environmental).is_none());
    }
}
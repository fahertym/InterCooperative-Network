use crate::entities::{Resource, Labor};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    ResourceTrade,
    LaborExchange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub transaction_type: TransactionType,
    pub from_id: String,
    pub to_id: String,
    pub resource: Option<Resource>,
    pub labor: Option<Labor>,
}

impl Transaction {
    pub fn new(
        transaction_type: TransactionType,
        from_id: String,
        to_id: String,
        resource: Option<Resource>,
        labor: Option<Labor>,
    ) -> Self {
        Transaction {
            transaction_type,
            from_id,
            to_id,
            resource,
            labor,
        }
    }
}

pub type TransactionResult = Result<(), String>;

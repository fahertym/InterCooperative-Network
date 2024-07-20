use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bond {
    pub bond_id: String,
    pub name: String,
    pub description: String,
    pub issuer: String,
    pub face_value: f64,
    pub maturity_date: DateTime<Utc>,
    pub interest_rate: f64,
    pub owner: String,
}

impl Bond {
    pub fn new(bond_id: String, name: String, description: String, issuer: String, face_value: f64, maturity_date: DateTime<Utc>, interest_rate: f64, owner: String) -> Self {
        Bond {
            bond_id,
            name,
            description,
            issuer,
            face_value,
            maturity_date,
            interest_rate,
            owner,
        }
    }

    pub fn transfer(&mut self, new_owner: String) {
        self.owner = new_owner;
    }

    pub fn calculate_current_value(&self, current_date: DateTime<Utc>) -> f64 {
        if current_date >= self.maturity_date {
            self.face_value
        } else {
            let years_to_maturity = (self.maturity_date - current_date).num_days() as f64 / 365.0;
            self.face_value * (1.0 + self.interest_rate).powf(years_to_maturity)
        }
    }
}

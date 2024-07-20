use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vote {
    pub voter: String,
    pub proposal_id: String,
    pub in_favor: bool,
    pub weight: f64,
    pub timestamp: DateTime<Utc>,
}

impl Vote {
    pub fn new(voter: String, proposal_id: String, in_favor: bool, weight: f64) -> Self {
        Vote {
            voter,
            proposal_id,
            in_favor,
            weight,
            timestamp: Utc::now(),
        }
    }
}

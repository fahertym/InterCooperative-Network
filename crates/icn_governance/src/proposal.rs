use crate::democracy::{ProposalCategory, ProposalType, ProposalStatus};
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
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

impl Proposal {
    pub fn new(
        id: String,
        title: String,
        description: String,
        proposer: String,
        voting_period: Duration,
        proposal_type: ProposalType,
        category: ProposalCategory,
        required_quorum: f64,
        execution_timestamp: Option<DateTime<Utc>>,
    ) -> Self {
        let now = Utc::now();
        Proposal {
            id,
            title,
            description,
            proposer,
            created_at: now,
            voting_ends_at: now + voting_period,
            status: ProposalStatus::Active,
            proposal_type,
            category,
            required_quorum,
            execution_timestamp,
        }
    }
}

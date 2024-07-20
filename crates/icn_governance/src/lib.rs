mod democracy;
mod proposal;
mod voting;

pub use democracy::{DemocraticSystem, ProposalCategory, ProposalType, ProposalStatus};
pub use proposal::Proposal;
pub use voting::Vote;

use icn_utils::error::{Error, Result};
use chrono::{DateTime, Utc, Duration};

pub struct GovernanceSystem {
    democratic_system: DemocraticSystem,
}

impl GovernanceSystem {
    pub fn new() -> Self {
        GovernanceSystem {
            democratic_system: DemocraticSystem::new(),
        }
    }

    pub fn create_proposal(
        &mut self,
        title: String,
        description: String,
        proposer: String,
        voting_period: Duration,
        proposal_type: ProposalType,
        category: ProposalCategory,
        required_quorum: f64,
        execution_timestamp: Option<DateTime<Utc>>,
    ) -> Result<String> {
        self.democratic_system.create_proposal(
            title,
            description,
            proposer,
            voting_period,
            proposal_type,
            category,
            required_quorum,
            execution_timestamp,
        )
    }

    pub fn vote(&mut self, voter: String, proposal_id: String, in_favor: bool, weight: f64) -> Result<()> {
        self.democratic_system.vote(voter, proposal_id, in_favor, weight)
    }

    pub fn tally_votes(&mut self, proposal_id: &str) -> Result<()> {
        self.democratic_system.tally_votes(proposal_id)
    }

    pub fn get_proposal(&self, proposal_id: &str) -> Option<&Proposal> {
        self.democratic_system.get_proposal(proposal_id)
    }

    pub fn list_active_proposals(&self) -> Vec<&Proposal> {
        self.democratic_system.list_active_proposals()
    }

    pub fn mark_as_implemented(&mut self, proposal_id: &str) -> Result<()> {
        self.democratic_system.mark_as_implemented(proposal_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_governance_system() {
        let mut gov_system = GovernanceSystem::new();

        // Create a proposal
        let proposal_id = gov_system.create_proposal(
            "Test Proposal".to_string(),
            "This is a test proposal".to_string(),
            "Alice".to_string(),
            Duration::days(7),
            ProposalType::Constitutional,
            ProposalCategory::Technical,
            0.5,
            None,
        ).unwrap();

        // Vote on the proposal
        assert!(gov_system.vote("Bob".to_string(), proposal_id.clone(), true, 1.0).is_ok());
        assert!(gov_system.vote("Charlie".to_string(), proposal_id.clone(), false, 1.0).is_ok());
        assert!(gov_system.vote("David".to_string(), proposal_id.clone(), true, 1.0).is_ok());

        // Tally votes
        assert!(gov_system.tally_votes(&proposal_id).is_ok());

        // Check proposal status
        let proposal = gov_system.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Passed);

        // Mark as implemented
        assert!(gov_system.mark_as_implemented(&proposal_id).is_ok());
        let updated_proposal = gov_system.get_proposal(&proposal_id).unwrap();
        assert_eq!(updated_proposal.status, ProposalStatus::Implemented);
    }
}

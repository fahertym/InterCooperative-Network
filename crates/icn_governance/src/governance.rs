// File: crates/icn_governance/src/governance.rs

use crate::{Proposal, ProposalStatus, Vote, ProposalType, ProposalCategory};
use icn_common::{IcnResult, IcnError};
use std::collections::HashMap;
use chrono::Utc;
use log::{info, warn};

/// Represents the governance system managing proposals and votes.
pub struct GovernanceSystem {
    pub proposals: HashMap<String, Proposal>,
    pub votes: HashMap<String, Vec<Vote>>,
}

impl GovernanceSystem {
    /// Creates a new Governance system.
    pub fn new() -> Self {
        GovernanceSystem {
            proposals: HashMap::new(),
            votes: HashMap::new(),
        }
    }

    /// Creates a new proposal in the governance system.
    ///
    /// # Arguments
    ///
    /// * `proposal` - The proposal to be created.
    ///
    /// # Returns
    ///
    /// The ID of the created proposal.
    ///
    /// # Errors
    ///
    /// Returns `IcnError::Governance` if the proposal already exists.
    pub fn create_proposal(&mut self, proposal: Proposal) -> IcnResult<String> {
        if self.proposals.contains_key(&proposal.id) {
            return Err(IcnError::Governance("Proposal ID already exists".into()));
        }
        let proposal_id = proposal.id.clone();
        self.proposals.insert(proposal_id.clone(), proposal);
        self.votes.insert(proposal_id.clone(), Vec::new());
        Ok(proposal_id)
    }

    pub fn get_proposal(&self, proposal_id: &str) -> IcnResult<&Proposal> {
        self.proposals.get(proposal_id)
            .ok_or_else(|| IcnError::Governance("Proposal not found".into()))
    }

    pub fn vote_on_proposal(&mut self, proposal_id: &str, voter: String, in_favor: bool, weight: f64) -> IcnResult<()> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or_else(|| IcnError::Governance("Proposal not found".into()))?;

        if proposal.status != ProposalStatus::Active {
            return Err(IcnError::Governance("Proposal is not active".into()));
        }

        if Utc::now() > proposal.voting_ends_at {
            return Err(IcnError::Governance("Voting period has ended".into()));
        }

        let votes = self.votes.get_mut(proposal_id)
            .ok_or_else(|| IcnError::Governance("Votes not found for proposal".into()))?;

        if votes.iter().any(|v| v.voter == voter) {
            return Err(IcnError::Governance("Voter has already voted on this proposal".into()));
        }

        votes.push(Vote::new(voter, proposal_id.to_string(), in_favor, weight));
        Ok(())
    }

    pub fn finalize_proposal(&mut self, proposal_id: &str) -> IcnResult<ProposalStatus> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or_else(|| IcnError::Governance("Proposal not found".into()))?;

        if proposal.status != ProposalStatus::Active {
            return Err(IcnError::Governance("Proposal is not active".into()));
        }

        if Utc::now() < proposal.voting_ends_at {
            return Err(IcnError::Governance("Voting period has not ended yet".into()));
        }

        let votes = self.votes.get(proposal_id)
            .ok_or_else(|| IcnError::Governance("Votes not found for proposal".into()))?;

        let total_votes = votes.len() as f64;
        let votes_in_favor = votes.iter().filter(|v| v.in_favor).count() as f64;

        if total_votes == 0.0 || total_votes / proposal.required_quorum < 1.0 {
            proposal.status = ProposalStatus::Rejected;
        } else if votes_in_favor / total_votes > 0.5 {
            proposal.status = ProposalStatus::Passed;
        } else {
            proposal.status = ProposalStatus::Rejected;
        }

        Ok(proposal.status.clone())
    }

    pub fn list_active_proposals(&self) -> Vec<&Proposal> {
        self.proposals.values()
            .filter(|p| p.status == ProposalStatus::Active)
            .collect()
    }
}

impl Default for GovernanceSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn create_test_proposal(id: &str) -> Proposal {
        Proposal {
            id: id.to_string(),
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            proposer: "Alice".to_string(),
            created_at: Utc::now(),
            voting_ends_at: Utc::now() + Duration::days(7),
            status: ProposalStatus::Active,
            proposal_type: ProposalType::Constitutional,
            category: ProposalCategory::Technical,
            required_quorum: 0.5,
            execution_timestamp: None,
        }
    }

    #[test]
    fn test_create_proposal() {
        let mut gov_system = GovernanceSystem::new();
        let proposal = create_test_proposal("prop1");
        let proposal_id = gov_system.create_proposal(proposal).unwrap();
        assert_eq!(proposal_id, "prop1");
        assert!(gov_system.get_proposal("prop1").is_ok());
    }

    #[test]
    fn test_vote_on_proposal() {
        let mut gov_system = GovernanceSystem::new();
        let proposal = create_test_proposal("prop1");
        gov_system.create_proposal(proposal).unwrap();

        assert!(gov_system.vote_on_proposal("prop1", "Alice".to_string(), true, 1.0).is_ok());
        assert!(gov_system.vote_on_proposal("prop1", "Bob".to_string(), false, 1.0).is_ok());

        // Test duplicate vote
        assert!(gov_system.vote_on_proposal("prop1", "Alice".to_string(), false, 1.0).is_err());

        // Test vote on non-existent proposal
        assert!(gov_system.vote_on_proposal("prop2", "Charlie".to_string(), true, 1.0).is_err());
    }

    #[test]
    fn test_finalize_proposal() {
        let mut gov_system = GovernanceSystem::new();
        let mut proposal = create_test_proposal("prop1");
        proposal.voting_ends_at = Utc::now() - Duration::hours(1); // Set voting period to have ended
        gov_system.create_proposal(proposal).unwrap();

        gov_system.vote_on_proposal("prop1", "Alice".to_string(), true, 1.0).unwrap();
        gov_system.vote_on_proposal("prop1", "Bob".to_string(), true, 1.0).unwrap();
        gov_system.vote_on_proposal("prop1", "Charlie".to_string(), false, 1.0).unwrap();

        let result = gov_system.finalize_proposal("prop1").unwrap();
        assert_eq!(result, ProposalStatus::Passed);

        // Test finalizing an already finalized proposal
        assert!(gov_system.finalize_proposal("prop1").is_err());
    }

    #[test]
    fn test_list_active_proposals() {
        let mut gov_system = GovernanceSystem::new();
        let proposal1 = create_test_proposal("prop1");
        let proposal2 = create_test_proposal("prop2");
        let mut proposal3 = create_test_proposal("prop3");
        proposal3.status = ProposalStatus::Passed;

        gov_system.create_proposal(proposal1).unwrap();
        gov_system.create_proposal(proposal2).unwrap();
        gov_system.create_proposal(proposal3).unwrap();

        let active_proposals = gov_system.list_active_proposals();
        assert_eq!(active_proposals.len(), 2);
        assert!(active_proposals.iter().any(|p| p.id == "prop1"));
        assert!(active_proposals.iter().any(|p| p.id == "prop2"));
    }
}

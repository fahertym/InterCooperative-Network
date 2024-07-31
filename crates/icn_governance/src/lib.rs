// File: icn_governance/src/lib.rs

use icn_common::{IcnResult, IcnError};
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: String,
    pub proposal_id: String,
    pub in_favor: bool,
    pub weight: f64,
    pub timestamp: DateTime<Utc>,
}

pub struct GovernanceSystem {
    proposals: HashMap<String, Proposal>,
    votes: HashMap<String, Vec<Vote>>,
}

impl GovernanceSystem {
    pub fn new() -> Self {
        GovernanceSystem {
            proposals: HashMap::new(),
            votes: HashMap::new(),
        }
    }

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

        votes.push(Vote {
            voter,
            proposal_id: proposal_id.to_string(),
            in_favor,
            weight,
            timestamp: Utc::now(),
        });

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

        let total_votes: f64 = votes.iter().map(|v| v.weight).sum();
        let votes_in_favor: f64 = votes.iter().filter(|v| v.in_favor).map(|v| v.weight).sum();

        if total_votes < proposal.required_quorum {
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

    pub fn mark_as_executed(&mut self, proposal_id: &str) -> IcnResult<()> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or_else(|| IcnError::Governance("Proposal not found".into()))?;
        
        if proposal.status != ProposalStatus::Passed {
            return Err(IcnError::Governance("Proposal has not passed".into()));
        }

        proposal.status = ProposalStatus::Executed;
        proposal.execution_timestamp = Some(Utc::now());
        Ok(())
    }

    pub fn get_votes(&self, proposal_id: &str) -> IcnResult<&Vec<Vote>> {
        self.votes.get(proposal_id)
            .ok_or_else(|| IcnError::Governance("Votes not found for proposal".into()))
    }

    pub fn get_proposal_result(&self, proposal_id: &str) -> IcnResult<(f64, f64)> {
        let votes = self.get_votes(proposal_id)?;
        let total_votes: f64 = votes.iter().map(|v| v.weight).sum();
        let votes_in_favor: f64 = votes.iter().filter(|v| v.in_favor).map(|v| v.weight).sum();
        Ok((votes_in_favor, total_votes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_proposal() -> Proposal {
        Proposal {
            id: "test_proposal".to_string(),
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            proposer: "Alice".to_string(),
            created_at: Utc::now(),
            voting_ends_at: Utc::now() + Duration::days(7),
            status: ProposalStatus::Active,
            proposal_type: ProposalType::Constitutional,
            category: ProposalCategory::Economic,
            required_quorum: 0.5,
            execution_timestamp: None,
        }
    }

    #[test]
    fn test_create_proposal() {
        let mut gov_system = GovernanceSystem::new();
        let proposal = create_test_proposal();
        let proposal_id = gov_system.create_proposal(proposal.clone()).unwrap();
        assert_eq!(proposal_id, "test_proposal");
        assert!(gov_system.get_proposal("test_proposal").is_ok());
    }

    #[test]
    fn test_vote_on_proposal() {
        let mut gov_system = GovernanceSystem::new();
        let proposal = create_test_proposal();
        gov_system.create_proposal(proposal).unwrap();

        assert!(gov_system.vote_on_proposal("test_proposal", "Alice".to_string(), true, 1.0).is_ok());
        assert!(gov_system.vote_on_proposal("test_proposal", "Bob".to_string(), false, 1.0).is_ok());

        // Test duplicate vote
        assert!(gov_system.vote_on_proposal("test_proposal", "Alice".to_string(), false, 1.0).is_err());

        // Test vote on non-existent proposal
        assert!(gov_system.vote_on_proposal("non_existent", "Charlie".to_string(), true, 1.0).is_err());
    }

    #[test]
    fn test_finalize_proposal() {
        let mut gov_system = GovernanceSystem::new();
        let mut proposal = create_test_proposal();
        proposal.voting_ends_at = Utc::now() - Duration::hours(1); // Set voting period to have ended
        gov_system.create_proposal(proposal).unwrap();

        gov_system.vote_on_proposal("test_proposal", "Alice".to_string(), true, 1.0).unwrap();
        gov_system.vote_on_proposal("test_proposal", "Bob".to_string(), true, 1.0).unwrap();
        gov_system.vote_on_proposal("test_proposal", "Charlie".to_string(), false, 1.0).unwrap();

        let result = gov_system.finalize_proposal("test_proposal").unwrap();
        assert_eq!(result, ProposalStatus::Passed);

        // Test finalizing an already finalized proposal
        assert!(gov_system.finalize_proposal("test_proposal").is_err());
    }

    #[test]
    fn test_list_active_proposals() {
        let mut gov_system = GovernanceSystem::new();
        let proposal1 = create_test_proposal();
        let mut proposal2 = create_test_proposal();
        proposal2.id = "test_proposal_2".to_string();
        let mut proposal3 = create_test_proposal();
        proposal3.id = "test_proposal_3".to_string();
        proposal3.status = ProposalStatus::Passed;

        gov_system.create_proposal(proposal1).unwrap();
        gov_system.create_proposal(proposal2).unwrap();
        gov_system.create_proposal(proposal3).unwrap();

        let active_proposals = gov_system.list_active_proposals();
        assert_eq!(active_proposals.len(), 2);
        assert!(active_proposals.iter().any(|p| p.id == "test_proposal"));
        assert!(active_proposals.iter().any(|p| p.id == "test_proposal_2"));
    }

    #[test]
    fn test_mark_as_executed() {
        let mut gov_system = GovernanceSystem::new();
        let mut proposal = create_test_proposal();
        proposal.status = ProposalStatus::Passed;
        gov_system.create_proposal(proposal).unwrap();

        assert!(gov_system.mark_as_executed("test_proposal").is_ok());
        let executed_proposal = gov_system.get_proposal("test_proposal").unwrap();
        assert_eq!(executed_proposal.status, ProposalStatus::Executed);
        assert!(executed_proposal.execution_timestamp.is_some());

        // Test marking a non-passed proposal as executed
        let mut proposal2 = create_test_proposal();
        proposal2.id = "test_proposal_2".to_string();
        gov_system.create_proposal(proposal2).unwrap();
        assert!(gov_system.mark_as_executed("test_proposal_2").is_err());
    }

    #[test]
    fn test_get_proposal_result() {
        let mut gov_system = GovernanceSystem::new();
        let proposal = create_test_proposal();
        gov_system.create_proposal(proposal).unwrap();

        gov_system.vote_on_proposal("test_proposal", "Alice".to_string(), true, 1.0).unwrap();
        gov_system.vote_on_proposal("test_proposal", "Bob".to_string(), true, 2.0).unwrap();
        gov_system.vote_on_proposal("test_proposal", "Charlie".to_string(), false, 1.5).unwrap();

        let (votes_in_favor, total_votes) = gov_system.get_proposal_result("test_proposal").unwrap();
        assert_eq!(votes_in_favor, 3.0);
        assert_eq!(total_votes, 4.5);
    }

    #[test]
    fn test_proposal_quorum() {
        let mut gov_system = GovernanceSystem::new();
        let mut proposal = create_test_proposal();
        proposal.required_quorum = 5.0;
        proposal.voting_ends_at = Utc::now() - Duration::hours(1);
        gov_system.create_proposal(proposal).unwrap();

        gov_system.vote_on_proposal("test_proposal", "Alice".to_string(), true, 2.0).unwrap();
        gov_system.vote_on_proposal("test_proposal", "Bob".to_string(), true, 2.0).unwrap();

        let result = gov_system.finalize_proposal("test_proposal").unwrap();
        assert_eq!(result, ProposalStatus::Rejected); // Rejected due to not meeting quorum

        // Test with meeting quorum
        let mut proposal2 = create_test_proposal();
        proposal2.id = "test_proposal_2".to_string();
        proposal2.required_quorum = 5.0;
        proposal2.voting_ends_at = Utc::now() - Duration::hours(1);
        gov_system.create_proposal(proposal2).unwrap();

        gov_system.vote_on_proposal("test_proposal_2", "Alice".to_string(), true, 3.0).unwrap();
        gov_system.vote_on_proposal("test_proposal_2", "Bob".to_string(), true, 3.0).unwrap();

        let result2 = gov_system.finalize_proposal("test_proposal_2").unwrap();
        assert_eq!(result2, ProposalStatus::Passed); // Passed due to meeting quorum and majority
    }
}
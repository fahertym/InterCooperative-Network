use icn_common::{IcnResult, IcnError, Proposal, ProposalStatus, Vote};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use log::{info, warn};

/// Represents the governance system managing proposals and votes.
pub struct Governance {
    pub proposals: HashMap<String, Proposal>,
    pub votes: HashMap<String, Vec<Vote>>,
}

impl Governance {
    /// Creates a new Governance system.
    pub fn new() -> Self {
        Governance {
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
            return Err(IcnError::Governance("Proposal already exists".into()));
        }
        self.proposals.insert(proposal.id.clone(), proposal);
        info!("Created proposal with ID: {}", proposal.id);
        Ok(proposal.id.clone())
    }

    /// Votes on an existing proposal.
    ///
    /// # Arguments
    ///
    /// * `vote` - The vote to be cast.
    ///
    /// # Errors
    ///
    /// Returns `IcnError::Governance` if the proposal is not found.
    pub fn vote_on_proposal(&mut self, vote: Vote) -> IcnResult<()> {
        if !self.proposals.contains_key(&vote.proposal_id) {
            return Err(IcnError::Governance("Proposal not found".into()));
        }
        self.votes.entry(vote.proposal_id.clone()).or_insert_with(Vec::new).push(vote);
        info!("Vote recorded for proposal ID: {}", vote.proposal_id);
        Ok(())
    }

    /// Fetches the status of a given proposal.
    ///
    /// # Arguments
    ///
    /// * `proposal_id` - The ID of the proposal to fetch the status of.
    ///
    /// # Returns
    ///
    /// The status of the specified proposal.
    ///
    /// # Errors
    ///
    /// Returns `IcnError::Governance` if the proposal is not found.
    pub fn get_proposal_status(&self, proposal_id: &str) -> IcnResult<ProposalStatus> {
        let proposal = self.proposals.get(proposal_id).ok_or_else(|| IcnError::Governance("Proposal not found".into()))?;
        Ok(proposal.status.clone())
    }

    /// Closes a proposal and determines its outcome.
    ///
    /// # Arguments
    ///
    /// * `proposal_id` - The ID of the proposal to close.
    ///
    /// # Returns
    ///
    /// The final status of the proposal.
    ///
    /// # Errors
    ///
    /// Returns `IcnError::Governance` if the proposal is not found or if the proposal is already closed.
    pub fn close_proposal(&mut self, proposal_id: &str) -> IcnResult<ProposalStatus> {
        let proposal = self.proposals.get_mut(proposal_id).ok_or_else(|| IcnError::Governance("Proposal not found".into()))?;
        if proposal.status != ProposalStatus::Active {
            return Err(IcnError::Governance("Proposal is already closed".into()));
        }

        let votes = self.votes.get(proposal_id).unwrap_or(&Vec::new());
        let mut in_favor = 0.0;
        let mut against = 0.0;
        for vote in votes {
            if vote.in_favor {
                in_favor += vote.weight;
            } else {
                against += vote.weight;
            }
        }

        if in_favor / (in_favor + against) >= proposal.required_quorum {
            proposal.status = ProposalStatus::Passed;
        } else {
            proposal.status = ProposalStatus::Rejected;
        }

        info!("Closed proposal ID: {} with status: {:?}", proposal_id, proposal.status);
        Ok(proposal.status.clone())
    }

    /// Lists all active proposals.
    ///
    /// # Returns
    ///
    /// A list of all active proposals.
    pub fn list_active_proposals(&self) -> Vec<&Proposal> {
        self.proposals.values().filter(|p| p.status == ProposalStatus::Active).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::{ProposalType, ProposalCategory};

    #[test]
    fn test_create_proposal() {
        let mut governance = Governance::new();
        let proposal = Proposal {
            id: "proposal1".to_string(),
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            proposer: "Alice".to_string(),
            created_at: Utc::now(),
            voting_ends_at: Utc::now() + chrono::Duration::days(7),
            status: ProposalStatus::Active,
            proposal_type: ProposalType::Constitutional,
            category: ProposalCategory::Economic,
            required_quorum: 0.66,
            execution_timestamp: None,
        };
        assert!(governance.create_proposal(proposal).is_ok());
        assert!(governance.create_proposal(proposal).is_err());
    }

    #[test]
    fn test_vote_on_proposal() {
        let mut governance = Governance::new();
        let proposal = Proposal {
            id: "proposal1".to_string(),
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            proposer: "Alice".to_string(),
            created_at: Utc::now(),
            voting_ends_at: Utc::now() + chrono::Duration::days(7),
            status: ProposalStatus::Active,
            proposal_type: ProposalType::Constitutional,
            category: ProposalCategory::Economic,
            required_quorum: 0.66,
            execution_timestamp: None,
        };
        governance.create_proposal(proposal).unwrap();
        let vote = Vote {
            voter: "Bob".to_string(),
            proposal_id: "proposal1".to_string(),
            in_favor: true,
            weight: 1.0,
            timestamp: Utc::now(),
            zkp: None,
        };
        assert!(governance.vote_on_proposal(vote).is_ok());
    }

    #[test]
    fn test_get_proposal_status() {
        let mut governance = Governance::new();
        let proposal = Proposal {
            id: "proposal1".to_string(),
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            proposer: "Alice".to_string(),
            created_at: Utc::now(),
            voting_ends_at: Utc::now() + chrono::Duration::days(7),
            status: ProposalStatus::Active,
            proposal_type: ProposalType::Constitutional,
            category: ProposalCategory::Economic,
            required_quorum: 0.66,
            execution_timestamp: None,
        };
        governance.create_proposal(proposal).unwrap();
        let status = governance.get_proposal_status("proposal1").unwrap();
        assert_eq!(status, ProposalStatus::Active);
    }

    #[test]
    fn test_close_proposal() {
        let mut governance = Governance::new();
        let proposal = Proposal {
            id: "proposal1".to_string(),
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            proposer: "Alice".to_string(),
            created_at: Utc::now(),
            voting_ends_at: Utc::now() + chrono::Duration::days(7),
            status: ProposalStatus::Active,
            proposal_type: ProposalType::Constitutional,
            category: ProposalCategory::Economic,
            required_quorum: 0.66,
            execution_timestamp: None,
        };
        governance.create_proposal(proposal).unwrap();
        let vote1 = Vote {
            voter: "Bob".to_string(),
            proposal_id: "proposal1".to_string(),
            in_favor: true,
            weight: 0.7,
            timestamp: Utc::now(),
            zkp: None,
        };
        let vote2 = Vote {
            voter: "Charlie".to_string(),
            proposal_id: "proposal1".to_string(),
            in_favor: false,
            weight: 0.3,
            timestamp: Utc::now(),
            zkp: None,
        };
        governance.vote_on_proposal(vote1).unwrap();
        governance.vote_on_proposal(vote2).unwrap();
        let status = governance.close_proposal("proposal1").unwrap();
        assert_eq!(status, ProposalStatus::Passed);
    }

    #[test]
    fn test_list_active_proposals() {
        let mut governance = Governance::new();
        let proposal1 = Proposal {
            id: "proposal1".to_string(),
            title: "Test Proposal 1".to_string(),
            description: "This is a test proposal 1".to_string(),
            proposer: "Alice".to_string(),
            created_at: Utc::now(),
            voting_ends_at: Utc::now() + chrono::Duration::days(7),
            status: ProposalStatus::Active,
            proposal_type: ProposalType::Constitutional,
            category: ProposalCategory::Economic,
            required_quorum: 0.66,
            execution_timestamp: None,
        };
        let proposal2 = Proposal {
            id: "proposal2".to_string(),
            title: "Test Proposal 2".to_string(),
            description: "This is a test proposal 2".to_string(),
            proposer: "Bob".to_string(),
            created_at: Utc::now(),
            voting_ends_at: Utc::now() + chrono::Duration::days(7),
            status: ProposalStatus::Passed,
            proposal_type: ProposalType::Constitutional,
            category: ProposalCategory::Economic,
            required_quorum: 0.66,
            execution_timestamp: None,
        };
        governance.create_proposal(proposal1).unwrap();
        governance.create_proposal(proposal2).unwrap();
        let active_proposals = governance.list_active_proposals();
        assert_eq!(active_proposals.len(), 1);
        assert_eq!(active_proposals[0].id, "proposal1");
    }
}

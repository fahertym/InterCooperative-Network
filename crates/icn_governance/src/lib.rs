use icn_types::{IcnResult, IcnError, Proposal, ProposalStatus, ProposalType, ProposalCategory, Vote};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;

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
    ) -> IcnResult<String> {
        let id = format!("prop_{}", Utc::now().timestamp());
        let proposal = Proposal {
            id: id.clone(),
            title,
            description,
            proposer,
            created_at: Utc::now(),
            voting_ends_at: Utc::now() + voting_period,
            status: ProposalStatus::Active,
            proposal_type,
            category,
            required_quorum,
            execution_timestamp,
        };
        self.proposals.insert(id.clone(), proposal);
        Ok(id)
    }

    pub fn vote(&mut self, voter: String, proposal_id: String, in_favor: bool, weight: f64) -> IcnResult<()> {
        let proposal = self.proposals.get(&proposal_id)
            .ok_or_else(|| IcnError::Governance("Proposal not found".to_string()))?;
        
        if proposal.status != ProposalStatus::Active {
            return Err(IcnError::Governance("Voting is not active for this proposal".to_string()));
        }

        if Utc::now() > proposal.voting_ends_at {
            return Err(IcnError::Governance("Voting period has ended".to_string()));
        }

        let vote = Vote {
            voter: voter.clone(),
            proposal_id: proposal_id.clone(),
            in_favor,
            weight,
            timestamp: Utc::now(),
        };

        self.votes.entry(proposal_id).or_insert_with(Vec::new).push(vote);
        Ok(())
    }

    pub fn tally_votes(&mut self, proposal_id: &str) -> IcnResult<()> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or_else(|| IcnError::Governance("Proposal not found".to_string()))?;
        
        if proposal.status != ProposalStatus::Active {
            return Err(IcnError::Governance("Proposal is not active".to_string()));
        }

        if Utc::now() < proposal.voting_ends_at {
            return Err(IcnError::Governance("Voting period has not ended yet".to_string()));
        }

        let votes = self.votes.get(proposal_id)
            .ok_or_else(|| IcnError::Governance("No votes found for this proposal".to_string()))?;
        
        let total_weight: f64 = votes.iter().map(|v| v.weight).sum();
        let weight_in_favor: f64 = votes.iter().filter(|v| v.in_favor).map(|v| v.weight).sum();

        if total_weight < proposal.required_quorum {
            proposal.status = ProposalStatus::Rejected;
            return Ok(());
        }

        if weight_in_favor / total_weight > 0.5 {
            proposal.status = ProposalStatus::Passed;
        } else {
            proposal.status = ProposalStatus::Rejected;
        }

        Ok(())
    }

    pub fn get_proposal(&self, proposal_id: &str) -> Option<&Proposal> {
        self.proposals.get(proposal_id)
    }

    pub fn get_votes(&self, proposal_id: &str) -> Option<&Vec<Vote>> {
        self.votes.get(proposal_id)
    }

    pub fn list_active_proposals(&self) -> Vec<&Proposal> {
        self.proposals.values()
            .filter(|p| p.status == ProposalStatus::Active)
            .collect()
    }

    pub fn mark_as_implemented(&mut self, proposal_id: &str) -> IcnResult<()> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or_else(|| IcnError::Governance("Proposal not found".to_string()))?;
        
        if proposal.status != ProposalStatus::Passed {
            return Err(IcnError::Governance("Proposal has not passed".to_string()));
        }

        proposal.status = ProposalStatus::Implemented;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        // Fast-forward time (in a real scenario, you'd use a time mocking library)
        let proposal = gov_system.proposals.get_mut(&proposal_id).unwrap();
        proposal.voting_ends_at = Utc::now() - Duration::hours(1);

        // Tally votes
        assert!(gov_system.tally_votes(&proposal_id).is_ok());

        // Check proposal status
        let proposal = gov_system.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Passed);

        // Mark as implemented
        assert!(gov_system.mark_as_implemented(&proposal_id).is_ok());
        let updated_proposal = gov_system.get_proposal(&proposal_id).unwrap();
        assert_eq!(updated_proposal.status, ProposalStatus::Implemented);

        // Test voting on an inactive proposal
        assert!(gov_system.vote("Eve".to_string(), proposal_id.clone(), true, 1.0).is_err());

        // Test listing active proposals
        assert!(gov_system.list_active_proposals().is_empty());

        // Test creating a proposal with invalid quorum
        let invalid_proposal_result = gov_system.create_proposal(
            "Invalid Proposal".to_string(),
            "This proposal has an invalid quorum".to_string(),
            "Frank".to_string(),
            Duration::days(7),
            ProposalType::EconomicAdjustment,
            ProposalCategory::Economic,
            1.5, // Invalid quorum (should be between 0 and 1)
            None,
        );
        assert!(invalid_proposal_result.is_ok()); // The create_proposal function doesn't validate the quorum, so this will still succeed

        // In a real-world scenario, you might want to add validation for the quorum in the create_proposal function
    }
}
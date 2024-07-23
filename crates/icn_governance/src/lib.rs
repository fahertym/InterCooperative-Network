// File: crates/icn_governance/src/lib.rs

use icn_common::{IcnResult, IcnError, Block, Transaction, CurrencyType};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Implemented,
    Expired
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalType {
    Constitutional,
    EconomicAdjustment,
    NetworkUpgrade,
    ResourceAllocation,
    Reputation,
    Custom(String)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub proposer: String,
    pub proposal_type: ProposalType,
    pub status: ProposalStatus,
    pub created_at: DateTime<Utc>,
    pub voting_ends_at: DateTime<Utc>,
    pub votes_for: u64,
    pub votes_against: u64,
    pub quorum_threshold: f64,
    pub approval_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: String,
    pub proposal_id: String,
    pub in_favor: bool,
    pub voting_power: f64,
    pub timestamp: DateTime<Utc>,
}

pub struct GovernanceSystem {
    proposals: Arc<RwLock<HashMap<String, Proposal>>>,
    votes: Arc<RwLock<HashMap<String, Vec<Vote>>>>,
    reputation_system: Arc<RwLock<dyn ReputationSystem>>,
}

pub trait ReputationSystem: Send + Sync {
    fn get_voting_power(&self, address: &str) -> f64;
    fn update_reputation(&mut self, address: &str, change: f64) -> IcnResult<()>;
}

impl GovernanceSystem {
    pub fn new(reputation_system: Arc<RwLock<dyn ReputationSystem>>) -> Self {
        GovernanceSystem {
            proposals: Arc::new(RwLock::new(HashMap::new())),
            votes: Arc::new(RwLock::new(HashMap::new())),
            reputation_system,
        }
    }

    pub fn create_proposal(&self, 
        title: String, 
        description: String, 
        proposer: String, 
        proposal_type: ProposalType,
        voting_duration: Duration,
        quorum_threshold: f64,
        approval_threshold: f64
    ) -> IcnResult<String> {
        let id = format!("prop_{}", Utc::now().timestamp());
        let proposal = Proposal {
            id: id.clone(),
            title,
            description,
            proposer,
            proposal_type,
            status: ProposalStatus::Active,
            created_at: Utc::now(),
            voting_ends_at: Utc::now() + voting_duration,
            votes_for: 0,
            votes_against: 0,
            quorum_threshold,
            approval_threshold,
        };

        self.proposals.write().map_err(|_| IcnError::Governance("Lock poisoned".into()))?.insert(id.clone(), proposal);
        Ok(id)
    }

    pub fn vote(&self, voter: String, proposal_id: String, in_favor: bool) -> IcnResult<()> {
        let mut proposals = self.proposals.write().map_err(|_| IcnError::Governance("Lock poisoned".into()))?;
        let proposal = proposals.get_mut(&proposal_id).ok_or(IcnError::Governance("Proposal not found".into()))?;

        if proposal.status != ProposalStatus::Active {
            return Err(IcnError::Governance("Proposal is not active".into()));
        }

        if Utc::now() > proposal.voting_ends_at {
            return Err(IcnError::Governance("Voting period has ended".into()));
        }

        let voting_power = self.reputation_system.read()
            .map_err(|_| IcnError::Governance("Lock poisoned".into()))?
            .get_voting_power(&voter);

        let vote = Vote {
            voter: voter.clone(),
            proposal_id: proposal_id.clone(),
            in_favor,
            voting_power,
            timestamp: Utc::now(),
        };

        if in_favor {
            proposal.votes_for += voting_power as u64;
        } else {
            proposal.votes_against += voting_power as u64;
        }

        self.votes.write()
            .map_err(|_| IcnError::Governance("Lock poisoned".into()))?
            .entry(proposal_id)
            .or_insert_with(Vec::new)
            .push(vote);

        Ok(())
    }

    pub fn tally_votes(&self, proposal_id: &str) -> IcnResult<ProposalStatus> {
        let mut proposals = self.proposals.write().map_err(|_| IcnError::Governance("Lock poisoned".into()))?;
        let proposal = proposals.get_mut(proposal_id).ok_or(IcnError::Governance("Proposal not found".into()))?;

        if proposal.status != ProposalStatus::Active {
            return Ok(proposal.status.clone());
        }

        if Utc::now() < proposal.voting_ends_at {
            return Err(IcnError::Governance("Voting period has not ended yet".into()));
        }

        let total_votes = proposal.votes_for + proposal.votes_against;
        let participation_rate = total_votes as f64 / self.get_total_voting_power()?;

        if participation_rate < proposal.quorum_threshold {
            proposal.status = ProposalStatus::Rejected;
        } else {
            let approval_rate = proposal.votes_for as f64 / total_votes as f64;
            if approval_rate > proposal.approval_threshold {
                proposal.status = ProposalStatus::Passed;
            } else {
                proposal.status = ProposalStatus::Rejected;
            }
        }

        Ok(proposal.status.clone())
    }

    pub fn get_proposal(&self, proposal_id: &str) -> IcnResult<Proposal> {
        self.proposals.read()
            .map_err(|_| IcnError::Governance("Lock poisoned".into()))?
            .get(proposal_id)
            .cloned()
            .ok_or(IcnError::Governance("Proposal not found".into()))
    }

    pub fn get_votes(&self, proposal_id: &str) -> IcnResult<Vec<Vote>> {
        self.votes.read()
            .map_err(|_| IcnError::Governance("Lock poisoned".into()))?
            .get(proposal_id)
            .cloned()
            .ok_or(IcnError::Governance("No votes found for this proposal".into()))
    }

    pub fn list_active_proposals(&self) -> IcnResult<Vec<Proposal>> {
        Ok(self.proposals.read()
            .map_err(|_| IcnError::Governance("Lock poisoned".into()))?
            .values()
            .filter(|p| p.status == ProposalStatus::Active)
            .cloned()
            .collect())
    }

    pub fn implement_proposal(&self, proposal_id: &str) -> IcnResult<()> {
        let mut proposals = self.proposals.write().map_err(|_| IcnError::Governance("Lock poisoned".into()))?;
        let proposal = proposals.get_mut(proposal_id).ok_or(IcnError::Governance("Proposal not found".into()))?;

        if proposal.status != ProposalStatus::Passed {
            return Err(IcnError::Governance("Proposal has not passed".into()));
        }

        proposal.status = ProposalStatus::Implemented;

        // Here you would implement the actual changes proposed
        // This could involve calling other parts of the system to make the necessary changes
        match proposal.proposal_type {
            ProposalType::Constitutional => {
                // Implement changes to the system's constitution
            },
            ProposalType::EconomicAdjustment => {
                // Implement economic changes, e.g., adjust currency issuance rates
            },
            ProposalType::NetworkUpgrade => {
                // Trigger a network upgrade process
            },
            ProposalType::ResourceAllocation => {
                // Adjust resource allocation in the system
            },
            ProposalType::Reputation => {
                // Make changes to the reputation system
            },
            ProposalType::Custom(_) => {
                // Handle custom proposal types
            },
        }

        Ok(())
    }

    fn get_total_voting_power(&self) -> IcnResult<f64> {
        // This is a placeholder. In a real system, you'd need to calculate this based on all users' voting power
        Ok(10000.0)
    }

    pub fn update_proposal_status(&self) -> IcnResult<()> {
        let mut proposals = self.proposals.write().map_err(|_| IcnError::Governance("Lock poisoned".into()))?;
        
        for proposal in proposals.values_mut() {
            if proposal.status == ProposalStatus::Active && Utc::now() > proposal.voting_ends_at {
                let total_votes = proposal.votes_for + proposal.votes_against;
                let participation_rate = total_votes as f64 / self.get_total_voting_power()?;

                if participation_rate < proposal.quorum_threshold {
                    proposal.status = ProposalStatus::Expired;
                } else {
                    let approval_rate = proposal.votes_for as f64 / total_votes as f64;
                    if approval_rate > proposal.approval_threshold {
                        proposal.status = ProposalStatus::Passed;
                    } else {
                        proposal.status = ProposalStatus::Rejected;
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockReputationSystem {
        voting_power: HashMap<String, f64>,
    }

    impl ReputationSystem for MockReputationSystem {
        fn get_voting_power(&self, address: &str) -> f64 {
            *self.voting_power.get(address).unwrap_or(&1.0)
        }

        fn update_reputation(&mut self, address: &str, change: f64) -> IcnResult<()> {
            *self.voting_power.entry(address.to_string()).or_insert(1.0) += change;
            Ok(())
        }
    }

    #[test]
    fn test_create_proposal() {
        let reputation_system = Arc::new(RwLock::new(MockReputationSystem {
            voting_power: HashMap::new(),
        }));
        let governance = GovernanceSystem::new(reputation_system);

        let proposal_id = governance.create_proposal(
            "Test Proposal".to_string(),
            "This is a test proposal".to_string(),
            "Alice".to_string(),
            ProposalType::Constitutional,
            Duration::days(7),
            0.5,
            0.66,
        ).unwrap();

        let proposal = governance.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.title, "Test Proposal");
        assert_eq!(proposal.status, ProposalStatus::Active);
    }

    #[test]
    fn test_voting_and_tallying() {
        let mut mock_reputation = MockReputationSystem {
            voting_power: HashMap::new(),
        };
        mock_reputation.voting_power.insert("Alice".to_string(), 100.0);
        mock_reputation.voting_power.insert("Bob".to_string(), 50.0);
        mock_reputation.voting_power.insert("Charlie".to_string(), 25.0);

        let reputation_system = Arc::new(RwLock::new(mock_reputation));
        let governance = GovernanceSystem::new(reputation_system);

        let proposal_id = governance.create_proposal(
            "Test Proposal".to_string(),
            "This is a test proposal".to_string(),
            "Alice".to_string(),
            ProposalType::Constitutional,
            Duration::days(7),
            0.5,
            0.66,
        ).unwrap();

        governance.vote("Alice".to_string(), proposal_id.clone(), true).unwrap();
        governance.vote("Bob".to_string(), proposal_id.clone(), false).unwrap();
        governance.vote("Charlie".to_string(), proposal_id.clone(), true).unwrap();

        // Force the voting period to end
        {
            let mut proposals = governance.proposals.write().unwrap();
            let proposal = proposals.get_mut(&proposal_id).unwrap();
            proposal.voting_ends_at = Utc::now() - Duration::hours(1);
        }

        let result = governance.tally_votes(&proposal_id).unwrap();
        assert_eq!(result, ProposalStatus::Passed);

        let proposal = governance.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.votes_for, 125);
        assert_eq!(proposal.votes_against, 50);
    }

    #[test]
    fn test_implement_proposal() {
        let reputation_system = Arc::new(RwLock::new(MockReputationSystem {
            voting_power: HashMap::new(),
        }));
        let governance = GovernanceSystem::new(reputation_system);

        let proposal_id = governance.create_proposal(
            "Test Proposal".to_string(),
            "This is a test proposal".to_string(),
            "Alice".to_string(),
            ProposalType::Constitutional,
            Duration::days(7),
            0.5,
            0.66,
        ).unwrap();

        // Force the proposal to pass
        {
            let mut proposals = governance.proposals.write().unwrap();
            let proposal = proposals.get_mut(&proposal_id).unwrap();
            proposal.status = ProposalStatus::Passed;
        }

        governance.implement_proposal(&proposal_id).unwrap();

        let proposal = governance.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Implemented);
    }
}
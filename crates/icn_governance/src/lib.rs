// File: crates/icn_governance/src/lib.rs

use icn_common::{IcnResult, IcnError, Proposal, ProposalStatus, ProposalType, ProposalCategory};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use log::{info, warn, error};

pub struct GovernanceSystem {
    proposals: Arc<RwLock<HashMap<String, Proposal>>>,
    votes: Arc<RwLock<HashMap<String, HashMap<String, bool>>>>,
    blockchain: Arc<RwLock<dyn BlockchainInterface>>,
    consensus: Arc<RwLock<dyn ConsensusInterface>>,
}

pub trait BlockchainInterface: Send + Sync {
    fn add_proposal(&mut self, proposal: Proposal) -> IcnResult<()>;
    fn update_proposal_status(&mut self, proposal_id: &str, status: ProposalStatus) -> IcnResult<()>;
}

pub trait ConsensusInterface: Send + Sync {
    fn validate_proposal(&self, proposal: &Proposal) -> IcnResult<bool>;
    fn execute_proposal(&mut self, proposal: &Proposal) -> IcnResult<()>;
}

impl GovernanceSystem {
    pub fn new(blockchain: Arc<RwLock<dyn BlockchainInterface>>, consensus: Arc<RwLock<dyn ConsensusInterface>>) -> Self {
        GovernanceSystem {
            proposals: Arc::new(RwLock::new(HashMap::new())),
            votes: Arc::new(RwLock::new(HashMap::new())),
            blockchain,
            consensus,
        }
    }

    pub fn create_proposal(&self, proposal: Proposal) -> IcnResult<String> {
        let mut proposals = self.proposals.write().map_err(|_| IcnError::Governance("Failed to lock proposals".into()))?;
        
        // Validate the proposal
        self.consensus.read().map_err(|_| IcnError::Governance("Failed to lock consensus".into()))?
            .validate_proposal(&proposal)?;
        
        // Add the proposal to the blockchain
        self.blockchain.write().map_err(|_| IcnError::Governance("Failed to lock blockchain".into()))?
            .add_proposal(proposal.clone())?;
        
        proposals.insert(proposal.id.clone(), proposal);
        Ok(proposal.id)
    }

    pub fn vote_on_proposal(&self, proposal_id: &str, voter: &str, vote: bool) -> IcnResult<()> {
        let mut votes = self.votes.write().map_err(|_| IcnError::Governance("Failed to lock votes".into()))?;
        
        let proposal_votes = votes.entry(proposal_id.to_string()).or_insert_with(HashMap::new);
        proposal_votes.insert(voter.to_string(), vote);
        
        Ok(())
    }

    pub fn get_proposal_status(&self, proposal_id: &str) -> IcnResult<ProposalStatus> {
        let proposals = self.proposals.read().map_err(|_| IcnError::Governance("Failed to lock proposals".into()))?;
        
        proposals.get(proposal_id)
            .map(|proposal| proposal.status.clone())
            .ok_or_else(|| IcnError::Governance("Proposal not found".into()))
    }

    pub fn update_proposal_status(&self, proposal_id: &str) -> IcnResult<()> {
        let mut proposals = self.proposals.write().map_err(|_| IcnError::Governance("Failed to lock proposals".into()))?;
        let votes = self.votes.read().map_err(|_| IcnError::Governance("Failed to lock votes".into()))?;
        
        let proposal = proposals.get_mut(proposal_id).ok_or_else(|| IcnError::Governance("Proposal not found".into()))?;
        
        if proposal.status != ProposalStatus::Active {
            return Ok(());
        }
        
        if Utc::now() < proposal.voting_ends_at {
            return Ok(());
        }
        
        let proposal_votes = votes.get(proposal_id).unwrap_or(&HashMap::new());
        let total_votes = proposal_votes.len() as f64;
        let positive_votes = proposal_votes.values().filter(|&&v| v).count() as f64;
        
        let new_status = if total_votes == 0.0 {
            ProposalStatus::Rejected
        } else if positive_votes / total_votes >= proposal.required_quorum {
            ProposalStatus::Passed
        } else {
            ProposalStatus::Rejected
        };
        
        proposal.status = new_status.clone();
        
        // Update the proposal status on the blockchain
        self.blockchain.write().map_err(|_| IcnError::Governance("Failed to lock blockchain".into()))?
            .update_proposal_status(proposal_id, new_status)?;
        
        Ok(())
    }

    pub fn execute_proposal(&self, proposal_id: &str) -> IcnResult<()> {
        let proposals = self.proposals.read().map_err(|_| IcnError::Governance("Failed to lock proposals".into()))?;
        
        let proposal = proposals.get(proposal_id).ok_or_else(|| IcnError::Governance("Proposal not found".into()))?;
        
        if proposal.status != ProposalStatus::Passed {
            return Err(IcnError::Governance("Cannot execute proposal that has not passed".into()));
        }
        
        self.consensus.write().map_err(|_| IcnError::Governance("Failed to lock consensus".into()))?
            .execute_proposal(proposal)?;
        
        Ok(())
    }

    pub fn list_active_proposals(&self) -> IcnResult<Vec<Proposal>> {
        let proposals = self.proposals.read().map_err(|_| IcnError::Governance("Failed to lock proposals".into()))?;
        
        Ok(proposals.values()
            .filter(|p| p.status == ProposalStatus::Active)
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockBlockchain {
        proposals: Mutex<Vec<Proposal>>,
    }

    impl BlockchainInterface for MockBlockchain {
        fn add_proposal(&mut self, proposal: Proposal) -> IcnResult<()> {
            self.proposals.lock().unwrap().push(proposal);
            Ok(())
        }

        fn update_proposal_status(&mut self, proposal_id: &str, status: ProposalStatus) -> IcnResult<()> {
            let mut proposals = self.proposals.lock().unwrap();
            if let Some(proposal) = proposals.iter_mut().find(|p| p.id == proposal_id) {
                proposal.status = status;
                Ok(())
            } else {
                Err(IcnError::Governance("Proposal not found".into()))
            }
        }
    }

    struct MockConsensus;

    impl ConsensusInterface for MockConsensus {
        fn validate_proposal(&self, _proposal: &Proposal) -> IcnResult<bool> {
            Ok(true)
        }

        fn execute_proposal(&mut self, _proposal: &Proposal) -> IcnResult<()> {
            Ok(())
        }
    }

    #[test]
    fn test_create_and_vote_on_proposal() {
        let blockchain = Arc::new(RwLock::new(MockBlockchain { proposals: Mutex::new(Vec::new()) }));
        let consensus = Arc::new(RwLock::new(MockConsensus));
        let governance = GovernanceSystem::new(blockchain.clone(), consensus);

        let proposal = Proposal {
            id: "test_proposal".to_string(),
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            proposer: "Alice".to_string(),
            created_at: Utc::now(),
            voting_ends_at: Utc::now() + chrono::Duration::days(7),
            status: ProposalStatus::Active,
            proposal_type: ProposalType::Constitutional,
            category: ProposalCategory::Economic,
            required_quorum: 0.5,
            execution_timestamp: None,
        };

        // Create proposal
        let proposal_id = governance.create_proposal(proposal).unwrap();
        assert_eq!(proposal_id, "test_proposal");

        // Vote on proposal
        assert!(governance.vote_on_proposal(&proposal_id, "Bob", true).is_ok());
        assert!(governance.vote_on_proposal(&proposal_id, "Charlie", false).is_ok());
        assert!(governance.vote_on_proposal(&proposal_id, "David", true).is_ok());

        // Check proposal status
        assert_eq!(governance.get_proposal_status(&proposal_id).unwrap(), ProposalStatus::Active);

        // Update proposal status (simulating time passage)
        governance.update_proposal_status(&proposal_id).unwrap();

        // Check updated status
        assert_eq!(governance.get_proposal_status(&proposal_id).unwrap(), ProposalStatus::Passed);

        // Execute proposal
        assert!(governance.execute_proposal(&proposal_id).is_ok());
    }

    #[test]
    fn test_list_active_proposals() {
        let blockchain = Arc::new(RwLock::new(MockBlockchain { proposals: Mutex::new(Vec::new()) }));
        let consensus = Arc::new(RwLock::new(MockConsensus));
        let governance = GovernanceSystem::new(blockchain.clone(), consensus);

        let proposal1 = Proposal {
            id: "proposal1".to_string(),
            title: "Active Proposal".to_string(),
            description: "This is an active proposal".to_string(),
            proposer: "Alice".to_string(),
            created_at: Utc::now(),
            voting_ends_at: Utc::now() + chrono::Duration::days(7),
            status: ProposalStatus::Active,
            proposal_type: ProposalType::Constitutional,
            category: ProposalCategory::Economic,
            required_quorum: 0.5,
            execution_timestamp: None,
        };

        let proposal2 = Proposal {
            id: "proposal2".to_string(),
            title: "Passed Proposal".to_string(),
            description: "This is a passed proposal".to_string(),
            proposer: "Bob".to_string(),
            created_at: Utc::now(),
            voting_ends_at: Utc::now() - chrono::Duration::days(1),
            status: ProposalStatus::Passed,
            proposal_type: ProposalType::EconomicAdjustment,
            category: ProposalCategory::Technical,
            required_quorum: 0.6,
            execution_timestamp: None,
        };

        governance.create_proposal(proposal1.clone()).unwrap();
        governance.create_proposal(proposal2).unwrap();

        let active_proposals = governance.list_active_proposals().unwrap();
        assert_eq!(active_proposals.len(), 1);
        assert_eq!(active_proposals[0].id, "proposal1");
    }

    #[test]
    fn test_proposal_rejection() {
        let blockchain = Arc::new(RwLock::new(MockBlockchain { proposals: Mutex::new(Vec::new()) }));
        let consensus = Arc::new(RwLock::new(MockConsensus));
        let governance = GovernanceSystem::new(blockchain.clone(), consensus);

        let proposal = Proposal {
            id: "reject_proposal".to_string(),
            title: "Proposal to be Rejected".to_string(),
            description: "This proposal should be rejected".to_string(),
            proposer: "Alice".to_string(),
            created_at: Utc::now(),
            voting_ends_at: Utc::now() + chrono::Duration::days(7),
            status: ProposalStatus::Active,
            proposal_type: ProposalType::Constitutional,
            category: ProposalCategory::Economic,
            required_quorum: 0.7,
            execution_timestamp: None,
        };

        let proposal_id = governance.create_proposal(proposal).unwrap();

        // Vote on proposal (not enough votes to pass)
        governance.vote_on_proposal(&proposal_id, "Bob", true).unwrap();
        governance.vote_on_proposal(&proposal_id, "Charlie", false).unwrap();
        governance.vote_on_proposal(&proposal_id, "David", false).unwrap();

        // Update proposal status (simulating time passage)
        governance.update_proposal_status(&proposal_id).unwrap();

        // Check updated status
        assert_eq!(governance.get_proposal_status(&proposal_id).unwrap(), ProposalStatus::Rejected);

        // Attempt to execute rejected proposal (should fail)
        assert!(governance.execute_proposal(&proposal_id).is_err());
    }
}
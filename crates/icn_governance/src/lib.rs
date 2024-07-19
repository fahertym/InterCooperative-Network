// src/governance/democracy.rs

use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use log::{info, error, debug, warn};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ProposalCategory {
    Constitutional,
    Economic,
    Technical,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Implemented,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ProposalType {
    Constitutional,
    EconomicAdjustment,
    NetworkUpgrade,
}

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vote {
    pub voter: String,
    pub proposal_id: String,
    pub in_favor: bool,
    pub weight: f64,
    pub timestamp: DateTime<Utc>,
}

pub struct DemocraticSystem {
    proposals: HashMap<String, Proposal>,
    votes: HashMap<String, Vec<Vote>>,
}

impl DemocraticSystem {
    pub fn new() -> Self {
        debug!("Creating new DemocraticSystem");
        DemocraticSystem {
            proposals: HashMap::new(),
            votes: HashMap::new(),
        }
    }

    pub fn create_proposal(
        &mut self,
        title: String,
        description: String,
        proposer: String,
        voting_duration: Duration,
        proposal_type: ProposalType,
        category: ProposalCategory,
        required_quorum: f64,
        execution_timestamp: Option<DateTime<Utc>>
    ) -> Result<String, String> {
        let id = format!("prop_{}", Utc::now().timestamp());
        let proposal = Proposal {
            id: id.clone(),
            title,
            description,
            proposer,
            created_at: Utc::now(),
            voting_ends_at: Utc::now() + voting_duration,
            status: ProposalStatus::Active,
            proposal_type,
            category,
            required_quorum,
            execution_timestamp,
        };
        self.proposals.insert(id.clone(), proposal);
        info!("New proposal created: {}", id);
        Ok(id)
    }

    pub fn vote(
        &mut self,
        voter: String,
        proposal_id: String,
        in_favor: bool,
        weight: f64
    ) -> Result<(), String> {
        let proposal = self.proposals.get(&proposal_id).ok_or("Proposal not found")?;
        
        if proposal.status != ProposalStatus::Active {
            error!("Attempted to vote on inactive proposal: {}", proposal_id);
            return Err("Voting is not active for this proposal".to_string());
        }

        if Utc::now() > proposal.voting_ends_at {
            error!("Attempted to vote on expired proposal: {}", proposal_id);
            return Err("Voting period has ended".to_string());
        }

        let vote = Vote {
            voter,
            proposal_id: proposal_id.clone(),
            in_favor,
            weight,
            timestamp: Utc::now(),
        };

        self.votes.entry(proposal_id.clone()).or_insert_with(Vec::new).push(vote);
        info!("Vote recorded for proposal: {}", proposal_id);
        Ok(())
    }

    pub fn tally_votes(&mut self, proposal_id: &str) -> Result<(), String> {
        let proposal = self.proposals.get_mut(proposal_id).ok_or("Proposal not found")?;
        
        if proposal.status != ProposalStatus::Active {
            error!("Attempted to tally votes for inactive proposal: {}", proposal_id);
            return Err("Proposal is not active".to_string());
        }

        if Utc::now() < proposal.voting_ends_at {
            warn!("Attempted to tally votes before voting period ended: {}", proposal_id);
            return Err("Voting period has not ended yet".to_string());
        }

        let votes = self.votes.get(proposal_id).ok_or("No votes found for this proposal")?;
        
        let total_weight: f64 = votes.iter().map(|v| v.weight).sum();
        let weight_in_favor: f64 = votes.iter().filter(|v| v.in_favor).map(|v| v.weight).sum();

        if total_weight < proposal.required_quorum {
            proposal.status = ProposalStatus::Rejected;
            info!("Proposal {} rejected due to insufficient quorum", proposal_id);
            return Ok(());
        }

        if weight_in_favor / total_weight > 0.5 {
            proposal.status = ProposalStatus::Passed;
            info!("Proposal {} passed", proposal_id);
        } else {
            proposal.status = ProposalStatus::Rejected;
            info!("Proposal {} rejected", proposal_id);
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

    pub fn mark_as_implemented(&mut self, proposal_id: &str) -> Result<(), String> {
        let proposal = self.proposals.get_mut(proposal_id).ok_or("Proposal not found")?;
        
        if proposal.status != ProposalStatus::Passed {
            error!("Attempted to mark non-passed proposal as implemented: {}", proposal_id);
            return Err("Proposal has not passed".to_string());
        }

        proposal.status = ProposalStatus::Implemented;
        info!("Proposal {} marked as implemented", proposal_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_proposal() {
        let mut system = DemocraticSystem::new();
        let result = system.create_proposal(
            "Test Proposal".to_string(),
            "This is a test proposal".to_string(),
            "Alice".to_string(),
            Duration::days(7),
            ProposalType::Constitutional,
            ProposalCategory::Technical,
            0.5,
            None,
        );
        assert!(result.is_ok());
        let proposal_id = result.unwrap();
        assert!(system.get_proposal(&proposal_id).is_some());
    }

    #[test]
    fn test_vote_and_tally() {
        let mut system = DemocraticSystem::new();
        let proposal_id = system.create_proposal(
            "Test Proposal".to_string(),
            "This is a test proposal".to_string(),
            "Alice".to_string(),
            Duration::seconds(1), // Short duration for testing
            ProposalType::Constitutional,
            ProposalCategory::Technical,
            0.5,
            None,
        ).unwrap();

        system.vote("Bob".to_string(), proposal_id.clone(), true, 1.0).unwrap();
        system.vote("Charlie".to_string(), proposal_id.clone(), false, 1.0).unwrap();
        system.vote("David".to_string(), proposal_id.clone(), true, 1.0).unwrap();

        // Wait for voting period to end
        std::thread::sleep(std::time::Duration::from_secs(2));

        let tally_result = system.tally_votes(&proposal_id);
        assert!(tally_result.is_ok());

        let proposal = system.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Passed);
    }
}
// crates/icn_dao/src/lib.rs

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use icn_common::{IcnResult, IcnError};
use uuid::Uuid;

/// Represents a member of a DAO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub id: String,
    pub name: String,
    pub joined_at: DateTime<Utc>,
    pub reputation: f64,
}

/// Represents a proposal in a DAO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub proposer: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: ProposalStatus,
    pub votes: HashMap<String, Vote>,
}

/// Represents the status of a proposal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
}

/// Represents a vote on a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub member: String,
    pub in_favor: bool,
    pub weight: f64,
}

/// Represents the type of a DAO
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DaoType {
    Cooperative,
    Community,
    Custom(String),
}

/// Represents a Decentralized Autonomous Organization (DAO)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dao {
    pub id: String,
    pub name: String,
    pub dao_type: DaoType,
    pub members: HashMap<String, Member>,
    pub proposals: HashMap<String, Proposal>,
    pub quorum: f64,
    pub majority: f64,
}

impl Dao {
    /// Creates a new DAO
    pub fn new(name: String, dao_type: DaoType, quorum: f64, majority: f64) -> Self {
        Dao {
            id: Uuid::new_v4().to_string(),
            name,
            dao_type,
            members: HashMap::new(),
            proposals: HashMap::new(),
            quorum,
            majority,
        }
    }

    /// Adds a new member to the DAO
    pub fn add_member(&mut self, id: String, name: String) -> IcnResult<()> {
        if self.members.contains_key(&id) {
            return Err(IcnError::Dao("Member already exists".into()));
        }

        let member = Member {
            id: id.clone(),
            name,
            joined_at: Utc::now(),
            reputation: 1.0,
        };

        self.members.insert(id, member);
        Ok(())
    }

    /// Creates a new proposal in the DAO
    pub fn create_proposal(&mut self, title: String, description: String, proposer: String, duration: chrono::Duration) -> IcnResult<String> {
        if !self.members.contains_key(&proposer) {
            return Err(IcnError::Dao("Proposer is not a member of the DAO".into()));
        }

        let id = Uuid::new_v4().to_string();
        let proposal = Proposal {
            id: id.clone(),
            title,
            description,
            proposer,
            created_at: Utc::now(),
            expires_at: Utc::now() + duration,
            status: ProposalStatus::Active,
            votes: HashMap::new(),
        };

        self.proposals.insert(id.clone(), proposal);
        Ok(id)
    }

    /// Casts a vote on a proposal
    pub fn vote(&mut self, proposal_id: &str, member_id: &str, in_favor: bool) -> IcnResult<()> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or_else(|| IcnError::Dao("Proposal not found".into()))?;

        if proposal.status != ProposalStatus::Active {
            return Err(IcnError::Dao("Proposal is not active".into()));
        }

        let member = self.members.get(member_id)
            .ok_or_else(|| IcnError::Dao("Member not found".into()))?;

        let vote = Vote {
            member: member_id.to_string(),
            in_favor,
            weight: member.reputation,
        };

        proposal.votes.insert(member_id.to_string(), vote);
        Ok(())
    }

    /// Finalizes a proposal, determining if it passed or failed
    pub fn finalize_proposal(&mut self, proposal_id: &str) -> IcnResult<ProposalStatus> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or_else(|| IcnError::Dao("Proposal not found".into()))?;

        if proposal.status != ProposalStatus::Active {
            return Err(IcnError::Dao("Proposal is not active".into()));
        }

        let total_votes: f64 = proposal.votes.values().map(|v| v.weight).sum();
        let total_members: f64 = self.members.values().map(|m| m.reputation).sum();

        if total_votes / total_members < self.quorum {
            proposal.status = ProposalStatus::Rejected;
            return Ok(ProposalStatus::Rejected);
        }

        let votes_in_favor: f64 = proposal.votes.values()
            .filter(|v| v.in_favor)
            .map(|v| v.weight)
            .sum();

        if votes_in_favor / total_votes > self.majority {
            proposal.status = ProposalStatus::Passed;
            Ok(ProposalStatus::Passed)
        } else {
            proposal.status = ProposalStatus::Rejected;
            Ok(ProposalStatus::Rejected)
        }
    }

    /// Executes a passed proposal
    pub fn execute_proposal(&mut self, proposal_id: &str) -> IcnResult<()> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or_else(|| IcnError::Dao("Proposal not found".into()))?;

        if proposal.status != ProposalStatus::Passed {
            return Err(IcnError::Dao("Proposal has not passed".into()));
        }

        // Here you would implement the logic to execute the proposal
        // For now, we'll just mark it as executed
        proposal.status = ProposalStatus::Executed;
        Ok(())
    }
}

/// Represents a Cooperative, which is a specific type of DAO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cooperative {
    pub dao: Dao,
    pub business_type: String,
    pub member_shares: HashMap<String, f64>,
}

impl Cooperative {
    pub fn new(name: String, business_type: String, quorum: f64, majority: f64) -> Self {
        Cooperative {
            dao: Dao::new(name, DaoType::Cooperative, quorum, majority),
            business_type,
            member_shares: HashMap::new(),
        }
    }

    pub fn issue_shares(&mut self, member_id: &str, shares: f64) -> IcnResult<()> {
        if !self.dao.members.contains_key(member_id) {
            return Err(IcnError::Dao("Member not found".into()));
        }

        *self.member_shares.entry(member_id.to_string()).or_insert(0.0) += shares;
        Ok(())
    }

    pub fn get_member_shares(&self, member_id: &str) -> IcnResult<f64> {
        self.member_shares.get(member_id)
            .cloned()
            .ok_or_else(|| IcnError::Dao("Member has no shares".into()))
    }

    pub fn distribute_profits(&mut self, total_profit: f64) -> IcnResult<()> {
        let total_shares: f64 = self.member_shares.values().sum();
        
        for (member_id, shares) in &self.member_shares {
            let profit_share = total_profit * (shares / total_shares);
            // Here you would typically update the member's balance
            println!("Member {} receives profit share: {}", member_id, profit_share);
        }

        Ok(())
    }
}

/// Represents a Community, which is another specific type of DAO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Community {
    pub dao: Dao,
    pub location: String,
    pub focus_areas: Vec<String>,
}

impl Community {
    pub fn new(name: String, location: String, focus_areas: Vec<String>, quorum: f64, majority: f64) -> Self {
        Community {
            dao: Dao::new(name, DaoType::Community, quorum, majority),
            location,
            focus_areas,
        }
    }

    pub fn add_focus_area(&mut self, focus_area: String) -> IcnResult<()> {
        if !self.focus_areas.contains(&focus_area) {
            self.focus_areas.push(focus_area);
            Ok(())
        } else {
            Err(IcnError::Dao("Focus area already exists".into()))
        }
    }

    pub fn remove_focus_area(&mut self, focus_area: &str) -> IcnResult<()> {
        if let Some(pos) = self.focus_areas.iter().position(|x| x == focus_area) {
            self.focus_areas.remove(pos);
            Ok(())
        } else {
            Err(IcnError::Dao("Focus area not found".into()))
        }
    }

    pub fn organize_event(&self, event_name: &str, event_description: &str) -> IcnResult<()> {
        // Here you would typically integrate with a calendar or event system
        println!("Community {} is organizing event: {}", self.dao.name, event_name);
        println!("Event description: {}", event_description);
        Ok(())
    }
}

/// A factory for creating different types of DAOs
pub struct DaoFactory;

impl DaoFactory {
    pub fn create_dao(dao_type: DaoType, name: String, quorum: f64, majority: f64) -> Box<dyn DaoTrait> {
        match dao_type {
            DaoType::Cooperative => Box::new(Cooperative::new(name, "General".to_string(), quorum, majority)),
            DaoType::Community => Box::new(Community::new(name, "Global".to_string(), Vec::new(), quorum, majority)),
            DaoType::Custom(custom_type) => {
                // Here you could implement logic to create custom DAO types
                println!("Creating custom DAO of type: {}", custom_type);
                Box::new(Dao::new(name, dao_type, quorum, majority))
            }
        }
    }
}

/// A trait that defines common behavior for all DAO types
pub trait DaoTrait {
    fn get_dao(&self) -> &Dao;
    fn get_dao_mut(&mut self) -> &mut Dao;
    
    fn add_member(&mut self, id: String, name: String) -> IcnResult<()> {
        self.get_dao_mut().add_member(id, name)
    }

    fn create_proposal(&mut self, title: String, description: String, proposer: String, duration: chrono::Duration) -> IcnResult<String> {
        self.get_dao_mut().create_proposal(title, description, proposer, duration)
    }

    fn vote(&mut self, proposal_id: &str, member_id: &str, in_favor: bool) -> IcnResult<()> {
        self.get_dao_mut().vote(proposal_id, member_id, in_favor)
    }

    fn finalize_proposal(&mut self, proposal_id: &str) -> IcnResult<ProposalStatus> {
        self.get_dao_mut().finalize_proposal(proposal_id)
    }

    fn execute_proposal(&mut self, proposal_id: &str) -> IcnResult<()> {
        self.get_dao_mut().execute_proposal(proposal_id)
    }
}

impl DaoTrait for Dao {
    fn get_dao(&self) -> &Dao { self }
    fn get_dao_mut(&mut self) -> &mut Dao { self }
}

impl DaoTrait for Cooperative {
    fn get_dao(&self) -> &Dao { &self.dao }
    fn get_dao_mut(&mut self) -> &mut Dao { &mut self.dao }
}

impl DaoTrait for Community {
    fn get_dao(&self) -> &Dao { &self.dao }
    fn get_dao_mut(&mut self) -> &mut Dao { &mut self.dao }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cooperative_creation_and_operations() {
        let mut coop = Cooperative::new("Test Coop".to_string(), "Agriculture".to_string(), 0.5, 0.6);
        
        assert_eq!(coop.dao.name, "Test Coop");
        assert_eq!(coop.dao.dao_type, DaoType::Cooperative);
        assert_eq!(coop.business_type, "Agriculture");

        coop.add_member("alice".to_string(), "Alice".to_string()).unwrap();
        coop.add_member("bob".to_string(), "Bob".to_string()).unwrap();

        coop.issue_shares("alice", 100.0).unwrap();
        coop.issue_shares("bob", 50.0).unwrap();

        assert_eq!(coop.get_member_shares("alice").unwrap(), 100.0);
        assert_eq!(coop.get_member_shares("bob").unwrap(), 50.0);

        let proposal_id = coop.create_proposal(
            "Test Proposal".to_string(),
            "This is a test proposal".to_string(),
            "alice".to_string(),
            chrono::Duration::days(7)
        ).unwrap();

        coop.vote(&proposal_id, "alice", true).unwrap();
        coop.vote(&proposal_id, "bob", true).unwrap();

        let status = coop.finalize_proposal(&proposal_id).unwrap();
        assert_eq!(status, ProposalStatus::Passed);

        coop.execute_proposal(&proposal_id).unwrap();
    }

    #[test]
    fn test_community_creation_and_operations() {
        let mut community = Community::new(
            "Test Community".to_string(),
            "Test City".to_string(),
            vec!["Education".to_string(), "Environment".to_string()],
            0.5,
            0.6
        );
        
        assert_eq!(community.dao.name, "Test Community");
        assert_eq!(community.dao.dao_type, DaoType::Community);
        assert_eq!(community.location, "Test City");
        assert_eq!(community.focus_areas, vec!["Education", "Environment"]);

        community.add_member("alice".to_string(), "Alice".to_string()).unwrap();
        community.add_member("bob".to_string(), "Bob".to_string()).unwrap();

        community.add_focus_area("Health".to_string()).unwrap();
        assert_eq!(community.focus_areas, vec!["Education", "Environment", "Health"]);

        community.remove_focus_area("Environment").unwrap();
        assert_eq!(community.focus_areas, vec!["Education", "Health"]);

        let proposal_id = community.create_proposal(
            "Community Event".to_string(),
            "Organize a community cleanup day".to_string(),
            "alice".to_string(),
            chrono::Duration::days(7)
        ).unwrap();

        community.vote(&proposal_id, "alice", true).unwrap();
        community.vote(&proposal_id, "bob", true).unwrap();

        let status = community.finalize_proposal(&proposal_id).unwrap();
        assert_eq!(status, ProposalStatus::Passed);

        community.execute_proposal(&proposal_id).unwrap();

        community.organize_event("Community Cleanup Day", "Let's clean up our neighborhood!").unwrap();
    }

    #[test]
    fn test_dao_factory() {
        let cooperative = DaoFactory::create_dao(
            DaoType::Cooperative,
            "Test Cooperative".to_string(),
            0.5,
            0.6
        );
        assert_eq!(cooperative.get_dao().dao_type, DaoType::Cooperative);

        let community = DaoFactory::create_dao(
            DaoType::Community,
            "Test Community".to_string(),
            0.5,
            0.6
        );
        assert_eq!(community.get_dao().dao_type, DaoType::Community);

        let custom_dao = DaoFactory::create_dao(
            DaoType::Custom("CustomType".to_string()),
            "Test Custom DAO".to_string(),
            0.5,
            0.6
        );
        if let DaoType::Custom(custom_type) = custom_dao.get_dao().dao_type {
            assert_eq!(custom_type, "CustomType");
        } else {
            panic!("Expected custom DAO type");
        }

        // Test common operations across different DAO types
        for mut dao in vec![cooperative, community, custom_dao] {
            dao.add_member("alice".to_string(), "Alice".to_string()).unwrap();
            dao.add_member("bob".to_string(), "Bob".to_string()).unwrap();

            let proposal_id = dao.create_proposal(
                "Test Proposal".to_string(),
                "This is a test proposal".to_string(),
                "alice".to_string(),
                chrono::Duration::days(7)
            ).unwrap();

            dao.vote(&proposal_id, "alice", true).unwrap();
            dao.vote(&proposal_id, "bob", true).unwrap();

            let status = dao.finalize_proposal(&proposal_id).unwrap();
            assert_eq!(status, ProposalStatus::Passed);

            dao.execute_proposal(&proposal_id).unwrap();
        }
    }
}
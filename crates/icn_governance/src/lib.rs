// File: crates/icn_governance/src/lib.rs

use icn_common::{IcnResult, IcnError, Block, Transaction, CurrencyType};
use icn_blockchain::Blockchain;
use icn_reputation::ReputationSystem;
use icn_currency::CurrencySystem;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

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
    pub voting_system: VotingSystem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub proposal_id: String,
    pub voter: String,
    pub vote_power: f64,
    pub in_favor: bool,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Implemented,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProposalType {
    Constitutional,
    EconomicAdjustment,
    NetworkUpgrade,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProposalCategory {
    Constitutional,
    Economic,
    Technical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VotingSystem {
    OnePersonOneVote,
    TokenWeighted { max_power: f64 },
    Quadratic { max_votes: u32 },
    ReputationWeighted,
    Hybrid { reputation_weight: f64, token_weight: f64, max_power: f64 },
}

pub struct GovernanceSystem {
    blockchain: Blockchain,
    reputation_system: ReputationSystem,
}

impl GovernanceSystem {
    pub fn new(blockchain: Blockchain, reputation_system: ReputationSystem) -> Self {
        GovernanceSystem {
            blockchain,
            reputation_system,
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
        voting_system: VotingSystem,
    ) -> IcnResult<String> {
        let proposal_id = format!("prop_{}", Utc::now().timestamp());
        let proposal = Proposal {
            id: proposal_id.clone(),
            title,
            description,
            proposer,
            created_at: Utc::now(),
            voting_ends_at: Utc::now() + voting_period,
            status: ProposalStatus::Active,
            proposal_type,
            category,
            required_quorum,
            execution_timestamp: None,
            voting_system,
        };

        let transaction = Transaction::new_governance(
            "SYSTEM".to_string(),
            "CreateProposal".to_string(),
            serde_json::to_vec(&proposal).map_err(|e| IcnError::Serialization(e.to_string()))?
        );

        self.blockchain.add_transaction(transaction)?;
        Ok(proposal_id)
    }

    pub fn vote(&mut self, proposal_id: &str, voter: String, in_favor: bool, vote_power: Option<f64>, currency_system: &mut CurrencySystem) -> IcnResult<()> {
        let proposal = self.get_proposal(proposal_id)?;

        if proposal.status != ProposalStatus::Active {
            return Err(IcnError::Governance("Proposal is not active".into()));
        }

        if Utc::now() > proposal.voting_ends_at {
            return Err(IcnError::Governance("Voting period has ended".into()));
        }

        let calculated_vote_power = match proposal.voting_system {
            VotingSystem::OnePersonOneVote => 1.0,
            VotingSystem::TokenWeighted { max_power } => {
                let balance = currency_system.get_balance(&voter, &CurrencyType::Governance)?;
                balance.min(max_power)
            },
            VotingSystem::Quadratic { max_votes } => {
                let vote_count = vote_power.ok_or(IcnError::Governance("Vote power must be provided for quadratic voting".into()))? as u32;
                if vote_count > max_votes {
                    return Err(IcnError::Governance("Exceeded maximum allowed votes".into()));
                }
                let cost = (vote_count as f64).powi(2);
                let balance = currency_system.get_balance(&voter, &CurrencyType::Governance)?;
                if balance < cost {
                    return Err(IcnError::Governance("Insufficient balance for quadratic vote".into()));
                }
                currency_system.transfer(&voter, "GOVERNANCE_POOL", &CurrencyType::Governance, cost)?;
                (vote_count as f64).sqrt()
            },
            VotingSystem::ReputationWeighted => {
                self.reputation_system.get_score(&voter).unwrap_or(0.0)
            },
            VotingSystem::Hybrid { reputation_weight, token_weight, max_power } => {
                let reputation = self.reputation_system.get_score(&voter).unwrap_or(0.0);
                let balance = currency_system.get_balance(&voter, &CurrencyType::Governance)?;
                (reputation * reputation_weight + balance * token_weight).min(max_power)
            },
        };

        let vote = Vote {
            proposal_id: proposal_id.to_string(),
            voter: voter.clone(),
            vote_power: calculated_vote_power,
            in_favor,
            timestamp: Utc::now(),
        };

        let transaction = Transaction::new_governance(
            voter,
            "Vote".to_string(),
            serde_json::to_vec(&vote).map_err(|e| IcnError::Serialization(e.to_string()))?
        );

        self.blockchain.add_transaction(transaction)
    }

    pub fn tally_votes(&mut self, proposal_id: &str) -> IcnResult<ProposalStatus> {
        let mut proposal = self.get_proposal(proposal_id)?;

        if proposal.status != ProposalStatus::Active {
            return Ok(proposal.status.clone());
        }

        if Utc::now() < proposal.voting_ends_at {
            return Err(IcnError::Governance("Voting period has not ended yet".into()));
        }

        let votes = self.get_votes(proposal_id)?;

        let mut total_power = 0.0;
        let mut in_favor_power = 0.0;

        for vote in votes {
            total_power += vote.vote_power;
            if vote.in_favor {
                in_favor_power += vote.vote_power;
            }
        }

        if total_power < proposal.required_quorum {
            proposal.status = ProposalStatus::Rejected;
        } else if in_favor_power > total_power / 2.0 {
            proposal.status = ProposalStatus::Passed;
        } else {
            proposal.status = ProposalStatus::Rejected;
        }

        let transaction = Transaction::new_governance(
            "SYSTEM".to_string(),
            "TallyVotes".to_string(),
            serde_json::to_vec(&proposal).map_err(|e| IcnError::Serialization(e.to_string()))?
        );

        self.blockchain.add_transaction(transaction)?;

        Ok(proposal.status)
    }

    pub fn get_proposal(&self, proposal_id: &str) -> IcnResult<Proposal> {
        for block in self.blockchain.chain().iter().rev() {
            for transaction in &block.transactions {
                if transaction.transaction_type == "CreateProposal" {
                    let proposal: Proposal = serde_json::from_slice(&transaction.data)
                        .map_err(|e| IcnError::Serialization(e.to_string()))?;
                    if proposal.id == proposal_id {
                        return Ok(proposal);
                    }
                }
            }
        }
        Err(IcnError::Governance("Proposal not found".into()))
    }

    pub fn get_votes(&self, proposal_id: &str) -> IcnResult<Vec<Vote>> {
        let mut votes = Vec::new();
        for block in self.blockchain.chain().iter().rev() {
            for transaction in &block.transactions {
                if transaction.transaction_type == "Vote" {
                    let vote: Vote = serde_json::from_slice(&transaction.data)
                        .map_err(|e| IcnError::Serialization(e.to_string()))?;
                    if vote.proposal_id == proposal_id {
                        votes.push(vote);
                    }
                }
            }
        }
        Ok(votes)
    }

    pub fn list_active_proposals(&self) -> IcnResult<Vec<Proposal>> {
        let mut active_proposals = Vec::new();
        for block in self.blockchain.chain().iter().rev() {
            for transaction in &block.transactions {
                if transaction.transaction_type == "CreateProposal" {
                    let proposal: Proposal = serde_json::from_slice(&transaction.data)
                        .map_err(|e| IcnError::Serialization(e.to_string()))?;
                    if proposal.status == ProposalStatus::Active && proposal.voting_ends_at > Utc::now() {
                        active_proposals.push(proposal);
                    }
                }
            }
        }
        Ok(active_proposals)
    }

    pub fn mark_as_implemented(&mut self, proposal_id: &str) -> IcnResult<()> {
        let mut proposal = self.get_proposal(proposal_id)?;
        
        if proposal.status != ProposalStatus::Passed {
            return Err(IcnError::Governance("Proposal has not passed".into()));
        }

        proposal.status = ProposalStatus::Implemented;
        proposal.execution_timestamp = Some(Utc::now());

        let transaction = Transaction::new_governance(
            "SYSTEM".to_string(),
            "ImplementProposal".to_string(),
            serde_json::to_vec(&proposal).map_err(|e| IcnError::Serialization(e.to_string()))?
        );

        self.blockchain.add_transaction(transaction)
    }

    pub fn get_proposal_result(&self, proposal_id: &str) -> IcnResult<ProposalResult> {
        let proposal = self.get_proposal(proposal_id)?;
        let votes = self.get_votes(proposal_id)?;

        let total_votes = votes.len();
        let total_power: f64 = votes.iter().map(|v| v.vote_power).sum();
        let in_favor_power: f64 = votes.iter().filter(|v| v.in_favor).map(|v| v.vote_power).sum();
        let against_power = total_power - in_favor_power;

        Ok(ProposalResult {
            proposal_id: proposal_id.to_string(),
            status: proposal.status,
            total_votes,
            total_power,
            in_favor_power,
            against_power,
            quorum_reached: total_power >= proposal.required_quorum,
            passed: proposal.status == ProposalStatus::Passed,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProposalResult {
    pub proposal_id: String,
    pub status: ProposalStatus,
    pub total_votes: usize,
    pub total_power: f64,
    pub in_favor_power: f64,
    pub against_power: f64,
    pub quorum_reached: bool,
    pub passed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_blockchain::Blockchain;
    use icn_reputation::ReputationSystem;
    use icn_currency::CurrencySystem;

    fn create_test_environment() -> (Blockchain, ReputationSystem, CurrencySystem) {
        let blockchain = Blockchain::new();
        let mut reputation_system = ReputationSystem::new(0.01, 0.0, 100.0);
        let mut currency_system = CurrencySystem::new();
        
        // Initialize some reputation and governance tokens for testing
        reputation_system.add_score("Alice", 50.0);
        reputation_system.add_score("Bob", 30.0);
        currency_system.mint("Alice", &CurrencyType::Governance, 1000.0).unwrap();
        currency_system.mint("Bob", &CurrencyType::Governance, 1000.0).unwrap();

        (blockchain, reputation_system, currency_system)
    }

    #[test]
    fn test_one_person_one_vote() {
        let (blockchain, reputation_system, mut currency_system) = create_test_environment();
        let mut gs = GovernanceSystem::new(blockchain, reputation_system);

        let proposal_id = gs.create_proposal(
            "Test Proposal".to_string(),
            "This is a test proposal".to_string(),
            "Alice".to_string(),
            Duration::days(7),
            ProposalType::Constitutional,
            ProposalCategory::Economic,
            0.5,
            VotingSystem::OnePersonOneVote,
        ).unwrap();

        gs.vote(&proposal_id, "Alice".to_string(), true, None, &mut currency_system).unwrap();
        gs.vote(&proposal_id, "Bob".to_string(), false, None, &mut currency_system).unwrap();

        // Fast-forward time
        let mut proposal = gs.get_proposal(&proposal_id).unwrap();
        proposal.voting_ends_at = Utc::now() - Duration::hours(1);
        let update_transaction = Transaction::new_governance(
            "SYSTEM".to_string(),
            "UpdateProposal".to_string(),
            serde_json::to_vec(&proposal).unwrap()
        );
        gs.blockchain.add_transaction(update_transaction).unwrap();

        let result = gs.tally_votes(&proposal_id).unwrap();
        assert_eq!(result, ProposalStatus::Passed);

        let proposal_result = gs.get_proposal_result(&proposal_id).unwrap();
        assert_eq!(proposal_result.total_votes, 2);
        assert_eq!(proposal_result.total_power, 2.0);
        assert_eq!(proposal_result.in_favor_power, 1.0);
        assert_eq!(proposal_result.against_power, 1.0);
        assert!(proposal_result.quorum_reached);
        assert!(proposal_result.passed);
    }

    #[test]
    fn test_token_weighted_voting() {
        let (blockchain, reputation_system, mut currency_system) = create_test_environment();
        let mut gs = GovernanceSystem::new(blockchain, reputation_system);

        let proposal_id = gs.create_proposal(
            "Token Weighted Proposal".to_string(),
            "This is a token weighted proposal".to_string(),
            "Alice".to_string(),
            Duration::days(7),
            ProposalType::EconomicAdjustment,
            ProposalCategory::Economic,
            500.0,
            VotingSystem::TokenWeighted { max_power: 800.0 },
        ).unwrap();

        gs.vote(&proposal_id, "Alice".to_string(), true, None, &mut currency_system).unwrap();
        gs.vote(&proposal_id, "Bob".to_string(), false, None, &mut currency_system).unwrap();

        // Fast-forward time
        let mut proposal = gs.get_proposal(&proposal_id).unwrap();
        proposal.voting_ends_at = Utc::now() - Duration::hours(1);
        let update_transaction = Transaction::new_governance(
            "SYSTEM".to_string(),
            "UpdateProposal".to_string(),
            serde_json::to_vec(&proposal).unwrap()
        );
        gs.blockchain.add_transaction(update_transaction).unwrap();

        let result = gs.tally_votes(&proposal_id).unwrap();
        assert_eq!(result, ProposalStatus::Passed);

        let proposal_result = gs.get_proposal_result(&proposal_id).unwrap();
        assert_eq!(proposal_result.total_votes, 2);
        assert_eq!(proposal_result.total_power, 1600.0);
        assert_eq!(proposal_result.in_favor_power, 800.0);
        assert_eq!(proposal_result.against_power, 800.0);
        assert!(proposal_result.quorum_reached);
        assert!(proposal_result.passed);
    }

    #[test]
    fn test_quadratic_voting() {
        let (blockchain, reputation_system, mut currency_system) = create_test_environment();
        let mut gs = GovernanceSystem::new(blockchain, reputation_system);

        let proposal_id = gs.create_proposal(
            "Quadratic Voting Proposal".to_string(),
            "This is a quadratic voting proposal".to_string(),
            "Alice".to_string(),
            Duration::days(7),
            ProposalType::NetworkUpgrade,
            ProposalCategory::Technical,
            5.0,
            VotingSystem::Quadratic { max_votes: 10 },
        ).unwrap();

        gs.vote(&proposal_id, "Alice".to_string(), true, Some(9.0), &mut currency_system).unwrap();
        gs.vote(&proposal_id, "Bob".to_string(), false, Some(4.0), &mut currency_system).unwrap();

        // Fast-forward time
        let mut proposal = gs.get_proposal(&proposal_id).unwrap();
        proposal.voting_ends_at = Utc::now() - Duration::hours(1);
        let update_transaction = Transaction::new_governance(
            "SYSTEM".to_string(),
            "UpdateProposal".to_string(),
            serde_json::to_vec(&proposal).unwrap()
        );
        gs.blockchain.add_transaction(update_transaction).unwrap();

        let result = gs.tally_votes(&proposal_id).unwrap();
        assert_eq!(result, ProposalStatus::Passed);

        let proposal_result = gs.get_proposal_result(&proposal_id).unwrap();
        assert_eq!(proposal_result.total_votes, 2);
        assert_eq!(proposal_result.total_power, 5.0); // 3 (sqrt of 9) + 2 (sqrt of 4)
        assert_eq!(proposal_result.in_favor_power, 3.0);
        assert_eq!(proposal_result.against_power, 2.0);
        assert!(proposal_result.quorum_reached);
        assert!(proposal_result.passed);
    }

    #[test]
    fn test_reputation_weighted_voting() {
        let (blockchain, reputation_system, mut currency_system) = create_test_environment();
        let mut gs = GovernanceSystem::new(blockchain, reputation_system);

        let proposal_id = gs.create_proposal(
            "Reputation Weighted Proposal".to_string(),
            "This is a reputation weighted proposal".to_string(),
            "Alice".to_string(),
            Duration::days(7),
            ProposalType::Constitutional,
            ProposalCategory::Economic,
            40.0,
            VotingSystem::ReputationWeighted,
        ).unwrap();

        gs.vote(&proposal_id, "Alice".to_string(), true, None, &mut currency_system).unwrap();
        gs.vote(&proposal_id, "Bob".to_string(), false, None, &mut currency_system).unwrap();

        // Fast-forward time
        let mut proposal = gs.get_proposal(&proposal_id).unwrap();
        proposal.voting_ends_at = Utc::now() - Duration::hours(1);
        let update_transaction = Transaction::new_governance(
            "SYSTEM".to_string(),
            "UpdateProposal".to_string(),
            serde_json::to_vec(&proposal).unwrap()
        );
        gs.blockchain.add_transaction(update_transaction).unwrap();

        let result = gs.tally_votes(&proposal_id).unwrap();
        assert_eq!(result, ProposalStatus::Passed);

        let proposal_result = gs.get_proposal_result(&proposal_id).unwrap();
        assert_eq!(proposal_result.total_votes, 2);
        assert_eq!(proposal_result.total_power, 80.0); // 50 (Alice) + 30 (Bob)
        assert_eq!(proposal_result.in_favor_power, 50.0);
        assert_eq!(proposal_result.against_power, 30.0);
        assert!(proposal_result.quorum_reached);
        assert!(proposal_result.passed);
    }

    #[test]
    fn test_hybrid_voting() {
        let (blockchain, reputation_system, mut currency_system) = create_test_environment();
        let mut gs = GovernanceSystem::new(blockchain, reputation_system);

        let proposal_id = gs.create_proposal(
            "Hybrid Voting Proposal".to_string(),
            "This is a hybrid voting proposal".to_string(),
            "Alice".to_string(),
            Duration::days(7),
            ProposalType::EconomicAdjustment,
            ProposalCategory::Economic,
            100.0,
            VotingSystem::Hybrid { 
                reputation_weight: 0.5, 
                token_weight: 0.001, 
                max_power: 100.0 
            },
        ).unwrap();

        gs.vote(&proposal_id, "Alice".to_string(), true, None, &mut currency_system).unwrap();
        gs.vote(&proposal_id, "Bob".to_string(), false, None, &mut currency_system).unwrap();

        // Fast-forward time
        let mut proposal = gs.get_proposal(&proposal_id).unwrap();
        proposal.voting_ends_at = Utc::now() - Duration::hours(1);
        let update_transaction = Transaction::new_governance(
            "SYSTEM".to_string(),
            "UpdateProposal".to_string(),
            serde_json::to_vec(&proposal).unwrap()
        );
        gs.blockchain.add_transaction(update_transaction).unwrap();

        let result = gs.tally_votes(&proposal_id).unwrap();
        assert_eq!(result, ProposalStatus::Passed);

        let proposal_result = gs.get_proposal_result(&proposal_id).unwrap();
        assert_eq!(proposal_result.total_votes, 2);
        // Alice: min(50 * 0.5 + 1000 * 0.001, 100) = 26
        // Bob: min(30 * 0.5 + 1000 * 0.001, 100) = 16
        assert_eq!(proposal_result.total_power, 42.0);
        assert_eq!(proposal_result.in_favor_power, 26.0);
        assert_eq!(proposal_result.against_power, 16.0);
        assert!(proposal_result.quorum_reached);
        assert!(proposal_result.passed);
    }

    #[test]
    fn test_quorum_not_reached() {
        let (blockchain, reputation_system, mut currency_system) = create_test_environment();
        let mut gs = GovernanceSystem::new(blockchain, reputation_system);

        let proposal_id = gs.create_proposal(
            "High Quorum Proposal".to_string(),
            "This proposal requires high quorum".to_string(),
            "Alice".to_string(),
            Duration::days(7),
            ProposalType::Constitutional,
            ProposalCategory::Technical,
            200.0, // High quorum requirement
            VotingSystem::TokenWeighted { max_power: 100.0 },
        ).unwrap();

        gs.vote(&proposal_id, "Alice".to_string(), true, None, &mut currency_system).unwrap();
        
        // Fast-forward time
        let mut proposal = gs.get_proposal(&proposal_id).unwrap();
        proposal.voting_ends_at = Utc::now() - Duration::hours(1);
        let update_transaction = Transaction::new_governance(
            "SYSTEM".to_string(),
            "UpdateProposal".to_string(),
            serde_json::to_vec(&proposal).unwrap()
        );
        gs.blockchain.add_transaction(update_transaction).unwrap();

        let result = gs.tally_votes(&proposal_id).unwrap();
        assert_eq!(result, ProposalStatus::Rejected);

        let proposal_result = gs.get_proposal_result(&proposal_id).unwrap();
        assert!(!proposal_result.quorum_reached);
        assert!(!proposal_result.passed);
    }

    #[test]
    fn test_vote_after_end_period() {
        let (blockchain, reputation_system, mut currency_system) = create_test_environment();
        let mut gs = GovernanceSystem::new(blockchain, reputation_system);

        let proposal_id = gs.create_proposal(
            "Test Proposal".to_string(),
            "This is a test proposal".to_string(),
            "Alice".to_string(),
            Duration::days(7),
            ProposalType::Constitutional,
            ProposalCategory::Economic,
            0.5,
            VotingSystem::OnePersonOneVote,
        ).unwrap();

        // Fast-forward time
        let mut proposal = gs.get_proposal(&proposal_id).unwrap();
        proposal.voting_ends_at = Utc::now() - Duration::hours(1);
        let update_transaction = Transaction::new_governance(
            "SYSTEM".to_string(),
            "UpdateProposal".to_string(),
            serde_json::to_vec(&proposal).unwrap()
        );
        gs.blockchain.add_transaction(update_transaction).unwrap();

        // Try to vote after the voting period has ended
        let vote_result = gs.vote(&proposal_id, "Alice".to_string(), true, None, &mut currency_system);
        assert!(vote_result.is_err());
    }

    #[test]
    fn test_multiple_proposals() {
        let (blockchain, reputation_system, mut currency_system) = create_test_environment();
        let mut gs = GovernanceSystem::new(blockchain, reputation_system);

        let proposal_id1 = gs.create_proposal(
            "Proposal 1".to_string(),
            "This is proposal 1".to_string(),
            "Alice".to_string(),
            Duration::days(7),
            ProposalType::Constitutional,
            ProposalCategory::Economic,
            0.5,
            VotingSystem::OnePersonOneVote,
        ).unwrap();

        let proposal_id2 = gs.create_proposal(
            "Proposal 2".to_string(),
            "This is proposal 2".to_string(),
            "Bob".to_string(),
            Duration::days(7),
            ProposalType::EconomicAdjustment,
            ProposalCategory::Economic,
            0.5,
            VotingSystem::TokenWeighted { max_power: 100.0 },
        ).unwrap();

        gs.vote(&proposal_id1, "Alice".to_string(), true, None, &mut currency_system).unwrap();
        gs.vote(&proposal_id1, "Bob".to_string(), false, None, &mut currency_system).unwrap();
        gs.vote(&proposal_id2, "Alice".to_string(), false, None, &mut currency_system).unwrap();
        gs.vote(&proposal_id2, "Bob".to_string(), true, None, &mut currency_system).unwrap();

        // Fast-forward time
        for proposal_id in &[&proposal_id1, &proposal_id2] {
            let mut proposal = gs.get_proposal(proposal_id).unwrap();
            proposal.voting_ends_at = Utc::now() - Duration::hours(1);
            let update_transaction = Transaction::new_governance(
                "SYSTEM".to_string(),
                "UpdateProposal".to_string(),
                serde_json::to_vec(&proposal).unwrap()
            );
            gs.blockchain.add_transaction(update_transaction).unwrap();
        }

        let result1 = gs.tally_votes(&proposal_id1).unwrap();
        let result2 = gs.tally_votes(&proposal_id2).unwrap();

        assert_eq!(result1, ProposalStatus::Passed);
        assert_eq!(result2, ProposalStatus::Passed);

        let active_proposals = gs.list_active_proposals().unwrap();
        assert_eq!(active_proposals.len(), 0);

        let proposal_result1 = gs.get_proposal_result(&proposal_id1).unwrap();
        let proposal_result2 = gs.get_proposal_result(&proposal_id2).unwrap();

        assert_eq!(proposal_result1.total_votes, 2);
        assert_eq!(proposal_result1.total_power, 2.0);
        assert_eq!(proposal_result1.in_favor_power, 1.0);
        assert_eq!(proposal_result1.against_power, 1.0);

        assert_eq!(proposal_result2.total_votes, 2);
        assert_eq!(proposal_result2.total_power, 200.0);
        assert_eq!(proposal_result2.in_favor_power, 100.0);
        assert_eq!(proposal_result2.against_power, 100.0);
    }

    #[test]
    fn test_proposal_implementation() {
        let (blockchain, reputation_system, mut currency_system) = create_test_environment();
        let mut gs = GovernanceSystem::new(blockchain, reputation_system);

        let proposal_id = gs.create_proposal(
            "Implementation Test".to_string(),
            "This proposal will be implemented".to_string(),
            "Alice".to_string(),
            Duration::days(1),
            ProposalType::NetworkUpgrade,
            ProposalCategory::Technical,
            0.5,
            VotingSystem::OnePersonOneVote,
        ).unwrap();

        gs.vote(&proposal_id, "Alice".to_string(), true, None, &mut currency_system).unwrap();
        gs.vote(&proposal_id, "Bob".to_string(), true, None, &mut currency_system).unwrap();

        // Fast-forward time
        let mut proposal = gs.get_proposal(&proposal_id).unwrap();
        proposal.voting_ends_at = Utc::now() - Duration::hours(1);
        let update_transaction = Transaction::new_governance(
            "SYSTEM".to_string(),
            "UpdateProposal".to_string(),
            serde_json::to_vec(&proposal).unwrap()
        );
        gs.blockchain.add_transaction(update_transaction).unwrap();

        gs.tally_votes(&proposal_id).unwrap();
        
        assert!(gs.mark_as_implemented(&proposal_id).is_ok());
        
        let implemented_proposal = gs.get_proposal(&proposal_id).unwrap();
        assert_eq!(implemented_proposal.status, ProposalStatus::Implemented);
        assert!(implemented_proposal.execution_timestamp.is_some());
    }

    #[test]
    fn test_vote_power_limits() {
        let (blockchain, reputation_system, mut currency_system) = create_test_environment();
        let mut gs = GovernanceSystem::new(blockchain, reputation_system);

        // Mint more tokens for Alice
        currency_system.mint("Alice", &CurrencyType::Governance, 9000.0).unwrap();

        let proposal_id = gs.create_proposal(
            "Vote Power Limit Test".to_string(),
            "This proposal tests vote power limits".to_string(),
            "Bob".to_string(),
            Duration::days(7),
            ProposalType::EconomicAdjustment,
            ProposalCategory::Economic,
            5000.0,
            VotingSystem::TokenWeighted { max_power: 5000.0 },
        ).unwrap();

        gs.vote(&proposal_id, "Alice".to_string(), true, None, &mut currency_system).unwrap();
        gs.vote(&proposal_id, "Bob".to_string(), false, None, &mut currency_system).unwrap();

        // Fast-forward time
        let mut proposal = gs.get_proposal(&proposal_id).unwrap();
        proposal.voting_ends_at = Utc::now() - Duration::hours(1);
        let update_transaction = Transaction::new_governance(
            "SYSTEM".to_string(),
            "UpdateProposal".to_string(),
            serde_json::to_vec(&proposal).unwrap()
        );
        gs.blockchain.add_transaction(update_transaction).unwrap();

        let result = gs.tally_votes(&proposal_id).unwrap();
        assert_eq!(result, ProposalStatus::Passed);

        let proposal_result = gs.get_proposal_result(&proposal_id).unwrap();
        assert_eq!(proposal_result.total_votes, 2);
        assert_eq!(proposal_result.total_power, 6000.0); // 5000 (Alice's limit) + 1000 (Bob)
        assert_eq!(proposal_result.in_favor_power, 5000.0);
        assert_eq!(proposal_result.against_power, 1000.0);
        assert!(proposal_result.quorum_reached);
        assert!(proposal_result.passed);
    }

    #[test]
    fn test_quadratic_voting_limits() {
        let (blockchain, reputation_system, mut currency_system) = create_test_environment();
        let mut gs = GovernanceSystem::new(blockchain, reputation_system);

        let proposal_id = gs.create_proposal(
            "Quadratic Voting Limit Test".to_string(),
            "This proposal tests quadratic voting limits".to_string(),
            "Alice".to_string(),
            Duration::days(7),
            ProposalType::NetworkUpgrade,
            ProposalCategory::Technical,
            10.0,
            VotingSystem::Quadratic { max_votes: 10 },
        ).unwrap();

        // This should succeed (9 votes, cost of 81 tokens)
        assert!(gs.vote(&proposal_id, "Alice".to_string(), true, Some(9.0), &mut currency_system).is_ok());

        // This should fail (11 votes, exceeds max_votes)
        assert!(gs.vote(&proposal_id, "Bob".to_string(), false, Some(11.0), &mut currency_system).is_err());

        // This should succeed (4 votes, cost of 16 tokens)
        assert!(gs.vote(&proposal_id, "Bob".to_string(), false, Some(4.0), &mut currency_system).is_ok());

        // Fast-forward time
        let mut proposal = gs.get_proposal(&proposal_id).unwrap();
        proposal.voting_ends_at = Utc::now() - Duration::hours(1);
        let update_transaction = Transaction::new_governance(
            "SYSTEM".to_string(),
            "UpdateProposal".to_string(),
            serde_json::to_vec(&proposal).unwrap()
        );
        gs.blockchain.add_transaction(update_transaction).unwrap();

        let result = gs.tally_votes(&proposal_id).unwrap();
        assert_eq!(result, ProposalStatus::Passed);

        let proposal_result = gs.get_proposal_result(&proposal_id).unwrap();
        assert_eq!(proposal_result.total_votes, 2);
        assert_eq!(proposal_result.total_power, 5.0); // 3 (sqrt of 9) + 2 (sqrt of 4)
        assert_eq!(proposal_result.in_favor_power, 3.0);
        assert_eq!(proposal_result.against_power, 2.0);
        assert!(proposal_result.quorum_reached);
        assert!(proposal_result.passed);

        // Check if Alice's balance was deducted correctly
        let alice_balance = currency_system.get_balance("Alice", &CurrencyType::Governance).unwrap();
        assert_eq!(alice_balance, 919.0); // 1000 - 81 (cost of 9 votes)

        // Check if Bob's balance was deducted correctly
        let bob_balance = currency_system.get_balance("Bob", &CurrencyType::Governance).unwrap();
        assert_eq!(bob_balance, 984.0); // 1000 - 16 (cost of 4 votes)
    }

    #[test]
    fn test_edge_cases() {
        let (blockchain, reputation_system, mut currency_system) = create_test_environment();
        let mut gs = GovernanceSystem::new(blockchain, reputation_system);

        // Test case 1: Create a proposal with 0 quorum requirement
        let proposal_id1 = gs.create_proposal(
            "Zero Quorum Proposal".to_string(),
            "This proposal has zero quorum requirement".to_string(),
            "Alice".to_string(),
            Duration::days(7),
            ProposalType::Constitutional,
            ProposalCategory::Economic,
            0.0,
            VotingSystem::OnePersonOneVote,
        ).unwrap();

        // Test case 2: Create a proposal with 100% quorum requirement
        let proposal_id2 = gs.create_proposal(
            "Full Quorum Proposal".to_string(),
            "This proposal requires full quorum".to_string(),
            "Bob".to_string(),
            Duration::days(7),
            ProposalType::EconomicAdjustment,
            ProposalCategory::Economic,
            f64::MAX,
            VotingSystem::TokenWeighted { max_power: f64::MAX },
        ).unwrap();

        // Vote on zero quorum proposal
        gs.vote(&proposal_id1, "Alice".to_string(), true, None, &mut currency_system).unwrap();

        // Vote on full quorum proposal
        gs.vote(&proposal_id2, "Alice".to_string(), true, None, &mut currency_system).unwrap();
        gs.vote(&proposal_id2, "Bob".to_string(), true, None, &mut currency_system).unwrap();

        // Fast-forward time
        for proposal_id in &[&proposal_id1, &proposal_id2] {
            let mut proposal = gs.get_proposal(proposal_id).unwrap();
            proposal.voting_ends_at = Utc::now() - Duration::hours(1);
            let update_transaction = Transaction::new_governance(
                "SYSTEM".to_string(),
                "UpdateProposal".to_string(),
                serde_json::to_vec(&proposal).unwrap()
            );
            gs.blockchain.add_transaction(update_transaction).unwrap();
        }

        // Tally votes for zero quorum proposal
        let result1 = gs.tally_votes(&proposal_id1).unwrap();
        assert_eq!(result1, ProposalStatus::Passed);

        let proposal_result1 = gs.get_proposal_result(&proposal_id1).unwrap();
        assert_eq!(proposal_result1.total_votes, 1);
        assert_eq!(proposal_result1.total_power, 1.0);
        assert_eq!(proposal_result1.in_favor_power, 1.0);
        assert_eq!(proposal_result1.against_power, 0.0);
        assert!(proposal_result1.quorum_reached);
        assert!(proposal_result1.passed);

        // Tally votes for full quorum proposal
        let result2 = gs.tally_votes(&proposal_id2).unwrap();
        assert_eq!(result2, ProposalStatus::Rejected);

        let proposal_result2 = gs.get_proposal_result(&proposal_id2).unwrap();
        assert_eq!(proposal_result2.total_votes, 2);
        assert_eq!(proposal_result2.total_power, 2000.0);
        assert_eq!(proposal_result2.in_favor_power, 2000.0);
        assert_eq!(proposal_result2.against_power, 0.0);
        assert!(!proposal_result2.quorum_reached);
        assert!(!proposal_result2.passed);

        // Test case 3: Try to vote with negative power (should fail)
        let proposal_id3 = gs.create_proposal(
            "Negative Vote Test".to_string(),
            "This proposal tests negative voting power".to_string(),
            "Alice".to_string(),
            Duration::days(7),
            ProposalType::NetworkUpgrade,
            ProposalCategory::Technical,
            1.0,
            VotingSystem::Quadratic { max_votes: 10 },
        ).unwrap();

        assert!(gs.vote(&proposal_id3, "Alice".to_string(), true, Some(-1.0), &mut currency_system).is_err());

        // Test case 4: Try to create a proposal with negative quorum (should fail)
        let proposal_id4 = gs.create_proposal(
            "Negative Quorum Test".to_string(),
            "This proposal has a negative quorum requirement".to_string(),
            "Bob".to_string(),
            Duration::days(7),
            ProposalType::Constitutional,
            ProposalCategory::Economic,
            -1.0,
            VotingSystem::OnePersonOneVote,
        );

        assert!(proposal_id4.is_err());
    }
}
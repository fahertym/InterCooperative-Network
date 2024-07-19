use crate::PoCConsensus;
use icn_core::error::{Error, Result};
use std::collections::HashMap;
use log::{info, warn, debug};

pub struct BFTPoC {
    consensus: PoCConsensus,
    proposals: HashMap<String, Proposal>,
}

struct Proposal {
    block_hash: String,
    proposer: String,
    votes: HashMap<String, bool>,
    status: ProposalStatus,
}

enum ProposalStatus {
    Proposed,
    Voting,
    Committed,
    Rejected,
}

impl BFTPoC {
    pub fn new(threshold: f64, quorum: f64) -> Self {
        BFTPoC {
            consensus: PoCConsensus::new(threshold, quorum),
            proposals: HashMap::new(),
        }
    }

    pub fn propose_block(&mut self, proposer: &str, block_hash: &str) -> Result<()> {
        debug!("Proposing block: {} by {}", block_hash, proposer);
        if !self.consensus.is_validator(proposer) {
            return Err(Error::ConsensusError("Proposer is not a validator".to_string()));
        }

        let proposal = Proposal {
            block_hash: block_hash.to_string(),
            proposer: proposer.to_string(),
            votes: HashMap::new(),
            status: ProposalStatus::Proposed,
        };

        self.proposals.insert(block_hash.to_string(), proposal);
        info!("Block proposed: {}", block_hash);
        Ok(())
    }

    pub fn vote_on_block(&mut self, voter: &str, block_hash: &str, vote: bool) -> Result<()> {
        debug!("Vote received for block {} from {}: {}", block_hash, voter, vote);
        let proposal = self.proposals.get_mut(block_hash)
            .ok_or_else(|| Error::ConsensusError("Proposal not found".to_string()))?;

        if !self.consensus.is_validator(voter) {
            return Err(Error::ConsensusError("Voter is not a validator".to_string()));
        }

        proposal.votes.insert(voter.to_string(), vote);
        proposal.status = ProposalStatus::Voting;
        info!("Vote recorded for block {}", block_hash);
        Ok(())
    }

    pub fn finalize_block(&mut self, block_hash: &str) -> Result<bool> {
        debug!("Finalizing block: {}", block_hash);
        let proposal = self.proposals.get(block_hash)
            .ok_or_else(|| Error::ConsensusError("Proposal not found".to_string()))?;

        let votes: Vec<(&str, bool)> = proposal.votes.iter()
            .map(|(k, v)| (k.as_str(), *v))
            .collect();

        match self.consensus.validate_block(block_hash, &votes) {
            Ok(true) => {
                info!("Block {} committed", block_hash);
                self.update_reputations(block_hash, true)?;
                self.proposals.get_mut(block_hash).unwrap().status = ProposalStatus::Committed;
                Ok(true)
            }
            Ok(false) => {
                warn!("Block {} rejected", block_hash);
                self.update_reputations(block_hash, false)?;
                self.proposals.get_mut(block_hash).unwrap().status = ProposalStatus::Rejected;
                Ok(false)
            }
            Err(e) => Err(e),
        }
    }

    fn update_reputations(&mut self, block_hash: &str, committed: bool) -> Result<()> {
        let proposal = self.proposals.get(block_hash)
            .ok_or_else(|| Error::ConsensusError("Proposal not found".to_string()))?;

        for (voter, vote) in &proposal.votes {
            let reputation_change = if *vote == committed { 0.1 } else { -0.1 };
            self.consensus.update_reputation(voter, reputation_change)?;
        }

        Ok(())
    }

    pub fn get_proposal_status(&self, block_hash: &str) -> Result<ProposalStatus> {
        self.proposals.get(block_hash)
            .map(|p| p.status.clone())
            .ok_or_else(|| Error::ConsensusError("Proposal not found".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_bft_poc() -> BFTPoC {
        let mut bft_poc = BFTPoC::new(0.5, 0.66);
        bft_poc.consensus.add_member("Alice".to_string(), true).unwrap();
        bft_poc.consensus.add_member("Bob".to_string(), true).unwrap();
        bft_poc.consensus.add_member("Charlie".to_string(), true).unwrap();
        bft_poc
    }

    #[test]
    fn test_propose_and_vote() {
        let mut bft_poc = setup_bft_poc();

        assert!(bft_poc.propose_block("Alice", "block1").is_ok());
        assert!(bft_poc.vote_on_block("Alice", "block1", true).is_ok());
        assert!(bft_poc.vote_on_block("Bob", "block1", true).is_ok());
        assert!(bft_poc.vote_on_block("Charlie", "block1", false).is_ok());

        assert!(matches!(bft_poc.get_proposal_status("block1"), Ok(ProposalStatus::Voting)));
    }

    #[test]
    fn test_finalize_block() {
        let mut bft_poc = setup_bft_poc();

        bft_poc.propose_block("Alice", "block1").unwrap();
        bft_poc.vote_on_block("Alice", "block1", true).unwrap();
        bft_poc.vote_on_block("Bob", "block1", true).unwrap();
        bft_poc.vote_on_block("Charlie", "block1", false).unwrap();

        assert!(bft_poc.finalize_block("block1").unwrap());
        assert!(matches!(bft_poc.get_proposal_status("block1"), Ok(ProposalStatus::Committed)));

        // Check reputation updates
        assert!(bft_poc.consensus.members.iter().any(|m| m.id == "Alice" && m.reputation > 1.0));
        assert!(bft_poc.consensus.members.iter().any(|m| m.id == "Bob" && m.reputation > 1.0));
        assert!(bft_poc.consensus.members.iter().any(|m| m.id == "Charlie" && m.reputation < 1.0));
    }

    #[test]
    fn test_reject_block() {
        let mut bft_poc = setup_bft_poc();

        bft_poc.propose_block("Alice", "block2").unwrap();
        bft_poc.vote_on_block("Alice", "block2", false).unwrap();
        bft_poc.vote_on_block("Bob", "block2", false).unwrap();
        bft_poc.vote_on_block("Charlie", "block2", true).unwrap();

        assert!(!bft_poc.finalize_block("block2").unwrap());
        assert!(matches!(bft_poc.get_proposal_status("block2"), Ok(ProposalStatus::Rejected)));

        // Check reputation updates
        assert!(bft_poc.consensus.members.iter().any(|m| m.id == "Alice" && m.reputation > 1.0));
        assert!(bft_poc.consensus.members.iter().any(|m| m.id == "Bob" && m.reputation > 1.0));
        assert!(bft_poc.consensus.members.iter().any(|m| m.id == "Charlie" && m.reputation < 1.0));
    }

    #[test]
    fn test_invalid_proposer() {
        let mut bft_poc = setup_bft_poc();

        assert!(bft_poc.propose_block("David", "block3").is_err());
    }

    #[test]
    fn test_invalid_voter() {
        let mut bft_poc = setup_bft_poc();

        bft_poc.propose_block("Alice", "block4").unwrap();
        assert!(bft_poc.vote_on_block("David", "block4", true).is_err());
    }

    #[test]
    fn test_finalize_non_existent_block() {
        let mut bft_poc = setup_bft_poc();

        assert!(bft_poc.finalize_block("non_existent_block").is_err());
    }

    #[test]
    fn test_get_status_non_existent_block() {
        let bft_poc = setup_bft_poc();

        assert!(bft_poc.get_proposal_status("non_existent_block").is_err());
    }
}
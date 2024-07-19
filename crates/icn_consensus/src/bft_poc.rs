use icn_common::{Error, Result};

#[derive(Debug)]
pub enum ProposalStatus {
    Pending,
    Approved,
    Rejected,
}

impl Clone for ProposalStatus {
    fn clone(&self) -> Self {
        match self {
            ProposalStatus::Pending => ProposalStatus::Pending,
            ProposalStatus::Approved => ProposalStatus::Approved,
            ProposalStatus::Rejected => ProposalStatus::Rejected,
        }
    }
}

pub struct Proposal {
    pub id: String,
    pub status: ProposalStatus,
    pub votes: Vec<Vote>,
}

#[derive(Clone)]
pub struct Vote {
    pub member_id: String,
    pub vote: bool,
}

pub struct BFTPoC {
    pub proposals: Vec<Proposal>,
}

impl BFTPoC {
    pub fn new() -> Self {
        BFTPoC {
            proposals: Vec::new(),
        }
    }

    pub fn create_proposal(&mut self, proposal_id: String) -> Result<()> {
        if self.proposals.iter().any(|p| p.id == proposal_id) {
            return Err(Error {
                message: "Proposal already exists".to_string(),
            });
        }

        let proposal = Proposal {
            id: proposal_id,
            status: ProposalStatus::Pending,
            votes: Vec::new(),
        };

        self.proposals.push(proposal);
        Ok(())
    }

    pub fn vote_on_proposal(&mut self, proposal_id: &str, member_id: String, vote: bool) -> Result<()> {
        let proposal = self.proposals.iter_mut().find(|p| p.id == proposal_id).ok_or_else(|| Error {
            message: "Proposal not found".to_string(),
        })?;

        if proposal.votes.iter().any(|v| v.member_id == member_id) {
            return Err(Error {
                message: "Member has already voted".to_string(),
            });
        }

        proposal.votes.push(Vote { member_id, vote });
        Ok(())
    }

    pub fn finalize_proposal(&mut self, proposal_id: &str) -> Result<ProposalStatus> {
        let proposal = self.proposals.iter_mut().find(|p| p.id == proposal_id).ok_or_else(|| Error {
            message: "Proposal not found".to_string(),
        })?;

        let positive_votes = proposal.votes.iter().filter(|v| v.vote).count();
        let negative_votes = proposal.votes.len() - positive_votes;

        proposal.status = if positive_votes > negative_votes {
            ProposalStatus::Approved
        } else {
            ProposalStatus::Rejected
        };

        Ok(proposal.status.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_proposal() {
        let mut bft_poc = BFTPoC::new();

        assert!(bft_poc.create_proposal("proposal1".to_string()).is_ok());
        assert_eq!(bft_poc.proposals.len(), 1);

        assert!(bft_poc.create_proposal("proposal1".to_string()).is_err());
    }

    #[test]
    fn test_vote_on_proposal() {
        let mut bft_poc = BFTPoC::new();
        bft_poc.create_proposal("proposal1".to_string()).unwrap();

        assert!(bft_poc.vote_on_proposal("proposal1", "member1".to_string(), true).is_ok());
        assert_eq!(bft_poc.proposals[0].votes.len(), 1);

        assert!(bft_poc.vote_on_proposal("proposal1", "member1".to_string(), false).is_err());
    }

    #[test]
    fn test_finalize_proposal() {
        let mut bft_poc = BFTPoC::new();
        bft_poc.create_proposal("proposal1".to_string()).unwrap();

        bft_poc.vote_on_proposal("proposal1", "member1".to_string(), true).unwrap();
        bft_poc.vote_on_proposal("proposal1", "member2".to_string(), false).unwrap();
        bft_poc.vote_on_proposal("proposal1", "member3".to_string(), true).unwrap();

        let status = bft_poc.finalize_proposal("proposal1").unwrap();
        assert_eq!(status, ProposalStatus::Approved);

        bft_poc.create_proposal("proposal2".to_string()).unwrap();
        bft_poc.vote_on_proposal("proposal2", "member1".to_string(), false).unwrap();
        bft_poc.vote_on_proposal("proposal2", "member2".to_string(), false).unwrap();
        bft_poc.vote_on_proposal("proposal2", "member3".to_string(), true).unwrap();

        let status = bft_poc.finalize_proposal("proposal2").unwrap();
        assert_eq!(status, ProposalStatus::Rejected);
    }
}

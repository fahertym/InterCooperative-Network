pub mod bft_poc;
pub mod consensus;
pub mod proof_of_cooperation;

pub use bft_poc::BFTPoC;
pub use consensus::PoCConsensus;
pub use proof_of_cooperation::ProofOfCooperation;

use log::error;
use icn_utils::{error::IcnError, IcnResult};
use icn_utils::types::{Block, Transaction};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consensus() {
        let mut consensus = PoCConsensus::new();
        assert!(consensus.add_member("Alice".to_string(), true).is_ok());
        assert!(consensus.add_member("Bob".to_string(), false).is_ok());

        let votes = vec![
            ("Alice", true),
            ("Bob", false),
        ];

        assert!(consensus.validate_block("block_hash", &votes).unwrap());
    }

    #[test]
    fn test_bft_poc() {
        let mut bft_poc = BFTPoC::new();
        assert!(bft_poc.create_proposal("proposal1".to_string()).is_ok());
        assert_eq!(bft_poc.proposals.len(), 1);

        assert!(bft_poc.vote_on_proposal("proposal1", "member1".to_string(), true).is_ok());
        assert_eq!(bft_poc.proposals[0].votes.len(), 1);

        let status = bft_poc.finalize_proposal("proposal1").unwrap();
        assert_eq!(status, ProposalStatus::Approved);
    }
}

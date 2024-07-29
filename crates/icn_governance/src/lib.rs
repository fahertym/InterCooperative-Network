// File: crates/icn_governance/src/lib.rs

mod governance;
mod proposal;
mod voting;

pub use governance::GovernanceSystem;
pub use proposal::{Proposal, ProposalCategory, ProposalStatus, ProposalType};
pub use voting::Vote;

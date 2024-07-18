//! Program error types.

use spl_program_error::*;

/// Errors that can be returned by the Paladin Governance program.
#[spl_program_error]
pub enum PaladinGovernanceError {
    /// Stake config accounts mismatch.
    #[error("Stake config accounts mismatch.")]
    StakeConfigMismatch,
    /// Incorrect proposal vote address.
    #[error("Incorrect proposal vote address.")]
    IncorrectProposalVoteAddress,
    /// Incorrect governance config address.
    #[error("Incorrect governance config address.")]
    IncorrectGovernanceConfigAddress,
    /// Proposal not accepted.
    #[error("Proposal not accepted.")]
    ProposalNotAccepted,
}

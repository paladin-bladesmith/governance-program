//! Program error types.

use spl_program_error::*;

/// Errors that can be returned by the Paladin Governance program.
#[spl_program_error]
pub enum PaladinGovernanceError {
    /// Stake config accounts mismatch.
    #[error("Stake config accounts mismatch.")]
    StakeConfigMismatch,
    /// Incorrect stake config.
    #[error("Incorrect stake config.")]
    IncorrectStakeConfig,
    /// Incorrect proposal transaction address.
    #[error("Incorrect proposal transaction address.")]
    IncorrectProposalTransactionAddress,
    /// Incorrect proposal vote address.
    #[error("Incorrect proposal vote address.")]
    IncorrectProposalVoteAddress,
    /// Incorrect governance config address.
    #[error("Incorrect governance config address.")]
    IncorrectGovernanceConfigAddress,
    /// Proposal not in voting stage.
    #[error("Proposal not in voting stage.")]
    ProposalNotInVotingStage,
    /// Proposal is immutable.
    #[error("Proposal is immutable.")]
    ProposalIsImmutable,
    /// Proposal not accepted.
    #[error("Proposal not accepted.")]
    ProposalNotAccepted,
    /// Invalid transaction index.
    #[error("Invalid transaction index.")]
    InvalidTransactionIndex,
}

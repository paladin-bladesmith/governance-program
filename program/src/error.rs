//! Program error types.

use {
    num_derive::FromPrimitive,
    solana_program::{
        decode_error::DecodeError,
        msg,
        program_error::{PrintProgramError, ProgramError},
    },
    thiserror::Error,
};

/// Errors that can be returned by the Paladin Governance program.
// Note: Shank does not export the type when we use `spl_program_error`.
#[derive(Error, Clone, Debug, Eq, PartialEq, FromPrimitive)]
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
    /// Incorrect treasury address.
    #[error("Incorrect treasury address.")]
    IncorrectTreasuryAddress,
    /// Proposal not in voting stage.
    #[error("Proposal not in voting stage.")]
    ProposalNotInVotingStage,
    /// Proposal is immutable.
    #[error("Proposal is immutable.")]
    ProposalIsImmutable,
    /// Proposal not accepted.
    #[error("Proposal not accepted.")]
    ProposalNotAccepted,
    /// Proposal voting period still active.
    #[error("Proposal voting period still active.")]
    ProposalVotingPeriodStillActive,
    /// Invalid transaction index.
    #[error("Invalid transaction index.")]
    InvalidTransactionIndex,
    /// Instruction already executed.
    #[error("Instruction already executed.")]
    InstructionAlreadyExecuted,
    /// Previous instruction has not been executed.
    #[error("Previous instruction has not been executed.")]
    PreviousInstructionHasNotBeenExecuted,
}

impl PrintProgramError for PaladinGovernanceError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl From<PaladinGovernanceError> for ProgramError {
    fn from(e: PaladinGovernanceError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for PaladinGovernanceError {
    fn type_of() -> &'static str {
        "PaladinGovernanceError"
    }
}

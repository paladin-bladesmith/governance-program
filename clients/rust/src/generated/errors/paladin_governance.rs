//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>

use {num_derive::FromPrimitive, thiserror::Error};

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum PaladinGovernanceError {
    /// 0 - Stake config accounts mismatch.
    #[error("Stake config accounts mismatch.")]
    StakeConfigMismatch = 0x0,
    /// 1 - Incorrect stake config.
    #[error("Incorrect stake config.")]
    IncorrectStakeConfig = 0x1,
    /// 2 - Incorrect proposal transaction address.
    #[error("Incorrect proposal transaction address.")]
    IncorrectProposalTransactionAddress = 0x2,
    /// 3 - Incorrect proposal vote address.
    #[error("Incorrect proposal vote address.")]
    IncorrectProposalVoteAddress = 0x3,
    /// 4 - Incorrect governance config address.
    #[error("Incorrect governance config address.")]
    IncorrectGovernanceConfigAddress = 0x4,
    /// 5 - Incorrect treasury address.
    #[error("Incorrect treasury address.")]
    IncorrectTreasuryAddress = 0x5,
    /// 6 - Proposal not in voting stage.
    #[error("Proposal not in voting stage.")]
    ProposalNotInVotingStage = 0x6,
    /// 7 - Proposal is immutable.
    #[error("Proposal is immutable.")]
    ProposalIsImmutable = 0x7,
    /// 8 - Proposal not accepted.
    #[error("Proposal not accepted.")]
    ProposalNotAccepted = 0x8,
    /// 9 - Proposal voting period still active.
    #[error("Proposal voting period still active.")]
    ProposalVotingPeriodStillActive = 0x9,
    /// 10 - Invalid transaction index.
    #[error("Invalid transaction index.")]
    InvalidTransactionIndex = 0xA,
    /// 11 - Instruction already executed.
    #[error("Instruction already executed.")]
    InstructionAlreadyExecuted = 0xB,
    /// 12 - Previous instruction has not been executed.
    #[error("Previous instruction has not been executed.")]
    PreviousInstructionHasNotBeenExecuted = 0xC,
}

impl solana_program::program_error::PrintProgramError for PaladinGovernanceError {
    fn print<E>(&self) {
        solana_program::msg!(&self.to_string());
    }
}
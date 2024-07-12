//! Program instruction types.

/// Instructions supported by the Paladin Governance program.
pub enum PaladinGovernanceInstruction {
    /// Create a new governance proposal.
    ///
    /// Creates a new proposal with an instruction. Some examples of
    /// instructions that can be configured:
    ///
    /// * Slash a validator.
    /// * Transfer X tokens from the treasury.
    /// * Burn X tokens from the treasury.
    ///
    /// Expects an uninitialized proposal account with enough rent-exempt
    /// lamports to store proposal state, owned by the Paladin Governance
    /// program.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[s]` Stake account.
    /// 1. `[w]` Proposal account.
    CreateProposal,
    /// Cancel a governance proposal.
    ///
    /// Stake account provided must be the proposal creator.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[s]` Stake account.
    /// 1. `[w]` Proposal account.
    CancelProposal,
    /// Vote on a governance proposal.
    ///
    /// Expects an uninitialized vote account with enough rent-exempt lamports
    /// to store vote state.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[s]` Stake account.
    /// 1. `[w]` Vote account.
    /// 2. `[w]` Proposal account.
    /// 3. `[ ]` Governance config account.
    Vote {
        /// Vote.
        ///
        /// * `true`: In favor.
        /// * `false`: Against.
        vote: bool,
    },
    /// Vote on a governance proposal.
    ///
    /// Expects an existing vote account, representing a previously cast vote.
    ///
    /// If the cast vote results in >= 50% majority:
    ///
    /// * In favor: Begins the cooldown period.
    /// * Against: Terminates the proposal immediately.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[s]` Stake account.
    /// 1. `[w]` Vote account.
    /// 2. `[w]` Proposal account.
    /// 3. `[ ]` Governance config account.
    SwitchVote {
        /// Vote.
        ///
        /// * `true`: In favor.
        /// * `false`: Against.
        vote: bool,
    },
    /// Process a governance proposal.
    ///
    /// Given an accepted proposal, execute it. An accepted proposal has at
    /// least 50% majority vote and has passed the cooldown period.
    ///
    /// Closes the proposal account after execution.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[w]` Proposal account.
    /// 1. `[ ]` Governance config account.
    ProcessProposal,
    /// Initialize the governance config.
    ///
    /// Initializes the configurations that will dictate governance
    /// constraints, including:
    ///
    /// * The cooldown period for proposal execution.
    /// * Minimum required majority threshold.
    ///
    /// This instruction can only be invoked once.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[w]` Governance config account.
    InitializeGovernance,
    /// Update the governance config.
    ///
    /// Allows modification of the governance config, including:
    ///
    /// * The cooldown period for proposal execution.
    /// * Minimum required majority threshold.
    ///
    /// This instruction can only be executed from an accepted proposal.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[w]` Governance config account.
    /// 1. `[ ]` Proposal account.
    UpdateGovernance,
}

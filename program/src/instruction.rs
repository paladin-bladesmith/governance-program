//! Program instruction types.

use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program,
};

/// Instructions supported by the Paladin Governance program.
#[derive(Clone, Copy, Debug, PartialEq)]
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
    /// 0. `[s]` Paladin stake authority account.
    /// 1. `[ ]` Paladin stake account.
    /// 2. `[w]` Proposal account.
    CreateProposal,
    /// Cancel a governance proposal.
    ///
    /// Authority account provided must be the proposal creator.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[s]` Paladin stake authority account.
    /// 1. `[w]` Proposal account.
    CancelProposal,
    /// Vote on a governance proposal.
    ///
    /// Expects an uninitialized proposal vote account with enough rent-exempt
    /// lamports to store proposal vote state.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[s]` Paladin stake authority account.
    /// 1. `[ ]` Paladin stake account.
    /// 2. `[ ]` Paladin stake config account.
    /// 3. `[w]` Proposal vote account.
    /// 4. `[w]` Proposal account.
    /// 5. `[ ]` Governance config account.
    /// 6. `[ ]` System program.
    Vote {
        /// Proposal vote.
        ///
        /// * `true`: In favor.
        /// * `false`: Against.
        vote: bool,
    },
    /// Vote on a governance proposal.
    ///
    /// Expects an existing proposal vote account, representing a previously
    /// cast proposal vote.
    ///
    /// If the cast proposal vote results in >= 50% majority:
    ///
    /// * In favor: Begins the cooldown period.
    /// * Against: Terminates the proposal immediately.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[s]` Paladin stake authority account.
    /// 1. `[ ]` Paladin stake account.
    /// 2. `[ ]` Paladin stake config account.
    /// 3. `[w]` Proposal vote account.
    /// 4. `[w]` Proposal account.
    /// 5. `[ ]` Governance config account.
    SwitchVote {
        /// Proposal vote.
        ///
        /// * `true`: In favor.
        /// * `false`: Against.
        new_vote: bool,
    },
    /// Process a governance proposal.
    ///
    /// Given an accepted proposal, execute it. An accepted proposal has at
    /// least 50% majority in favor and has passed the cooldown period.
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
    /// 1. `[ ]` Paladin stake config account.
    /// 2. `[ ]` System program.
    InitializeGovernance {
        /// The cooldown period that begins when a proposal reaches the
        /// `proposal_acceptance_threshold` and upon its conclusion will execute
        /// the proposal's instruction.
        cooldown_period_seconds: u64,
        /// The minimum required threshold of proposal acceptance to begin the
        /// cooldown period.
        proposal_acceptance_threshold: u64,
        /// The minimum required threshold of proposal rejection to terminate
        /// the proposal.
        proposal_rejection_threshold: u64,
    },
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
    UpdateGovernance {
        /// The cooldown period that begins when a proposal reaches the
        /// `proposal_acceptance_threshold` and upon its conclusion will execute
        /// the proposal's instruction.
        cooldown_period_seconds: u64,
        /// The minimum required threshold of proposal acceptance to begin the
        /// cooldown period.
        proposal_acceptance_threshold: u64,
        /// The minimum required threshold of proposal rejection to terminate
        /// the proposal.
        proposal_rejection_threshold: u64,
    },
}

impl PaladinGovernanceInstruction {
    /// Packs a
    /// [PaladinGovernanceInstruction](enum.PaladinGovernanceInstruction.html)
    /// into a byte buffer.
    pub fn pack(&self) -> Vec<u8> {
        match self {
            Self::CreateProposal => vec![0],
            Self::CancelProposal => vec![1],
            Self::Vote { vote } => {
                let mut buf = vec![2];
                buf.push(if *vote { 1 } else { 0 });
                buf
            }
            Self::SwitchVote { new_vote } => {
                let mut buf = vec![3];
                buf.push(if *new_vote { 1 } else { 0 });
                buf
            }
            Self::ProcessProposal => vec![4],
            Self::InitializeGovernance {
                cooldown_period_seconds,
                proposal_acceptance_threshold,
                proposal_rejection_threshold,
            } => {
                let mut buf = vec![5];
                buf.extend_from_slice(&cooldown_period_seconds.to_le_bytes());
                buf.extend_from_slice(&proposal_acceptance_threshold.to_le_bytes());
                buf.extend_from_slice(&proposal_rejection_threshold.to_le_bytes());
                buf
            }
            Self::UpdateGovernance {
                cooldown_period_seconds,
                proposal_acceptance_threshold,
                proposal_rejection_threshold,
            } => {
                let mut buf = vec![6];
                buf.extend_from_slice(&cooldown_period_seconds.to_le_bytes());
                buf.extend_from_slice(&proposal_acceptance_threshold.to_le_bytes());
                buf.extend_from_slice(&proposal_rejection_threshold.to_le_bytes());
                buf
            }
        }
    }

    /// Unpacks a byte buffer into a
    /// [PaladinGovernanceInstruction](enum.PaladinGovernanceInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        match input.split_first() {
            Some((&0, _)) => Ok(Self::CreateProposal),
            Some((&1, _)) => Ok(Self::CancelProposal),
            Some((&2, rest)) if rest.len() == 1 => Ok(Self::Vote { vote: rest[0] == 1 }),
            Some((&3, rest)) if rest.len() == 1 => Ok(Self::SwitchVote {
                new_vote: rest[0] == 1,
            }),
            Some((&4, _)) => Ok(Self::ProcessProposal),
            Some((&5, rest)) if rest.len() == 24 => {
                let cooldown_period_seconds = u64::from_le_bytes(rest[..8].try_into().unwrap());
                let proposal_acceptance_threshold =
                    u64::from_le_bytes(rest[8..16].try_into().unwrap());
                let proposal_rejection_threshold =
                    u64::from_le_bytes(rest[16..24].try_into().unwrap());
                Ok(Self::InitializeGovernance {
                    cooldown_period_seconds,
                    proposal_acceptance_threshold,
                    proposal_rejection_threshold,
                })
            }
            Some((&6, rest)) if rest.len() == 24 => {
                let cooldown_period_seconds = u64::from_le_bytes(rest[..8].try_into().unwrap());
                let proposal_acceptance_threshold =
                    u64::from_le_bytes(rest[8..16].try_into().unwrap());
                let proposal_rejection_threshold =
                    u64::from_le_bytes(rest[16..24].try_into().unwrap());
                Ok(Self::UpdateGovernance {
                    cooldown_period_seconds,
                    proposal_acceptance_threshold,
                    proposal_rejection_threshold,
                })
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

/// Creates a
/// [CreateProposal](enum.PaladinGovernanceInstruction.html)
/// instruction.
pub fn create_proposal(
    stake_authority_address: &Pubkey,
    stake_address: &Pubkey,
    proposal_address: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*stake_authority_address, true),
        AccountMeta::new_readonly(*stake_address, false),
        AccountMeta::new(*proposal_address, false),
    ];
    let data = PaladinGovernanceInstruction::CreateProposal.pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

/// Creates a
/// [CancelProposal](enum.PaladinGovernanceInstruction.html)
/// instruction.
pub fn cancel_proposal(stake_authority_address: &Pubkey, proposal_address: &Pubkey) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*stake_authority_address, true),
        AccountMeta::new(*proposal_address, false),
    ];
    let data = PaladinGovernanceInstruction::CancelProposal.pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

/// Creates a
/// [Vote](enum.PaladinGovernanceInstruction.html)
/// instruction.
pub fn vote(
    stake_authority_address: &Pubkey,
    stake_address: &Pubkey,
    stake_config_address: &Pubkey,
    proposal_vote_address: &Pubkey,
    proposal_address: &Pubkey,
    governance_config_address: &Pubkey,
    vote: bool,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*stake_authority_address, true),
        AccountMeta::new_readonly(*stake_address, false),
        AccountMeta::new_readonly(*stake_config_address, false),
        AccountMeta::new(*proposal_vote_address, false),
        AccountMeta::new(*proposal_address, false),
        AccountMeta::new_readonly(*governance_config_address, false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    let data = PaladinGovernanceInstruction::Vote { vote }.pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

/// Creates a
/// [SwitchVote](enum.PaladinGovernanceInstruction.html)
/// instruction.
pub fn switch_vote(
    stake_authority_address: &Pubkey,
    stake_address: &Pubkey,
    stake_config_address: &Pubkey,
    proposal_vote_address: &Pubkey,
    proposal_address: &Pubkey,
    governance_config_address: &Pubkey,
    new_vote: bool,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*stake_authority_address, true),
        AccountMeta::new_readonly(*stake_address, false),
        AccountMeta::new_readonly(*stake_config_address, false),
        AccountMeta::new(*proposal_vote_address, false),
        AccountMeta::new(*proposal_address, false),
        AccountMeta::new_readonly(*governance_config_address, false),
    ];
    let data = PaladinGovernanceInstruction::SwitchVote { new_vote }.pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

/// Creates a
/// [ProcessProposal](enum.PaladinGovernanceInstruction.html)
/// instruction.
pub fn process_proposal(
    proposal_address: &Pubkey,
    governance_config_address: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*proposal_address, false),
        AccountMeta::new_readonly(*governance_config_address, false),
    ];
    let data = PaladinGovernanceInstruction::ProcessProposal.pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

/// Creates a
/// [InitializeGovernance](enum.PaladinGovernanceInstruction.html)
/// instruction.
pub fn initialize_governance(
    governance_config_address: &Pubkey,
    stake_config_address: &Pubkey,
    cooldown_period_seconds: u64,
    proposal_acceptance_threshold: u64,
    proposal_rejection_threshold: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*governance_config_address, false),
        AccountMeta::new_readonly(*stake_config_address, false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    let data = PaladinGovernanceInstruction::InitializeGovernance {
        cooldown_period_seconds,
        proposal_acceptance_threshold,
        proposal_rejection_threshold,
    }
    .pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

/// Creates a
/// [UpdateGovernance](enum.PaladinGovernanceInstruction.html)
/// instruction.
pub fn update_governance(
    governance_config_address: &Pubkey,
    proposal_address: &Pubkey,
    cooldown_period_seconds: u64,
    proposal_acceptance_threshold: u64,
    proposal_rejection_threshold: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*governance_config_address, false),
        AccountMeta::new_readonly(*proposal_address, false),
    ];
    let data = PaladinGovernanceInstruction::UpdateGovernance {
        cooldown_period_seconds,
        proposal_acceptance_threshold,
        proposal_rejection_threshold,
    }
    .pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_pack_unpack(instruction: PaladinGovernanceInstruction) {
        let packed = instruction.pack();
        let unpacked = PaladinGovernanceInstruction::unpack(&packed).unwrap();
        assert_eq!(instruction, unpacked);
    }

    #[test]
    fn test_pack_unpack_create_proposal() {
        test_pack_unpack(PaladinGovernanceInstruction::CreateProposal);
    }

    #[test]
    fn test_pack_unpack_cancel_proposal() {
        test_pack_unpack(PaladinGovernanceInstruction::CancelProposal);
    }

    #[test]
    fn test_pack_unpack_vote() {
        test_pack_unpack(PaladinGovernanceInstruction::Vote { vote: true });
        test_pack_unpack(PaladinGovernanceInstruction::Vote { vote: false });
    }

    #[test]
    fn test_pack_unpack_switch_vote() {
        test_pack_unpack(PaladinGovernanceInstruction::SwitchVote { new_vote: true });
        test_pack_unpack(PaladinGovernanceInstruction::SwitchVote { new_vote: false });
    }

    #[test]
    fn test_pack_unpack_process_proposal() {
        test_pack_unpack(PaladinGovernanceInstruction::ProcessProposal);
    }

    #[test]
    fn test_pack_unpack_initialize_governance() {
        test_pack_unpack(PaladinGovernanceInstruction::InitializeGovernance {
            cooldown_period_seconds: 1,
            proposal_acceptance_threshold: 2,
            proposal_rejection_threshold: 3,
        });
    }

    #[test]
    fn test_pack_unpack_update_governance() {
        test_pack_unpack(PaladinGovernanceInstruction::UpdateGovernance {
            cooldown_period_seconds: 1,
            proposal_acceptance_threshold: 2,
            proposal_rejection_threshold: 3,
        });
    }
}

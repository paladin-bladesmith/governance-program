//! Program instruction types.

use {
    crate::state::{ProposalAccountMeta, ProposalVoteElection},
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        instruction::{AccountMeta, Instruction},
        program_error::ProgramError,
        pubkey::Pubkey,
        system_program,
    },
};

/// Instructions supported by the Paladin Governance program.
#[derive(Clone, Debug, PartialEq)]
pub enum PaladinGovernanceInstruction {
    /// Create a new governance proposal.
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
    /// 3. `[w]` Proposal transaction account.
    /// 4. `[ ]` Governance config account.
    /// 5. `[ ]` System program.
    CreateProposal,
    /// Insert an instruction into a governance proposal.
    ///
    /// Expects an initialized proposal and proposal transaction account.
    ///
    /// Authority account provided must be the proposal creator.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[s]` Paladin stake authority account.
    /// 1. `[ ]` Proposal account.
    /// 2. `[w]` Proposal transaction account.
    PushInstruction {
        /// The program ID to invoke.
        instruction_program_id: Pubkey,
        /// The accounts to pass to the program.
        instruction_account_metas: Vec<ProposalAccountMeta>,
        /// The data to pass to the program.
        instruction_data: Vec<u8>,
    },
    /// Removes an instruction from a governance proposal.
    ///
    /// Authority account provided must be the proposal creator.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[s]` Paladin stake authority account.
    /// 1. `[ ]` Proposal account.
    /// 2. `[w]` Proposal transaction account.
    RemoveInstruction {
        /// The index of the instruction to remove.
        instruction_index: u32,
    },
    /// Cancel a governance proposal.
    ///
    /// Authority account provided must be the proposal creator.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[s]` Paladin stake authority account.
    /// 1. `[w]` Proposal account.
    CancelProposal,
    /// Finalize a draft governance proposal and begin voting.
    ///
    /// Authority account provided must be the proposal creator.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[s]` Paladin stake authority account.
    /// 1. `[w]` Proposal account.
    BeginVoting,
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
    /// 6. `[ ]` System program.
    Vote {
        /// Proposal vote election.
        election: ProposalVoteElection,
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
    SwitchVote {
        /// New proposal vote election.
        new_election: ProposalVoteElection,
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
        proposal_acceptance_threshold: u32,
        /// The minimum required threshold of proposal rejection to terminate
        /// the proposal.
        proposal_rejection_threshold: u32,
        /// The voting period for proposals.
        voting_period_seconds: u64,
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
        proposal_acceptance_threshold: u32,
        /// The minimum required threshold of proposal rejection to terminate
        /// the proposal.
        proposal_rejection_threshold: u32,
        /// The voting period for proposals.
        voting_period_seconds: u64,
    },
}

impl PaladinGovernanceInstruction {
    /// Packs a
    /// [PaladinGovernanceInstruction](enum.PaladinGovernanceInstruction.html)
    /// into a byte buffer.
    pub fn pack(&self) -> Vec<u8> {
        match self {
            Self::CreateProposal => vec![0],
            Self::PushInstruction {
                instruction_program_id,
                instruction_account_metas,
                instruction_data,
            } => {
                let mut buf = vec![1];
                instruction_program_id.serialize(&mut buf).unwrap();
                instruction_account_metas.serialize(&mut buf).unwrap();
                instruction_data.serialize(&mut buf).unwrap();
                buf
            }
            Self::RemoveInstruction { instruction_index } => {
                let mut buf = vec![2];
                buf.extend_from_slice(&instruction_index.to_le_bytes());
                buf
            }
            Self::CancelProposal => vec![3],
            Self::BeginVoting => vec![4],
            Self::Vote { election } => vec![5, (*election).into()],
            Self::SwitchVote { new_election } => vec![6, (*new_election).into()],
            Self::ProcessProposal => vec![7],
            Self::InitializeGovernance {
                cooldown_period_seconds,
                proposal_acceptance_threshold,
                proposal_rejection_threshold,
                voting_period_seconds,
            } => {
                let mut buf = vec![8];
                buf.extend_from_slice(&cooldown_period_seconds.to_le_bytes());
                buf.extend_from_slice(&proposal_acceptance_threshold.to_le_bytes());
                buf.extend_from_slice(&proposal_rejection_threshold.to_le_bytes());
                buf.extend_from_slice(&voting_period_seconds.to_le_bytes());
                buf
            }
            Self::UpdateGovernance {
                cooldown_period_seconds,
                proposal_acceptance_threshold,
                proposal_rejection_threshold,
                voting_period_seconds,
            } => {
                let mut buf = vec![9];
                buf.extend_from_slice(&cooldown_period_seconds.to_le_bytes());
                buf.extend_from_slice(&proposal_acceptance_threshold.to_le_bytes());
                buf.extend_from_slice(&proposal_rejection_threshold.to_le_bytes());
                buf.extend_from_slice(&voting_period_seconds.to_le_bytes());
                buf
            }
        }
    }

    /// Unpacks a byte buffer into a
    /// [PaladinGovernanceInstruction](enum.PaladinGovernanceInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        match input.split_first() {
            Some((&0, _)) => Ok(Self::CreateProposal),
            Some((&1, rest)) => {
                #[derive(BorshDeserialize)]
                struct Instruction {
                    instruction_program_id: Pubkey,
                    instruction_account_metas: Vec<ProposalAccountMeta>,
                    instruction_data: Vec<u8>,
                }
                let Instruction {
                    instruction_program_id,
                    instruction_account_metas,
                    instruction_data,
                } = Instruction::try_from_slice(rest)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(Self::PushInstruction {
                    instruction_program_id,
                    instruction_account_metas,
                    instruction_data,
                })
            }
            Some((&2, rest)) if rest.len() == 4 => {
                let instruction_index = u32::from_le_bytes(rest.try_into().unwrap());
                Ok(Self::RemoveInstruction { instruction_index })
            }
            Some((&3, _)) => Ok(Self::CancelProposal),
            Some((&4, _)) => Ok(Self::BeginVoting),
            Some((&5, rest)) if rest.len() == 1 => {
                let election = rest[0]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(Self::Vote { election })
            }
            Some((&6, rest)) if rest.len() == 1 => {
                let new_election = rest[0]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(Self::SwitchVote { new_election })
            }
            Some((&7, _)) => Ok(Self::ProcessProposal),
            Some((&8, rest)) if rest.len() == 24 => {
                let cooldown_period_seconds = u64::from_le_bytes(rest[..8].try_into().unwrap());
                let proposal_acceptance_threshold =
                    u32::from_le_bytes(rest[8..12].try_into().unwrap());
                let proposal_rejection_threshold =
                    u32::from_le_bytes(rest[12..16].try_into().unwrap());
                let voting_period_seconds = u64::from_le_bytes(rest[16..24].try_into().unwrap());
                Ok(Self::InitializeGovernance {
                    cooldown_period_seconds,
                    proposal_acceptance_threshold,
                    proposal_rejection_threshold,
                    voting_period_seconds,
                })
            }
            Some((&9, rest)) if rest.len() == 24 => {
                let cooldown_period_seconds = u64::from_le_bytes(rest[..8].try_into().unwrap());
                let proposal_acceptance_threshold =
                    u32::from_le_bytes(rest[8..12].try_into().unwrap());
                let proposal_rejection_threshold =
                    u32::from_le_bytes(rest[12..16].try_into().unwrap());
                let voting_period_seconds = u64::from_le_bytes(rest[16..24].try_into().unwrap());
                Ok(Self::UpdateGovernance {
                    cooldown_period_seconds,
                    proposal_acceptance_threshold,
                    proposal_rejection_threshold,
                    voting_period_seconds,
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
    proposal_transaction_address: &Pubkey,
    governance_config_address: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*stake_authority_address, true),
        AccountMeta::new_readonly(*stake_address, false),
        AccountMeta::new(*proposal_address, false),
        AccountMeta::new(*proposal_transaction_address, false),
        AccountMeta::new_readonly(*governance_config_address, false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    let data = PaladinGovernanceInstruction::CreateProposal.pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

/// Creates a
/// [PushInstruction](enum.PaladinGovernanceInstruction.html)
/// instruction.
pub fn push_instruction(
    stake_authority_address: &Pubkey,
    proposal_address: &Pubkey,
    proposal_transaction_address: &Pubkey,
    instruction_program_id: &Pubkey,
    instruction_account_metas: Vec<ProposalAccountMeta>,
    instruction_data: Vec<u8>,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*stake_authority_address, true),
        AccountMeta::new_readonly(*proposal_address, false),
        AccountMeta::new(*proposal_transaction_address, false),
    ];
    let data = PaladinGovernanceInstruction::PushInstruction {
        instruction_program_id: *instruction_program_id,
        instruction_account_metas,
        instruction_data,
    }
    .pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

/// Creates a
/// [RemoveInstruction](enum.PaladinGovernanceInstruction.html)
/// instruction.
pub fn remove_instruction(
    stake_authority_address: &Pubkey,
    proposal_address: &Pubkey,
    proposal_transaction_address: &Pubkey,
    instruction_index: u32,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*stake_authority_address, true),
        AccountMeta::new_readonly(*proposal_address, false),
        AccountMeta::new(*proposal_transaction_address, false),
    ];
    let data = PaladinGovernanceInstruction::RemoveInstruction { instruction_index }.pack();
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
/// [BeginVoting](enum.PaladinGovernanceInstruction.html)
/// instruction.
pub fn begin_voting(stake_authority_address: &Pubkey, proposal_address: &Pubkey) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*stake_authority_address, true),
        AccountMeta::new(*proposal_address, false),
    ];
    let data = PaladinGovernanceInstruction::BeginVoting.pack();
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
    election: ProposalVoteElection,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*stake_authority_address, true),
        AccountMeta::new_readonly(*stake_address, false),
        AccountMeta::new_readonly(*stake_config_address, false),
        AccountMeta::new(*proposal_vote_address, false),
        AccountMeta::new(*proposal_address, false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    let data = PaladinGovernanceInstruction::Vote { election }.pack();
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
    new_election: ProposalVoteElection,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*stake_authority_address, true),
        AccountMeta::new_readonly(*stake_address, false),
        AccountMeta::new_readonly(*stake_config_address, false),
        AccountMeta::new(*proposal_vote_address, false),
        AccountMeta::new(*proposal_address, false),
    ];
    let data = PaladinGovernanceInstruction::SwitchVote { new_election }.pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

/// Creates a
/// [ProcessProposal](enum.PaladinGovernanceInstruction.html)
/// instruction.
pub fn process_proposal(proposal_address: &Pubkey) -> Instruction {
    let accounts = vec![AccountMeta::new(*proposal_address, false)];
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
    proposal_acceptance_threshold: u32,
    proposal_rejection_threshold: u32,
    voting_period_seconds: u64,
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
        voting_period_seconds,
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
    proposal_acceptance_threshold: u32,
    proposal_rejection_threshold: u32,
    voting_period_seconds: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*governance_config_address, false),
        AccountMeta::new_readonly(*proposal_address, false),
    ];
    let data = PaladinGovernanceInstruction::UpdateGovernance {
        cooldown_period_seconds,
        proposal_acceptance_threshold,
        proposal_rejection_threshold,
        voting_period_seconds,
    }
    .pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

#[cfg(test)]
mod tests {
    use {super::*, crate::state::ProposalAccountMeta};

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
    fn test_pack_unpack_push_instruction() {
        let program_id = Pubkey::new_unique();
        let account_metas = vec![
            ProposalAccountMeta {
                pubkey: Pubkey::new_unique(),
                is_signer: false,
                is_writable: false,
            },
            ProposalAccountMeta {
                pubkey: Pubkey::new_unique(),
                is_signer: false,
                is_writable: true,
            },
        ];
        let data = vec![1, 2, 3];
        test_pack_unpack(PaladinGovernanceInstruction::PushInstruction {
            instruction_program_id: program_id,
            instruction_account_metas: account_metas,
            instruction_data: data,
        });
    }

    #[test]
    fn test_pack_unpack_remove_instruction() {
        test_pack_unpack(PaladinGovernanceInstruction::RemoveInstruction {
            instruction_index: 45,
        });
    }

    #[test]
    fn test_pack_unpack_cancel_proposal() {
        test_pack_unpack(PaladinGovernanceInstruction::CancelProposal);
    }

    #[test]
    fn test_pack_unpack_begin_voting() {
        test_pack_unpack(PaladinGovernanceInstruction::BeginVoting);
    }

    #[test]
    fn test_pack_unpack_vote() {
        test_pack_unpack(PaladinGovernanceInstruction::Vote {
            election: ProposalVoteElection::DidNotVote,
        });
        test_pack_unpack(PaladinGovernanceInstruction::Vote {
            election: ProposalVoteElection::For,
        });
        test_pack_unpack(PaladinGovernanceInstruction::Vote {
            election: ProposalVoteElection::Against,
        });
    }

    #[test]
    fn test_pack_unpack_switch_vote() {
        test_pack_unpack(PaladinGovernanceInstruction::SwitchVote {
            new_election: ProposalVoteElection::DidNotVote,
        });
        test_pack_unpack(PaladinGovernanceInstruction::SwitchVote {
            new_election: ProposalVoteElection::For,
        });
        test_pack_unpack(PaladinGovernanceInstruction::SwitchVote {
            new_election: ProposalVoteElection::Against,
        });
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
            voting_period_seconds: 4,
        });
    }

    #[test]
    fn test_pack_unpack_update_governance() {
        test_pack_unpack(PaladinGovernanceInstruction::UpdateGovernance {
            cooldown_period_seconds: 1,
            proposal_acceptance_threshold: 2,
            proposal_rejection_threshold: 3,
            voting_period_seconds: 4,
        });
    }
}

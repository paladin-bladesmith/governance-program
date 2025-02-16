//! Program instruction types.

use {
    crate::state::{ProposalAccountMeta, ProposalVoteElection},
    arrayref::{array_ref, array_refs},
    borsh::{BorshDeserialize, BorshSerialize},
    shank::ShankInstruction,
    solana_program::{
        instruction::{AccountMeta, Instruction},
        program_error::ProgramError,
        pubkey::Pubkey,
        system_program,
    },
};

/// Instructions supported by the Paladin Governance program.
#[rustfmt::skip]
#[derive(Clone, Debug, PartialEq, ShankInstruction)]
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
    /// 1. `[w]` Author account.
    /// 2. `[ ]` Paladin stake account.
    /// 3. `[w]` Proposal account.
    /// 4. `[w]` Proposal transaction account.
    /// 5. `[ ]` Governance config account.
    /// 6. `[ ]` System program.
    #[account(
        0,
        signer,
        name = "stake_authority",
        description = "Paladin stake authority account"
    )]
    #[account(
        1,
        writable,
        name = "author",
        description = "Stake authority author account"
    )]
    #[account(
        2,
        name = "stake",
        description = "Paladin stake account"
    )]
    #[account(
        3,
        writable,
        name = "proposal",
        description = "Proposal account"
    )]
    #[account(
        4,
        writable,
        name = "proposal_transaction",
        description = "Proposal transaction account"
    )]
    #[account(
        5,
        name = "governance_config",
        description = "Governance config account"
    )]
    #[account(
        6,
        name = "system_program",
        description = "System program"
    )]
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
    #[account(
        0,
        signer,
        name = "stake_authority",
        description = "Paladin stake authority account"
    )]
    #[account(
        1,
        name = "proposal",
        description = "Proposal account"
    )]
    #[account(
        2,
        writable,
        name = "proposal_transaction",
        description = "Proposal transaction account"
    )]
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
    #[account(
        0,
        signer,
        name = "stake_authority",
        description = "Paladin stake authority account"
    )]
    #[account(
        1,
        name = "proposal",
        description = "Proposal account"
    )]
    #[account(
        2,
        writable,
        name = "proposal_transaction",
        description = "Proposal transaction account"
    )]
    RemoveInstruction {
        /// The index of the instruction to remove.
        instruction_index: u32,
    },
    /// Delete a governance proposal.
    ///
    /// Authority account provided must be the proposal creator.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[s,w]` Paladin stake authority account.
    /// 1. `[w]` Author account.
    /// 2. `[w]` Proposal account.
    #[account(
        0,
        signer,
        writable,
        name = "stake_authority",
        description = "Paladin stake authority account"
    )]
    #[account(
        1,
        writable,
        name = "author",
        description = "Stake authority author account"
    )]
    #[account(
        2,
        writable,
        name = "proposal",
        description = "Proposal account"
    )]
    DeleteProposal,
    /// Finalize a draft governance proposal and begin voting.
    ///
    /// Authority account provided must be the proposal creator.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[s]` Paladin stake authority account.
    /// 1. `[w]` Proposal account.
    #[account(
        0,
        signer,
        name = "stake_authority",
        description = "Paladin stake authority account"
    )]
    #[account(
        1,
        writable,
        name = "proposal",
        description = "Proposal account"
    )]
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
    /// 5. `[ ]` System program.
    #[account(
        0,
        signer,
        name = "stake_authority",
        description = "Paladin stake authority account"
    )]
    #[account(
        1,
        name = "stake",
        description = "Paladin stake account"
    )]
    #[account(
        2,
        name = "stake_config",
        description = "Paladin stake config account"
    )]
    #[account(
        3,
        writable,
        name = "vote",
        description = "Proposal vote account"
    )]
    #[account(
        4,
        writable,
        name = "proposal",
        description = "Proposal account"
    )]
    #[account(
        5,
        name = "system_program",
        description = "System program"
    )]
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
    #[account(
        0,
        signer,
        name = "stake_authority",
        description = "Paladin stake authority account"
    )]
    #[account(
        1,
        name = "stake",
        description = "Paladin stake account"
    )]
    #[account(
        2,
        name = "stake_config",
        description = "Paladin stake config account"
    )]
    #[account(
        3,
        writable,
        name = "vote",
        description = "Proposal vote account"
    )]
    #[account(
        4,
        writable,
        name = "proposal",
        description = "Proposal account"
    )]
    SwitchVote {
        /// New proposal vote election.
        new_election: ProposalVoteElection,
    },
    /// Finish voting on a proposal. Marks a proposal as `Accepted` or
    /// `Rejected`.
    ///
    /// Permissionless instruction. Only succeeds under two conditions.
    ///
    /// * If a proposal has reached the acceptance threshold _and_ the cooldown
    ///   period has ended, marks the proposal as `Accepted`.
    /// * If a proposal's voting period has ended, and no cooldown period is
    ///   active, marks the proposal as `Rejected`.
    ///
    /// This way, accepted or expired proposals can be finalized without the
    /// need for an additional vote or vote switch to be cast.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[w]` Proposal account.
    /// 1. `[ ]` Paladin stake config account.
    #[account(
        0,
        writable,
        name = "proposal",
        description = "Proposal account"
    )]
    FinishVoting,
    /// Delete's a vote account and recovers its' rent once the proposal has
    /// been finalized.
    #[account(0, name = "proposal")]
    #[account(1, writable, name = "vote")]
    #[account(2, writable, name = "authority")]
    DeleteVote,
    #[allow(clippy::doc_lazy_continuation)]
    /// Process an instruction in an accepted governance proposal.
    ///
    /// Given an accepted proposal and one of its instructions, executes it.
    /// If the proposal has been accepted, executes the instruction via CPI
    /// and applies the governance treasury PDA signature, then marks the
    /// instruction as executed.
    ///
    /// Note: Returns an error if the previous instruction in this proposal has
    /// not been executed.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[ ]` Proposal account.
    /// 1. `[w]` Proposal transaction account.
    /// 2..N.    Instruction accounts.
    #[account(
        0,
        writable,
        name = "proposal",
        description = "Proposal account"
    )]
    #[account(
        1,
        writable,
        name = "proposal_transaction",
        description = "Proposal transaction account"
    )]
    ProcessInstruction {
        /// The index of the instruction to execute.
        instruction_index: u32,
    },
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
    #[account(
        0,
        writable,
        name = "governance_config",
        description = "Governance config account"
    )]
    #[account(
        1,
        name = "stake_config",
        description = "Paladin stake config account"
    )]
    #[account(
        2,
        name = "system_program",
        description = "System program"
    )]
    InitializeGovernance {
        governance_id: u64,
        cooldown_period_seconds: u64,
        proposal_minimum_quorum: u32,
        proposal_pass_threshold: u32,
        voting_period_seconds: u64,
        stake_per_proposal: u64,
    },
    /// Update the governance config.
    ///
    /// Allows modification of the governance config, including:
    ///
    /// * The cooldown period for proposal execution.
    /// * Minimum required majority threshold.
    ///
    /// This instruction can only be executed from an accepted proposal, thus
    /// it requires the PDA signature of the treasury.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[s]` Treasury account.
    /// 1. `[w]` Governance config account.
    #[account(
        0,
        signer,
        name = "treasury",
        description = "Treasury account"
    )]
    #[account(
        1,
        name = "governance_config",
        description = "Governance config account"
    )]
    UpdateGovernance {
        governance_id: u64,
        cooldown_period_seconds: u64,
        proposal_minimum_quorum: u32,
        proposal_pass_threshold: u32,
        voting_period_seconds: u64,
        stake_per_proposal: u64,
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
            Self::DeleteProposal => vec![3],
            Self::BeginVoting => vec![4],
            Self::Vote { election } => vec![5, (*election).into()],
            Self::SwitchVote { new_election } => vec![6, (*new_election).into()],
            Self::FinishVoting => vec![7],
            Self::DeleteVote => vec![8],
            Self::ProcessInstruction { instruction_index } => {
                let mut buf = vec![9];
                buf.extend_from_slice(&instruction_index.to_le_bytes());
                buf
            }
            Self::InitializeGovernance {
                governance_id,
                cooldown_period_seconds,
                proposal_minimum_quorum,
                proposal_pass_threshold,
                voting_period_seconds,
                stake_per_proposal,
            } => {
                let mut buf = vec![10];
                buf.extend_from_slice(&governance_id.to_le_bytes());
                buf.extend_from_slice(&cooldown_period_seconds.to_le_bytes());
                buf.extend_from_slice(&proposal_minimum_quorum.to_le_bytes());
                buf.extend_from_slice(&proposal_pass_threshold.to_le_bytes());
                buf.extend_from_slice(&voting_period_seconds.to_le_bytes());
                buf.extend_from_slice(&stake_per_proposal.to_le_bytes());
                buf
            }
            Self::UpdateGovernance {
                governance_id,
                cooldown_period_seconds,
                proposal_minimum_quorum,
                proposal_pass_threshold,
                voting_period_seconds,
                stake_per_proposal,
            } => {
                let mut buf = vec![11];
                buf.extend_from_slice(&governance_id.to_le_bytes());
                buf.extend_from_slice(&cooldown_period_seconds.to_le_bytes());
                buf.extend_from_slice(&proposal_minimum_quorum.to_le_bytes());
                buf.extend_from_slice(&proposal_pass_threshold.to_le_bytes());
                buf.extend_from_slice(&voting_period_seconds.to_le_bytes());
                buf.extend_from_slice(&stake_per_proposal.to_le_bytes());
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
            Some((&3, _)) => Ok(Self::DeleteProposal),
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
            Some((&7, _)) => Ok(Self::FinishVoting),
            Some((&8, _)) => Ok(Self::DeleteVote),
            Some((&9, rest)) if rest.len() == 4 => {
                let instruction_index = u32::from_le_bytes(rest.try_into().unwrap());
                Ok(Self::ProcessInstruction { instruction_index })
            }
            Some((&10, rest)) if rest.len() == 40 => {
                let rest = array_ref![rest, 0, 40];
                let (
                    governance_id,
                    cooldown_period_seconds,
                    proposal_minimum_quorum,
                    proposal_pass_threshold,
                    voting_period_seconds,
                    stake_per_proposal,
                ) = array_refs![rest, 8, 8, 4, 4, 8, 8];

                let governance_id = u64::from_le_bytes(*governance_id);
                let cooldown_period_seconds = u64::from_le_bytes(*cooldown_period_seconds);
                let proposal_minimum_quorum = u32::from_le_bytes(*proposal_minimum_quorum);
                let proposal_pass_threshold = u32::from_le_bytes(*proposal_pass_threshold);
                let voting_period_seconds = u64::from_le_bytes(*voting_period_seconds);
                let stake_per_proposal = u64::from_le_bytes(*stake_per_proposal);

                Ok(Self::InitializeGovernance {
                    governance_id,
                    cooldown_period_seconds,
                    proposal_minimum_quorum,
                    proposal_pass_threshold,
                    voting_period_seconds,
                    stake_per_proposal,
                })
            }
            Some((&11, rest)) if rest.len() == 40 => {
                let rest = array_ref![rest, 0, 40];
                let (
                    governance_id,
                    cooldown_period_seconds,
                    proposal_minimum_quorum,
                    proposal_pass_threshold,
                    voting_period_seconds,
                    stake_per_proposal,
                ) = array_refs![rest, 8, 8, 4, 4, 8, 8];

                let governance_id = u64::from_le_bytes(*governance_id);
                let cooldown_period_seconds = u64::from_le_bytes(*cooldown_period_seconds);
                let proposal_minimum_quorum = u32::from_le_bytes(*proposal_minimum_quorum);
                let proposal_pass_threshold = u32::from_le_bytes(*proposal_pass_threshold);
                let voting_period_seconds = u64::from_le_bytes(*voting_period_seconds);
                let stake_per_proposal = u64::from_le_bytes(*stake_per_proposal);

                Ok(Self::UpdateGovernance {
                    governance_id,
                    cooldown_period_seconds,
                    proposal_minimum_quorum,
                    proposal_pass_threshold,
                    voting_period_seconds,
                    stake_per_proposal,
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
        AccountMeta::new(
            crate::state::get_proposal_author_address(stake_authority_address, &crate::id()),
            false,
        ),
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
/// [DeleteProposal](enum.PaladinGovernanceInstruction.html)
/// instruction.
pub fn delete_proposal(stake_authority_address: Pubkey, proposal_address: Pubkey) -> Instruction {
    let accounts = vec![
        AccountMeta::new(stake_authority_address, true),
        AccountMeta::new(
            crate::state::get_proposal_author_address(&stake_authority_address, &crate::id()),
            false,
        ),
        AccountMeta::new(proposal_address, false),
    ];
    let data = PaladinGovernanceInstruction::DeleteProposal.pack();
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
/// [SwitchVote](enum.PaladinGovernanceInstruction.html)
/// instruction.
pub fn delete_vote(proposal: Pubkey, vote: Pubkey, authority: Pubkey) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(proposal, false),
        AccountMeta::new(vote, false),
        AccountMeta::new(authority, false),
    ];
    let data = PaladinGovernanceInstruction::DeleteVote.pack();

    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

/// Creates a
/// [FinishVoting](enum.PaladinGovernanceInstruction.html)
/// instruction.
pub fn finish_voting(proposal_address: &Pubkey) -> Instruction {
    let accounts = vec![AccountMeta::new(*proposal_address, false)];
    let data = PaladinGovernanceInstruction::FinishVoting.pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

/// Creates a
/// [ProcessInstruction](enum.PaladinGovernanceInstruction.html)
/// instruction.
pub fn process_instruction(
    proposal_address: &Pubkey,
    proposal_transaction_address: &Pubkey,
    account_metas: &[AccountMeta],
    instruction_index: u32,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(*proposal_address, false),
        AccountMeta::new(*proposal_transaction_address, false),
    ];
    accounts.extend_from_slice(account_metas);
    let data = PaladinGovernanceInstruction::ProcessInstruction { instruction_index }.pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

/// Creates a
/// [InitializeGovernance](enum.PaladinGovernanceInstruction.html)
/// instruction.
#[allow(clippy::too_many_arguments)]
pub fn initialize_governance(
    governance_config_address: &Pubkey,
    stake_config_address: &Pubkey,
    governance_id: u64,
    cooldown_period_seconds: u64,
    proposal_minimum_quorum: u32,
    proposal_pass_threshold: u32,
    voting_period_seconds: u64,
    stake_per_proposal: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*governance_config_address, false),
        AccountMeta::new_readonly(*stake_config_address, false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    let data = PaladinGovernanceInstruction::InitializeGovernance {
        governance_id,
        cooldown_period_seconds,
        proposal_minimum_quorum,
        proposal_pass_threshold,
        voting_period_seconds,
        stake_per_proposal,
    }
    .pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

/// Creates a
/// [UpdateGovernance](enum.PaladinGovernanceInstruction.html)
/// instruction.
#[allow(clippy::too_many_arguments)]
pub fn update_governance(
    treasury_address: &Pubkey,
    governance_config_address: &Pubkey,
    governance_id: u64,
    cooldown_period_seconds: u64,
    proposal_minimum_quorum: u32,
    proposal_pass_threshold: u32,
    voting_period_seconds: u64,
    stake_per_proposal: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*treasury_address, true),
        AccountMeta::new(*governance_config_address, false),
    ];
    let data = PaladinGovernanceInstruction::UpdateGovernance {
        governance_id,
        cooldown_period_seconds,
        proposal_minimum_quorum,
        proposal_pass_threshold,
        voting_period_seconds,
        stake_per_proposal,
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
        test_pack_unpack(PaladinGovernanceInstruction::DeleteProposal);
    }

    #[test]
    fn test_pack_unpack_begin_voting() {
        test_pack_unpack(PaladinGovernanceInstruction::BeginVoting);
    }

    #[test]
    fn test_pack_unpack_vote() {
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
            new_election: ProposalVoteElection::For,
        });
        test_pack_unpack(PaladinGovernanceInstruction::SwitchVote {
            new_election: ProposalVoteElection::Against,
        });
    }

    #[test]
    fn test_pack_unpack_finish_voting() {
        test_pack_unpack(PaladinGovernanceInstruction::FinishVoting);
    }

    #[test]
    fn test_pack_unpack_process_instruction() {
        test_pack_unpack(PaladinGovernanceInstruction::ProcessInstruction {
            instruction_index: 45,
        });
    }

    #[test]
    fn test_pack_unpack_initialize_governance() {
        test_pack_unpack(PaladinGovernanceInstruction::InitializeGovernance {
            governance_id: 1,
            cooldown_period_seconds: 2,
            proposal_minimum_quorum: 3,
            proposal_pass_threshold: 4,
            voting_period_seconds: 5,
            stake_per_proposal: 6,
        });
    }

    #[test]
    fn test_pack_unpack_update_governance() {
        test_pack_unpack(PaladinGovernanceInstruction::UpdateGovernance {
            governance_id: 1,
            cooldown_period_seconds: 2,
            proposal_minimum_quorum: 3,
            proposal_pass_threshold: 4,
            voting_period_seconds: 5,
            stake_per_proposal: 6,
        });
    }
}

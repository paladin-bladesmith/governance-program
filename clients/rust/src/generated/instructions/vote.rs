//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>

use {
    crate::generated::types::ProposalVoteElection,
    borsh::{BorshDeserialize, BorshSerialize},
};

/// Accounts.
pub struct Vote {
    /// Paladin stake authority account
    pub stake_authority: solana_program::pubkey::Pubkey,
    /// Paladin stake account
    pub stake: solana_program::pubkey::Pubkey,
    /// Paladin stake config account
    pub stake_config: solana_program::pubkey::Pubkey,
    /// Proposal vote account
    pub vote: solana_program::pubkey::Pubkey,
    /// Proposal account
    pub proposal: solana_program::pubkey::Pubkey,
    /// System program
    pub system_program: solana_program::pubkey::Pubkey,
}

impl Vote {
    pub fn instruction(
        &self,
        args: VoteInstructionArgs,
    ) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(args, &[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        args: VoteInstructionArgs,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(6 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.stake_authority,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.stake, false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.stake_config,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.vote, false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.proposal,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.system_program,
            false,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let mut data = VoteInstructionData::new().try_to_vec().unwrap();
        let mut args = args.try_to_vec().unwrap();
        data.append(&mut args);

        solana_program::instruction::Instruction {
            program_id: crate::PALADIN_GOVERNANCE_ID,
            accounts,
            data,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct VoteInstructionData {
    discriminator: u8,
}

impl VoteInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 6 }
    }
}

impl Default for VoteInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VoteInstructionArgs {
    pub election: ProposalVoteElection,
}

/// Instruction builder for `Vote`.
///
/// ### Accounts:
///
///   0. `[signer]` stake_authority
///   1. `[]` stake
///   2. `[]` stake_config
///   3. `[writable]` vote
///   4. `[writable]` proposal
///   5. `[optional]` system_program (default to
///      `11111111111111111111111111111111`)
#[derive(Clone, Debug, Default)]
pub struct VoteBuilder {
    stake_authority: Option<solana_program::pubkey::Pubkey>,
    stake: Option<solana_program::pubkey::Pubkey>,
    stake_config: Option<solana_program::pubkey::Pubkey>,
    vote: Option<solana_program::pubkey::Pubkey>,
    proposal: Option<solana_program::pubkey::Pubkey>,
    system_program: Option<solana_program::pubkey::Pubkey>,
    election: Option<ProposalVoteElection>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl VoteBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// Paladin stake authority account
    #[inline(always)]
    pub fn stake_authority(
        &mut self,
        stake_authority: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.stake_authority = Some(stake_authority);
        self
    }
    /// Paladin stake account
    #[inline(always)]
    pub fn stake(&mut self, stake: solana_program::pubkey::Pubkey) -> &mut Self {
        self.stake = Some(stake);
        self
    }
    /// Paladin stake config account
    #[inline(always)]
    pub fn stake_config(&mut self, stake_config: solana_program::pubkey::Pubkey) -> &mut Self {
        self.stake_config = Some(stake_config);
        self
    }
    /// Proposal vote account
    #[inline(always)]
    pub fn vote(&mut self, vote: solana_program::pubkey::Pubkey) -> &mut Self {
        self.vote = Some(vote);
        self
    }
    /// Proposal account
    #[inline(always)]
    pub fn proposal(&mut self, proposal: solana_program::pubkey::Pubkey) -> &mut Self {
        self.proposal = Some(proposal);
        self
    }
    /// `[optional account, default to '11111111111111111111111111111111']`
    /// System program
    #[inline(always)]
    pub fn system_program(&mut self, system_program: solana_program::pubkey::Pubkey) -> &mut Self {
        self.system_program = Some(system_program);
        self
    }
    #[inline(always)]
    pub fn election(&mut self, election: ProposalVoteElection) -> &mut Self {
        self.election = Some(election);
        self
    }
    /// Add an aditional account to the instruction.
    #[inline(always)]
    pub fn add_remaining_account(
        &mut self,
        account: solana_program::instruction::AccountMeta,
    ) -> &mut Self {
        self.__remaining_accounts.push(account);
        self
    }
    /// Add additional accounts to the instruction.
    #[inline(always)]
    pub fn add_remaining_accounts(
        &mut self,
        accounts: &[solana_program::instruction::AccountMeta],
    ) -> &mut Self {
        self.__remaining_accounts.extend_from_slice(accounts);
        self
    }
    #[allow(clippy::clone_on_copy)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        let accounts = Vote {
            stake_authority: self.stake_authority.expect("stake_authority is not set"),
            stake: self.stake.expect("stake is not set"),
            stake_config: self.stake_config.expect("stake_config is not set"),
            vote: self.vote.expect("vote is not set"),
            proposal: self.proposal.expect("proposal is not set"),
            system_program: self
                .system_program
                .unwrap_or(solana_program::pubkey!("11111111111111111111111111111111")),
        };
        let args = VoteInstructionArgs {
            election: self.election.clone().expect("election is not set"),
        };

        accounts.instruction_with_remaining_accounts(args, &self.__remaining_accounts)
    }
}

/// `vote` CPI accounts.
pub struct VoteCpiAccounts<'a, 'b> {
    /// Paladin stake authority account
    pub stake_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// Paladin stake account
    pub stake: &'b solana_program::account_info::AccountInfo<'a>,
    /// Paladin stake config account
    pub stake_config: &'b solana_program::account_info::AccountInfo<'a>,
    /// Proposal vote account
    pub vote: &'b solana_program::account_info::AccountInfo<'a>,
    /// Proposal account
    pub proposal: &'b solana_program::account_info::AccountInfo<'a>,
    /// System program
    pub system_program: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `vote` CPI instruction.
pub struct VoteCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,
    /// Paladin stake authority account
    pub stake_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// Paladin stake account
    pub stake: &'b solana_program::account_info::AccountInfo<'a>,
    /// Paladin stake config account
    pub stake_config: &'b solana_program::account_info::AccountInfo<'a>,
    /// Proposal vote account
    pub vote: &'b solana_program::account_info::AccountInfo<'a>,
    /// Proposal account
    pub proposal: &'b solana_program::account_info::AccountInfo<'a>,
    /// System program
    pub system_program: &'b solana_program::account_info::AccountInfo<'a>,
    /// The arguments for the instruction.
    pub __args: VoteInstructionArgs,
}

impl<'a, 'b> VoteCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: VoteCpiAccounts<'a, 'b>,
        args: VoteInstructionArgs,
    ) -> Self {
        Self {
            __program: program,
            stake_authority: accounts.stake_authority,
            stake: accounts.stake,
            stake_config: accounts.stake_config,
            vote: accounts.vote,
            proposal: accounts.proposal,
            system_program: accounts.system_program,
            __args: args,
        }
    }
    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(&[], &[])
    }
    #[inline(always)]
    pub fn invoke_with_remaining_accounts(
        &self,
        remaining_accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(&[], remaining_accounts)
    }
    #[inline(always)]
    pub fn invoke_signed(
        &self,
        signers_seeds: &[&[&[u8]]],
    ) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(signers_seeds, &[])
    }
    #[allow(clippy::clone_on_copy)]
    #[allow(clippy::vec_init_then_push)]
    pub fn invoke_signed_with_remaining_accounts(
        &self,
        signers_seeds: &[&[&[u8]]],
        remaining_accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> solana_program::entrypoint::ProgramResult {
        let mut accounts = Vec::with_capacity(6 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.stake_authority.key,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.stake.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.stake_config.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.vote.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.proposal.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.system_program.key,
            false,
        ));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let mut data = VoteInstructionData::new().try_to_vec().unwrap();
        let mut args = self.__args.try_to_vec().unwrap();
        data.append(&mut args);

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::PALADIN_GOVERNANCE_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(6 + 1 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.stake_authority.clone());
        account_infos.push(self.stake.clone());
        account_infos.push(self.stake_config.clone());
        account_infos.push(self.vote.clone());
        account_infos.push(self.proposal.clone());
        account_infos.push(self.system_program.clone());
        remaining_accounts
            .iter()
            .for_each(|remaining_account| account_infos.push(remaining_account.0.clone()));

        if signers_seeds.is_empty() {
            solana_program::program::invoke(&instruction, &account_infos)
        } else {
            solana_program::program::invoke_signed(&instruction, &account_infos, signers_seeds)
        }
    }
}

/// Instruction builder for `Vote` via CPI.
///
/// ### Accounts:
///
///   0. `[signer]` stake_authority
///   1. `[]` stake
///   2. `[]` stake_config
///   3. `[writable]` vote
///   4. `[writable]` proposal
///   5. `[]` system_program
#[derive(Clone, Debug)]
pub struct VoteCpiBuilder<'a, 'b> {
    instruction: Box<VoteCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> VoteCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(VoteCpiBuilderInstruction {
            __program: program,
            stake_authority: None,
            stake: None,
            stake_config: None,
            vote: None,
            proposal: None,
            system_program: None,
            election: None,
            __remaining_accounts: Vec::new(),
        });
        Self { instruction }
    }
    /// Paladin stake authority account
    #[inline(always)]
    pub fn stake_authority(
        &mut self,
        stake_authority: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.stake_authority = Some(stake_authority);
        self
    }
    /// Paladin stake account
    #[inline(always)]
    pub fn stake(&mut self, stake: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.stake = Some(stake);
        self
    }
    /// Paladin stake config account
    #[inline(always)]
    pub fn stake_config(
        &mut self,
        stake_config: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.stake_config = Some(stake_config);
        self
    }
    /// Proposal vote account
    #[inline(always)]
    pub fn vote(&mut self, vote: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.vote = Some(vote);
        self
    }
    /// Proposal account
    #[inline(always)]
    pub fn proposal(
        &mut self,
        proposal: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.proposal = Some(proposal);
        self
    }
    /// System program
    #[inline(always)]
    pub fn system_program(
        &mut self,
        system_program: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.system_program = Some(system_program);
        self
    }
    #[inline(always)]
    pub fn election(&mut self, election: ProposalVoteElection) -> &mut Self {
        self.instruction.election = Some(election);
        self
    }
    /// Add an additional account to the instruction.
    #[inline(always)]
    pub fn add_remaining_account(
        &mut self,
        account: &'b solana_program::account_info::AccountInfo<'a>,
        is_writable: bool,
        is_signer: bool,
    ) -> &mut Self {
        self.instruction
            .__remaining_accounts
            .push((account, is_writable, is_signer));
        self
    }
    /// Add additional accounts to the instruction.
    ///
    /// Each account is represented by a tuple of the `AccountInfo`, a `bool`
    /// indicating whether the account is writable or not, and a `bool`
    /// indicating whether the account is a signer or not.
    #[inline(always)]
    pub fn add_remaining_accounts(
        &mut self,
        accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> &mut Self {
        self.instruction
            .__remaining_accounts
            .extend_from_slice(accounts);
        self
    }
    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed(&[])
    }
    #[allow(clippy::clone_on_copy)]
    #[allow(clippy::vec_init_then_push)]
    pub fn invoke_signed(
        &self,
        signers_seeds: &[&[&[u8]]],
    ) -> solana_program::entrypoint::ProgramResult {
        let args = VoteInstructionArgs {
            election: self
                .instruction
                .election
                .clone()
                .expect("election is not set"),
        };
        let instruction = VoteCpi {
            __program: self.instruction.__program,

            stake_authority: self
                .instruction
                .stake_authority
                .expect("stake_authority is not set"),

            stake: self.instruction.stake.expect("stake is not set"),

            stake_config: self
                .instruction
                .stake_config
                .expect("stake_config is not set"),

            vote: self.instruction.vote.expect("vote is not set"),

            proposal: self.instruction.proposal.expect("proposal is not set"),

            system_program: self
                .instruction
                .system_program
                .expect("system_program is not set"),
            __args: args,
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct VoteCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    stake_authority: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    stake: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    stake_config: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    vote: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    proposal: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    system_program: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    election: Option<ProposalVoteElection>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}

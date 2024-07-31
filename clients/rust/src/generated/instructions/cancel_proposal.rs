//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>

use borsh::{BorshDeserialize, BorshSerialize};

/// Accounts.
pub struct CancelProposal {
    /// Paladin stake authority account
    pub stake_authority: solana_program::pubkey::Pubkey,
    /// Proposal account
    pub proposal: solana_program::pubkey::Pubkey,
}

impl CancelProposal {
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(&[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(2 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.stake_authority,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.proposal,
            false,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let data = CancelProposalInstructionData::new().try_to_vec().unwrap();

        solana_program::instruction::Instruction {
            program_id: crate::PALADIN_GOVERNANCE_PROGRAM_ID,
            accounts,
            data,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct CancelProposalInstructionData {
    discriminator: u8,
}

impl CancelProposalInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 3 }
    }
}

impl Default for CancelProposalInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

/// Instruction builder for `CancelProposal`.
///
/// ### Accounts:
///
///   0. `[signer]` stake_authority
///   1. `[writable]` proposal
#[derive(Clone, Debug, Default)]
pub struct CancelProposalBuilder {
    stake_authority: Option<solana_program::pubkey::Pubkey>,
    proposal: Option<solana_program::pubkey::Pubkey>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl CancelProposalBuilder {
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
    /// Proposal account
    #[inline(always)]
    pub fn proposal(&mut self, proposal: solana_program::pubkey::Pubkey) -> &mut Self {
        self.proposal = Some(proposal);
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
        let accounts = CancelProposal {
            stake_authority: self.stake_authority.expect("stake_authority is not set"),
            proposal: self.proposal.expect("proposal is not set"),
        };

        accounts.instruction_with_remaining_accounts(&self.__remaining_accounts)
    }
}

/// `cancel_proposal` CPI accounts.
pub struct CancelProposalCpiAccounts<'a, 'b> {
    /// Paladin stake authority account
    pub stake_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// Proposal account
    pub proposal: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `cancel_proposal` CPI instruction.
pub struct CancelProposalCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,
    /// Paladin stake authority account
    pub stake_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// Proposal account
    pub proposal: &'b solana_program::account_info::AccountInfo<'a>,
}

impl<'a, 'b> CancelProposalCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: CancelProposalCpiAccounts<'a, 'b>,
    ) -> Self {
        Self {
            __program: program,
            stake_authority: accounts.stake_authority,
            proposal: accounts.proposal,
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
        let mut accounts = Vec::with_capacity(2 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.stake_authority.key,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.proposal.key,
            false,
        ));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let data = CancelProposalInstructionData::new().try_to_vec().unwrap();

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::PALADIN_GOVERNANCE_PROGRAM_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(2 + 1 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.stake_authority.clone());
        account_infos.push(self.proposal.clone());
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

/// Instruction builder for `CancelProposal` via CPI.
///
/// ### Accounts:
///
///   0. `[signer]` stake_authority
///   1. `[writable]` proposal
#[derive(Clone, Debug)]
pub struct CancelProposalCpiBuilder<'a, 'b> {
    instruction: Box<CancelProposalCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> CancelProposalCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(CancelProposalCpiBuilderInstruction {
            __program: program,
            stake_authority: None,
            proposal: None,
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
    /// Proposal account
    #[inline(always)]
    pub fn proposal(
        &mut self,
        proposal: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.proposal = Some(proposal);
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
        let instruction = CancelProposalCpi {
            __program: self.instruction.__program,

            stake_authority: self
                .instruction
                .stake_authority
                .expect("stake_authority is not set"),

            proposal: self.instruction.proposal.expect("proposal is not set"),
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct CancelProposalCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    stake_authority: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    proposal: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}

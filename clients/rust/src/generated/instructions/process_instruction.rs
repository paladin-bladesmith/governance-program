//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>

use borsh::{BorshDeserialize, BorshSerialize};

/// Accounts.
pub struct ProcessInstruction {
    /// Proposal account
    pub proposal: solana_program::pubkey::Pubkey,
    /// Proposal transaction account
    pub proposal_transaction: solana_program::pubkey::Pubkey,
}

impl ProcessInstruction {
    pub fn instruction(
        &self,
        args: ProcessInstructionInstructionArgs,
    ) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(args, &[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        args: ProcessInstructionInstructionArgs,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(2 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.proposal,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.proposal_transaction,
            false,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let mut data = ProcessInstructionInstructionData::new()
            .try_to_vec()
            .unwrap();
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
pub struct ProcessInstructionInstructionData {
    discriminator: u8,
}

impl ProcessInstructionInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 9 }
    }
}

impl Default for ProcessInstructionInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProcessInstructionInstructionArgs {
    pub instruction_index: u32,
}

/// Instruction builder for `ProcessInstruction`.
///
/// ### Accounts:
///
///   0. `[writable]` proposal
///   1. `[writable]` proposal_transaction
#[derive(Clone, Debug, Default)]
pub struct ProcessInstructionBuilder {
    proposal: Option<solana_program::pubkey::Pubkey>,
    proposal_transaction: Option<solana_program::pubkey::Pubkey>,
    instruction_index: Option<u32>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl ProcessInstructionBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// Proposal account
    #[inline(always)]
    pub fn proposal(&mut self, proposal: solana_program::pubkey::Pubkey) -> &mut Self {
        self.proposal = Some(proposal);
        self
    }
    /// Proposal transaction account
    #[inline(always)]
    pub fn proposal_transaction(
        &mut self,
        proposal_transaction: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.proposal_transaction = Some(proposal_transaction);
        self
    }
    #[inline(always)]
    pub fn instruction_index(&mut self, instruction_index: u32) -> &mut Self {
        self.instruction_index = Some(instruction_index);
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
        let accounts = ProcessInstruction {
            proposal: self.proposal.expect("proposal is not set"),
            proposal_transaction: self
                .proposal_transaction
                .expect("proposal_transaction is not set"),
        };
        let args = ProcessInstructionInstructionArgs {
            instruction_index: self
                .instruction_index
                .clone()
                .expect("instruction_index is not set"),
        };

        accounts.instruction_with_remaining_accounts(args, &self.__remaining_accounts)
    }
}

/// `process_instruction` CPI accounts.
pub struct ProcessInstructionCpiAccounts<'a, 'b> {
    /// Proposal account
    pub proposal: &'b solana_program::account_info::AccountInfo<'a>,
    /// Proposal transaction account
    pub proposal_transaction: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `process_instruction` CPI instruction.
pub struct ProcessInstructionCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,
    /// Proposal account
    pub proposal: &'b solana_program::account_info::AccountInfo<'a>,
    /// Proposal transaction account
    pub proposal_transaction: &'b solana_program::account_info::AccountInfo<'a>,
    /// The arguments for the instruction.
    pub __args: ProcessInstructionInstructionArgs,
}

impl<'a, 'b> ProcessInstructionCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: ProcessInstructionCpiAccounts<'a, 'b>,
        args: ProcessInstructionInstructionArgs,
    ) -> Self {
        Self {
            __program: program,
            proposal: accounts.proposal,
            proposal_transaction: accounts.proposal_transaction,
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
        let mut accounts = Vec::with_capacity(2 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.proposal.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.proposal_transaction.key,
            false,
        ));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let mut data = ProcessInstructionInstructionData::new()
            .try_to_vec()
            .unwrap();
        let mut args = self.__args.try_to_vec().unwrap();
        data.append(&mut args);

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::PALADIN_GOVERNANCE_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(2 + 1 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.proposal.clone());
        account_infos.push(self.proposal_transaction.clone());
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

/// Instruction builder for `ProcessInstruction` via CPI.
///
/// ### Accounts:
///
///   0. `[writable]` proposal
///   1. `[writable]` proposal_transaction
#[derive(Clone, Debug)]
pub struct ProcessInstructionCpiBuilder<'a, 'b> {
    instruction: Box<ProcessInstructionCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> ProcessInstructionCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(ProcessInstructionCpiBuilderInstruction {
            __program: program,
            proposal: None,
            proposal_transaction: None,
            instruction_index: None,
            __remaining_accounts: Vec::new(),
        });
        Self { instruction }
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
    /// Proposal transaction account
    #[inline(always)]
    pub fn proposal_transaction(
        &mut self,
        proposal_transaction: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.proposal_transaction = Some(proposal_transaction);
        self
    }
    #[inline(always)]
    pub fn instruction_index(&mut self, instruction_index: u32) -> &mut Self {
        self.instruction.instruction_index = Some(instruction_index);
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
        let args = ProcessInstructionInstructionArgs {
            instruction_index: self
                .instruction
                .instruction_index
                .clone()
                .expect("instruction_index is not set"),
        };
        let instruction = ProcessInstructionCpi {
            __program: self.instruction.__program,

            proposal: self.instruction.proposal.expect("proposal is not set"),

            proposal_transaction: self
                .instruction
                .proposal_transaction
                .expect("proposal_transaction is not set"),
            __args: args,
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct ProcessInstructionCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    proposal: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    proposal_transaction: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    instruction_index: Option<u32>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}

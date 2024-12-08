//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>

use borsh::{BorshDeserialize, BorshSerialize};

/// Accounts.
pub struct UpdateGovernance {
    /// Treasury account
    pub treasury: solana_program::pubkey::Pubkey,
    /// Governance config account
    pub governance_config: solana_program::pubkey::Pubkey,
}

impl UpdateGovernance {
    pub fn instruction(
        &self,
        args: UpdateGovernanceInstructionArgs,
    ) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(args, &[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        args: UpdateGovernanceInstructionArgs,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(2 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.treasury,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.governance_config,
            false,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let mut data = UpdateGovernanceInstructionData::new().try_to_vec().unwrap();
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
pub struct UpdateGovernanceInstructionData {
    discriminator: u8,
}

impl UpdateGovernanceInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 10 }
    }
}

impl Default for UpdateGovernanceInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UpdateGovernanceInstructionArgs {
    pub governance_id: u64,
    pub cooldown_period_seconds: u64,
    pub proposal_minimum_quorum: u32,
    pub proposal_pass_threshold: u32,
    pub voting_period_seconds: u64,
    pub stake_per_proposal: u64,
}

/// Instruction builder for `UpdateGovernance`.
///
/// ### Accounts:
///
///   0. `[signer]` treasury
///   1. `[]` governance_config
#[derive(Clone, Debug, Default)]
pub struct UpdateGovernanceBuilder {
    treasury: Option<solana_program::pubkey::Pubkey>,
    governance_config: Option<solana_program::pubkey::Pubkey>,
    governance_id: Option<u64>,
    cooldown_period_seconds: Option<u64>,
    proposal_minimum_quorum: Option<u32>,
    proposal_pass_threshold: Option<u32>,
    voting_period_seconds: Option<u64>,
    stake_per_proposal: Option<u64>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl UpdateGovernanceBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// Treasury account
    #[inline(always)]
    pub fn treasury(&mut self, treasury: solana_program::pubkey::Pubkey) -> &mut Self {
        self.treasury = Some(treasury);
        self
    }
    /// Governance config account
    #[inline(always)]
    pub fn governance_config(
        &mut self,
        governance_config: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.governance_config = Some(governance_config);
        self
    }
    #[inline(always)]
    pub fn governance_id(&mut self, governance_id: u64) -> &mut Self {
        self.governance_id = Some(governance_id);
        self
    }
    #[inline(always)]
    pub fn cooldown_period_seconds(&mut self, cooldown_period_seconds: u64) -> &mut Self {
        self.cooldown_period_seconds = Some(cooldown_period_seconds);
        self
    }
    #[inline(always)]
    pub fn proposal_minimum_quorum(&mut self, proposal_minimum_quorum: u32) -> &mut Self {
        self.proposal_minimum_quorum = Some(proposal_minimum_quorum);
        self
    }
    #[inline(always)]
    pub fn proposal_pass_threshold(&mut self, proposal_pass_threshold: u32) -> &mut Self {
        self.proposal_pass_threshold = Some(proposal_pass_threshold);
        self
    }
    #[inline(always)]
    pub fn voting_period_seconds(&mut self, voting_period_seconds: u64) -> &mut Self {
        self.voting_period_seconds = Some(voting_period_seconds);
        self
    }
    #[inline(always)]
    pub fn stake_per_proposal(&mut self, stake_per_proposal: u64) -> &mut Self {
        self.stake_per_proposal = Some(stake_per_proposal);
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
        let accounts = UpdateGovernance {
            treasury: self.treasury.expect("treasury is not set"),
            governance_config: self
                .governance_config
                .expect("governance_config is not set"),
        };
        let args = UpdateGovernanceInstructionArgs {
            governance_id: self
                .governance_id
                .clone()
                .expect("governance_id is not set"),
            cooldown_period_seconds: self
                .cooldown_period_seconds
                .clone()
                .expect("cooldown_period_seconds is not set"),
            proposal_minimum_quorum: self
                .proposal_minimum_quorum
                .clone()
                .expect("proposal_minimum_quorum is not set"),
            proposal_pass_threshold: self
                .proposal_pass_threshold
                .clone()
                .expect("proposal_pass_threshold is not set"),
            voting_period_seconds: self
                .voting_period_seconds
                .clone()
                .expect("voting_period_seconds is not set"),
            stake_per_proposal: self
                .stake_per_proposal
                .clone()
                .expect("stake_per_proposal is not set"),
        };

        accounts.instruction_with_remaining_accounts(args, &self.__remaining_accounts)
    }
}

/// `update_governance` CPI accounts.
pub struct UpdateGovernanceCpiAccounts<'a, 'b> {
    /// Treasury account
    pub treasury: &'b solana_program::account_info::AccountInfo<'a>,
    /// Governance config account
    pub governance_config: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `update_governance` CPI instruction.
pub struct UpdateGovernanceCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,
    /// Treasury account
    pub treasury: &'b solana_program::account_info::AccountInfo<'a>,
    /// Governance config account
    pub governance_config: &'b solana_program::account_info::AccountInfo<'a>,
    /// The arguments for the instruction.
    pub __args: UpdateGovernanceInstructionArgs,
}

impl<'a, 'b> UpdateGovernanceCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: UpdateGovernanceCpiAccounts<'a, 'b>,
        args: UpdateGovernanceInstructionArgs,
    ) -> Self {
        Self {
            __program: program,
            treasury: accounts.treasury,
            governance_config: accounts.governance_config,
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
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.treasury.key,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.governance_config.key,
            false,
        ));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let mut data = UpdateGovernanceInstructionData::new().try_to_vec().unwrap();
        let mut args = self.__args.try_to_vec().unwrap();
        data.append(&mut args);

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::PALADIN_GOVERNANCE_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(2 + 1 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.treasury.clone());
        account_infos.push(self.governance_config.clone());
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

/// Instruction builder for `UpdateGovernance` via CPI.
///
/// ### Accounts:
///
///   0. `[signer]` treasury
///   1. `[]` governance_config
#[derive(Clone, Debug)]
pub struct UpdateGovernanceCpiBuilder<'a, 'b> {
    instruction: Box<UpdateGovernanceCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> UpdateGovernanceCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(UpdateGovernanceCpiBuilderInstruction {
            __program: program,
            treasury: None,
            governance_config: None,
            governance_id: None,
            cooldown_period_seconds: None,
            proposal_minimum_quorum: None,
            proposal_pass_threshold: None,
            voting_period_seconds: None,
            stake_per_proposal: None,
            __remaining_accounts: Vec::new(),
        });
        Self { instruction }
    }
    /// Treasury account
    #[inline(always)]
    pub fn treasury(
        &mut self,
        treasury: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.treasury = Some(treasury);
        self
    }
    /// Governance config account
    #[inline(always)]
    pub fn governance_config(
        &mut self,
        governance_config: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.governance_config = Some(governance_config);
        self
    }
    #[inline(always)]
    pub fn governance_id(&mut self, governance_id: u64) -> &mut Self {
        self.instruction.governance_id = Some(governance_id);
        self
    }
    #[inline(always)]
    pub fn cooldown_period_seconds(&mut self, cooldown_period_seconds: u64) -> &mut Self {
        self.instruction.cooldown_period_seconds = Some(cooldown_period_seconds);
        self
    }
    #[inline(always)]
    pub fn proposal_minimum_quorum(&mut self, proposal_minimum_quorum: u32) -> &mut Self {
        self.instruction.proposal_minimum_quorum = Some(proposal_minimum_quorum);
        self
    }
    #[inline(always)]
    pub fn proposal_pass_threshold(&mut self, proposal_pass_threshold: u32) -> &mut Self {
        self.instruction.proposal_pass_threshold = Some(proposal_pass_threshold);
        self
    }
    #[inline(always)]
    pub fn voting_period_seconds(&mut self, voting_period_seconds: u64) -> &mut Self {
        self.instruction.voting_period_seconds = Some(voting_period_seconds);
        self
    }
    #[inline(always)]
    pub fn stake_per_proposal(&mut self, stake_per_proposal: u64) -> &mut Self {
        self.instruction.stake_per_proposal = Some(stake_per_proposal);
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
        let args = UpdateGovernanceInstructionArgs {
            governance_id: self
                .instruction
                .governance_id
                .clone()
                .expect("governance_id is not set"),
            cooldown_period_seconds: self
                .instruction
                .cooldown_period_seconds
                .clone()
                .expect("cooldown_period_seconds is not set"),
            proposal_minimum_quorum: self
                .instruction
                .proposal_minimum_quorum
                .clone()
                .expect("proposal_minimum_quorum is not set"),
            proposal_pass_threshold: self
                .instruction
                .proposal_pass_threshold
                .clone()
                .expect("proposal_pass_threshold is not set"),
            voting_period_seconds: self
                .instruction
                .voting_period_seconds
                .clone()
                .expect("voting_period_seconds is not set"),
            stake_per_proposal: self
                .instruction
                .stake_per_proposal
                .clone()
                .expect("stake_per_proposal is not set"),
        };
        let instruction = UpdateGovernanceCpi {
            __program: self.instruction.__program,

            treasury: self.instruction.treasury.expect("treasury is not set"),

            governance_config: self
                .instruction
                .governance_config
                .expect("governance_config is not set"),
            __args: args,
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct UpdateGovernanceCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    treasury: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    governance_config: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    governance_id: Option<u64>,
    cooldown_period_seconds: Option<u64>,
    proposal_minimum_quorum: Option<u32>,
    proposal_pass_threshold: Option<u32>,
    voting_period_seconds: Option<u64>,
    stake_per_proposal: Option<u64>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}

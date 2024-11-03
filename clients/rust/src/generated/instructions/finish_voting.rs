//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>
//!

use borsh::BorshDeserialize;
use borsh::BorshSerialize;

/// Accounts.
pub struct FinishVoting {
            /// Proposal account

    
              
          pub proposal: solana_program::pubkey::Pubkey,
                /// Paladin stake config account

    
              
          pub stake_config: solana_program::pubkey::Pubkey,
      }

impl FinishVoting {
  pub fn instruction(&self) -> solana_program::instruction::Instruction {
    self.instruction_with_remaining_accounts(&[])
  }
  #[allow(clippy::vec_init_then_push)]
  pub fn instruction_with_remaining_accounts(&self, remaining_accounts: &[solana_program::instruction::AccountMeta]) -> solana_program::instruction::Instruction {
    let mut accounts = Vec::with_capacity(2 + remaining_accounts.len());
                            accounts.push(solana_program::instruction::AccountMeta::new(
            self.proposal,
            false
          ));
                                          accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.stake_config,
            false
          ));
                      accounts.extend_from_slice(remaining_accounts);
    let data = FinishVotingInstructionData::new().try_to_vec().unwrap();
    
    solana_program::instruction::Instruction {
      program_id: crate::PALADIN_GOVERNANCE_ID,
      accounts,
      data,
    }
  }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct FinishVotingInstructionData {
            discriminator: u8,
      }

impl FinishVotingInstructionData {
  pub fn new() -> Self {
    Self {
                        discriminator: 7,
                  }
  }
}

impl Default for FinishVotingInstructionData {
  fn default() -> Self {
    Self::new()
  }
}



/// Instruction builder for `FinishVoting`.
///
/// ### Accounts:
///
                ///   0. `[writable]` proposal
          ///   1. `[]` stake_config
#[derive(Clone, Debug, Default)]
pub struct FinishVotingBuilder {
            proposal: Option<solana_program::pubkey::Pubkey>,
                stake_config: Option<solana_program::pubkey::Pubkey>,
                __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl FinishVotingBuilder {
  pub fn new() -> Self {
    Self::default()
  }
            /// Proposal account
#[inline(always)]
    pub fn proposal(&mut self, proposal: solana_program::pubkey::Pubkey) -> &mut Self {
                        self.proposal = Some(proposal);
                    self
    }
            /// Paladin stake config account
#[inline(always)]
    pub fn stake_config(&mut self, stake_config: solana_program::pubkey::Pubkey) -> &mut Self {
                        self.stake_config = Some(stake_config);
                    self
    }
            /// Add an aditional account to the instruction.
  #[inline(always)]
  pub fn add_remaining_account(&mut self, account: solana_program::instruction::AccountMeta) -> &mut Self {
    self.__remaining_accounts.push(account);
    self
  }
  /// Add additional accounts to the instruction.
  #[inline(always)]
  pub fn add_remaining_accounts(&mut self, accounts: &[solana_program::instruction::AccountMeta]) -> &mut Self {
    self.__remaining_accounts.extend_from_slice(accounts);
    self
  }
  #[allow(clippy::clone_on_copy)]
  pub fn instruction(&self) -> solana_program::instruction::Instruction {
    let accounts = FinishVoting {
                              proposal: self.proposal.expect("proposal is not set"),
                                        stake_config: self.stake_config.expect("stake_config is not set"),
                      };
    
    accounts.instruction_with_remaining_accounts(&self.__remaining_accounts)
  }
}

  /// `finish_voting` CPI accounts.
  pub struct FinishVotingCpiAccounts<'a, 'b> {
                  /// Proposal account

      
                    
              pub proposal: &'b solana_program::account_info::AccountInfo<'a>,
                        /// Paladin stake config account

      
                    
              pub stake_config: &'b solana_program::account_info::AccountInfo<'a>,
            }

/// `finish_voting` CPI instruction.
pub struct FinishVotingCpi<'a, 'b> {
  /// The program to invoke.
  pub __program: &'b solana_program::account_info::AccountInfo<'a>,
            /// Proposal account

    
              
          pub proposal: &'b solana_program::account_info::AccountInfo<'a>,
                /// Paladin stake config account

    
              
          pub stake_config: &'b solana_program::account_info::AccountInfo<'a>,
        }

impl<'a, 'b> FinishVotingCpi<'a, 'b> {
  pub fn new(
    program: &'b solana_program::account_info::AccountInfo<'a>,
          accounts: FinishVotingCpiAccounts<'a, 'b>,
          ) -> Self {
    Self {
      __program: program,
              proposal: accounts.proposal,
              stake_config: accounts.stake_config,
                }
  }
  #[inline(always)]
  pub fn invoke(&self) -> solana_program::entrypoint::ProgramResult {
    self.invoke_signed_with_remaining_accounts(&[], &[])
  }
  #[inline(always)]
  pub fn invoke_with_remaining_accounts(&self, remaining_accounts: &[(&'b solana_program::account_info::AccountInfo<'a>, bool, bool)]) -> solana_program::entrypoint::ProgramResult {
    self.invoke_signed_with_remaining_accounts(&[], remaining_accounts)
  }
  #[inline(always)]
  pub fn invoke_signed(&self, signers_seeds: &[&[&[u8]]]) -> solana_program::entrypoint::ProgramResult {
    self.invoke_signed_with_remaining_accounts(signers_seeds, &[])
  }
  #[allow(clippy::clone_on_copy)]
  #[allow(clippy::vec_init_then_push)]
  pub fn invoke_signed_with_remaining_accounts(
    &self,
    signers_seeds: &[&[&[u8]]],
    remaining_accounts: &[(&'b solana_program::account_info::AccountInfo<'a>, bool, bool)]
  ) -> solana_program::entrypoint::ProgramResult {
    let mut accounts = Vec::with_capacity(2 + remaining_accounts.len());
                            accounts.push(solana_program::instruction::AccountMeta::new(
            *self.proposal.key,
            false
          ));
                                          accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.stake_config.key,
            false
          ));
                      remaining_accounts.iter().for_each(|remaining_account| {
      accounts.push(solana_program::instruction::AccountMeta {
          pubkey: *remaining_account.0.key,
          is_signer: remaining_account.1,
          is_writable: remaining_account.2,
      })
    });
    let data = FinishVotingInstructionData::new().try_to_vec().unwrap();
    
    let instruction = solana_program::instruction::Instruction {
      program_id: crate::PALADIN_GOVERNANCE_ID,
      accounts,
      data,
    };
    let mut account_infos = Vec::with_capacity(2 + 1 + remaining_accounts.len());
    account_infos.push(self.__program.clone());
                  account_infos.push(self.proposal.clone());
                        account_infos.push(self.stake_config.clone());
              remaining_accounts.iter().for_each(|remaining_account| account_infos.push(remaining_account.0.clone()));

    if signers_seeds.is_empty() {
      solana_program::program::invoke(&instruction, &account_infos)
    } else {
      solana_program::program::invoke_signed(&instruction, &account_infos, signers_seeds)
    }
  }
}

/// Instruction builder for `FinishVoting` via CPI.
///
/// ### Accounts:
///
                ///   0. `[writable]` proposal
          ///   1. `[]` stake_config
#[derive(Clone, Debug)]
pub struct FinishVotingCpiBuilder<'a, 'b> {
  instruction: Box<FinishVotingCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> FinishVotingCpiBuilder<'a, 'b> {
  pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
    let instruction = Box::new(FinishVotingCpiBuilderInstruction {
      __program: program,
              proposal: None,
              stake_config: None,
                                __remaining_accounts: Vec::new(),
    });
    Self { instruction }
  }
      /// Proposal account
#[inline(always)]
    pub fn proposal(&mut self, proposal: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
                        self.instruction.proposal = Some(proposal);
                    self
    }
      /// Paladin stake config account
#[inline(always)]
    pub fn stake_config(&mut self, stake_config: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
                        self.instruction.stake_config = Some(stake_config);
                    self
    }
            /// Add an additional account to the instruction.
  #[inline(always)]
  pub fn add_remaining_account(&mut self, account: &'b solana_program::account_info::AccountInfo<'a>, is_writable: bool, is_signer: bool) -> &mut Self {
    self.instruction.__remaining_accounts.push((account, is_writable, is_signer));
    self
  }
  /// Add additional accounts to the instruction.
  ///
  /// Each account is represented by a tuple of the `AccountInfo`, a `bool` indicating whether the account is writable or not,
  /// and a `bool` indicating whether the account is a signer or not.
  #[inline(always)]
  pub fn add_remaining_accounts(&mut self, accounts: &[(&'b solana_program::account_info::AccountInfo<'a>, bool, bool)]) -> &mut Self {
    self.instruction.__remaining_accounts.extend_from_slice(accounts);
    self
  }
  #[inline(always)]
  pub fn invoke(&self) -> solana_program::entrypoint::ProgramResult {
    self.invoke_signed(&[])
  }
  #[allow(clippy::clone_on_copy)]
  #[allow(clippy::vec_init_then_push)]
  pub fn invoke_signed(&self, signers_seeds: &[&[&[u8]]]) -> solana_program::entrypoint::ProgramResult {
        let instruction = FinishVotingCpi {
        __program: self.instruction.__program,
                  
          proposal: self.instruction.proposal.expect("proposal is not set"),
                  
          stake_config: self.instruction.stake_config.expect("stake_config is not set"),
                    };
    instruction.invoke_signed_with_remaining_accounts(signers_seeds, &self.instruction.__remaining_accounts)
  }
}

#[derive(Clone, Debug)]
struct FinishVotingCpiBuilderInstruction<'a, 'b> {
  __program: &'b solana_program::account_info::AccountInfo<'a>,
            proposal: Option<&'b solana_program::account_info::AccountInfo<'a>>,
                stake_config: Option<&'b solana_program::account_info::AccountInfo<'a>>,
                /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
  __remaining_accounts: Vec<(&'b solana_program::account_info::AccountInfo<'a>, bool, bool)>,
}


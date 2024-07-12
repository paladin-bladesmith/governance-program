//! Program processor.

use {
    crate::{instruction::PaladinGovernanceInstruction, state::Proposal},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        clock::Clock,
        entrypoint::ProgramResult,
        incinerator, msg,
        program_error::ProgramError,
        pubkey::Pubkey,
        system_program,
        sysvar::Sysvar,
    },
    spl_discriminator::SplDiscriminate,
};

/// Processes a
/// [CreateProposal](enum.PaladinGovernanceInstruction.html)
/// instruction.
fn process_create_proposal(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let validator_info = next_account_info(accounts_iter)?;
    let _stake_info = next_account_info(accounts_iter)?;
    let proposal_info = next_account_info(accounts_iter)?;

    // Ensure the validator vote account is a signer.
    if !validator_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Ensure the provided stake account belongs to the validator.
    // TODO: Requires imports from stake program.

    // Ensure the proposal account is owned by the Paladin Governance program.
    if proposal_info.owner != program_id {
        return Err(ProgramError::InvalidAccountOwner);
    }

    // Ensure the proposal account has enough space.
    if proposal_info.data_len() != std::mem::size_of::<Proposal>() {
        return Err(ProgramError::InvalidAccountData);
    }

    // Ensure the proposal account is not initialized.
    if &proposal_info.try_borrow_data()?[0..8] == Proposal::SPL_DISCRIMINATOR_SLICE {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let clock = <Clock as Sysvar>::get()?;
    let creation_timestamp = clock.unix_timestamp as u64;
    let instruction = 0; // TODO!

    // Write the data.
    let mut proposal_data = proposal_info.try_borrow_mut_data()?;
    *bytemuck::try_from_bytes_mut::<Proposal>(&mut proposal_data)
        .map_err(|_| ProgramError::InvalidAccountData)? =
        Proposal::new(validator_info.key, creation_timestamp, instruction);

    Ok(())
}

/// Processes a
/// [CancelProposal](enum.PaladinGovernanceInstruction.html)
/// instruction.
fn process_cancel_proposal(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let validator_info = next_account_info(accounts_iter)?;
    let _stake_info = next_account_info(accounts_iter)?;
    let proposal_info = next_account_info(accounts_iter)?;
    let incinerator_info = next_account_info(accounts_iter)?;

    // Ensure the validator vote account is a signer.
    if !validator_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Ensure the provided stake account belongs to the validator.
    // TODO: Requires imports from stake program.

    // Ensure the proposal account is owned by the Paladin Governance program.
    if proposal_info.owner != program_id {
        return Err(ProgramError::InvalidAccountOwner);
    }

    // Ensure the proposal account is initialized.
    if !(proposal_info.data_len() == std::mem::size_of::<Proposal>()
        && &proposal_info.try_borrow_data()?[0..8] == Proposal::SPL_DISCRIMINATOR_SLICE)
    {
        return Err(ProgramError::UninitializedAccount);
    }

    // Close the proposal account.
    if incinerator_info.key != &incinerator::id() {
        return Err(ProgramError::InvalidArgument);
    }

    let new_incinerator_lamports = proposal_info
        .lamports()
        .checked_add(incinerator_info.lamports())
        .ok_or::<ProgramError>(ProgramError::ArithmeticOverflow)?;

    **proposal_info.try_borrow_mut_lamports()? = 0;
    **incinerator_info.try_borrow_mut_lamports()? = new_incinerator_lamports;

    proposal_info.realloc(0, true)?;
    proposal_info.assign(&system_program::id());

    Ok(())
}

/// Processes a
/// [Vote](enum.PaladinGovernanceInstruction.html)
/// instruction.
fn process_vote(_program_id: &Pubkey, _accounts: &[AccountInfo], _vote: bool) -> ProgramResult {
    Ok(())
}

/// Processes a
/// [SwitchVote](enum.PaladinGovernanceInstruction.html)
/// instruction.
fn process_switch_vote(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _vote: bool,
) -> ProgramResult {
    Ok(())
}

/// Processes a
/// [ProcessProposal](enum.PaladinGovernanceInstruction.html)
/// instruction.
fn process_process_proposal(_program_id: &Pubkey, _accounts: &[AccountInfo]) -> ProgramResult {
    Ok(())
}

/// Processes a
/// [InitializeGovernance](enum.PaladinGovernanceInstruction.html)
/// instruction.
fn process_initialize_governance(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _cooldown_period_seconds: u64,
    _proposal_acceptance_threshold: u64,
    _proposal_rejection_threshold: u64,
) -> ProgramResult {
    Ok(())
}

/// Processes a
/// [UpdateGovernance](enum.PaladinGovernanceInstruction.html)
/// instruction.
fn process_update_governance(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _cooldown_period_seconds: u64,
    _proposal_acceptance_threshold: u64,
    _proposal_rejection_threshold: u64,
) -> ProgramResult {
    Ok(())
}

/// Processes a
/// [PaladinGovernanceInstruction](enum.PaladinGovernanceInstruction.html).
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    let instruction = PaladinGovernanceInstruction::unpack(input)?;
    match instruction {
        PaladinGovernanceInstruction::CreateProposal => {
            msg!("Instruction: CreateProposal");
            process_create_proposal(program_id, accounts)
        }
        PaladinGovernanceInstruction::CancelProposal => {
            msg!("Instruction: CancelProposal");
            process_cancel_proposal(program_id, accounts)
        }
        PaladinGovernanceInstruction::Vote { vote } => {
            msg!("Instruction: Vote");
            process_vote(program_id, accounts, vote)
        }
        PaladinGovernanceInstruction::SwitchVote { vote } => {
            msg!("Instruction: SwitchVote");
            process_switch_vote(program_id, accounts, vote)
        }
        PaladinGovernanceInstruction::ProcessProposal => {
            msg!("Instruction: ProcessProposal");
            process_process_proposal(program_id, accounts)
        }
        PaladinGovernanceInstruction::InitializeGovernance {
            cooldown_period_seconds,
            proposal_acceptance_threshold,
            proposal_rejection_threshold,
        } => {
            msg!("Instruction: InitializeGovernance");
            process_initialize_governance(
                program_id,
                accounts,
                cooldown_period_seconds,
                proposal_acceptance_threshold,
                proposal_rejection_threshold,
            )
        }
        PaladinGovernanceInstruction::UpdateGovernance {
            cooldown_period_seconds,
            proposal_acceptance_threshold,
            proposal_rejection_threshold,
        } => {
            msg!("Instruction: UpdateGovernance");
            process_update_governance(
                program_id,
                accounts,
                cooldown_period_seconds,
                proposal_acceptance_threshold,
                proposal_rejection_threshold,
            )
        }
    }
}

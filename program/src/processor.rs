//! Program processor.

use {
    crate::{
        error::PaladinGovernanceError,
        instruction::PaladinGovernanceInstruction,
        state::{
            collect_governance_signer_seeds, collect_vote_signer_seeds, get_governance_address,
            get_governance_address_and_bump_seed, get_vote_address, get_vote_address_and_bump_seed,
            Config, Proposal, ProposalVote,
        },
    },
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        clock::Clock,
        entrypoint::ProgramResult,
        incinerator, msg,
        program::invoke_signed,
        program_error::ProgramError,
        pubkey::Pubkey,
        system_instruction, system_program,
        sysvar::Sysvar,
    },
    spl_discriminator::SplDiscriminate,
    spl_pod::primitives::PodBool,
};

fn check_governance_exists(program_id: &Pubkey, governance_info: &AccountInfo) -> ProgramResult {
    // Ensure the provided governance address is the correct address derived from
    // the program.
    if !governance_info.key.eq(&get_governance_address(program_id)) {
        return Err(PaladinGovernanceError::IncorrectGovernanceConfigAddress.into());
    }

    // Ensure the governance account is owned by the Paladin Governance program.
    if governance_info.owner != program_id {
        return Err(ProgramError::InvalidAccountOwner);
    }

    // Ensure the governance account is initialized.
    if governance_info.data_len() != std::mem::size_of::<Config>() {
        return Err(ProgramError::UninitializedAccount);
    }

    Ok(())
}

fn check_proposal_exists(program_id: &Pubkey, proposal_info: &AccountInfo) -> ProgramResult {
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

    Ok(())
}

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

    check_proposal_exists(program_id, proposal_info)?;

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
fn process_vote(program_id: &Pubkey, accounts: &[AccountInfo], vote: bool) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let validator_info = next_account_info(accounts_iter)?;
    let _stake_info = next_account_info(accounts_iter)?;
    let _vault_info = next_account_info(accounts_iter)?;
    let vote_info = next_account_info(accounts_iter)?;
    let proposal_info = next_account_info(accounts_iter)?;
    let governance_info = next_account_info(accounts_iter)?;
    let _system_program_info = next_account_info(accounts_iter)?;

    // Ensure the validator vote account is a signer.
    if !validator_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Ensure the provided stake account belongs to the validator.
    // TODO: Requires imports from stake program.
    let stake = 0; // TODO!

    // Ensure the proper vault account was provided.
    // TODO: Requires imports from stake program.

    check_governance_exists(program_id, governance_info)?;
    check_proposal_exists(program_id, proposal_info)?;

    // Create the proposal vote account.
    {
        let (vote_address, bump_seed) =
            get_vote_address_and_bump_seed(validator_info.key, proposal_info.key, program_id);
        let bump_seed = [bump_seed];
        let vote_signer_seeds =
            collect_vote_signer_seeds(validator_info.key, proposal_info.key, &bump_seed);

        // Ensure the provided vote address is the correct address derived from
        // the validator and proposal.
        if !vote_info.key.eq(&vote_address) {
            return Err(PaladinGovernanceError::IncorrectProposalVoteAddress.into());
        }

        // Ensure the vote account has not already been initialized.
        if vote_info.data_len() != 0 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        // Allocate & assign.
        invoke_signed(
            &system_instruction::allocate(
                &vote_address,
                std::mem::size_of::<ProposalVote>() as u64,
            ),
            &[vote_info.clone()],
            &[&vote_signer_seeds],
        )?;
        invoke_signed(
            &system_instruction::assign(&vote_address, program_id),
            &[vote_info.clone()],
            &[&vote_signer_seeds],
        )?;

        // Write the data.
        let mut data = vote_info.try_borrow_mut_data()?;
        *bytemuck::try_from_bytes_mut(&mut data).map_err(|_| ProgramError::InvalidAccountData)? =
            ProposalVote::new(proposal_info.key, stake, validator_info.key, vote);
    }

    // Update the proposal with the newly cast vote.
    let mut proposal_data = proposal_info.try_borrow_mut_data()?;
    let state = bytemuck::try_from_bytes_mut::<Proposal>(&mut proposal_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    if vote {
        state.stake_for = state
            .stake_for
            .checked_add(stake)
            .ok_or(ProgramError::ArithmeticOverflow)?;
    } else {
        state.stake_against = state
            .stake_against
            .checked_add(stake)
            .ok_or(ProgramError::ArithmeticOverflow)?;
    }

    // Evaluate the new proposal votes.
    let governance_data = governance_info.try_borrow_data()?;
    let governance_config = bytemuck::try_from_bytes::<Config>(&governance_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    #[allow(clippy::if_same_then_else)]
    if state.stake_for >= governance_config.proposal_acceptance_threshold {
        // If the proposal has met the acceptance threshold, begin the cooldown
        // period.
        // TODO: Requires imports from stake program.
    } else if state.stake_against >= governance_config.proposal_rejection_threshold {
        // If the proposal has met the rejection threshold, cancel the proposal.
        // TODO: Requires imports from stake program.
    }

    Ok(())
}

/// Processes a
/// [SwitchVote](enum.PaladinGovernanceInstruction.html)
/// instruction.
fn process_switch_vote(program_id: &Pubkey, accounts: &[AccountInfo], vote: bool) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let validator_info = next_account_info(accounts_iter)?;
    let _stake_info = next_account_info(accounts_iter)?;
    let _vault_info = next_account_info(accounts_iter)?;
    let vote_info = next_account_info(accounts_iter)?;
    let proposal_info = next_account_info(accounts_iter)?;
    let governance_info = next_account_info(accounts_iter)?;

    // Ensure the validator vote account is a signer.
    if !validator_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Ensure the provided stake account belongs to the validator.
    // TODO: Requires imports from stake program.
    let stake = 0; // TODO!

    // Ensure the proper vault account was provided.

    check_governance_exists(program_id, governance_info)?;
    check_proposal_exists(program_id, proposal_info)?;

    // Update the proposal vote account.
    {
        // Ensure the provided vote address is the correct address derived from
        // the validator and proposal.
        if !vote_info.key.eq(&get_vote_address(
            validator_info.key,
            proposal_info.key,
            program_id,
        )) {
            return Err(PaladinGovernanceError::IncorrectProposalVoteAddress.into());
        }

        // Ensure the vote account is owned by the Paladin Governance program.
        if vote_info.owner != program_id {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // Ensure the vote account is initialized.
        if vote_info.data_len() != std::mem::size_of::<ProposalVote>() {
            return Err(ProgramError::UninitializedAccount);
        }

        // Update the vote.
        let mut data = vote_info.try_borrow_mut_data()?;
        let state = bytemuck::try_from_bytes_mut::<ProposalVote>(&mut data)
            .map_err(|_| ProgramError::InvalidAccountData)?;

        let vote = PodBool::from(vote);
        if state.vote == vote {
            // End early if the vote wasn't changed.
            // Skip updating the proposal.
            return Ok(());
        } else {
            state.vote = vote;
        }
    }

    // Update the proposal with the updated vote.
    // If the program hasn't terminated by this point, the vote has changed.
    // Simply update the proposal by inversing the vote stake.
    let mut proposal_data = proposal_info.try_borrow_mut_data()?;
    let state = bytemuck::try_from_bytes_mut::<Proposal>(&mut proposal_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    if vote {
        // Previous vote against was now switched to a vote for.
        // Move stake from against to for.
        state.stake_for = state
            .stake_for
            .checked_add(stake)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        state.stake_against = state
            .stake_against
            .checked_sub(stake)
            .ok_or(ProgramError::ArithmeticOverflow)?;
    } else {
        // Previous vote for was now switched to a vote against.
        // Move stake from for to against.
        state.stake_for = state
            .stake_for
            .checked_sub(stake)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        state.stake_against = state
            .stake_against
            .checked_add(stake)
            .ok_or(ProgramError::ArithmeticOverflow)?;
    }

    // Evaluate the new proposal votes.
    let governance_data = governance_info.try_borrow_data()?;
    let governance_config = bytemuck::try_from_bytes::<Config>(&governance_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    #[allow(clippy::if_same_then_else)]
    if state.stake_for >= governance_config.proposal_acceptance_threshold {
        // If the proposal has met the acceptance threshold, begin the cooldown
        // period.
        // TODO: Requires imports from stake program.
    } else if state.stake_against >= governance_config.proposal_rejection_threshold {
        // If the proposal has met the rejection threshold, cancel the proposal.
        // TODO: Requires imports from stake program.
    }

    Ok(())
}

/// Processes a
/// [ProcessProposal](enum.PaladinGovernanceInstruction.html)
/// instruction.
fn process_process_proposal(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let proposal_info = next_account_info(accounts_iter)?;
    let _vault_info = next_account_info(accounts_iter)?;
    let governance_info = next_account_info(accounts_iter)?;
    let incinerator_info = next_account_info(accounts_iter)?;

    // Ensure the proper vault account was provided.
    // TODO: Requires imports from stake program.

    check_governance_exists(program_id, governance_info)?;
    check_proposal_exists(program_id, proposal_info)?;

    {
        // Ensure the proposal meets the acceptance threshold.
        let proposal_data = proposal_info.try_borrow_data()?;
        let proposal_state = bytemuck::try_from_bytes::<Proposal>(&proposal_data)
            .map_err(|_| ProgramError::InvalidAccountData)?;

        let governance_data = governance_info.try_borrow_data()?;
        let governance_config = bytemuck::try_from_bytes::<Config>(&governance_data)
            .map_err(|_| ProgramError::InvalidAccountData)?;

        if proposal_state.stake_for < governance_config.proposal_acceptance_threshold {
            // If the proposal has met the acceptance threshold, begin the cooldown
            // period.
            // TODO: Requires imports from stake program.
            return Err(PaladinGovernanceError::ProposalNotAccepted.into());
        }

        // Process the proposal instruction.
        // TODO!
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
/// [InitializeGovernance](enum.PaladinGovernanceInstruction.html)
/// instruction.
fn process_initialize_governance(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    cooldown_period_seconds: u64,
    proposal_acceptance_threshold: u64,
    proposal_rejection_threshold: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let governance_info = next_account_info(accounts_iter)?;
    let _system_program_info = next_account_info(accounts_iter)?;

    // Create the governance config account.
    {
        let (governance_address, bump_seed) = get_governance_address_and_bump_seed(program_id);
        let bump_seed = [bump_seed];
        let governance_signer_seeds = collect_governance_signer_seeds(&bump_seed);

        // Ensure the provided governance address is the correct address
        // derived from the program.
        if !governance_info.key.eq(&governance_address) {
            return Err(PaladinGovernanceError::IncorrectGovernanceConfigAddress.into());
        }

        // Ensure the governance account has not already been initialized.
        if governance_info.data_len() != 0 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        // Allocate & assign.
        invoke_signed(
            &system_instruction::allocate(
                &governance_address,
                std::mem::size_of::<Config>() as u64,
            ),
            &[governance_info.clone()],
            &[&governance_signer_seeds],
        )?;
        invoke_signed(
            &system_instruction::assign(&governance_address, program_id),
            &[governance_info.clone()],
            &[&governance_signer_seeds],
        )?;

        // Write the data.
        let mut data = governance_info.try_borrow_mut_data()?;
        *bytemuck::try_from_bytes_mut(&mut data).map_err(|_| ProgramError::InvalidAccountData)? =
            Config {
                cooldown_period_seconds,
                proposal_acceptance_threshold,
                proposal_rejection_threshold,
                total_staked: 0,
            };
    }

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

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
    paladin_stake_program::state::{find_stake_pda, Config as StakeConfig, Stake},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        clock::Clock,
        entrypoint::ProgramResult,
        msg,
        program::invoke_signed,
        program_error::ProgramError,
        pubkey::Pubkey,
        system_instruction, system_program,
        sysvar::Sysvar,
    },
    spl_discriminator::SplDiscriminate,
    spl_pod::primitives::PodBool,
    std::num::NonZeroU64,
};

const THRESHOLD_SCALING_FACTOR: u64 = 1_000_000_000; // 1e9

fn calculate_vote_threshold(stake: u64, total_stake: u64) -> Result<u64, ProgramError> {
    if total_stake == 0 {
        return Ok(0);
    }
    // Calculation: stake / total_stake
    //
    // Scaled by 1e9 to store 9 decimal places of precision.
    stake
        .checked_mul(THRESHOLD_SCALING_FACTOR)
        .and_then(|product| product.checked_div(total_stake))
        .ok_or(ProgramError::ArithmeticOverflow)
}

fn get_validator_stake_checked(
    validator_key: &Pubkey,
    stake_info: &AccountInfo,
) -> Result<u64, ProgramError> {
    // Ensure the stake account is owned by the Paladin Stake program.
    if stake_info.owner != &paladin_stake_program::id() {
        return Err(ProgramError::InvalidAccountOwner);
    }

    let data = stake_info.try_borrow_data()?;
    let state =
        bytemuck::try_from_bytes::<Stake>(&data).map_err(|_| ProgramError::InvalidAccountData)?;

    // Ensure the stake account is initialized.
    if !state.is_initialized() {
        return Err(ProgramError::UninitializedAccount);
    }

    // Ensure the stake account belongs to the validator.
    if state.validator != *validator_key {
        return Err(PaladinGovernanceError::ValidatorStakeAccountMismatch.into());
    }

    // Return the currently active stake amount.
    Ok(state.amount)
}

fn get_total_stake_checked(
    validator_key: &Pubkey,
    stake_key: &Pubkey,
    stake_config_info: &AccountInfo,
) -> Result<u64, ProgramError> {
    // Ensure the stake address is derived from the validator and config keys.
    if stake_key
        != &find_stake_pda(
            validator_key,
            stake_config_info.key,
            &paladin_stake_program::id(),
        )
        .0
    {
        return Err(PaladinGovernanceError::IncorrectStakeConfig.into());
    }

    // Ensure the config account is owned by the Paladin Stake program.
    if stake_config_info.owner != &paladin_stake_program::id() {
        return Err(ProgramError::InvalidAccountOwner);
    }

    let data = stake_config_info.try_borrow_data()?;
    let state = bytemuck::try_from_bytes::<StakeConfig>(&data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // Ensure the config account is initialized.
    if !state.is_initialized() {
        return Err(ProgramError::UninitializedAccount);
    }

    // Return the total stake amount.
    Ok(state.token_amount_delegated)
}

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

fn check_proposal_cooldown(
    proposal: &Proposal,
    governance_config: &Config,
    clock: &Clock,
) -> ProgramResult {
    if let Some(cooldown_timestamp) = proposal.cooldown_timestamp {
        if (clock.unix_timestamp as u64).saturating_sub(governance_config.cooldown_period_seconds)
            >= cooldown_timestamp.get()
        {
            return Ok(());
        }
    }
    Err(PaladinGovernanceError::ProposalNotAccepted.into())
}

fn close_proposal_account(proposal_info: &AccountInfo) -> ProgramResult {
    proposal_info.realloc(0, true)?;
    proposal_info.assign(&system_program::id());
    Ok(())
}

/// Processes a
/// [CreateProposal](enum.PaladinGovernanceInstruction.html)
/// instruction.
fn process_create_proposal(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let validator_info = next_account_info(accounts_iter)?;
    let stake_info = next_account_info(accounts_iter)?;
    let proposal_info = next_account_info(accounts_iter)?;

    // Ensure the validator vote account is a signer.
    if !validator_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Ensure the provided stake account belongs to the validator.
    let _ = get_validator_stake_checked(validator_info.key, stake_info)?;

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
    let stake_info = next_account_info(accounts_iter)?;
    let proposal_info = next_account_info(accounts_iter)?;

    // Ensure the validator vote account is a signer.
    if !validator_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Ensure the provided stake account belongs to the validator.
    let _ = get_validator_stake_checked(validator_info.key, stake_info)?;

    check_proposal_exists(program_id, proposal_info)?;

    // Ensure the validator is the proposal author.
    {
        let proposal_data = proposal_info.try_borrow_data()?;
        let proposal_state = bytemuck::try_from_bytes::<Proposal>(&proposal_data)
            .map_err(|_| ProgramError::InvalidAccountData)?;

        if proposal_state.author != *validator_info.key {
            return Err(ProgramError::IncorrectAuthority);
        }
    }

    close_proposal_account(proposal_info)?;

    Ok(())
}

/// Processes a
/// [Vote](enum.PaladinGovernanceInstruction.html)
/// instruction.
fn process_vote(program_id: &Pubkey, accounts: &[AccountInfo], vote: bool) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let validator_info = next_account_info(accounts_iter)?;
    let stake_info = next_account_info(accounts_iter)?;
    let stake_config_info = next_account_info(accounts_iter)?;
    let vote_info = next_account_info(accounts_iter)?;
    let proposal_info = next_account_info(accounts_iter)?;
    let governance_info = next_account_info(accounts_iter)?;
    let _system_program_info = next_account_info(accounts_iter)?;

    // Ensure the validator vote account is a signer.
    if !validator_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let stake = get_validator_stake_checked(validator_info.key, stake_info)?;
    let total_stake =
        get_total_stake_checked(validator_info.key, stake_info.key, stake_config_info)?;

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
    let proposal_state = bytemuck::try_from_bytes_mut::<Proposal>(&mut proposal_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    let governance_data = governance_info.try_borrow_data()?;
    let governance_config = bytemuck::try_from_bytes::<Config>(&governance_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    let clock = <Clock as Sysvar>::get()?;

    if vote {
        // The vote was in favor. Increase the stake for the proposal.
        proposal_state.stake_for = proposal_state
            .stake_for
            .checked_add(stake)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        if calculate_vote_threshold(proposal_state.stake_for, total_stake)?
            >= governance_config.proposal_acceptance_threshold
            && proposal_state.cooldown_timestamp.is_none()
        {
            // If the proposal has met the acceptance threshold, and it's
            // currently not in a cooldown period, begin a new cooldown period.
            proposal_state.cooldown_timestamp = NonZeroU64::new(clock.unix_timestamp as u64);
        }
    } else {
        // The vote was against. Increase the stake against the proposal.
        proposal_state.stake_against = proposal_state
            .stake_against
            .checked_add(stake)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        if calculate_vote_threshold(proposal_state.stake_against, total_stake)?
            >= governance_config.proposal_rejection_threshold
        {
            // If the proposal has met the rejection threshold, cancel the proposal.
            // This is done regardless of any cooldown period.
            drop(proposal_data);
            return close_proposal_account(proposal_info);
        }
    }

    Ok(())
}

/// Processes a
/// [SwitchVote](enum.PaladinGovernanceInstruction.html)
/// instruction.
fn process_switch_vote(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    new_vote: bool,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let validator_info = next_account_info(accounts_iter)?;
    let stake_info = next_account_info(accounts_iter)?;
    let stake_config_info = next_account_info(accounts_iter)?;
    let vote_info = next_account_info(accounts_iter)?;
    let proposal_info = next_account_info(accounts_iter)?;
    let governance_info = next_account_info(accounts_iter)?;

    // Ensure the validator vote account is a signer.
    if !validator_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let stake = get_validator_stake_checked(validator_info.key, stake_info)?;
    let total_stake =
        get_total_stake_checked(validator_info.key, stake_info.key, stake_config_info)?;

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

        let pod_new_vote = PodBool::from(new_vote);
        if state.vote == pod_new_vote {
            // End early if the vote wasn't changed.
            // Skip updating the proposal.
            return Ok(());
        } else {
            state.vote = pod_new_vote;
        }
    }

    // Update the proposal with the updated vote.
    // If the program hasn't terminated by this point, the vote has changed.
    // Simply update the proposal by inversing the vote stake.
    let mut proposal_data = proposal_info.try_borrow_mut_data()?;
    let proposal_state = bytemuck::try_from_bytes_mut::<Proposal>(&mut proposal_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    let governance_data = governance_info.try_borrow_data()?;
    let governance_config = bytemuck::try_from_bytes::<Config>(&governance_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    let clock = <Clock as Sysvar>::get()?;

    if new_vote {
        // Previous vote against was now switched to a vote for.
        // Move stake from against to for.
        proposal_state.stake_against = proposal_state
            .stake_against
            .checked_sub(stake)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        proposal_state.stake_for = proposal_state
            .stake_for
            .checked_add(stake)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        if calculate_vote_threshold(proposal_state.stake_for, total_stake)?
            >= governance_config.proposal_acceptance_threshold
            && proposal_state.cooldown_timestamp.is_none()
        {
            // If the proposal has met the acceptance threshold, and it's
            // currently not in a cooldown period, begin a new cooldown period.
            proposal_state.cooldown_timestamp = NonZeroU64::new(clock.unix_timestamp as u64);
        }
    } else {
        // Previous vote for was now switched to a vote against.
        // Move stake from for to against.
        proposal_state.stake_for = proposal_state
            .stake_for
            .checked_sub(stake)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        proposal_state.stake_against = proposal_state
            .stake_against
            .checked_add(stake)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        if calculate_vote_threshold(proposal_state.stake_against, total_stake)?
            >= governance_config.proposal_rejection_threshold
        {
            // If the proposal has met the rejection threshold, cancel the proposal.
            // This is done regardless of any cooldown period.
            drop(proposal_data);
            return close_proposal_account(proposal_info);
        }

        if calculate_vote_threshold(proposal_state.stake_for, total_stake)?
            < governance_config.proposal_acceptance_threshold
            && proposal_state.cooldown_timestamp.is_some()
        {
            // If the proposal has fallen below the acceptance threshold, and
            // it's currently in a cooldown period, terminate the cooldown
            // period.
            proposal_state.cooldown_timestamp = None;
        }
    }

    Ok(())
}

/// Processes a
/// [ProcessProposal](enum.PaladinGovernanceInstruction.html)
/// instruction.
fn process_process_proposal(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let proposal_info = next_account_info(accounts_iter)?;
    // TODO: I've cut the stake config from this instruction, in favor of a
    // more robust cooldown mechanism, which won't need the config account
    // here.
    // It also shouldn't need the governance account, but I'll leave that
    // one be for now.
    let governance_info = next_account_info(accounts_iter)?;

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

        let clock = <Clock as Sysvar>::get()?;

        check_proposal_cooldown(proposal_state, governance_config, &clock)?;

        // Process the proposal instruction.
        // TODO!
    }

    close_proposal_account(proposal_info)?;

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
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    cooldown_period_seconds: u64,
    proposal_acceptance_threshold: u64,
    proposal_rejection_threshold: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let governance_info = next_account_info(accounts_iter)?;
    let proposal_info = next_account_info(accounts_iter)?;
    // Same note as `process_process_proposal` applies here for cutting
    // the stake config account.

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

        let clock = <Clock as Sysvar>::get()?;

        check_proposal_cooldown(proposal_state, governance_config, &clock)?;

        // TODO: This instruction requires a gate to ensure it can only be
        // invoked from a proposal.
    }

    // Update the governance config.
    let mut data = governance_info.try_borrow_mut_data()?;
    *bytemuck::try_from_bytes_mut(&mut data).map_err(|_| ProgramError::InvalidAccountData)? =
        Config {
            cooldown_period_seconds,
            proposal_acceptance_threshold,
            proposal_rejection_threshold,
            total_staked: 0,
        };

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

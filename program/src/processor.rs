//! Program processor.
use {
    crate::{
        error::PaladinGovernanceError,
        instruction::PaladinGovernanceInstruction,
        state::{
            collect_governance_signer_seeds, collect_proposal_transaction_signer_seeds,
            collect_proposal_vote_signer_seeds, collect_treasury_signer_seeds,
            get_governance_address, get_governance_address_and_bump_seed,
            get_proposal_transaction_address, get_proposal_transaction_address_and_bump_seed,
            get_proposal_vote_address, get_proposal_vote_address_and_bump_seed,
            get_treasury_address, get_treasury_address_and_bump_seed, Author, GovernanceConfig,
            Proposal, ProposalAccountMeta, ProposalInstruction, ProposalStatus,
            ProposalTransaction, ProposalVote, ProposalVoteElection,
        },
    },
    borsh::BorshDeserialize,
    paladin_stake_program::{
        require,
        state::{find_validator_stake_pda, Config as StakeConfig, ValidatorStake},
    },
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        borsh1::get_instance_packed_len,
        clock::Clock,
        entrypoint::ProgramResult,
        instruction::Instruction,
        msg,
        program::invoke_signed,
        program_error::ProgramError,
        pubkey::Pubkey,
        rent::Rent,
        system_instruction, system_program,
        sysvar::Sysvar,
    },
    spl_discriminator::{ArrayDiscriminator, SplDiscriminate},
    std::num::NonZeroU64,
};

pub const THRESHOLD_SCALING_FACTOR: u32 = 10u32.pow(9); // 1e9

#[allow(clippy::arithmetic_side_effects)]
fn calculate_maximum_proposals(governance_config: &GovernanceConfig, author_stake: u64) -> u64 {
    if governance_config.stake_per_proposal == 0 {
        return u64::MAX;
    }

    // NB: Division by zero is not possible as we have already handle this case
    // above.
    author_stake / governance_config.stake_per_proposal
}

fn calculate_voter_turnout(
    proposal_state: &Proposal,
    total_stake: u64,
) -> Result<u32, ProgramError> {
    if total_stake == 0 {
        return Ok(0);
    }

    // Calculation: stake / total_stake
    //
    // Scaled by 1e9 to store 9 decimal places of precision.
    u128::from(proposal_state.stake_for)
        .checked_mul(u128::from(THRESHOLD_SCALING_FACTOR))
        .and_then(|scaled_stake| scaled_stake.checked_div(total_stake as u128))
        .and_then(|result| u32::try_from(result).ok())
        .ok_or(ProgramError::ArithmeticOverflow)
}

fn calculate_for_percentage(stake_for: u64, stake_against: u64) -> Result<u32, ProgramError> {
    let total_stake = stake_for
        .checked_add(stake_against)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    if total_stake == 0 {
        return Ok(0);
    }

    // Calculation: stake_for / total_stake
    //
    // Scaled by 1e9 to store 9 decimal places of precision.
    u128::from(stake_for)
        .checked_mul(u128::from(THRESHOLD_SCALING_FACTOR))
        .and_then(|scaled_for| scaled_for.checked_div(total_stake as u128))
        .and_then(|result| u32::try_from(result).ok())
        .ok_or(ProgramError::ArithmeticOverflow)
}

fn get_stake_checked(
    authority_key: &Pubkey,
    stake_config_address: &Pubkey,
    stake_info: &AccountInfo,
) -> Result<u64, ProgramError> {
    check_stake_exists(stake_info)?;

    let data = stake_info.try_borrow_data()?;
    let state = bytemuck::try_from_bytes::<ValidatorStake>(&data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // Ensure the stake account belongs to the authority.
    if state.delegation.authority != *authority_key {
        return Err(ProgramError::IncorrectAuthority);
    }

    // Ensure the stake account has the correct address derived from the
    // validator vote account and the stake config account.
    if stake_info.key
        != &find_validator_stake_pda(
            &state.delegation.validator_vote,
            stake_config_address,
            &paladin_stake_program::id(),
        )
        .0
    {
        return Err(PaladinGovernanceError::StakeConfigMismatch.into());
    }

    Ok(state.delegation.effective_amount)
}

fn check_stake_config_exists(stake_config_info: &AccountInfo) -> ProgramResult {
    // Ensure the stake config account is owned by the Paladin Stake program.
    if stake_config_info.owner != &paladin_stake_program::id() {
        return Err(ProgramError::InvalidAccountOwner);
    }

    // Ensure the stake account is initialized.
    if !(stake_config_info.data_len() == std::mem::size_of::<StakeConfig>()
        && &stake_config_info.try_borrow_data()?[0..8] == StakeConfig::SPL_DISCRIMINATOR_SLICE)
    {
        return Err(ProgramError::UninitializedAccount);
    }

    Ok(())
}

fn check_stake_exists(stake_info: &AccountInfo) -> ProgramResult {
    // Ensure the stake account is owned by the Paladin Stake program.
    if stake_info.owner != &paladin_stake_program::id() {
        return Err(ProgramError::InvalidAccountOwner);
    }

    // Ensure the stake account is initialized.
    if !(stake_info.data_len() == std::mem::size_of::<ValidatorStake>()
        && &stake_info.try_borrow_data()?[0..8] == ValidatorStake::SPL_DISCRIMINATOR_SLICE)
    {
        return Err(ProgramError::UninitializedAccount);
    }

    Ok(())
}

fn check_governance_exists(program_id: &Pubkey, governance_info: &AccountInfo) -> ProgramResult {
    // Ensure the governance account is owned by the Paladin Governance program.
    if governance_info.owner != program_id {
        return Err(ProgramError::InvalidAccountOwner);
    }

    // Ensure the governance account is initialized.
    if governance_info.data_len() != std::mem::size_of::<GovernanceConfig>() {
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

fn check_proposal_transaction_exists(
    program_id: &Pubkey,
    proposal_transaction_info: &AccountInfo,
) -> ProgramResult {
    // Ensure the proposal transaction account is owned by the Paladin
    // Governance program.
    if proposal_transaction_info.owner != program_id {
        return Err(ProgramError::InvalidAccountOwner);
    }

    // Ensure the proposal transaction account is initialized.
    if proposal_transaction_info.data_len() == 0 {
        return Err(ProgramError::UninitializedAccount);
    }

    Ok(())
}

fn process_initialize_author(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let _system_program = next_account_info(accounts_iter)?;
    let stake_authority_info = next_account_info(accounts_iter)?;
    let author_info = next_account_info(accounts_iter)?;

    // Check the author account.
    let (author_pda, author_bump) =
        crate::state::get_proposal_author_address_and_bump(stake_authority_info.key, program_id);
    solana_program::msg!("{} == {}", author_info.key, author_pda);
    require!(author_info.key == &author_pda, ProgramError::InvalidSeeds);
    require!(
        author_info.data_is_empty(),
        ProgramError::AccountAlreadyInitialized
    );
    require!(
        author_info.lamports() >= Rent::get()?.minimum_balance(Author::LEN),
        ProgramError::AccountNotRentExempt
    );

    // Assign & allocate the desired size.
    invoke_signed(
        &system_instruction::allocate(author_info.key, Author::LEN as u64),
        &[author_info.clone()],
        &[&crate::state::collect_proposal_author_seeds(
            stake_authority_info.key,
            &[author_bump],
        )],
    )?;
    invoke_signed(
        &system_instruction::assign(author_info.key, program_id),
        &[author_info.clone()],
        &[&crate::state::collect_proposal_author_seeds(
            stake_authority_info.key,
            &[author_bump],
        )],
    )?;

    Ok(())
}

/// Processes a
/// [CreateProposal](enum.PaladinGovernanceInstruction.html)
/// instruction.
fn process_create_proposal(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let stake_authority_info = next_account_info(accounts_iter)?;
    let author_info = next_account_info(accounts_iter)?;
    let stake_info = next_account_info(accounts_iter)?;
    let proposal_info = next_account_info(accounts_iter)?;
    let proposal_transaction_info = next_account_info(accounts_iter)?;
    let governance_info = next_account_info(accounts_iter)?;
    // NB: Must be loaded for CPIs but never directly accessed.
    let _system_program_info = next_account_info(accounts_iter)?;

    // Ensure the stake authority is a signer.
    if !stake_authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Check & deserialize author.
    if author_info.key
        != &crate::state::get_proposal_author_address(stake_authority_info.key, program_id)
    {
        return Err(ProgramError::InvalidSeeds);
    }
    let mut author_data = author_info.try_borrow_mut_data()?;
    let author_state = bytemuck::try_from_bytes_mut::<Author>(&mut author_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // Ensure a valid stake account was provided.
    {
        check_stake_exists(stake_info)?;

        let data = stake_info.try_borrow_data()?;
        let state = bytemuck::try_from_bytes::<ValidatorStake>(&data)
            .map_err(|_| ProgramError::InvalidAccountData)?;

        // Ensure the stake account belongs to the authority.
        if state.delegation.authority != *stake_authority_info.key {
            return Err(ProgramError::IncorrectAuthority);
        }
    }

    // Check & deserialize governance config.
    check_governance_exists(program_id, governance_info)?;
    let governance_config = {
        let governance_data = governance_info.try_borrow_data()?;
        *bytemuck::try_from_bytes::<GovernanceConfig>(&governance_data)
            .map_err(|_| ProgramError::InvalidAccountData)?
    };

    // Ensure cooldown period has passed
    if Clock::get()?.unix_timestamp < governance_config.cooldown_expires as i64 {
        return Err(PaladinGovernanceError::CooldownPeriodNotOver.into());
    }

    // Increment the active proposal count.
    author_state.active_proposals = author_state
        .active_proposals
        .checked_add(1)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // Ensure the author does not have too many active proposals.
    let author_stake = get_stake_checked(
        stake_authority_info.key,
        &governance_config.stake_config_address,
        stake_info,
    )?;
    if author_state.active_proposals > calculate_maximum_proposals(&governance_config, author_stake)
    {
        return Err(PaladinGovernanceError::TooManyActiveProposals.into());
    }

    // Initialize the proposal account.
    {
        // Ensure the proposal account is owned by the Paladin Governance program.
        if proposal_info.owner != program_id {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // Ensure the proposal account has enough space.
        if proposal_info.data_len() != Proposal::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        // Ensure the proposal account is not initialized.
        if &proposal_info.try_borrow_data()?[0..8] != ArrayDiscriminator::UNINITIALIZED.as_slice() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        let clock = <Clock as Sysvar>::get()?;
        let creation_timestamp = clock.unix_timestamp;

        // Ensure the account is rent exempt.
        require!(
            proposal_info.lamports() >= Rent::get()?.minimum_balance(Proposal::LEN),
            ProgramError::AccountNotRentExempt
        );

        // Write the data.
        let mut proposal_data = proposal_info.try_borrow_mut_data()?;
        *bytemuck::try_from_bytes_mut::<Proposal>(&mut proposal_data)
            .map_err(|_| ProgramError::InvalidAccountData)? = Proposal::new(
            stake_authority_info.key,
            creation_timestamp,
            governance_config,
        );
    }

    // Initialize the proposal transaction account.
    {
        let (proposal_transaction_address, signer_bump_seed) =
            get_proposal_transaction_address_and_bump_seed(proposal_info.key, program_id);
        let bump_seed = [signer_bump_seed];
        let proposal_transaction_signer_seeds =
            collect_proposal_transaction_signer_seeds(proposal_info.key, &bump_seed);

        // Ensure the provided proposal transaction address is the correct
        // address derived from the program.
        if !proposal_transaction_info
            .key
            .eq(&proposal_transaction_address)
        {
            return Err(PaladinGovernanceError::IncorrectProposalTransactionAddress.into());
        }

        // Ensure the proposal transaction account has not already been
        // initialized.
        if proposal_transaction_info.data_len() != 0 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        let state = ProposalTransaction::default();
        let space = get_instance_packed_len(&state)?;

        // Ensure the account is rent exempt.
        require!(
            proposal_transaction_info.lamports() >= Rent::get()?.minimum_balance(space),
            ProgramError::AccountNotRentExempt
        );

        // Allocate & assign.
        invoke_signed(
            &system_instruction::allocate(&proposal_transaction_address, space as u64),
            &[proposal_transaction_info.clone()],
            &[&proposal_transaction_signer_seeds],
        )?;
        invoke_signed(
            &system_instruction::assign(&proposal_transaction_address, program_id),
            &[proposal_transaction_info.clone()],
            &[&proposal_transaction_signer_seeds],
        )?;

        // Write the data.
        borsh::to_writer(&mut proposal_transaction_info.data.borrow_mut()[..], &state)?;
    }

    Ok(())
}

/// Processes a
/// [PushInstruction](enum.PaladinGovernanceInstruction.html)
/// instruction.
fn process_push_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_program_id: Pubkey,
    instruction_account_metas: Vec<ProposalAccountMeta>,
    instruction_data: Vec<u8>,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let stake_authority_info = next_account_info(accounts_iter)?;
    let proposal_info = next_account_info(accounts_iter)?;
    let proposal_transaction_info = next_account_info(accounts_iter)?;

    // Ensure the stake authority is a signer.
    if !stake_authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    check_proposal_exists(program_id, proposal_info)?;

    let proposal_data = proposal_info.try_borrow_data()?;
    let proposal_state = bytemuck::try_from_bytes::<Proposal>(&proposal_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // Ensure the stake authority is the proposal author.
    proposal_state.check_author(stake_authority_info.key)?;

    // Ensure the proposal is in draft stage.
    if proposal_state.status != ProposalStatus::Draft {
        return Err(PaladinGovernanceError::ProposalIsImmutable.into());
    }

    // Ensure the provided proposal transaction address is the correct address
    // derived from the proposal.
    if !proposal_transaction_info
        .key
        .eq(&get_proposal_transaction_address(
            proposal_info.key,
            program_id,
        ))
    {
        return Err(PaladinGovernanceError::IncorrectProposalTransactionAddress.into());
    }

    check_proposal_transaction_exists(program_id, proposal_transaction_info)?;

    let mut proposal_transaction_state =
        ProposalTransaction::try_from_slice(&proposal_transaction_info.try_borrow_data()?)?;

    // Insert the instruction.
    let new_instruction = ProposalInstruction::new(
        &instruction_program_id,
        instruction_account_metas,
        instruction_data,
    );
    proposal_transaction_state
        .instructions
        .push(new_instruction);

    // Reallocate the account.
    let new_len = get_instance_packed_len(&proposal_transaction_state)?;
    proposal_transaction_info.realloc(new_len, true)?;

    // Ensure the account is still rent exempt.
    let rent = Rent::get().unwrap().minimum_balance(new_len);
    require!(
        proposal_transaction_info.lamports() >= rent,
        ProgramError::AccountNotRentExempt,
    );

    // Write the data.
    borsh::to_writer(
        &mut proposal_transaction_info.data.borrow_mut()[..],
        &proposal_transaction_state,
    )?;

    Ok(())
}

/// Processes a
/// [DeleteProposal](enum.PaladinGovernanceInstruction.html)
/// instruction.
fn process_delete_proposal(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let stake_authority_info = next_account_info(accounts_iter)?;
    let author_info = next_account_info(accounts_iter)?;
    let proposal_info = next_account_info(accounts_iter)?;
    let proposal_transaction_info = next_account_info(accounts_iter)?;

    // Ensure the stake authority is a signer.
    if !stake_authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Check & deserialize proposal.
    check_proposal_exists(program_id, proposal_info)?;
    let mut proposal_data = proposal_info.try_borrow_mut_data()?;
    let proposal_state = bytemuck::try_from_bytes_mut::<Proposal>(&mut proposal_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // Check & deserialize proposal author state.
    if author_info.key
        != &crate::state::get_proposal_author_address(stake_authority_info.key, program_id)
    {
        return Err(ProgramError::InvalidSeeds);
    }
    let mut author_data = author_info.try_borrow_mut_data()?;
    let author_state = bytemuck::try_from_bytes_mut::<Author>(&mut author_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // Ensure the stake authority is the proposal author.
    proposal_state.check_author(stake_authority_info.key)?;

    // Ensure the proposal is eligible for deletion.
    if proposal_state.status.is_active() {
        return Err(PaladinGovernanceError::ProposalIsActive.into());
    }

    // Validate the proposal transaction account.
    let (proposal_transaction_address, _) =
        get_proposal_transaction_address_and_bump_seed(proposal_info.key, program_id);
    if proposal_transaction_info.key != &proposal_transaction_address {
        return Err(PaladinGovernanceError::IncorrectProposalTransactionAddress.into());
    }

    // Decrease the user's active proposal count.
    author_state.active_proposals = author_state
        .active_proposals
        .checked_sub(1)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // Delete the proposal & refund the rent.
    drop(proposal_data);
    // NB: The runtime will revert us if we overflow as the sum of balances
    // before/after will not match.
    #[allow(clippy::arithmetic_side_effects)]
    {
        **stake_authority_info.lamports.borrow_mut() +=
            proposal_info.lamports() + proposal_transaction_info.lamports();
    }
    **proposal_info.lamports.borrow_mut() = 0;
    **proposal_transaction_info.lamports.borrow_mut() = 0;
    proposal_info.realloc(0, true)?;
    proposal_transaction_info.realloc(0, true)?;

    Ok(())
}

/// Processes a
/// [BeginVoting](enum.PaladinGovernanceInstruction.html)
/// instruction.
fn process_begin_voting(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let stake_authority_info = next_account_info(accounts_iter)?;
    let proposal_info = next_account_info(accounts_iter)?;

    // Ensure the stake authority is a signer.
    if !stake_authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    check_proposal_exists(program_id, proposal_info)?;

    let mut proposal_data = proposal_info.try_borrow_mut_data()?;
    let proposal_state = bytemuck::try_from_bytes_mut::<Proposal>(&mut proposal_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // Ensure the stake authority is the proposal author.
    proposal_state.check_author(stake_authority_info.key)?;

    // Ensure the proposal is in draft stage.
    if proposal_state.status != ProposalStatus::Draft {
        return Err(PaladinGovernanceError::ProposalIsImmutable.into());
    }

    // Set the proposal's status to voting.
    proposal_state.status = ProposalStatus::Voting;

    // Set the proposal's voting start timestamp.
    let clock = <Clock as Sysvar>::get()?;
    proposal_state.voting_start_timestamp = NonZeroU64::new(clock.unix_timestamp as u64);

    Ok(())
}

/// Processes a
/// [Vote](enum.PaladinGovernanceInstruction.html)
/// instruction.
fn process_vote(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    election: ProposalVoteElection,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let stake_authority_info = next_account_info(accounts_iter)?;
    let stake_info = next_account_info(accounts_iter)?;
    let stake_config_info = next_account_info(accounts_iter)?;
    let proposal_vote_info = next_account_info(accounts_iter)?;
    let proposal_info = next_account_info(accounts_iter)?;
    let _system_program_info = next_account_info(accounts_iter)?;

    // Ensure the stake authority is a signer.
    if !stake_authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let stake = get_stake_checked(stake_authority_info.key, stake_config_info.key, stake_info)?;

    check_stake_config_exists(stake_config_info)?;
    let total_stake =
        bytemuck::try_from_bytes::<StakeConfig>(&stake_config_info.try_borrow_data()?)
            .map_err(|_| ProgramError::InvalidAccountData)?
            .token_amount_effective;

    check_proposal_exists(program_id, proposal_info)?;

    let mut proposal_data = proposal_info.try_borrow_mut_data()?;
    let proposal_state = bytemuck::try_from_bytes_mut::<Proposal>(&mut proposal_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    let governance_config = proposal_state.governance_config;

    // Ensure the address of the provided stake config account matches the one
    // stored in the proposal's governance config.
    governance_config.check_stake_config(stake_config_info.key)?;

    // Ensure the proposal is in the voting stage.
    if proposal_state.status != ProposalStatus::Voting {
        return Err(PaladinGovernanceError::ProposalNotInVotingStage.into());
    }

    let clock = <Clock as Sysvar>::get()?;

    // If the proposal has an active cooldown period, ensure it has not ended.
    if proposal_state.cooldown_has_ended(&clock) {
        return Err(PaladinGovernanceError::ProposalNotInVotingStage.into());
    }

    // Cooldown periods take precedence over voting periods. For example, if a
    // voting period expires, but a cooldown period still has time remaining,
    // the proposal will remain open for voting until the cooldown period ends.
    // Cooldown periods end only in an accepted or rejected proposal.
    if proposal_state.cooldown_timestamp.is_none() && proposal_state.voting_has_ended(&clock) {
        return Err(PaladinGovernanceError::ProposalNotInVotingStage.into());
    }

    // Create the proposal vote account.
    {
        let (proposal_vote_address, bump_seed) =
            get_proposal_vote_address_and_bump_seed(stake_info.key, proposal_info.key, program_id);
        let bump_seed = [bump_seed];
        let proposal_vote_signer_seeds =
            collect_proposal_vote_signer_seeds(stake_info.key, proposal_info.key, &bump_seed);

        // Ensure the provided proposal vote address is the correct address derived from
        // the stake authority and proposal.
        if !proposal_vote_info.key.eq(&proposal_vote_address) {
            return Err(PaladinGovernanceError::IncorrectProposalVoteAddress.into());
        }

        // Ensure the proposal vote account has not already been initialized.
        if proposal_vote_info.data_len() != 0 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        // Ensure the account is rent exempt.
        let size = std::mem::size_of::<ProposalVote>();
        if proposal_vote_info.lamports() < Rent::get()?.minimum_balance(size) {
            return Err(ProgramError::AccountNotRentExempt);
        }

        // Allocate & assign.
        invoke_signed(
            &system_instruction::allocate(&proposal_vote_address, size as u64),
            &[proposal_vote_info.clone()],
            &[&proposal_vote_signer_seeds],
        )?;
        invoke_signed(
            &system_instruction::assign(&proposal_vote_address, program_id),
            &[proposal_vote_info.clone()],
            &[&proposal_vote_signer_seeds],
        )?;

        // Write the data.
        let mut data = proposal_vote_info.try_borrow_mut_data()?;
        *bytemuck::try_from_bytes_mut(&mut data).map_err(|_| ProgramError::InvalidAccountData)? =
            ProposalVote {
                proposal: *proposal_info.key,
                stake,
                authority: *stake_authority_info.key,
                election,
                _padding: Default::default(),
            };
    }

    match election {
        ProposalVoteElection::For => {
            // The vote was in favor. Increase the stake for the proposal.
            proposal_state.stake_for = proposal_state
                .stake_for
                .checked_add(stake)
                .ok_or(ProgramError::ArithmeticOverflow)?;
        }
        ProposalVoteElection::Against => {
            // The vote was against. Increase the stake against the proposal.
            proposal_state.stake_against = proposal_state
                .stake_against
                .checked_add(stake)
                .ok_or(ProgramError::ArithmeticOverflow)?;
        }
    }

    // If we have met quorum and the cooldown has not started yet, start it.
    if calculate_voter_turnout(proposal_state, total_stake)?
        >= governance_config.proposal_minimum_quorum
        && proposal_state.cooldown_timestamp.is_none()
    {
        proposal_state.cooldown_timestamp = NonZeroU64::new(clock.unix_timestamp as u64);
    }

    Ok(())
}

/// Processes a
/// [SwitchVote](enum.PaladinGovernanceInstruction.html)
/// instruction.
fn process_switch_vote(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    new_election: ProposalVoteElection,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let stake_authority_info = next_account_info(accounts_iter)?;
    let stake_info = next_account_info(accounts_iter)?;
    let stake_config_info = next_account_info(accounts_iter)?;
    let proposal_vote_info = next_account_info(accounts_iter)?;
    let proposal_info = next_account_info(accounts_iter)?;

    // Ensure the stake authority is a signer.
    if !stake_authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let stake = get_stake_checked(stake_authority_info.key, stake_config_info.key, stake_info)?;

    check_stake_config_exists(stake_config_info)?;
    let total_stake =
        bytemuck::try_from_bytes::<StakeConfig>(&stake_config_info.try_borrow_data()?)
            .map_err(|_| ProgramError::InvalidAccountData)?
            .token_amount_effective;

    check_proposal_exists(program_id, proposal_info)?;

    let mut proposal_data = proposal_info.try_borrow_mut_data()?;
    let proposal_state = bytemuck::try_from_bytes_mut::<Proposal>(&mut proposal_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    let governance_config = proposal_state.governance_config;

    // Ensure the address of the provided stake config account matches the one
    // stored in the proposal's governance config.
    governance_config.check_stake_config(stake_config_info.key)?;

    // Ensure the proposal is in the voting stage.
    if proposal_state.status != ProposalStatus::Voting {
        return Err(PaladinGovernanceError::ProposalNotInVotingStage.into());
    }

    let clock = <Clock as Sysvar>::get()?;

    // If the proposal has an active cooldown period, ensure it has not ended.
    if proposal_state.cooldown_has_ended(&clock) {
        return Err(PaladinGovernanceError::ProposalNotInVotingStage.into());
    }

    // Cooldown periods take precedence over voting periods. For example, if a
    // voting period expires, but a cooldown period still has time remaining,
    // the proposal will remain open for voting until the cooldown period ends.
    // Cooldown periods end only in an accepted or rejected proposal.
    if proposal_state.cooldown_timestamp.is_none() && proposal_state.voting_has_ended(&clock) {
        return Err(PaladinGovernanceError::ProposalNotInVotingStage.into());
    }

    // Update the proposal vote account.
    let (last_election, last_stake) = {
        // Ensure the provided proposal vote address is the correct address
        // derived from the stake authority and proposal.
        if !proposal_vote_info.key.eq(&get_proposal_vote_address(
            stake_info.key,
            proposal_info.key,
            program_id,
        )) {
            return Err(PaladinGovernanceError::IncorrectProposalVoteAddress.into());
        }

        // Ensure the proposal vote account is owned by the Paladin Governance
        // program.
        if proposal_vote_info.owner != program_id {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // Ensure the proposal vote account is initialized.
        if proposal_vote_info.data_len() != std::mem::size_of::<ProposalVote>() {
            return Err(ProgramError::UninitializedAccount);
        }

        // Update the vote.
        let mut data = proposal_vote_info.try_borrow_mut_data()?;
        let state = bytemuck::try_from_bytes_mut::<ProposalVote>(&mut data)
            .map_err(|_| ProgramError::InvalidAccountData)?;

        (
            std::mem::replace(&mut state.election, new_election),
            std::mem::replace(&mut state.stake, stake),
        )
    };

    // If the program hasn't terminated by this point, the vote has changed.
    // Simply update the proposal by inversing the vote stake.
    match last_election {
        ProposalVoteElection::For => {
            // Previous vote was in favor. Deduct stake for.
            proposal_state.stake_for = proposal_state
                .stake_for
                .checked_sub(last_stake)
                .ok_or(ProgramError::ArithmeticOverflow)?;
        }
        ProposalVoteElection::Against => {
            // Previous vote was against. Deduct stake against.
            proposal_state.stake_against = proposal_state
                .stake_against
                .checked_sub(last_stake)
                .ok_or(ProgramError::ArithmeticOverflow)?;
        }
    }

    // Add the new stake based on new election.
    match new_election {
        ProposalVoteElection::For => {
            // New vote is in favor. Increment stake for.
            proposal_state.stake_for = proposal_state
                .stake_for
                .checked_add(stake)
                .ok_or(ProgramError::ArithmeticOverflow)?;
        }
        ProposalVoteElection::Against => {
            // New vote is against. Increment stake against.
            proposal_state.stake_against = proposal_state
                .stake_against
                .checked_add(stake)
                .ok_or(ProgramError::ArithmeticOverflow)?;
        }
    }

    // If we have met quorum and the cooldown has not started yet, start it.
    if calculate_voter_turnout(proposal_state, total_stake)?
        >= governance_config.proposal_minimum_quorum
        && proposal_state.cooldown_timestamp.is_none()
    {
        proposal_state.cooldown_timestamp = NonZeroU64::new(clock.unix_timestamp as u64);
    }

    Ok(())
}

/// Processes a
/// [FinishVoting](enum.PaladinGovernanceInstruction.html)
/// instruction.
fn process_finish_voting(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let stake_config_info = next_account_info(accounts_iter)?;
    let proposal_info = next_account_info(accounts_iter)?;

    // Validate & deserialize stake config.
    check_stake_config_exists(stake_config_info)?;
    let total_stake =
        bytemuck::try_from_bytes::<StakeConfig>(&stake_config_info.try_borrow_data()?)
            .map_err(|_| ProgramError::InvalidAccountData)?
            .token_amount_effective;

    // Validate & deserialize proposal state.
    check_proposal_exists(program_id, proposal_info)?;
    let mut proposal_data = proposal_info.try_borrow_mut_data()?;
    let proposal_state = bytemuck::try_from_bytes_mut::<Proposal>(&mut proposal_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // Ensure the proposal is in the voting stage.
    if proposal_state.status != ProposalStatus::Voting {
        return Err(PaladinGovernanceError::ProposalNotInVotingStage.into());
    }

    let clock = <Clock as Sysvar>::get()?;
    match proposal_state.cooldown_timestamp {
        Some(_) => {
            // If the proposal is in a cooldown period, check if it has ended.
            if proposal_state.cooldown_has_ended(&clock) {
                // The proposal must still be above quourum and have reached pass threshold.
                let reached_quorum = calculate_voter_turnout(proposal_state, total_stake)?
                    >= proposal_state.governance_config.proposal_minimum_quorum;
                let passed = calculate_for_percentage(
                    proposal_state.stake_for,
                    proposal_state.stake_against,
                )? >= proposal_state.governance_config.proposal_pass_threshold;

                match reached_quorum && passed {
                    true => proposal_state.status = ProposalStatus::Accepted,
                    false => proposal_state.status = ProposalStatus::Rejected,
                }

                Ok(())
            } else {
                Err(PaladinGovernanceError::ProposalVotingPeriodStillActive.into())
            }
        }
        None => {
            // If voting has ended and there is no cooldown period, then this proposal has
            // failed.
            if proposal_state.voting_has_ended(&clock) {
                proposal_state.status = ProposalStatus::Rejected;

                Ok(())
            } else {
                Err(PaladinGovernanceError::ProposalVotingPeriodStillActive.into())
            }
        }
    }
}

/// Processes a
/// [FinishVoting](enum.PaladinGovernanceInstruction.html)
/// instruction.
fn process_delete_vote(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let proposal_info = next_account_info(accounts_iter)?;
    let vote_info = next_account_info(accounts_iter)?;
    let authority_info = next_account_info(accounts_iter)?;

    // Validate the proposal.
    {
        let proposal_data = proposal_info.data.borrow();
        if !proposal_data.is_empty() {
            // Ensure the proposal is not active..
            check_proposal_exists(program_id, proposal_info)?;
            let proposal_data = proposal_info.try_borrow_data()?;
            let proposal_state = bytemuck::try_from_bytes::<Proposal>(&proposal_data)
                .map_err(|_| ProgramError::InvalidAccountData)?;
            if proposal_state.status.is_active() {
                return Err(PaladinGovernanceError::ProposalIsActive.into());
            }
        }
    }

    // Validate the vote account.
    const _: () = assert!(std::mem::size_of::<ProposalVote>() != std::mem::size_of::<Proposal>());
    let (proposal, authority) = {
        let vote = vote_info.data.borrow();
        let vote = bytemuck::from_bytes::<ProposalVote>(&vote);

        (vote.proposal, vote.authority)
    };
    if &proposal != proposal_info.key {
        return Err(PaladinGovernanceError::IncorrectProposalAddress.into());
    }
    if &authority != authority_info.key {
        return Err(ProgramError::IncorrectAuthority);
    }

    // Refund the rent.
    let authority_lamports = authority_info
        .lamports()
        .checked_add(vote_info.lamports())
        .unwrap();
    **authority_info.lamports.borrow_mut() = authority_lamports;

    // Close the vote account.
    vote_info.realloc(0, true)?;
    **vote_info.lamports.borrow_mut() = 0;
    vote_info.assign(&system_program::ID);

    Ok(())
}

/// Processes a
/// [ProcessInstruction](enum.PaladinGovernanceInstruction.html)
/// instruction.
fn process_process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_index: u32,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let proposal_info = next_account_info(accounts_iter)?;
    let proposal_transaction_info = next_account_info(accounts_iter)?;

    check_proposal_exists(program_id, proposal_info)?;

    let mut proposal_data = proposal_info.try_borrow_mut_data()?;
    let proposal_state = bytemuck::try_from_bytes_mut::<Proposal>(&mut proposal_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // Ensure the proposal was accepted.
    if proposal_state.status != ProposalStatus::Accepted {
        return Err(PaladinGovernanceError::ProposalNotAccepted.into());
    }

    // Ensure the provided proposal transaction account has the correct address
    // derived from the proposal.
    if !proposal_transaction_info
        .key
        .eq(&get_proposal_transaction_address(
            proposal_info.key,
            program_id,
        ))
    {
        return Err(PaladinGovernanceError::IncorrectProposalTransactionAddress.into());
    }

    check_proposal_transaction_exists(program_id, proposal_transaction_info)?;

    let mut proposal_transaction_state =
        ProposalTransaction::try_from_slice(&proposal_transaction_info.try_borrow_data()?)?;

    // Ensure the index is valid.
    let instruction_index = instruction_index as usize;
    if instruction_index >= proposal_transaction_state.instructions.len() {
        return Err(PaladinGovernanceError::InvalidTransactionIndex.into());
    }
    let instruction = &proposal_transaction_state.instructions[instruction_index];

    // Ensure the instruction has not already been executed.
    if instruction.executed {
        return Err(PaladinGovernanceError::InstructionAlreadyExecuted.into());
    }

    // Ensure the previous instruction has been executed.
    if instruction_index > 0
        && !proposal_transaction_state.instructions[instruction_index.saturating_sub(1)].executed
    {
        return Err(PaladinGovernanceError::PreviousInstructionHasNotBeenExecuted.into());
    }

    // Execute the instruction.
    {
        let (_treasury_address, signer_bump_seed) = get_treasury_address_and_bump_seed(
            &proposal_state.governance_config.governance_config,
            program_id,
        );
        let bump_seed = [signer_bump_seed];
        let treasury_signer_seeds = collect_treasury_signer_seeds(
            &proposal_state.governance_config.governance_config,
            &bump_seed,
        );

        invoke_signed(
            &Instruction::from(instruction),
            accounts_iter.as_slice(),
            &[&treasury_signer_seeds],
        )?;
    }

    // Mark the instruction as executed.
    proposal_transaction_state.instructions[instruction_index].executed = true;

    // If this was the last instruction, mark the proposal as processed.
    #[allow(clippy::arithmetic_side_effects)]
    {
        if instruction_index == proposal_transaction_state.instructions.len() - 1 {
            proposal_state.status = ProposalStatus::Processed;
        }
    }

    // Write the data (no reallocation necessary).
    borsh::to_writer(
        &mut proposal_transaction_info.data.borrow_mut()[..],
        &proposal_transaction_state,
    )?;

    Ok(())
}

/// Processes a
/// [InitializeGovernance](enum.PaladinGovernanceInstruction.html)
/// instruction.
#[allow(clippy::too_many_arguments)]
fn process_initialize_governance(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    governance_id: u64,
    cooldown_period_seconds: u64,
    proposal_minimum_quorum: u32,
    proposal_pass_threshold: u32,
    voting_period_seconds: u64,
    stake_per_proposal: u64,
    cooldown_seconds: u64,
) -> ProgramResult {
    // Sanity check arguments.
    // 0.1% <= proposal_minimum_quorum < 100%.
    assert!(
        ((THRESHOLD_SCALING_FACTOR / 1000)..THRESHOLD_SCALING_FACTOR)
            .contains(&proposal_minimum_quorum),
        "invalid proposal_minimum_quorum"
    );
    // 10% <= proposal_pass_threshold < 100%.
    assert!(
        ((THRESHOLD_SCALING_FACTOR / 10)..THRESHOLD_SCALING_FACTOR)
            .contains(&proposal_pass_threshold),
        "invalid proposal_pass_threshold"
    );

    // Load accounts.
    let accounts_iter = &mut accounts.iter();
    let governance_info = next_account_info(accounts_iter)?;
    let stake_config_info = next_account_info(accounts_iter)?;
    let _system_program_info = next_account_info(accounts_iter)?;

    check_stake_config_exists(stake_config_info)?;

    // Create the governance config account.
    {
        // Get expiration timestamp for the cooldown period.
        let cooldown_expires =
            (Clock::get()?.unix_timestamp as u64).saturating_add(cooldown_seconds);

        let (governance_address, signer_bump_seed) =
            get_governance_address_and_bump_seed(stake_config_info.key, &governance_id, program_id);
        let bump_seed = [signer_bump_seed];
        let governance_signer_seeds =
            collect_governance_signer_seeds(stake_config_info.key, &governance_id, &bump_seed);

        // Ensure the provided governance address is the correct address
        // derived from the program.
        if !governance_info.key.eq(&governance_address) {
            return Err(PaladinGovernanceError::IncorrectGovernanceConfigAddress.into());
        }

        // Ensure the governance account has not already been initialized.
        if governance_info.data_len() != 0 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        // Ensure the account is rent exempt.
        let size = std::mem::size_of::<GovernanceConfig>();
        if governance_info.lamports() < Rent::get()?.minimum_balance(size) {
            return Err(ProgramError::AccountNotRentExempt);
        }

        // Allocate & assign.
        invoke_signed(
            &system_instruction::allocate(&governance_address, size as u64),
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
            GovernanceConfig {
                cooldown_period_seconds,
                proposal_minimum_quorum,
                proposal_pass_threshold,
                stake_config_address: *stake_config_info.key,
                voting_period_seconds,
                stake_per_proposal,
                governance_config: governance_address,
                cooldown_expires,
            };
    }

    Ok(())
}

/// Processes an
/// [UpdateGovernance](enum.PaladinGovernanceInstruction.html)
/// instruction.
#[allow(clippy::too_many_arguments)]
fn process_update_governance(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    governance_id: u64,
    cooldown_period_seconds: u64,
    proposal_minimum_quorum: u32,
    proposal_pass_threshold: u32,
    voting_period_seconds: u64,
    stake_per_proposal: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let treasury_info = next_account_info(accounts_iter)?;
    let governance_info = next_account_info(accounts_iter)?;

    // Ensure the treasury is a signer.
    if !treasury_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    check_governance_exists(program_id, governance_info)?;

    let mut data = governance_info.try_borrow_mut_data()?;
    let state = bytemuck::try_from_bytes_mut::<GovernanceConfig>(&mut data)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let stake_config_address = state.stake_config_address;

    // Ensure the provided governance account has the correct address derived
    // from the stake config.
    let governance_address =
        get_governance_address(&stake_config_address, &governance_id, program_id);
    if governance_info.key != &governance_address {
        return Err(PaladinGovernanceError::IncorrectGovernanceConfigAddress.into());
    }

    // Ensure the provided treasury account has the correct address derived
    // from the stake config.
    if treasury_info.key != &get_treasury_address(&governance_address, program_id) {
        return Err(PaladinGovernanceError::IncorrectTreasuryAddress.into());
    }

    // Update the governance config.
    state.cooldown_period_seconds = cooldown_period_seconds;
    state.proposal_minimum_quorum = proposal_minimum_quorum;
    state.proposal_pass_threshold = proposal_pass_threshold;
    state.voting_period_seconds = voting_period_seconds;
    state.stake_per_proposal = stake_per_proposal;

    Ok(())
}

/// Processes a
/// [PaladinGovernanceInstruction](enum.PaladinGovernanceInstruction.html).
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    let instruction = PaladinGovernanceInstruction::unpack(input)?;
    match instruction {
        PaladinGovernanceInstruction::InitializeAuthor => {
            msg!("Instruction: InitializeAuthor");
            process_initialize_author(program_id, accounts)
        }
        PaladinGovernanceInstruction::CreateProposal => {
            msg!("Instruction: CreateProposal");
            process_create_proposal(program_id, accounts)
        }
        PaladinGovernanceInstruction::PushInstruction {
            instruction_program_id,
            instruction_account_metas,
            instruction_data,
        } => {
            msg!("Instruction: PushInstruction");
            process_push_instruction(
                program_id,
                accounts,
                instruction_program_id,
                instruction_account_metas,
                instruction_data,
            )
        }
        PaladinGovernanceInstruction::DeleteProposal => {
            msg!("Instruction: DeleteProposal");
            process_delete_proposal(program_id, accounts)
        }
        PaladinGovernanceInstruction::BeginVoting => {
            msg!("Instruction: BeginVoting");
            process_begin_voting(program_id, accounts)
        }
        PaladinGovernanceInstruction::Vote { election } => {
            msg!("Instruction: Vote");
            process_vote(program_id, accounts, election)
        }
        PaladinGovernanceInstruction::SwitchVote { new_election } => {
            msg!("Instruction: SwitchVote");
            process_switch_vote(program_id, accounts, new_election)
        }
        PaladinGovernanceInstruction::FinishVoting => {
            msg!("Instruction: FinishVoting");
            process_finish_voting(program_id, accounts)
        }
        PaladinGovernanceInstruction::DeleteVote => {
            msg!("Instruction: DeleteVote");
            process_delete_vote(program_id, accounts)
        }
        PaladinGovernanceInstruction::ProcessInstruction { instruction_index } => {
            msg!("Instruction: ProcessInstruction");
            process_process_instruction(program_id, accounts, instruction_index)
        }
        PaladinGovernanceInstruction::InitializeGovernance {
            governance_id,
            cooldown_period_seconds,
            proposal_minimum_quorum,
            proposal_pass_threshold,
            voting_period_seconds,
            stake_per_proposal,
            cooldown_seconds,
        } => {
            msg!("Instruction: InitializeGovernance");
            process_initialize_governance(
                program_id,
                accounts,
                governance_id,
                cooldown_period_seconds,
                proposal_minimum_quorum,
                proposal_pass_threshold,
                voting_period_seconds,
                stake_per_proposal,
                cooldown_seconds,
            )
        }
        PaladinGovernanceInstruction::UpdateGovernance {
            governance_id,
            cooldown_period_seconds,
            proposal_minimum_quorum,
            proposal_pass_threshold,
            voting_period_seconds,
            stake_per_proposal,
        } => {
            msg!("Instruction: UpdateGovernance");
            process_update_governance(
                program_id,
                accounts,
                governance_id,
                cooldown_period_seconds,
                proposal_minimum_quorum,
                proposal_pass_threshold,
                voting_period_seconds,
                stake_per_proposal,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use {super::*, proptest::prelude::*};

    // Ensures the (intermediate) stake value is never greater than the total
    // stake value, within the range 0 to the max value provided.
    prop_compose! {
        fn total_and_intermediate(max_value: u64)(total in 0..=max_value)
                        (intermediate in 0..=total, total in Just(total))
                        -> (u64, u64) {
           (intermediate, total)
       }
    }

    proptest! {
        #[test]
        fn test_calculate_proposal_vote_threshold(
            (stake, total_stake) in total_and_intermediate(u64::MAX)
        ) {
            let mut proposal_state = Box::new([0; std::mem::size_of::<Proposal>()]);
            let proposal_state = bytemuck::from_bytes_mut::<Proposal>(&mut proposal_state[..]);
            proposal_state.stake_for = stake;

            // Calculate.
            //
            // Since we've configured limits on the input values, we can safely
            // unwrap the result.
            let result = calculate_voter_turnout(proposal_state, total_stake).unwrap();
            // Evaluate.
            if total_stake == 0 {
                prop_assert_eq!(result, 0);
            } else {
                // The scaling multiplication and subsequent division should
                // always succeed, thanks to the limits on the input values.
                let scaled_stake_ratio = u128::from(stake)
                    .checked_mul(u128::from(THRESHOLD_SCALING_FACTOR))
                    .and_then(|scaled_stake| scaled_stake.checked_div(total_stake as u128))
                    .unwrap();

                // Since a failure to convert to `u32` can only occur if the
                // stake is greater than the total stake, which is not possible
                // thanks to our inputs, we can safely unwrap here.
                let expected = u32::try_from(scaled_stake_ratio).unwrap();

                // The result should be the expected value.
                prop_assert_eq!(result, expected);
            }
        }
    }
}

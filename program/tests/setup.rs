#![cfg(feature = "test-sbf")]
#![allow(dead_code)]

use {
    paladin_governance_program::state::{
        Config, Proposal, ProposalStatus, ProposalVote, ProposalVoteElection,
    },
    paladin_stake_program::state::{Config as StakeConfig, Stake},
    solana_program_test::*,
    solana_sdk::{
        account::{Account, AccountSharedData},
        clock::UnixTimestamp,
        pubkey::Pubkey,
    },
    spl_discriminator::SplDiscriminate,
    std::num::NonZeroU64,
};

pub fn setup() -> ProgramTest {
    ProgramTest::new(
        "paladin_governance_program",
        paladin_governance_program::id(),
        processor!(paladin_governance_program::processor::process),
    )
}

pub async fn setup_stake(
    context: &mut ProgramTestContext,
    stake_address: &Pubkey,
    authority_address: &Pubkey,
    validator_vote_address: &Pubkey,
    amount: u64,
) {
    let mut state = Stake::new(*authority_address, *validator_vote_address);
    state.amount = amount;
    let data = bytemuck::bytes_of(&state).to_vec();

    let rent = context.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(data.len());

    context.set_account(
        stake_address,
        &AccountSharedData::from(Account {
            lamports,
            data,
            owner: paladin_stake_program::id(),
            ..Account::default()
        }),
    );
}

pub async fn setup_stake_config(
    context: &mut ProgramTestContext,
    stake_config_address: &Pubkey,
    total_stake: u64,
) {
    let state = StakeConfig {
        discriminator: StakeConfig::SPL_DISCRIMINATOR.into(),
        token_amount_delegated: total_stake,
        ..Default::default()
    };
    let data = bytemuck::bytes_of(&state).to_vec();

    let rent = context.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(data.len());

    context.set_account(
        stake_config_address,
        &AccountSharedData::from(Account {
            lamports,
            data,
            owner: paladin_stake_program::id(),
            ..Account::default()
        }),
    );
}

pub async fn setup_governance(
    context: &mut ProgramTestContext,
    governance_address: &Pubkey,
    cooldown_period_seconds: u64,
    proposal_acceptance_threshold: u32,
    proposal_rejection_threshold: u32,
    stake_config_address: &Pubkey,
    voting_period_seconds: u64,
) {
    let state = Config::new(
        cooldown_period_seconds,
        proposal_acceptance_threshold,
        proposal_rejection_threshold,
        /* signer_bump_seed */ 0, // TODO: Unused right now.
        stake_config_address,
        voting_period_seconds,
    );
    let data = bytemuck::bytes_of(&state).to_vec();

    let rent = context.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(data.len());

    context.set_account(
        governance_address,
        &AccountSharedData::from(Account {
            lamports,
            data,
            owner: paladin_governance_program::id(),
            ..Account::default()
        }),
    );
}

#[allow(clippy::too_many_arguments)]
async fn _setup_proposal_inner(
    context: &mut ProgramTestContext,
    proposal_address: &Pubkey,
    author: &Pubkey,
    creation_timestamp: UnixTimestamp,
    instruction: u64,
    stake_for: u64,
    stake_against: u64,
    stake_abstained: u64,
    status: ProposalStatus,
    cooldown: Option<NonZeroU64>,
) {
    let mut state = Proposal::new(author, creation_timestamp, instruction);
    state.stake_for = stake_for;
    state.stake_against = stake_against;
    state.stake_abstained = stake_abstained;
    state.status = status;

    if cooldown.is_some() {
        state.cooldown_timestamp = cooldown;
    }

    let data = bytemuck::bytes_of(&state).to_vec();

    let rent = context.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(data.len());

    context.set_account(
        proposal_address,
        &AccountSharedData::from(Account {
            lamports,
            data,
            owner: paladin_governance_program::id(),
            ..Account::default()
        }),
    );
}

#[allow(clippy::too_many_arguments)]
pub async fn setup_proposal_with_stake_and_cooldown(
    context: &mut ProgramTestContext,
    proposal_address: &Pubkey,
    author: &Pubkey,
    creation_timestamp: UnixTimestamp,
    instruction: u64,
    stake_for: u64,
    stake_against: u64,
    stake_abstained: u64,
    status: ProposalStatus,
    cooldown: Option<NonZeroU64>,
) {
    _setup_proposal_inner(
        context,
        proposal_address,
        author,
        creation_timestamp,
        instruction,
        stake_for,
        stake_against,
        stake_abstained,
        status,
        cooldown,
    )
    .await;
}

#[allow(clippy::too_many_arguments)]
pub async fn setup_proposal_with_stake(
    context: &mut ProgramTestContext,
    proposal_address: &Pubkey,
    author: &Pubkey,
    creation_timestamp: UnixTimestamp,
    instruction: u64,
    stake_for: u64,
    stake_against: u64,
    stake_abstained: u64,
    status: ProposalStatus,
) {
    _setup_proposal_inner(
        context,
        proposal_address,
        author,
        creation_timestamp,
        instruction,
        stake_for,
        stake_against,
        stake_abstained,
        status,
        None,
    )
    .await;
}

pub async fn setup_proposal(
    context: &mut ProgramTestContext,
    proposal_address: &Pubkey,
    author: &Pubkey,
    creation_timestamp: UnixTimestamp,
    instruction: u64,
    status: ProposalStatus,
) {
    setup_proposal_with_stake(
        context,
        proposal_address,
        author,
        creation_timestamp,
        instruction,
        0,
        0,
        0,
        status,
    )
    .await;
}

pub async fn setup_proposal_vote(
    context: &mut ProgramTestContext,
    proposal_vote_address: &Pubkey,
    proposal_address: &Pubkey,
    stake: u64,
    stake_authority_address: &Pubkey,
    election: ProposalVoteElection,
) {
    let state = ProposalVote::new(proposal_address, stake, stake_authority_address, election);
    let data = bytemuck::bytes_of(&state).to_vec();

    let rent = context.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(data.len());

    context.set_account(
        proposal_vote_address,
        &AccountSharedData::from(Account {
            lamports,
            data,
            owner: paladin_governance_program::id(),
            ..Account::default()
        }),
    );
}

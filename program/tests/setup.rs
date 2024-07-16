#![cfg(feature = "test-sbf")]
#![allow(dead_code)]

use {
    paladin_governance_program::state::{Config, Proposal, ProposalVote},
    paladin_stake_program::state::{Config as StakeConfig, Stake},
    solana_program_test::*,
    solana_sdk::{
        account::{Account, AccountSharedData},
        pubkey::Pubkey,
    },
    spl_discriminator::SplDiscriminate,
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
    validator_address: &Pubkey,
    amount: u64,
) {
    let mut state = Stake::new(*authority_address, *validator_address);
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
    let mut state = StakeConfig::default();
    state.discriminator = StakeConfig::SPL_DISCRIMINATOR.into();
    state.token_amount_delegated = total_stake;
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
    proposal_acceptance_threshold: u64,
    proposal_rejection_threshold: u64,
    total_staked: u64,
) {
    let state = Config {
        cooldown_period_seconds,
        proposal_acceptance_threshold,
        proposal_rejection_threshold,
        total_staked,
    };
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

pub async fn setup_proposal(
    context: &mut ProgramTestContext,
    proposal_address: &Pubkey,
    author: &Pubkey,
    creation_timestamp: u64,
    instruction: u64,
) {
    let state = Proposal::new(author, creation_timestamp, instruction);
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

pub async fn setup_vote(
    context: &mut ProgramTestContext,
    proposal_vote_address: &Pubkey,
    proposal_address: &Pubkey,
    stake: u64,
    validator_address: &Pubkey,
    vote: bool,
) {
    let state = ProposalVote::new(proposal_address, stake, validator_address, vote);
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

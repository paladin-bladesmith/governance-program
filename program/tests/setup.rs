#![cfg(feature = "test-sbf")]
#![allow(dead_code)]

use {
    borsh::BorshSerialize,
    paladin_governance_program::state::{
        get_proposal_author_address, GovernanceConfig, Proposal, ProposalAccountMeta,
        ProposalInstruction, ProposalStatus, ProposalTransaction, ProposalVote,
        ProposalVoteElection,
    },
    paladin_stake_program::state::{Config as StakeConfig, Delegation, ValidatorStake},
    solana_program_test::*,
    solana_sdk::{
        account::{Account, AccountSharedData},
        clock::{Clock, UnixTimestamp},
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
    authority_address: Pubkey,
    validator_vote: Pubkey,
    amount: u64,
) {
    let mut state = ValidatorStake {
        _discriminator: ValidatorStake::SPL_DISCRIMINATOR.into(),
        delegation: Delegation {
            staked_amount: amount,
            effective_amount: amount,
            authority: authority_address,
            validator_vote,
            ..Default::default()
        },
        total_staked_lamports_amount: 0,
        total_staked_lamports_amount_min: 0,
    };
    state.delegation.staked_amount = amount;
    state.delegation.effective_amount = amount;
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

pub async fn setup_author(
    context: &mut ProgramTestContext,
    authority_address: &Pubkey,
    active_proposals: u64,
) {
    let rent = context.banks_client.get_rent().await.unwrap();
    let data = active_proposals.to_le_bytes().to_vec();
    let lamports = rent.minimum_balance(data.len());

    context.set_account(
        &get_proposal_author_address(authority_address, &paladin_governance_program::ID),
        &AccountSharedData::from(Account {
            lamports,
            data,
            owner: paladin_governance_program::ID,
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
        authority: Some(Pubkey::new_unique()).try_into().unwrap(),
        slash_authority: Some(Pubkey::new_unique()).try_into().unwrap(),
        vault: Pubkey::new_unique(),
        cooldown_time_seconds: 0,
        max_deactivation_basis_points: 0,
        sync_rewards_lamports: 0,
        vault_authority_bump: 0,
        lamports_last: 0,
        token_amount_effective: total_stake,
        accumulated_stake_rewards_per_token: 0.into(),
        duna_document_hash: [0;32],
        _padding: [0; 5],
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
    config: &GovernanceConfig,
) {
    let data = bytemuck::bytes_of(config).to_vec();

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
    governance_config: GovernanceConfig,
    stake_for: u64,
    stake_against: u64,
    status: ProposalStatus,
    voting_start_timestamp: Option<NonZeroU64>,
    cooldown_timestamp: Option<NonZeroU64>,
) {
    let mut state = Proposal::new(author, creation_timestamp, governance_config);
    state.cooldown_timestamp = cooldown_timestamp;
    state.stake_for = stake_for;
    state.stake_against = stake_against;
    state.status = status;
    state.voting_start_timestamp = voting_start_timestamp;

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
    governance_config: GovernanceConfig,
    stake_for: u64,
    stake_against: u64,
    status: ProposalStatus,
    voting_start_timestamp: Option<NonZeroU64>,
    cooldown_timestamp: Option<NonZeroU64>,
) {
    _setup_proposal_inner(
        context,
        proposal_address,
        author,
        creation_timestamp,
        governance_config,
        stake_for,
        stake_against,
        status,
        voting_start_timestamp,
        cooldown_timestamp,
    )
    .await;
}

#[allow(clippy::too_many_arguments)]
pub async fn setup_proposal_with_stake(
    context: &mut ProgramTestContext,
    proposal_address: &Pubkey,
    author: &Pubkey,
    creation_timestamp: UnixTimestamp,
    governance_config: GovernanceConfig,
    stake_for: u64,
    stake_against: u64,
    status: ProposalStatus,
    voting_start_timestamp: Option<NonZeroU64>,
) {
    _setup_proposal_inner(
        context,
        proposal_address,
        author,
        creation_timestamp,
        governance_config,
        stake_for,
        stake_against,
        status,
        voting_start_timestamp,
        None,
    )
    .await;
}

pub async fn setup_proposal(
    context: &mut ProgramTestContext,
    proposal_address: &Pubkey,
    author: &Pubkey,
    creation_timestamp: UnixTimestamp,
    governance_config: GovernanceConfig,
    status: ProposalStatus,
) {
    setup_proposal_with_stake(
        context,
        proposal_address,
        author,
        creation_timestamp,
        governance_config,
        0,
        0,
        status,
        None,
    )
    .await;
}

pub async fn setup_proposal_transaction(
    context: &mut ProgramTestContext,
    proposal_transaction_address: &Pubkey,
    proposal_transaction: ProposalTransaction,
) {
    let mut data = Vec::new();
    proposal_transaction.serialize(&mut data).unwrap();

    let rent = context.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(data.len());

    context.set_account(
        proposal_transaction_address,
        &AccountSharedData::from(Account {
            lamports,
            data,
            owner: paladin_governance_program::id(),
            ..Account::default()
        }),
    );
}

pub async fn setup_proposal_vote(
    context: &mut ProgramTestContext,
    proposal_vote_address: &Pubkey,
    proposal_address: Pubkey,
    stake: u64,
    stake_authority_address: Pubkey,
    election: ProposalVoteElection,
) {
    let state = ProposalVote {
        proposal: proposal_address,
        stake,
        authority: stake_authority_address,
        election,
        _padding: Default::default(),
    };
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

pub fn create_mock_proposal_transaction(program_ids: &[&Pubkey]) -> ProposalTransaction {
    let mut instructions = Vec::new();
    for instruction_program_id in program_ids {
        let instruction_account_metas = vec![
            ProposalAccountMeta {
                pubkey: Pubkey::new_unique(),
                is_signer: false,
                is_writable: false,
            },
            ProposalAccountMeta {
                pubkey: Pubkey::new_unique(),
                is_signer: false,
                is_writable: true,
            },
        ];
        let instruction_data = vec![4; 12];
        instructions.push(ProposalInstruction::new(
            instruction_program_id,
            instruction_account_metas,
            instruction_data,
        ));
    }
    ProposalTransaction { instructions }
}

pub async fn get_clock(context: &mut ProgramTestContext) -> Clock {
    context
        .banks_client
        .get_sysvar::<Clock>()
        .await
        .expect("Failed to get Clock sysvar")
}

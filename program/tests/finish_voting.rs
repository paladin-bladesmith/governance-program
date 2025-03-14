#![cfg(feature = "test-sbf")]
#![allow(clippy::arithmetic_side_effects)]

mod setup;

use {
    paladin_governance_program::{
        error::PaladinGovernanceError,
        instruction::finish_voting,
        state::{GovernanceConfig, Proposal, ProposalStatus},
    },
    setup::{setup, setup_proposal, setup_proposal_with_stake_and_cooldown, setup_stake_config},
    solana_program_test::*,
    solana_sdk::{
        account::AccountSharedData,
        clock::Clock,
        instruction::InstructionError,
        pubkey::Pubkey,
        signer::Signer,
        transaction::{Transaction, TransactionError},
    },
    std::num::NonZeroU64,
};

#[tokio::test]
async fn fail_proposal_not_initialized() {
    let proposal = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, 0).await;

    // Set up the proposal account uninitialized.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let space = std::mem::size_of::<Proposal>();
        let lamports = rent.minimum_balance(space);
        context.set_account(
            &proposal,
            &AccountSharedData::new(lamports, space, &paladin_governance_program::id()),
        );
    }

    let instruction = finish_voting(stake_config, &proposal);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let err = context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap_err()
        .unwrap();

    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::UninitializedAccount)
    );
}

#[tokio::test]
async fn fail_proposal_not_in_voting_stage() {
    let proposal = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();

    let mut governance_config = GovernanceConfig::default();
    governance_config.stake_config_address = stake_config;

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, 0).await;
    setup_proposal(
        &mut context,
        &proposal,
        &Pubkey::new_unique(),
        0,
        governance_config,
        ProposalStatus::Draft, // Not in voting stage.
    )
    .await;

    let instruction = finish_voting(stake_config, &proposal);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let err = context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap_err()
        .unwrap();

    assert_eq!(
        err,
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(PaladinGovernanceError::ProposalNotInVotingStage as u32)
        )
    );
}

#[tokio::test]
async fn fail_proposal_has_cooldown_but_has_not_ended() {
    let proposal = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();

    let mut governance_config = GovernanceConfig::default();
    governance_config.stake_config_address = stake_config;
    governance_config.cooldown_period_seconds = 10; // 10 seconds.

    let mut context = setup().start_with_context().await;

    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    let cooldown_timestamp = clock.unix_timestamp.saturating_sub(5); // Only 5 seconds ago.

    setup_stake_config(&mut context, &stake_config, 0).await;
    // Note that since there's no cooldown, stake doesn't play a role here.
    setup_proposal_with_stake_and_cooldown(
        &mut context,
        &proposal,
        /* author */ &Pubkey::new_unique(),
        /* creation_timestamp */ 0,
        governance_config,
        /* stake_for */ 0,
        /* stake_against */ 0,
        ProposalStatus::Voting,
        /* voting_start_timestamp */ NonZeroU64::new(1),
        /* cooldown_timestamp */ NonZeroU64::new(cooldown_timestamp as u64),
    )
    .await;

    let instruction = finish_voting(stake_config, &proposal);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let err = context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap_err()
        .unwrap();

    assert_eq!(
        err,
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(
                PaladinGovernanceError::ProposalVotingPeriodStillActive as u32
            )
        )
    );
}

#[tokio::test]
async fn success_cooldown_result_is_accepted() {
    let proposal = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();

    let mut governance_config = GovernanceConfig::default();
    governance_config.stake_config_address = stake_config;
    governance_config.cooldown_period_seconds = 10; // 10 seconds.
    governance_config.proposal_minimum_quorum = 500_000_000; // 50%
    governance_config.proposal_pass_threshold = 500_000_000; // 50%

    let total_stake = 100_000_000_000;

    let mut context = setup().start_with_context().await;

    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    let cooldown_timestamp = clock.unix_timestamp.saturating_sub(10); // Ended.

    setup_stake_config(&mut context, &stake_config, total_stake).await;
    setup_proposal_with_stake_and_cooldown(
        &mut context,
        &proposal,
        /* author */ &Pubkey::new_unique(),
        /* creation_timestamp */ 0,
        governance_config,
        /* stake_for */ total_stake / 2, // 50%, accepted.
        /* stake_against */ 0,
        ProposalStatus::Voting,
        /* voting_start_timestamp */ NonZeroU64::new(1),
        /* cooldown_timestamp */ NonZeroU64::new(cooldown_timestamp as u64),
    )
    .await;

    let instruction = finish_voting(stake_config, &proposal);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    // Assert the proposal was marked with accepted status.
    let proposal_account = context
        .banks_client
        .get_account(proposal)
        .await
        .unwrap()
        .unwrap();
    let proposal_state = bytemuck::from_bytes::<Proposal>(&proposal_account.data);
    assert_eq!(proposal_state.status, ProposalStatus::Accepted);
}

#[tokio::test]
async fn success_cooldown_result_is_rejected() {
    let proposal = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();

    let mut governance_config = GovernanceConfig::default();
    governance_config.stake_config_address = stake_config;
    governance_config.cooldown_period_seconds = 10; // 10 seconds.
    governance_config.proposal_minimum_quorum = 500_000_000; // 50%
    governance_config.proposal_pass_threshold = 500_000_000; // 50%

    let total_stake = 100_000_000_000;

    let mut context = setup().start_with_context().await;

    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    let cooldown_timestamp = clock.unix_timestamp.saturating_sub(10); // Ended.

    setup_stake_config(&mut context, &stake_config, total_stake).await;
    setup_proposal_with_stake_and_cooldown(
        &mut context,
        &proposal,
        /* author */ &Pubkey::new_unique(),
        /* creation_timestamp */ 0,
        governance_config,
        /* stake_for */ total_stake / 4, // 25%, not accepted.
        /* stake_against */ total_stake * 3 / 4,
        ProposalStatus::Voting,
        /* voting_start_timestamp */ NonZeroU64::new(1),
        /* cooldown_timestamp */ NonZeroU64::new(cooldown_timestamp as u64),
    )
    .await;

    let instruction = finish_voting(stake_config, &proposal);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    // Assert the proposal was marked with rejected status.
    let proposal_account = context
        .banks_client
        .get_account(proposal)
        .await
        .unwrap()
        .unwrap();
    let proposal_state = bytemuck::from_bytes::<Proposal>(&proposal_account.data);
    assert_eq!(proposal_state.status, ProposalStatus::Rejected);
}

#[tokio::test]
async fn fail_proposal_vote_period_not_ended() {
    let proposal = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();

    let mut governance_config = GovernanceConfig::default();
    governance_config.stake_config_address = stake_config;
    governance_config.voting_period_seconds = 10; // 10 seconds.

    let mut context = setup().start_with_context().await;

    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    let voting_start_timestamp = clock.unix_timestamp.saturating_sub(5); // Only 5 seconds ago.

    setup_stake_config(&mut context, &stake_config, 0).await;
    setup_proposal_with_stake_and_cooldown(
        &mut context,
        &proposal,
        /* author */ &Pubkey::new_unique(),
        /* creation_timestamp */ 0,
        governance_config,
        /* stake_for */ 0,
        /* stake_against */ 0,
        ProposalStatus::Voting,
        /* voting_start_timestamp */ NonZeroU64::new(voting_start_timestamp as u64),
        /* cooldown_timestamp */ None, // No cooldown.
    )
    .await;

    let instruction = finish_voting(stake_config, &proposal);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let err = context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap_err()
        .unwrap();

    assert_eq!(
        err,
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(
                PaladinGovernanceError::ProposalVotingPeriodStillActive as u32
            )
        )
    );
}

#[tokio::test]
async fn success_vote_period_ended_result_rejected() {
    let proposal = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();

    let mut governance_config = GovernanceConfig::default();
    governance_config.stake_config_address = stake_config;
    governance_config.voting_period_seconds = 10; // 10 seconds.

    let mut context = setup().start_with_context().await;

    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    let voting_start_timestamp = clock.unix_timestamp.saturating_sub(10); // Ended.

    setup_stake_config(&mut context, &stake_config, 0).await;
    setup_proposal_with_stake_and_cooldown(
        &mut context,
        &proposal,
        /* author */ &Pubkey::new_unique(),
        /* creation_timestamp */ 0,
        governance_config,
        /* stake_for */ 0,
        /* stake_against */ 0,
        ProposalStatus::Voting,
        /* voting_start_timestamp */ NonZeroU64::new(voting_start_timestamp as u64),
        /* cooldown_timestamp */ None, // No cooldown.
    )
    .await;

    let instruction = finish_voting(stake_config, &proposal);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    // Assert the proposal was marked with rejected status.
    let proposal_account = context
        .banks_client
        .get_account(proposal)
        .await
        .unwrap()
        .unwrap();
    let proposal_state = bytemuck::from_bytes::<Proposal>(&proposal_account.data);
    assert_eq!(proposal_state.status, ProposalStatus::Rejected);
}

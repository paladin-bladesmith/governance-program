#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_governance_program::{
        error::PaladinGovernanceError,
        instruction::initialize_governance,
        state::{get_governance_address, GovernanceConfig},
    },
    paladin_stake_program::state::Config as StakeConfig,
    setup::{setup, setup_governance, setup_stake_config},
    solana_program_test::*,
    solana_sdk::{
        account::AccountSharedData,
        instruction::InstructionError,
        pubkey::Pubkey,
        signer::Signer,
        system_program,
        transaction::{Transaction, TransactionError},
    },
};

#[tokio::test]
async fn fail_stake_config_incorrect_owner() {
    let stake_config = Pubkey::new_unique();
    let governance = get_governance_address(&stake_config, &paladin_governance_program::id());

    let mut context = setup().start_with_context().await;

    // Set up a stake config account with an incorrect owner.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let space = std::mem::size_of::<StakeConfig>();
        let lamports = rent.minimum_balance(space);
        context.set_account(
            &stake_config,
            &AccountSharedData::new(lamports, space, &Pubkey::new_unique()), // Incorrect owner.
        );
    }

    let instruction = initialize_governance(
        &governance,
        &stake_config,
        /* cooldown_period_seconds */ 0,
        /* proposal_acceptance_threshold */ 0,
        /* proposal_rejection_threshold */ 0,
        /* voting_period_seconds */ 0,
    );

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
        TransactionError::InstructionError(0, InstructionError::InvalidAccountOwner)
    );
}

#[tokio::test]
async fn fail_stake_config_not_initialized() {
    let stake_config = Pubkey::new_unique();
    let governance = get_governance_address(&stake_config, &paladin_governance_program::id());

    let mut context = setup().start_with_context().await;

    // Set up an uninitialized stake config account.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let space = std::mem::size_of::<StakeConfig>();
        let lamports = rent.minimum_balance(space);
        context.set_account(
            &stake_config,
            &AccountSharedData::new(lamports, space, &paladin_stake_program::id()),
        );
    }

    let instruction = initialize_governance(
        &governance,
        &stake_config,
        /* cooldown_period_seconds */ 0,
        /* proposal_acceptance_threshold */ 0,
        /* proposal_rejection_threshold */ 0,
        /* voting_period_seconds */ 0,
    );

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
async fn fail_governance_incorrect_address() {
    let governance = Pubkey::new_unique(); // Incorrect governance address.
    let stake_config = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, /* total_stake */ 100).await;

    let instruction = initialize_governance(
        &governance,
        /* stake_config_address */ &stake_config,
        /* cooldown_period_seconds */ 0,
        /* proposal_acceptance_threshold */ 0,
        /* proposal_rejection_threshold */ 0,
        /* voting_period_seconds */ 0,
    );

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
                PaladinGovernanceError::IncorrectGovernanceConfigAddress as u32
            )
        )
    );
}

#[tokio::test]
async fn fail_governance_already_initialized() {
    let stake_config = Pubkey::new_unique();
    let governance = get_governance_address(&stake_config, &paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, /* total_stake */ 100).await;

    // Set up an already initialized governance account.
    setup_governance(&mut context, &governance, 0, 0, 0, &stake_config, 0).await;

    let instruction = initialize_governance(
        &governance,
        &stake_config,
        /* cooldown_period_seconds */ 0,
        /* proposal_acceptance_threshold */ 0,
        /* proposal_rejection_threshold */ 0,
        /* voting_period_seconds */ 0,
    );

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
        TransactionError::InstructionError(0, InstructionError::AccountAlreadyInitialized)
    );
}

#[tokio::test]
async fn success() {
    let stake_config = Pubkey::new_unique();
    let governance = get_governance_address(&stake_config, &paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, /* total_stake */ 100).await;

    // Fund the governance account.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let lamports = rent.minimum_balance(std::mem::size_of::<GovernanceConfig>());
        context.set_account(
            &governance,
            &AccountSharedData::new(lamports, 0, &system_program::id()),
        );
    }

    let instruction = initialize_governance(
        &governance,
        &stake_config,
        /* cooldown_period_seconds */ 0,
        /* proposal_acceptance_threshold */ 0,
        /* proposal_rejection_threshold */ 0,
        /* voting_period_seconds */ 0,
    );

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

    // Assert the governance account was created.
    let governance_account = context
        .banks_client
        .get_account(governance)
        .await
        .unwrap()
        .unwrap();
    let governance_state = bytemuck::from_bytes::<GovernanceConfig>(&governance_account.data);
    assert_eq!(governance_state.cooldown_period_seconds, 0);
    assert_eq!(governance_state.proposal_acceptance_threshold, 0);
    assert_eq!(governance_state.proposal_rejection_threshold, 0);
    assert_eq!(governance_state.stake_config_address, stake_config);
}

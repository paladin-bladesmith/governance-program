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
    let governance = get_governance_address(&stake_config, &0, &paladin_governance_program::id());

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
        /* governance_id */ 0,
        /* cooldown_period_seconds */ 0,
        /* proposal_acceptance_threshold */ 0,
        /* proposal_rejection_threshold */ 0,
        /* voting_period_seconds */ 0,
        /* stake_per_proposal */ 0,
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
    let governance = get_governance_address(&stake_config, &0, &paladin_governance_program::id());

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
        /* governance_id */ 0,
        /* cooldown_period_seconds */ 0,
        /* proposal_acceptance_threshold */ 0,
        /* proposal_rejection_threshold */ 0,
        /* voting_period_seconds */ 0,
        /* stake_per_proposal */ 0,
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
        &stake_config,
        /* governance_id */ 0,
        /* cooldown_period_seconds */ 0,
        /* proposal_acceptance_threshold */ 0,
        /* proposal_rejection_threshold */ 0,
        /* voting_period_seconds */ 0,
        /* stake_per_proposal */ 0,
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
    let governance = get_governance_address(&stake_config, &0, &paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, /* total_stake */ 100).await;

    // Set up an already initialized governance account.
    let governance_config = GovernanceConfig {
        cooldown_period_seconds: 0,
        proposal_minimum_quorum: 0,
        proposal_pass_threshold: 0,
        stake_config_address: stake_config,
        voting_period_seconds: 0,
        stake_per_proposal: 0,
        governance_config: governance,
    };
    setup_governance(&mut context, &governance, &governance_config).await;

    let instruction = initialize_governance(
        &governance,
        &stake_config,
        /* governance_id */ 0,
        /* cooldown_period_seconds */ 0,
        /* proposal_acceptance_threshold */ 0,
        /* proposal_rejection_threshold */ 0,
        /* voting_period_seconds */ 0,
        /* stake_per_proposal */ 0,
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
    let governance = get_governance_address(&stake_config, &0, &paladin_governance_program::id());

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
        /* governance_id */ 0,
        /* cooldown_period_seconds */ 0,
        /* proposal_acceptance_threshold */ 0,
        /* proposal_rejection_threshold */ 0,
        /* voting_period_seconds */ 0,
        /* stake_per_proposal */ 0,
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
    assert_eq!(governance_state.proposal_minimum_quorum, 0);
    assert_eq!(governance_state.proposal_pass_threshold, 0);
    assert_eq!(governance_state.stake_per_proposal, 0);
    assert_eq!(governance_state.stake_config_address, stake_config);
}

#[tokio::test]
async fn setup_second_governance_same_stake_config() {
    let stake_config = Pubkey::new_unique();
    let governance_0 = get_governance_address(&stake_config, &0, &paladin_governance_program::id());
    let governance_1 = get_governance_address(&stake_config, &1, &paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, /* total_stake */ 100).await;

    // Fund the governance accounts.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let lamports = rent.minimum_balance(std::mem::size_of::<GovernanceConfig>());
        context.set_account(
            &governance_0,
            &AccountSharedData::new(lamports, 0, &system_program::id()),
        );
        context.set_account(
            &governance_1,
            &AccountSharedData::new(lamports, 0, &system_program::id()),
        );
    }

    // Initialize governance 0 with one config.
    let instruction = initialize_governance(
        &governance_0,
        &stake_config,
        /* governance_id */ 0,
        /* cooldown_period_seconds */ 100,
        /* proposal_acceptance_threshold */ 200,
        /* proposal_rejection_threshold */ 300,
        /* voting_period_seconds */ 400,
        /* stake_per_proposal */ 500,
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
    let instruction = initialize_governance(
        &governance_1,
        &stake_config,
        /* governance_id */ 1,
        /* cooldown_period_seconds */ 1000,
        /* proposal_acceptance_threshold */ 2000,
        /* proposal_rejection_threshold */ 3000,
        /* voting_period_seconds */ 4000,
        /* stake_per_proposal */ 5000,
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

    // Assert - Both governance configs exist and are different.
    let governance_account_0 = context
        .banks_client
        .get_account(governance_0)
        .await
        .unwrap()
        .unwrap();
    let governance_account_1 = context
        .banks_client
        .get_account(governance_1)
        .await
        .unwrap()
        .unwrap();
    let governance_state_0 = bytemuck::from_bytes::<GovernanceConfig>(&governance_account_0.data);
    assert_eq!(governance_state_0.cooldown_period_seconds, 100);
    assert_eq!(governance_state_0.proposal_minimum_quorum, 200);
    assert_eq!(governance_state_0.proposal_pass_threshold, 300);
    assert_eq!(governance_state_0.voting_period_seconds, 400);
    assert_eq!(governance_state_0.stake_per_proposal, 500);
    assert_eq!(governance_state_0.stake_config_address, stake_config);
    let governance_state_1 = bytemuck::from_bytes::<GovernanceConfig>(&governance_account_1.data);
    assert_eq!(governance_state_1.cooldown_period_seconds, 1000);
    assert_eq!(governance_state_1.proposal_minimum_quorum, 2000);
    assert_eq!(governance_state_1.proposal_pass_threshold, 3000);
    assert_eq!(governance_state_1.voting_period_seconds, 4000);
    assert_eq!(governance_state_1.stake_per_proposal, 5000);
    assert_eq!(governance_state_1.stake_config_address, stake_config);

    // Assert - The treasury accounts are also different.
    assert_ne!(
        paladin_governance_program::state::get_treasury_address(
            &governance_0,
            &paladin_governance_program::ID
        ),
        paladin_governance_program::state::get_treasury_address(
            &governance_1,
            &paladin_governance_program::ID
        ),
    );
}

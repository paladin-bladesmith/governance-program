#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_governance_program::{
        error::PaladinGovernanceError,
        instruction::initialize_governance,
        state::{get_governance_address, Config},
    },
    setup::{setup, setup_governance},
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
async fn fail_governance_incorrect_address() {
    let governance = Pubkey::new_unique(); // Incorrect governance address.

    let mut context = setup().start_with_context().await;

    let instruction = initialize_governance(
        &governance,
        /* cooldown_period_seconds */ 0,
        /* proposal_acceptance_threshold */ 0,
        /* proposal_rejection_threshold */ 0,
        /* stake_config_address */ &Pubkey::new_unique(), // Doesn't matter here.
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
    let governance = get_governance_address(&paladin_governance_program::id());

    let stake_config_address = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;

    // Set up an already initialized governance account.
    setup_governance(&mut context, &governance, 0, 0, 0, &stake_config_address).await;

    let instruction = initialize_governance(
        &governance,
        /* cooldown_period_seconds */ 0,
        /* proposal_acceptance_threshold */ 0,
        /* proposal_rejection_threshold */ 0,
        &stake_config_address,
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
    let governance = get_governance_address(&paladin_governance_program::id());

    let stake_config_address = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;

    // Fund the governance account.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let lamports = rent.minimum_balance(std::mem::size_of::<Config>());
        context.set_account(
            &governance,
            &AccountSharedData::new(lamports, 0, &system_program::id()),
        );
    }

    let instruction = initialize_governance(
        &governance,
        /* cooldown_period_seconds */ 0,
        /* proposal_acceptance_threshold */ 0,
        /* proposal_rejection_threshold */ 0,
        &stake_config_address,
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
    assert_eq!(
        bytemuck::from_bytes::<Config>(&governance_account.data),
        &Config {
            cooldown_period_seconds: 0,
            proposal_acceptance_threshold: 0,
            proposal_rejection_threshold: 0,
            stake_config_address
        }
    );
}

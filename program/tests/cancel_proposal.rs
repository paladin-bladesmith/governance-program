#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_governance_program::{instruction::cancel_proposal, state::Proposal},
    setup::{setup, setup_proposal},
    solana_program_test::*,
    solana_sdk::{
        account::AccountSharedData,
        instruction::InstructionError,
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
        transaction::{Transaction, TransactionError},
    },
};

#[tokio::test]
async fn fail_validator_not_signer() {
    let validator = Keypair::new();
    let stake = Pubkey::new_unique(); // TODO!
    let proposal = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;

    let mut instruction = cancel_proposal(&validator.pubkey(), &stake, &proposal);
    instruction.accounts[0].is_signer = false; // Validator not signer.

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer], // Validator not signer.
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
        TransactionError::InstructionError(0, InstructionError::MissingRequiredSignature)
    );
}

#[tokio::test]
async fn fail_incorrect_stake_account() {
    // TODO!
}

#[tokio::test]
async fn fail_proposal_incorrect_owner() {
    let validator = Keypair::new();
    let stake = Pubkey::new_unique(); // TODO!
    let proposal = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;

    // Set up the proposal account with the incorrect owner.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let space = std::mem::size_of::<Proposal>();
        let lamports = rent.minimum_balance(space);
        context.set_account(
            &proposal,
            &AccountSharedData::new(lamports, space, &Pubkey::new_unique()), // Incorrect owner.
        );
    }

    let instruction = cancel_proposal(&validator.pubkey(), &stake, &proposal);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &validator],
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
async fn fail_proposal_not_initialized() {
    let validator = Keypair::new();
    let stake = Pubkey::new_unique(); // TODO!
    let proposal = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;

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

    let instruction = cancel_proposal(&validator.pubkey(), &stake, &proposal);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &validator],
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
async fn fail_destination_not_incinerator() {
    let validator = Keypair::new();
    let stake = Pubkey::new_unique(); // TODO!
    let proposal = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;
    setup_proposal(&mut context, &proposal, &validator.pubkey(), 0, 0).await;

    let mut instruction = cancel_proposal(&validator.pubkey(), &stake, &proposal);
    instruction.accounts[3].pubkey = Pubkey::new_unique(); // Destination not incinerator.

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &validator],
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
        TransactionError::InstructionError(0, InstructionError::InvalidArgument)
    );
}

#[tokio::test]
async fn success() {
    let validator = Keypair::new();
    let stake = Pubkey::new_unique(); // TODO!
    let proposal = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;
    setup_proposal(&mut context, &proposal, &validator.pubkey(), 0, 0).await;

    let instruction = cancel_proposal(&validator.pubkey(), &stake, &proposal);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &validator],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    // Assert the proposal was closed.
    assert!(context
        .banks_client
        .get_account(proposal)
        .await
        .unwrap()
        .is_none());
}

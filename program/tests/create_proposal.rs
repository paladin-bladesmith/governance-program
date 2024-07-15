#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_governance_program::{instruction::create_proposal, state::Proposal},
    setup::{setup, setup_proposal},
    solana_program_test::*,
    solana_sdk::{
        account::AccountSharedData,
        clock::Clock,
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

    let mut instruction = create_proposal(&validator.pubkey(), &stake, &proposal);
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

    let instruction = create_proposal(&validator.pubkey(), &stake, &proposal);

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
async fn fail_proposal_not_enough_space() {
    let validator = Keypair::new();
    let stake = Pubkey::new_unique(); // TODO!
    let proposal = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;

    // Set up the proposal account with not enough space.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let space = std::mem::size_of::<Proposal>() - 1; // Not enough space.
        let lamports = rent.minimum_balance(space);
        context.set_account(
            &proposal,
            &AccountSharedData::new(lamports, space, &paladin_governance_program::id()),
        );
    }

    let instruction = create_proposal(&validator.pubkey(), &stake, &proposal);

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
        TransactionError::InstructionError(0, InstructionError::InvalidAccountData)
    );
}

#[tokio::test]
async fn fail_proposal_already_initialized() {
    let validator = Keypair::new();
    let stake = Pubkey::new_unique(); // TODO!
    let proposal = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;

    // Set up an initialized proposal account.
    setup_proposal(&mut context, &proposal, &validator.pubkey(), 0, 0).await;

    let instruction = create_proposal(&validator.pubkey(), &stake, &proposal);

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
        TransactionError::InstructionError(0, InstructionError::AccountAlreadyInitialized)
    );
}

#[tokio::test]
async fn success() {
    let validator = Keypair::new();
    let stake = Pubkey::new_unique(); // TODO!
    let proposal = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;

    // Fund the proposal account.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let space = std::mem::size_of::<Proposal>();
        let lamports = rent.minimum_balance(space);
        context.set_account(
            &proposal,
            &AccountSharedData::new(lamports, space, &paladin_governance_program::id()),
        );
    }

    // For checks later.
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    let timestamp = clock.unix_timestamp as u64;

    let instruction = create_proposal(&validator.pubkey(), &stake, &proposal);

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

    // Assert the proposal was created.
    let proposal_account = context
        .banks_client
        .get_account(proposal)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        bytemuck::from_bytes::<Proposal>(&proposal_account.data),
        &Proposal::new(&validator.pubkey(), timestamp, 0)
    );
}

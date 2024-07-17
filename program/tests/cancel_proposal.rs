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
    let proposal = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;

    let mut instruction = cancel_proposal(&validator.pubkey(), &proposal);
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
async fn fail_proposal_incorrect_owner() {
    let validator = Keypair::new();
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

    let instruction = cancel_proposal(&validator.pubkey(), &proposal);

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

    let instruction = cancel_proposal(&validator.pubkey(), &proposal);

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
async fn fail_validator_not_author() {
    let validator = Keypair::new();
    let proposal = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;
    setup_proposal(
        &mut context,
        &proposal,
        &Pubkey::new_unique(), // Validator not author.
        0,
        0,
    )
    .await;

    let instruction = cancel_proposal(&validator.pubkey(), &proposal);

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
        TransactionError::InstructionError(0, InstructionError::IncorrectAuthority)
    );
}

#[tokio::test]
async fn success() {
    let validator = Keypair::new();
    let proposal = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;
    setup_proposal(&mut context, &proposal, &validator.pubkey(), 0, 0).await;

    let instruction = cancel_proposal(&validator.pubkey(), &proposal);

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

    // Assert the proposal was cleared and reassigned to the system program.
    let proposal_account = context
        .banks_client
        .get_account(proposal)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(proposal_account.owner, solana_program::system_program::id());
    assert_eq!(proposal_account.data.len(), 0);
}

#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_governance_program::{instruction::initialize_author, state::Author},
    setup::setup,
    solana_program_test::*,
    solana_sdk::{
        account::Account,
        instruction::InstructionError,
        pubkey::Pubkey,
        rent::Rent,
        signer::Signer,
        system_instruction,
        transaction::{Transaction, TransactionError},
    },
};

#[tokio::test]
async fn fail_incorrect_author() {
    let context = setup().start_with_context().await;
    let author = Pubkey::new_unique();

    // Try and initialize an author account with the wrong author address.
    let fund = system_instruction::transfer(
        &context.payer.pubkey(),
        &author,
        Rent::default().minimum_balance(Author::LEN),
    );
    let mut initialize = initialize_author(context.payer.pubkey());
    initialize.accounts[1].pubkey = author; // Invalid author address.
    let transaction = Transaction::new_signed_with_payer(
        &[fund, initialize],
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

    // Seeds should not match.
    assert_eq!(
        err,
        TransactionError::InstructionError(1, InstructionError::InvalidSeeds)
    );
}

#[tokio::test]
async fn fail_already_initialized() {
    let context = setup().start_with_context().await;

    let fund = system_instruction::transfer(
        &context.payer.pubkey(),
        &paladin_governance_program::state::get_proposal_author_address(
            &context.payer.pubkey(),
            &paladin_governance_program::ID,
        ),
        Rent::default().minimum_balance(Author::LEN),
    );
    let initialize = initialize_author(context.payer.pubkey());

    // Initialize once.
    let transaction = Transaction::new_signed_with_payer(
        &[fund, initialize.clone()],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    // Initialize again.
    let transaction = Transaction::new_signed_with_payer(
        &[initialize],
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

    // Seeds should not match.
    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::AccountAlreadyInitialized)
    );
}

#[tokio::test]
async fn success() {
    let context = setup().start_with_context().await;
    let author = paladin_governance_program::state::get_proposal_author_address(
        &context.payer.pubkey(),
        &paladin_governance_program::ID,
    );

    let rent = Rent::default().minimum_balance(Author::LEN);
    let fund = system_instruction::transfer(&context.payer.pubkey(), &author, rent);
    let initialize = initialize_author(context.payer.pubkey());

    // Initialize.
    let transaction = Transaction::new_signed_with_payer(
        &[fund, initialize.clone()],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    // Confirm state.
    let author = context
        .banks_client
        .get_account(author)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        author,
        Account {
            lamports: rent,
            data: vec![0; 8],
            owner: paladin_governance_program::ID,
            executable: false,
            rent_epoch: u64::MAX,
        }
    );
}

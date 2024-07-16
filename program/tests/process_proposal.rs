#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_governance_program::{
        error::PaladinGovernanceError,
        instruction::process_proposal,
        state::{get_governance_address, Config, Proposal},
    },
    setup::{setup, setup_governance, setup_proposal},
    solana_program_test::*,
    solana_sdk::{
        account::AccountSharedData,
        instruction::InstructionError,
        pubkey::Pubkey,
        signer::Signer,
        transaction::{Transaction, TransactionError},
    },
};

#[tokio::test]
async fn fail_governance_incorrect_address() {
    let proposal = Pubkey::new_unique();
    let governance = Pubkey::new_unique(); // Incorrect governance address.

    let mut context = setup().start_with_context().await;

    let instruction = process_proposal(&proposal, &governance);

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
async fn fail_governance_incorrect_owner() {
    let proposal = Pubkey::new_unique();
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;

    // Set up the governance account with the incorrect owner.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let space = std::mem::size_of::<Config>();
        let lamports = rent.minimum_balance(space);
        context.set_account(
            &governance,
            &AccountSharedData::new(lamports, space, &Pubkey::new_unique()), // Incorrect owner.
        );
    }

    let instruction = process_proposal(&proposal, &governance);

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
async fn fail_governance_not_initialized() {
    let proposal = Pubkey::new_unique();
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;

    // Set up the governance account uninitialized.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let lamports = rent.minimum_balance(std::mem::size_of::<Config>());
        context.set_account(
            &governance,
            &AccountSharedData::new(lamports, 0, &paladin_governance_program::id()),
        );
    }

    let instruction = process_proposal(&proposal, &governance);

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
async fn fail_proposal_incorrect_owner() {
    let proposal = Pubkey::new_unique();
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_governance(&mut context, &governance, 0, 0, 0, 0).await;

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

    let instruction = process_proposal(&proposal, &governance);

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
async fn fail_proposal_not_initialized() {
    let proposal = Pubkey::new_unique();
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_governance(&mut context, &governance, 0, 0, 0, 0).await;

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

    let instruction = process_proposal(&proposal, &governance);

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
async fn fail_proposal_not_accepted() {
    let proposal = Pubkey::new_unique();
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_governance(&mut context, &governance, 0, 0, 0, 0).await;

    // Set up an unaccepted proposal.
    // Simply set the required threshold, then initialize the proposal to zero
    // stake for.
    setup_governance(&mut context, &governance, 0, 100_000, 0, 0).await;
    setup_proposal(&mut context, &proposal, &Pubkey::new_unique(), 0, 0).await;

    let instruction = process_proposal(&proposal, &governance);

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
            InstructionError::Custom(PaladinGovernanceError::ProposalNotAccepted as u32)
        )
    );
}

#[tokio::test]
async fn fail_destination_not_incinerator() {
    let proposal = Pubkey::new_unique();
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_governance(&mut context, &governance, 0, 0, 0, 0).await;
    setup_proposal(&mut context, &proposal, &Pubkey::new_unique(), 0, 0).await;

    let mut instruction = process_proposal(&proposal, &governance);
    instruction.accounts[2].pubkey = Pubkey::new_unique(); // Destination not incinerator.

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
        TransactionError::InstructionError(0, InstructionError::InvalidArgument)
    );
}

#[tokio::test]
async fn success() {
    let proposal = Pubkey::new_unique();
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_governance(&mut context, &governance, 0, 0, 0, 0).await;
    setup_proposal(&mut context, &proposal, &Pubkey::new_unique(), 0, 0).await;

    let instruction = process_proposal(&proposal, &governance);

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

    // Assert the proposal was closed.
    assert!(context
        .banks_client
        .get_account(proposal)
        .await
        .unwrap()
        .is_none());

    // TODO: Assert the instruction was processed.
}

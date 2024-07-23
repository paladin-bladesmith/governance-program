#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_governance_program::{instruction::create_proposal, state::Proposal},
    paladin_stake_program::state::Stake,
    setup::{setup, setup_proposal, setup_stake},
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
async fn fail_stake_authority_not_signer() {
    let stake_authority = Keypair::new();
    let stake = Pubkey::new_unique(); // PDA doesn't matter here.
    let proposal = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;

    let mut instruction = create_proposal(&stake_authority.pubkey(), &stake, &proposal);
    instruction.accounts[0].is_signer = false; // Stake authority not signer.

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer], // Stake authority not signer.
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
async fn fail_stake_incorrect_owner() {
    let stake_authority = Keypair::new();
    let stake = Pubkey::new_unique(); // PDA doesn't matter here.
    let proposal = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;

    // Set up the stake account with the incorrect owner.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let space = std::mem::size_of::<Stake>();
        let lamports = rent.minimum_balance(space);
        context.set_account(
            &stake,
            &AccountSharedData::new(lamports, space, &Pubkey::new_unique()), // Incorrect owner.
        );
    }

    let instruction = create_proposal(&stake_authority.pubkey(), &stake, &proposal);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &stake_authority],
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
async fn fail_stake_not_initialized() {
    let stake_authority = Keypair::new();
    let stake = Pubkey::new_unique(); // PDA doesn't matter here.
    let proposal = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;

    // Set up an uninitialized stake account.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let space = std::mem::size_of::<Stake>();
        let lamports = rent.minimum_balance(space);
        context.set_account(
            &stake,
            &AccountSharedData::new(lamports, space, &paladin_stake_program::id()),
        );
    }

    let instruction = create_proposal(&stake_authority.pubkey(), &stake, &proposal);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &stake_authority],
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
async fn fail_stake_incorrect_stake_authority() {
    let stake_authority = Keypair::new();
    let stake = Pubkey::new_unique(); // PDA doesn't matter here.
    let proposal = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;

    // Set up a stake account with the wrong stake authority address.
    setup_stake(
        &mut context,
        &stake,
        &Pubkey::new_unique(), // Incorrect stake authority.
        /* validator_vote_address */ &Pubkey::new_unique(), // Unused here.
        0,
    )
    .await;

    let instruction = create_proposal(&stake_authority.pubkey(), &stake, &proposal);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &stake_authority],
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
async fn fail_proposal_incorrect_owner() {
    let stake_authority = Keypair::new();
    let stake = Pubkey::new_unique(); // PDA doesn't matter here.
    let proposal = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;
    setup_stake(
        &mut context,
        &stake,
        &stake_authority.pubkey(),
        /* validator_vote_address */ &Pubkey::new_unique(), // Unused here.
        0,
    )
    .await;

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

    let instruction = create_proposal(&stake_authority.pubkey(), &stake, &proposal);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &stake_authority],
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
    let stake_authority = Keypair::new();
    let stake = Pubkey::new_unique(); // PDA doesn't matter here.
    let proposal = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;
    setup_stake(
        &mut context,
        &stake,
        &stake_authority.pubkey(),
        /* validator_vote_address */ &Pubkey::new_unique(), // Unused here.
        0,
    )
    .await;

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

    let instruction = create_proposal(&stake_authority.pubkey(), &stake, &proposal);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &stake_authority],
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
    let stake_authority = Keypair::new();
    let stake = Pubkey::new_unique(); // PDA doesn't matter here.
    let proposal = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;
    setup_stake(
        &mut context,
        &stake,
        &stake_authority.pubkey(),
        /* validator_vote_address */ &Pubkey::new_unique(), // Unused here.
        0,
    )
    .await;

    // Set up an initialized proposal account.
    setup_proposal(&mut context, &proposal, &stake_authority.pubkey(), 0, 0).await;

    let instruction = create_proposal(&stake_authority.pubkey(), &stake, &proposal);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &stake_authority],
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
    let stake_authority = Keypair::new();
    let stake = Pubkey::new_unique(); // PDA doesn't matter here.
    let proposal = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;
    setup_stake(
        &mut context,
        &stake,
        &stake_authority.pubkey(),
        /* validator_vote_address */ &Pubkey::new_unique(), // Unused here.
        0,
    )
    .await;

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
    let timestamp = clock.unix_timestamp;

    let instruction = create_proposal(&stake_authority.pubkey(), &stake, &proposal);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &stake_authority],
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
        &Proposal::new(&stake_authority.pubkey(), timestamp, 0)
    );
}

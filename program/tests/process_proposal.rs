#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_governance_program::{
        error::PaladinGovernanceError,
        instruction::process_proposal,
        state::{Config, Proposal, ProposalStatus},
    },
    setup::{setup, setup_governance, setup_proposal_with_stake_and_cooldown},
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
async fn fail_governance_incorrect_owner() {
    let proposal = Pubkey::new_unique();
    let governance = Pubkey::new_unique(); // PDA doesn't matter here.

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
    let governance = Pubkey::new_unique(); // PDA doesn't matter here.

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
    let governance = Pubkey::new_unique(); // PDA doesn't matter here.

    let mut context = setup().start_with_context().await;
    setup_governance(
        &mut context,
        &governance,
        0,
        0,
        0,
        /* stake_config_address */ &Pubkey::new_unique(), // Doesn't matter here.
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
    let governance = Pubkey::new_unique(); // PDA doesn't matter here.

    let mut context = setup().start_with_context().await;
    setup_governance(
        &mut context,
        &governance,
        0,
        0,
        0,
        /* stake_config_address */ &Pubkey::new_unique(), // Doesn't matter here.
    )
    .await;

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
async fn fail_proposal_cooldown_in_progress() {
    let proposal = Pubkey::new_unique();
    let governance = Pubkey::new_unique(); // PDA doesn't matter here.

    let mut context = setup().start_with_context().await;

    // Set up an unaccepted proposal.
    // Simply set the cooldown timestamp to the current clock timestamp,
    // and require more than 0 seconds for cooldown.
    setup_governance(
        &mut context,
        &governance,
        1_000_000,
        0,
        0,
        /* stake_config_address */ &Pubkey::new_unique(), // Doesn't matter here.
    )
    .await;
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    setup_proposal_with_stake_and_cooldown(
        &mut context,
        &proposal,
        &Pubkey::new_unique(),
        0,
        0,
        0,
        0,
        0,
        ProposalStatus::Accepted,
        NonZeroU64::new(clock.unix_timestamp as u64),
    )
    .await;

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
async fn fail_proposal_not_accepted() {
    let proposal = Pubkey::new_unique();
    let governance = Pubkey::new_unique(); // PDA doesn't matter here.

    let mut context = setup().start_with_context().await;

    setup_governance(
        &mut context,
        &governance,
        0,
        0,
        0,
        /* stake_config_address */ &Pubkey::new_unique(), // Doesn't matter here.
    )
    .await;
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    setup_proposal_with_stake_and_cooldown(
        &mut context,
        &proposal,
        &Pubkey::new_unique(),
        0,
        0,
        0,
        0,
        0,
        ProposalStatus::Voting, // Not accepted.
        NonZeroU64::new(clock.unix_timestamp as u64),
    )
    .await;

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
async fn success() {
    let proposal = Pubkey::new_unique();
    let governance = Pubkey::new_unique(); // PDA doesn't matter here.

    let mut context = setup().start_with_context().await;
    setup_governance(
        &mut context,
        &governance,
        0,
        0,
        0,
        /* stake_config_address */ &Pubkey::new_unique(), // Doesn't matter here.
    )
    .await;
    setup_proposal_with_stake_and_cooldown(
        &mut context,
        &proposal,
        &Pubkey::new_unique(),
        0,
        0,
        0,
        0,
        0,
        ProposalStatus::Accepted,
        NonZeroU64::new(1),
    )
    .await;

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

    // Assert the proposal was marked with processed status.
    let proposal_account = context
        .banks_client
        .get_account(proposal)
        .await
        .unwrap()
        .unwrap();
    let proposal_state = bytemuck::from_bytes::<Proposal>(&proposal_account.data);
    assert_eq!(proposal_state.status, ProposalStatus::Processed);

    // TODO: Assert the instruction was processed.
}

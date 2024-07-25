#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_governance_program::{
        error::PaladinGovernanceError,
        instruction::process_proposal,
        state::{Config, Proposal, ProposalStatus},
    },
    setup::{setup, setup_proposal_with_stake_and_cooldown},
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
async fn fail_proposal_incorrect_owner() {
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

    let instruction = process_proposal(&proposal);

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

    let instruction = process_proposal(&proposal);

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

    let governance_config = Config::new(
        /* cooldown_period_seconds */ 100_000_000,
        /* proposal_acceptance_threshold */ 500_000_000, // 50%
        /* proposal_rejection_threshold */ 500_000_000, // 50%
        /* signer_bump_seed */ 0,
        /* stake_config_address */ &Pubkey::new_unique(), // Doesn't matter here.
        /* voting_period_seconds */ 100_000_000,
    );

    let mut context = setup().start_with_context().await;
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();

    // Set up an unaccepted proposal.
    // Simply set the cooldown timestamp to the current clock timestamp,
    // and require more than 0 seconds for cooldown.
    setup_proposal_with_stake_and_cooldown(
        &mut context,
        &proposal,
        &Pubkey::new_unique(),
        0,
        governance_config,
        0,
        0,
        0,
        0,
        ProposalStatus::Accepted,
        /* voting_start_timestamp */ NonZeroU64::new(clock.unix_timestamp as u64),
        /* voting_start_timestamp */ NonZeroU64::new(clock.unix_timestamp as u64),
    )
    .await;

    let instruction = process_proposal(&proposal);

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

    let governance_config = Config::new(
        /* cooldown_period_seconds */ 100_000_000,
        /* proposal_acceptance_threshold */ 500_000_000, // 50%
        /* proposal_rejection_threshold */ 500_000_000, // 50%
        /* signer_bump_seed */ 0,
        /* stake_config_address */ &Pubkey::new_unique(), // Doesn't matter here.
        /* voting_period_seconds */ 100_000_000,
    );

    let mut context = setup().start_with_context().await;
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();

    setup_proposal_with_stake_and_cooldown(
        &mut context,
        &proposal,
        &Pubkey::new_unique(),
        0,
        governance_config,
        0,
        0,
        0,
        0,
        ProposalStatus::Voting, // Not accepted.
        /* voting_start_timestamp */ NonZeroU64::new(clock.unix_timestamp as u64),
        /* voting_start_timestamp */ NonZeroU64::new(clock.unix_timestamp as u64),
    )
    .await;

    let instruction = process_proposal(&proposal);

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

    let governance_config = Config::new(
        /* cooldown_period_seconds */ 100_000_000,
        /* proposal_acceptance_threshold */ 500_000_000, // 50%
        /* proposal_rejection_threshold */ 500_000_000, // 50%
        /* signer_bump_seed */ 0,
        /* stake_config_address */ &Pubkey::new_unique(), // Doesn't matter here.
        /* voting_period_seconds */ 100_000_000,
    );

    let mut context = setup().start_with_context().await;
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();

    setup_proposal_with_stake_and_cooldown(
        &mut context,
        &proposal,
        &Pubkey::new_unique(),
        0,
        governance_config,
        0,
        0,
        0,
        0,
        ProposalStatus::Accepted,
        /* voting_start_timestamp */ NonZeroU64::new(clock.unix_timestamp as u64),
        /* voting_start_timestamp */ NonZeroU64::new(1),
    )
    .await;

    let instruction = process_proposal(&proposal);

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

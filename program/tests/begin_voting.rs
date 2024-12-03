#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_governance_program::{
        error::PaladinGovernanceError,
        instruction::begin_voting,
        state::{GovernanceConfig, Proposal, ProposalStatus},
    },
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
async fn fail_stake_authority_not_signer() {
    let stake_authority = Keypair::new();
    let proposal = Pubkey::new_unique();

    let context = setup().start_with_context().await;

    let mut instruction = begin_voting(&stake_authority.pubkey(), &proposal);
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
async fn fail_proposal_incorrect_owner() {
    let stake_authority = Keypair::new();
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

    let instruction = begin_voting(&stake_authority.pubkey(), &proposal);

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
async fn fail_proposal_not_initialized() {
    let stake_authority = Keypair::new();
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

    let instruction = begin_voting(&stake_authority.pubkey(), &proposal);

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
async fn fail_stake_authority_not_author() {
    let stake_authority = Keypair::new();
    let proposal = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;
    setup_proposal(
        &mut context,
        &proposal,
        &Pubkey::new_unique(), // Stake authority not author.
        0,
        GovernanceConfig::default(),
        ProposalStatus::Draft,
    )
    .await;

    let instruction = begin_voting(&stake_authority.pubkey(), &proposal);

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
async fn fail_proposal_not_in_draft_stage() {
    let stake_authority = Keypair::new();
    let proposal = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;
    setup_proposal(
        &mut context,
        &proposal,
        &stake_authority.pubkey(),
        0,
        GovernanceConfig::default(),
        ProposalStatus::Voting, // Not in draft stage.
    )
    .await;

    let instruction = begin_voting(&stake_authority.pubkey(), &proposal);

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
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(PaladinGovernanceError::ProposalIsImmutable as u32)
        )
    );
}

#[tokio::test]
async fn success() {
    let stake_authority = Keypair::new();
    let proposal = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;
    setup_proposal(
        &mut context,
        &proposal,
        &stake_authority.pubkey(),
        0,
        GovernanceConfig::default(),
        ProposalStatus::Draft,
    )
    .await;

    let instruction = begin_voting(&stake_authority.pubkey(), &proposal);

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

    // Assert the proposal was marked with voting status.
    let proposal_account = context
        .banks_client
        .get_account(proposal)
        .await
        .unwrap()
        .unwrap();
    let proposal_state = bytemuck::from_bytes::<Proposal>(&proposal_account.data);
    assert_eq!(proposal_state.status, ProposalStatus::Voting);
}

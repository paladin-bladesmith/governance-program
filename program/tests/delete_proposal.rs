#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_governance_program::{
        error::PaladinGovernanceError,
        instruction::delete_proposal,
        state::{Author, GovernanceConfig, Proposal, ProposalStatus},
    },
    setup::{setup, setup_author, setup_proposal},
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

    let mut instruction = delete_proposal(stake_authority.pubkey(), proposal);
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

    let instruction = delete_proposal(stake_authority.pubkey(), proposal);

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

    let instruction = delete_proposal(stake_authority.pubkey(), proposal);

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
    setup_author(&mut context, &stake_authority.pubkey(), 1).await;
    setup_proposal(
        &mut context,
        &proposal,
        &Pubkey::new_unique(), // Stake authority not author.
        0,
        GovernanceConfig::default(),
        ProposalStatus::Draft,
    )
    .await;

    let instruction = delete_proposal(stake_authority.pubkey(), proposal);

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
async fn fail_proposal_immutable() {
    let stake_authority = Keypair::new();
    let proposal = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;
    setup_author(&mut context, &stake_authority.pubkey(), 1).await;
    setup_proposal(
        &mut context,
        &proposal,
        &stake_authority.pubkey(),
        0,
        GovernanceConfig::default(),
        ProposalStatus::Accepted, // Proposal is immutable.
    )
    .await;

    let instruction = delete_proposal(stake_authority.pubkey(), proposal);

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
            InstructionError::Custom(PaladinGovernanceError::ProposalIsActive as u32)
        )
    );
}

#[tokio::test]
async fn success() {
    let stake_authority = Keypair::new();
    let proposal = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;
    setup_author(&mut context, &stake_authority.pubkey(), 1).await;
    setup_proposal(
        &mut context,
        &proposal,
        &stake_authority.pubkey(),
        0,
        GovernanceConfig::default(),
        ProposalStatus::Draft,
    )
    .await;

    // Sanity - Open proposal account is zero.
    let author = paladin_governance_program::state::get_proposal_author_address(
        &stake_authority.pubkey(),
        &paladin_governance_program::ID,
    );
    let author = context
        .banks_client
        .get_account(author)
        .await
        .unwrap()
        .unwrap();
    let author = bytemuck::from_bytes::<Author>(&author.data);
    assert_eq!(author.active_proposals, 1);

    // Act - Execute delete proposal transaction.
    let instruction = delete_proposal(stake_authority.pubkey(), proposal);
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

    // Assert - The proposal was deleted.
    assert!(context
        .banks_client
        .get_account(proposal)
        .await
        .unwrap()
        .is_none());

    // Assert - Open proposal account is zero.
    let author = paladin_governance_program::state::get_proposal_author_address(
        &stake_authority.pubkey(),
        &paladin_governance_program::ID,
    );
    let author = context
        .banks_client
        .get_account(author)
        .await
        .unwrap()
        .unwrap();
    let author = bytemuck::from_bytes::<Author>(&author.data);
    assert_eq!(author.active_proposals, 0);
}

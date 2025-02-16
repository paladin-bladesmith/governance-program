#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_governance_program::{
        error::PaladinGovernanceError,
        instruction::delete_vote,
        state::{Author, GovernanceConfig, ProposalStatus, ProposalVoteElection},
    },
    setup::{setup, setup_author, setup_proposal, setup_proposal_vote},
    solana_program_test::*,
    solana_sdk::{
        instruction::InstructionError,
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
        transaction::{Transaction, TransactionError},
    },
};

#[tokio::test]
async fn fail_proposal_is_active() {
    let stake_authority = Keypair::new();
    let proposal = Pubkey::new_unique();
    let authority = Pubkey::new_unique();
    let vote = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;
    setup_author(&mut context, &stake_authority.pubkey(), 1).await;
    setup_proposal(
        &mut context,
        &proposal,
        &stake_authority.pubkey(),
        0,
        GovernanceConfig::default(),
        ProposalStatus::Voting,
    )
    .await;
    setup_proposal_vote(
        &mut context,
        &vote,
        proposal,
        100,
        authority,
        ProposalVoteElection::Against,
    )
    .await;

    // Act - Execute delete vote transaction.
    let instruction = delete_vote(proposal, vote, authority);
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
            InstructionError::Custom(PaladinGovernanceError::ProposalIsActive as u32)
        )
    );
}

#[tokio::test]
async fn fail_incorrect_proposal() {
    let proposal = Pubkey::new_unique();
    let authority = Pubkey::new_unique();
    let vote = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;
    setup_proposal_vote(
        &mut context,
        &vote,
        proposal,
        100,
        authority,
        ProposalVoteElection::Against,
    )
    .await;

    // Act - Execute delete vote transaction.
    let instruction = delete_vote(Pubkey::new_unique(), vote, authority);
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
            InstructionError::Custom(PaladinGovernanceError::IncorrectProposalAddress as u32)
        )
    );
}

#[tokio::test]
async fn fail_incorrect_authority() {
    let stake_authority = Keypair::new();
    let proposal = Pubkey::new_unique();
    let authority = Pubkey::new_unique();
    let vote = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;
    setup_author(&mut context, &stake_authority.pubkey(), 1).await;
    setup_proposal(
        &mut context,
        &proposal,
        &stake_authority.pubkey(),
        0,
        GovernanceConfig::default(),
        ProposalStatus::Rejected,
    )
    .await;
    setup_proposal_vote(
        &mut context,
        &vote,
        proposal,
        100,
        authority,
        ProposalVoteElection::Against,
    )
    .await;

    // Act - Execute delete vote transaction.
    let instruction = delete_vote(proposal, vote, Pubkey::new_unique());
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
        TransactionError::InstructionError(0, InstructionError::IncorrectAuthority)
    );
}

#[tokio::test]
async fn success_delete_after_proposal_reject() {
    let stake_authority = Keypair::new();
    let proposal = Pubkey::new_unique();
    let authority = Pubkey::new_unique();
    let vote = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;
    setup_author(&mut context, &stake_authority.pubkey(), 1).await;
    setup_proposal(
        &mut context,
        &proposal,
        &stake_authority.pubkey(),
        0,
        GovernanceConfig::default(),
        ProposalStatus::Rejected,
    )
    .await;
    setup_proposal_vote(
        &mut context,
        &vote,
        proposal,
        100,
        authority,
        ProposalVoteElection::Against,
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

    // Check our pre close rent.
    let rent = context
        .banks_client
        .get_account(vote)
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_ne!(rent, 0);

    // Act - Execute delete vote transaction.
    let instruction = delete_vote(proposal, vote, authority);
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

    // Assert - The vote was deleted.
    assert!(context
        .banks_client
        .get_account(vote)
        .await
        .unwrap()
        .is_none());
    assert_eq!(
        context
            .banks_client
            .get_account(authority)
            .await
            .unwrap()
            .unwrap()
            .lamports,
        rent
    );
}

#[tokio::test]
async fn success_delete_after_proposal_delete() {
    let stake_authority = Keypair::new();
    let proposal = Pubkey::new_unique();
    let authority = Pubkey::new_unique();
    let vote = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;
    setup_author(&mut context, &stake_authority.pubkey(), 1).await;
    setup_proposal_vote(
        &mut context,
        &vote,
        proposal,
        100,
        authority,
        ProposalVoteElection::Against,
    )
    .await;

    // Check our pre close rent.
    let rent = context
        .banks_client
        .get_account(vote)
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_ne!(rent, 0);

    // Act - Execute delete vote transaction.
    let instruction = delete_vote(proposal, vote, authority);
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

    // Assert - The vote was deleted.
    assert!(context
        .banks_client
        .get_account(vote)
        .await
        .unwrap()
        .is_none());
    assert_eq!(
        context
            .banks_client
            .get_account(authority)
            .await
            .unwrap()
            .unwrap()
            .lamports,
        rent
    );
}

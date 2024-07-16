#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_governance_program::{
        error::PaladinGovernanceError,
        state::{get_governance_address, get_vote_address, Config, Proposal, ProposalVote},
    },
    paladin_stake_program::state::Stake,
    setup::{setup, setup_governance, setup_proposal, setup_stake, setup_vote},
    solana_program_test::*,
    solana_sdk::{
        account::AccountSharedData,
        instruction::InstructionError,
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
        system_program,
        transaction::{Transaction, TransactionError},
    },
    test_case::test_case,
};

#[tokio::test]
async fn fail_validator_not_signer() {
    let validator = Keypair::new();
    let stake = Pubkey::new_unique(); // TODO!
    let vault = Pubkey::new_unique(); // TODO!
    let proposal = Pubkey::new_unique();

    let proposal_vote = get_vote_address(
        &validator.pubkey(),
        &proposal,
        &paladin_governance_program::id(),
    );
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;

    let mut instruction = paladin_governance_program::instruction::vote(
        &validator.pubkey(),
        &stake,
        &vault,
        &proposal_vote,
        &proposal,
        &governance,
        /* vote */ true,
    );
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
async fn fail_stake_incorrect_owner() {
    let validator = Keypair::new();
    let stake = Pubkey::new_unique(); // TODO!
    let vault = Pubkey::new_unique(); // TODO!
    let proposal = Pubkey::new_unique();

    let proposal_vote = get_vote_address(
        &validator.pubkey(),
        &proposal,
        &paladin_governance_program::id(),
    );
    let governance = get_governance_address(&paladin_governance_program::id());

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

    let instruction = paladin_governance_program::instruction::vote(
        &validator.pubkey(),
        &stake,
        &vault,
        &proposal_vote,
        &proposal,
        &governance,
        /* vote */ true,
    );

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
async fn fail_stake_not_initialized() {
    let validator = Keypair::new();
    let stake = Pubkey::new_unique(); // TODO!
    let vault = Pubkey::new_unique(); // TODO!
    let proposal = Pubkey::new_unique();

    let proposal_vote = get_vote_address(
        &validator.pubkey(),
        &proposal,
        &paladin_governance_program::id(),
    );
    let governance = get_governance_address(&paladin_governance_program::id());

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

    let instruction = paladin_governance_program::instruction::vote(
        &validator.pubkey(),
        &stake,
        &vault,
        &proposal_vote,
        &proposal,
        &governance,
        /* vote */ true,
    );

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
async fn fail_stake_incorrect_validator() {
    let validator = Keypair::new();
    let stake = Pubkey::new_unique(); // TODO!
    let vault = Pubkey::new_unique(); // TODO!
    let proposal = Pubkey::new_unique();

    let proposal_vote = get_vote_address(
        &validator.pubkey(),
        &proposal,
        &paladin_governance_program::id(),
    );
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;

    // Set up a stake account with the wrong validator address.
    setup_stake(
        &mut context,
        &stake,
        /* authority_address */ &Pubkey::new_unique(),
        &Pubkey::new_unique(), // Incorrect validator.
        0,
    )
    .await;

    let instruction = paladin_governance_program::instruction::vote(
        &validator.pubkey(),
        &stake,
        &vault,
        &proposal_vote,
        &proposal,
        &governance,
        /* vote */ true,
    );

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
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(PaladinGovernanceError::ValidatorStakeAccountMismatch as u32)
        )
    );
}

#[tokio::test]
async fn fail_incorrect_vault_account() {
    // TODO!
}

#[tokio::test]
async fn fail_governance_incorrect_address() {
    let validator = Keypair::new();
    let stake = Pubkey::new_unique(); // TODO!
    let vault = Pubkey::new_unique(); // TODO!
    let proposal = Pubkey::new_unique();

    let proposal_vote = get_vote_address(
        &validator.pubkey(),
        &proposal,
        &paladin_governance_program::id(),
    );
    let governance = Pubkey::new_unique(); // Incorrect governance address.

    let mut context = setup().start_with_context().await;
    setup_stake(
        &mut context,
        &stake,
        /* authority_address */ &Pubkey::new_unique(),
        &validator.pubkey(),
        0,
    )
    .await;
    setup_proposal(&mut context, &proposal, &validator.pubkey(), 0, 0).await;

    let instruction = paladin_governance_program::instruction::vote(
        &validator.pubkey(),
        &stake,
        &vault,
        &proposal_vote,
        &proposal,
        &governance,
        /* vote */ true,
    );

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
    let validator = Keypair::new();
    let stake = Pubkey::new_unique(); // TODO!
    let vault = Pubkey::new_unique(); // TODO!
    let proposal = Pubkey::new_unique();

    let proposal_vote = get_vote_address(
        &validator.pubkey(),
        &proposal,
        &paladin_governance_program::id(),
    );
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake(
        &mut context,
        &stake,
        /* authority_address */ &Pubkey::new_unique(),
        &validator.pubkey(),
        0,
    )
    .await;

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

    let instruction = paladin_governance_program::instruction::vote(
        &validator.pubkey(),
        &stake,
        &vault,
        &proposal_vote,
        &proposal,
        &governance,
        /* vote */ true,
    );

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
async fn fail_governance_not_initialized() {
    let validator = Keypair::new();
    let stake = Pubkey::new_unique(); // TODO!
    let vault = Pubkey::new_unique(); // TODO!
    let proposal = Pubkey::new_unique();

    let proposal_vote = get_vote_address(
        &validator.pubkey(),
        &proposal,
        &paladin_governance_program::id(),
    );
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake(
        &mut context,
        &stake,
        /* authority_address */ &Pubkey::new_unique(),
        &validator.pubkey(),
        0,
    )
    .await;

    // Set up the governance account uninitialized.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let lamports = rent.minimum_balance(std::mem::size_of::<Config>());
        context.set_account(
            &governance,
            &AccountSharedData::new(lamports, 0, &paladin_governance_program::id()),
        );
    }

    let instruction = paladin_governance_program::instruction::vote(
        &validator.pubkey(),
        &stake,
        &vault,
        &proposal_vote,
        &proposal,
        &governance,
        /* vote */ true,
    );

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
async fn fail_proposal_incorrect_owner() {
    let validator = Keypair::new();
    let stake = Pubkey::new_unique(); // TODO!
    let vault = Pubkey::new_unique(); // TODO!
    let proposal = Pubkey::new_unique();

    let proposal_vote = get_vote_address(
        &validator.pubkey(),
        &proposal,
        &paladin_governance_program::id(),
    );
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake(
        &mut context,
        &stake,
        /* authority_address */ &Pubkey::new_unique(),
        &validator.pubkey(),
        0,
    )
    .await;
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

    let instruction = paladin_governance_program::instruction::vote(
        &validator.pubkey(),
        &stake,
        &vault,
        &proposal_vote,
        &proposal,
        &governance,
        /* vote */ true,
    );

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
    let stake = Pubkey::new_unique(); // TODO!
    let vault = Pubkey::new_unique(); // TODO!
    let proposal = Pubkey::new_unique();

    let proposal_vote = get_vote_address(
        &validator.pubkey(),
        &proposal,
        &paladin_governance_program::id(),
    );
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake(
        &mut context,
        &stake,
        /* authority_address */ &Pubkey::new_unique(),
        &validator.pubkey(),
        0,
    )
    .await;
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

    let instruction = paladin_governance_program::instruction::vote(
        &validator.pubkey(),
        &stake,
        &vault,
        &proposal_vote,
        &proposal,
        &governance,
        /* vote */ true,
    );

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
async fn fail_vote_incorrect_address() {
    let validator = Keypair::new();
    let stake = Pubkey::new_unique(); // TODO!
    let vault = Pubkey::new_unique(); // TODO!
    let proposal = Pubkey::new_unique();

    let proposal_vote = Pubkey::new_unique(); // Incorrect vote address.
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake(
        &mut context,
        &stake,
        /* authority_address */ &Pubkey::new_unique(),
        &validator.pubkey(),
        0,
    )
    .await;
    setup_governance(&mut context, &governance, 0, 0, 0, 0).await;
    setup_proposal(&mut context, &proposal, &validator.pubkey(), 0, 0).await;

    let instruction = paladin_governance_program::instruction::vote(
        &validator.pubkey(),
        &stake,
        &vault,
        &proposal_vote,
        &proposal,
        &governance,
        /* vote */ true,
    );

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
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(PaladinGovernanceError::IncorrectProposalVoteAddress as u32)
        )
    );
}

#[tokio::test]
async fn fail_vote_already_initialized() {
    let validator = Keypair::new();
    let stake = Pubkey::new_unique(); // TODO!
    let vault = Pubkey::new_unique(); // TODO!
    let proposal = Pubkey::new_unique();

    let proposal_vote = get_vote_address(
        &validator.pubkey(),
        &proposal,
        &paladin_governance_program::id(),
    );
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake(
        &mut context,
        &stake,
        /* authority_address */ &Pubkey::new_unique(),
        &validator.pubkey(),
        0,
    )
    .await;
    setup_governance(&mut context, &governance, 0, 0, 0, 0).await;
    setup_proposal(&mut context, &proposal, &validator.pubkey(), 0, 0).await;

    // Set up an initialized vote account.
    setup_vote(
        &mut context,
        &proposal_vote,
        &proposal,
        0,
        &validator.pubkey(),
        true,
    )
    .await;

    let instruction = paladin_governance_program::instruction::vote(
        &validator.pubkey(),
        &stake,
        &vault,
        &proposal_vote,
        &proposal,
        &governance,
        /* vote */ true,
    );

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

#[test_case(true)]
#[test_case(false)]
#[tokio::test]
async fn success(vote: bool) {
    let validator = Keypair::new();
    let stake = Pubkey::new_unique(); // TODO!
    let vault = Pubkey::new_unique(); // TODO!
    let proposal = Pubkey::new_unique();

    let proposal_vote = get_vote_address(
        &validator.pubkey(),
        &proposal,
        &paladin_governance_program::id(),
    );
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake(
        &mut context,
        &stake,
        /* authority_address */ &Pubkey::new_unique(),
        &validator.pubkey(),
        0,
    )
    .await;
    setup_governance(&mut context, &governance, 0, 0, 0, 0).await;
    setup_proposal(&mut context, &proposal, &validator.pubkey(), 0, 0).await;

    // Fund the vote account.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let lamports = rent.minimum_balance(std::mem::size_of::<ProposalVote>());
        context.set_account(
            &proposal_vote,
            &AccountSharedData::new(lamports, 0, &system_program::id()),
        );
    }

    let instruction = paladin_governance_program::instruction::vote(
        &validator.pubkey(),
        &stake,
        &vault,
        &proposal_vote,
        &proposal,
        &governance,
        vote,
    );

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

    // Assert the vote was created.
    let vote_account = context
        .banks_client
        .get_account(proposal_vote)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        bytemuck::from_bytes::<ProposalVote>(&vote_account.data),
        &ProposalVote::new(
            &proposal,
            /* stake */ 0, // TODO!
            &validator.pubkey(),
            vote,
        )
    );

    // Assert the vote count was updated in the proposal.
    // TODO: All stake is zero right now...
    // let proposal_account = context
    //     .banks_client
    //     .get_account(proposal)
    //     .await
    //     .unwrap()
    //     .unwrap();
    // let proposal_state =
    // bytemuck::from_bytes::<Proposal>(&proposal_account.data);
}

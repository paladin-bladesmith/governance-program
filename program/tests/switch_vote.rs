#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_governance_program::{
        error::PaladinGovernanceError,
        state::{
            get_governance_address, get_proposal_vote_address, Config, Proposal, ProposalStatus,
            ProposalVote, ProposalVoteElection,
        },
    },
    paladin_stake_program::state::{find_stake_pda, Config as StakeConfig, Stake},
    setup::{
        setup, setup_governance, setup_proposal, setup_proposal_vote, setup_proposal_with_stake,
        setup_proposal_with_stake_and_cooldown, setup_stake, setup_stake_config,
    },
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
    std::num::NonZeroU64,
    test_case::test_case,
};

#[tokio::test]
async fn fail_stake_authority_not_signer() {
    let stake_authority = Keypair::new();
    let validator_vote = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
    let governance = get_governance_address(&stake_config, &paladin_governance_program::id());

    let mut context = setup().start_with_context().await;

    let mut instruction = paladin_governance_program::instruction::switch_vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        &governance,
        ProposalVoteElection::For,
    );
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
    let validator_vote = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
    let governance = get_governance_address(&stake_config, &paladin_governance_program::id());

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

    let instruction = paladin_governance_program::instruction::switch_vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        &governance,
        ProposalVoteElection::For,
    );

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
    let validator_vote = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
    let governance = get_governance_address(&stake_config, &paladin_governance_program::id());

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

    let instruction = paladin_governance_program::instruction::switch_vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        &governance,
        ProposalVoteElection::For,
    );

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
    let validator_vote = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
    let governance = get_governance_address(&stake_config, &paladin_governance_program::id());

    let mut context = setup().start_with_context().await;

    // Set up a stake account with the wrong stake_authority address.
    setup_stake(
        &mut context,
        &stake,
        &Pubkey::new_unique(), // Incorrect stake_authority.
        &validator_vote,
        0,
    )
    .await;

    let instruction = paladin_governance_program::instruction::switch_vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        &governance,
        ProposalVoteElection::For,
    );

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
async fn fail_stake_config_incorrect_owner() {
    let stake_authority = Keypair::new();
    let validator_vote = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
    let governance = get_governance_address(&stake_config, &paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake(
        &mut context,
        &stake,
        &stake_authority.pubkey(),
        &validator_vote,
        0,
    )
    .await;
    setup_proposal(
        &mut context,
        &proposal,
        &stake_authority.pubkey(),
        0,
        0,
        ProposalStatus::Voting,
    )
    .await;

    // Set up a stake config account with an incorrect owner.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let space = std::mem::size_of::<StakeConfig>();
        let lamports = rent.minimum_balance(space);
        context.set_account(
            &stake_config,
            &AccountSharedData::new(lamports, space, &Pubkey::new_unique()), // Incorrect owner.
        );
    }

    let instruction = paladin_governance_program::instruction::switch_vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        &governance,
        ProposalVoteElection::For,
    );

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
async fn fail_stake_config_not_initialized() {
    let stake_authority = Keypair::new();
    let validator_vote = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
    let governance = get_governance_address(&stake_config, &paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake(
        &mut context,
        &stake,
        &stake_authority.pubkey(),
        &validator_vote,
        0,
    )
    .await;
    setup_proposal(
        &mut context,
        &proposal,
        &stake_authority.pubkey(),
        0,
        0,
        ProposalStatus::Voting,
    )
    .await;

    // Set up an uninitialized stake config account.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let space = std::mem::size_of::<StakeConfig>();
        let lamports = rent.minimum_balance(space);
        context.set_account(
            &stake_config,
            &AccountSharedData::new(lamports, space, &paladin_stake_program::id()),
        );
    }

    let instruction = paladin_governance_program::instruction::switch_vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        &governance,
        ProposalVoteElection::For,
    );

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
async fn fail_governance_incorrect_address() {
    let stake_authority = Keypair::new();
    let validator_vote = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
    let governance = Pubkey::new_unique(); // Incorrect governance address.

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, 0).await;
    setup_stake(
        &mut context,
        &stake,
        &stake_authority.pubkey(),
        &validator_vote,
        0,
    )
    .await;
    setup_proposal(
        &mut context,
        &proposal,
        &stake_authority.pubkey(),
        0,
        0,
        ProposalStatus::Voting,
    )
    .await;

    let instruction = paladin_governance_program::instruction::switch_vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        &governance,
        ProposalVoteElection::For,
    );

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
            InstructionError::Custom(
                PaladinGovernanceError::IncorrectGovernanceConfigAddress as u32
            )
        )
    );
}

#[tokio::test]
async fn fail_governance_incorrect_owner() {
    let stake_authority = Keypair::new();
    let validator_vote = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
    let governance = get_governance_address(&stake_config, &paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, 0).await;
    setup_stake(
        &mut context,
        &stake,
        &stake_authority.pubkey(),
        &validator_vote,
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

    let instruction = paladin_governance_program::instruction::switch_vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        &governance,
        ProposalVoteElection::For,
    );

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
async fn fail_governance_not_initialized() {
    let stake_authority = Keypair::new();
    let validator_vote = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
    let governance = get_governance_address(&stake_config, &paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, 0).await;
    setup_stake(
        &mut context,
        &stake,
        &stake_authority.pubkey(),
        &validator_vote,
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

    let instruction = paladin_governance_program::instruction::switch_vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        &governance,
        ProposalVoteElection::For,
    );

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
async fn fail_proposal_incorrect_owner() {
    let stake_authority = Keypair::new();
    let validator_vote = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
    let governance = get_governance_address(&stake_config, &paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, 0).await;
    setup_stake(
        &mut context,
        &stake,
        &stake_authority.pubkey(),
        &validator_vote,
        0,
    )
    .await;
    setup_governance(&mut context, &governance, 0, 0, 0, &stake_config, 0).await;

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

    let instruction = paladin_governance_program::instruction::switch_vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        &governance,
        ProposalVoteElection::For,
    );

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
    let validator_vote = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
    let governance = get_governance_address(&stake_config, &paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, 0).await;
    setup_stake(
        &mut context,
        &stake,
        &stake_authority.pubkey(),
        &validator_vote,
        0,
    )
    .await;
    setup_governance(&mut context, &governance, 0, 0, 0, &stake_config, 0).await;

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

    let instruction = paladin_governance_program::instruction::switch_vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        &governance,
        ProposalVoteElection::For,
    );

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
async fn fail_proposal_not_voting() {
    let stake_authority = Keypair::new();
    let validator_vote = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
    let governance = get_governance_address(&stake_config, &paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, 0).await;
    setup_stake(
        &mut context,
        &stake,
        &stake_authority.pubkey(),
        &validator_vote,
        0,
    )
    .await;
    setup_governance(&mut context, &governance, 0, 0, 0, &stake_config, 0).await;
    setup_proposal(
        &mut context,
        &proposal,
        &stake_authority.pubkey(),
        0,
        0,
        ProposalStatus::Draft, // Not in voting stage.
    )
    .await;

    let instruction = paladin_governance_program::instruction::switch_vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        &governance,
        ProposalVoteElection::For,
    );

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
            InstructionError::Custom(PaladinGovernanceError::ProposalNotInVotingStage as u32)
        )
    );
}

#[tokio::test]
async fn fail_proposal_vote_incorrect_address() {
    let stake_authority = Keypair::new();
    let validator_vote = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote = Pubkey::new_unique(); // Incorrect proposal vote address.
    let governance = get_governance_address(&stake_config, &paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, 0).await;
    setup_stake(
        &mut context,
        &stake,
        &stake_authority.pubkey(),
        &validator_vote,
        0,
    )
    .await;
    setup_governance(&mut context, &governance, 0, 0, 0, &stake_config, 0).await;
    setup_proposal(
        &mut context,
        &proposal,
        &stake_authority.pubkey(),
        0,
        0,
        ProposalStatus::Voting,
    )
    .await;

    let instruction = paladin_governance_program::instruction::switch_vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        &governance,
        ProposalVoteElection::For,
    );

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
            InstructionError::Custom(PaladinGovernanceError::IncorrectProposalVoteAddress as u32)
        )
    );
}

#[tokio::test]
async fn fail_proposal_vote_not_initialized() {
    let stake_authority = Keypair::new();
    let validator_vote = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
    let governance = get_governance_address(&stake_config, &paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, 0).await;
    setup_stake(
        &mut context,
        &stake,
        &stake_authority.pubkey(),
        &validator_vote,
        0,
    )
    .await;
    setup_governance(&mut context, &governance, 0, 0, 0, &stake_config, 0).await;
    setup_proposal(
        &mut context,
        &proposal,
        &stake_authority.pubkey(),
        0,
        0,
        ProposalStatus::Voting,
    )
    .await;

    // Set up an uninitialized proposal vote account.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let lamports = rent.minimum_balance(std::mem::size_of::<ProposalVote>());
        context.set_account(
            &proposal_vote,
            &AccountSharedData::new(lamports, 0, &paladin_governance_program::id()),
        );
    }

    let instruction = paladin_governance_program::instruction::switch_vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        &governance,
        ProposalVoteElection::For,
    );

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

const ACCEPTANCE_THRESHOLD: u32 = 500_000_000; // 50%
const REJECTION_THRESHOLD: u32 = 500_000_000; // 50%
const VOTING_PERIOD_SECONDS: u64 = 100_000_000;
const TOTAL_STAKE: u64 = 100_000_000;

struct ProposalStarting {
    cooldown_active: bool,
    stake_for: u64,
    stake_against: u64,
    stake_abstained: u64,
}
struct VoteSwitch {
    previous_vote_stake: u64,
    new_vote_stake: u64,
    previous_election: ProposalVoteElection,
    new_election: ProposalVoteElection,
}
enum Expect {
    Cast {
        cooldown: bool,
        stake_for: u64,
        stake_against: u64,
        stake_abstained: u64,
    },
    Terminated,
}

// TODO: Until we decide whether or not to keep the cooldown running, or to
// reset it when acceptance dips below the threshold, the test case is missing
// for determining, not just that a cooldown exists post-vost-switch, but that
// it was in fact reset with a new timestamp.

#[test_case(
    ProposalStarting {
        cooldown_active: false,
        stake_for: TOTAL_STAKE / 4, // 25% of total stake.
        stake_against: TOTAL_STAKE / 4, // 25% of total stake.
        stake_abstained: TOTAL_STAKE / 4, // 25% of total stake.
    },
    VoteSwitch {
        previous_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        new_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        previous_election: ProposalVoteElection::DidNotVote,
        new_election: ProposalVoteElection::DidNotVote,
    },
    Expect::Cast {
        cooldown: false,
        stake_for: TOTAL_STAKE / 4,
        stake_against: TOTAL_STAKE / 4,
        stake_abstained: TOTAL_STAKE / 4,
    };
    "dnv_to_dnv_same_stake_does_nothing"
)]
#[test_case(
    ProposalStarting {
        cooldown_active: false,
        stake_for: TOTAL_STAKE / 4, // 25% of total stake.
        stake_against: TOTAL_STAKE / 4, // 25% of total stake.
        stake_abstained: TOTAL_STAKE / 4, // 25% of total stake.
    },
    VoteSwitch {
        previous_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        new_vote_stake: TOTAL_STAKE / 20, // 5% of total stake.
        previous_election: ProposalVoteElection::DidNotVote,
        new_election: ProposalVoteElection::DidNotVote,
    },
    Expect::Cast {
        cooldown: false,
        stake_for: TOTAL_STAKE / 4,
        stake_against: TOTAL_STAKE / 4,
        stake_abstained: TOTAL_STAKE / 4 - TOTAL_STAKE / 20, // 20% of total stake.
    };
    "dnv_to_dnv_less_stake_decrements_stake_abstained"
)]
#[test_case(
    ProposalStarting {
        cooldown_active: false,
        stake_for: TOTAL_STAKE / 4, // 25% of total stake.
        stake_against: TOTAL_STAKE / 4, // 25% of total stake.
        stake_abstained: TOTAL_STAKE / 4, // 25% of total stake.
    },
    VoteSwitch {
        previous_vote_stake: TOTAL_STAKE / 20, // 5% of total stake.
        new_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        previous_election: ProposalVoteElection::DidNotVote,
        new_election: ProposalVoteElection::DidNotVote,
    },
    Expect::Cast {
        cooldown: false,
        stake_for: TOTAL_STAKE / 4,
        stake_against: TOTAL_STAKE / 4,
        stake_abstained: TOTAL_STAKE / 4 + TOTAL_STAKE / 20, // 30% of total stake.
    };
    "dnv_to_dnv_more_stake_increments_stake_abstained"
)]
#[test_case(
    ProposalStarting {
        cooldown_active: false,
        stake_for: TOTAL_STAKE / 4, // 25% of total stake.
        stake_against: TOTAL_STAKE / 4, // 25% of total stake.
        stake_abstained: TOTAL_STAKE / 4, // 25% of total stake.
    },
    VoteSwitch {
        previous_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        new_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        previous_election: ProposalVoteElection::DidNotVote,
        new_election: ProposalVoteElection::For,
    },
    Expect::Cast {
        cooldown: false,
        stake_for: TOTAL_STAKE / 4 + TOTAL_STAKE / 10, // 35% of total stake.
        stake_against: TOTAL_STAKE / 4, // Unchanged.
        stake_abstained: TOTAL_STAKE / 4 - TOTAL_STAKE / 10, // 15% of total stake.
    };
    "dnv_to_for_decrements_stake_abstained_increments_stake_for"
)]
#[test_case(
    ProposalStarting {
        cooldown_active: false,
        stake_for: TOTAL_STAKE / 5 * 2, // 40% of total stake.
        stake_against: TOTAL_STAKE / 4, // 25% of total stake.
        stake_abstained: TOTAL_STAKE / 4, // 25% of total stake.
    },
    VoteSwitch {
        previous_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        new_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        previous_election: ProposalVoteElection::DidNotVote,
        new_election: ProposalVoteElection::For,
    },
    Expect::Cast {
        cooldown: true, // Cooldown activated.
        stake_for: TOTAL_STAKE / 5 * 2 + TOTAL_STAKE / 10, // 50% of total stake.
        stake_against: TOTAL_STAKE / 4, // Unchanged.
        stake_abstained: TOTAL_STAKE / 4 - TOTAL_STAKE / 10, // 15% of total stake.
    };
    "dnv_to_for_beyond_treshold_decrements_stake_abstained_increments_stake_for_and_activates_cooldown"
)]
#[test_case(
    ProposalStarting {
        cooldown_active: false,
        stake_for: TOTAL_STAKE / 4, // 25% of total stake.
        stake_against: TOTAL_STAKE / 4, // 25% of total stake.
        stake_abstained: TOTAL_STAKE / 4, // 25% of total stake.
    },
    VoteSwitch {
        previous_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        new_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        previous_election: ProposalVoteElection::DidNotVote,
        new_election: ProposalVoteElection::Against,
    },
    Expect::Cast {
        cooldown: false,
        stake_for: TOTAL_STAKE / 4, // Unchanged.
        stake_against: TOTAL_STAKE / 4 + TOTAL_STAKE / 10, // 35% of total stake.
        stake_abstained: TOTAL_STAKE / 4 - TOTAL_STAKE / 10, // 15% of total stake.
    };
    "dnv_to_against_decrements_stake_abstained_increments_stake_against"
)]
#[test_case(
    ProposalStarting {
        cooldown_active: false,
        stake_for: TOTAL_STAKE / 4, // 25% of total stake.
        stake_against: TOTAL_STAKE / 5 * 2, // 40% of total stake.
        stake_abstained: TOTAL_STAKE / 4, // 25% of total stake.
    },
    VoteSwitch {
        previous_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        new_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        previous_election: ProposalVoteElection::DidNotVote,
        new_election: ProposalVoteElection::Against,
    },
    Expect::Terminated;
    "dnv_to_against_beyond_threshold_terminates"
)]
#[test_case(
    ProposalStarting {
        cooldown_active: false,
        stake_for: TOTAL_STAKE / 4, // 25% of total stake.
        stake_against: TOTAL_STAKE / 4, // 25% of total stake.
        stake_abstained: TOTAL_STAKE / 4, // 25% of total stake.
    },
    VoteSwitch {
        previous_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        new_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        previous_election: ProposalVoteElection::For,
        new_election: ProposalVoteElection::For,
    },
    Expect::Cast {
        cooldown: false,
        stake_for: TOTAL_STAKE / 4,
        stake_against: TOTAL_STAKE / 4,
        stake_abstained: TOTAL_STAKE / 4, // Unchanged.
    };
    "for_to_for_same_stake_does_nothing"
)]
#[test_case(
    ProposalStarting {
        cooldown_active: false,
        stake_for: TOTAL_STAKE / 4, // 25% of total stake.
        stake_against: TOTAL_STAKE / 4, // 25% of total stake.
        stake_abstained: TOTAL_STAKE / 4, // 25% of total stake.
    },
    VoteSwitch {
        previous_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        new_vote_stake: TOTAL_STAKE / 20, // 5% of total stake.
        previous_election: ProposalVoteElection::For,
        new_election: ProposalVoteElection::For,
    },
    Expect::Cast {
        cooldown: false,
        stake_for: TOTAL_STAKE / 4 - TOTAL_STAKE / 10 + TOTAL_STAKE / 20, // 20% of total stake.
        stake_against: TOTAL_STAKE / 4,
        stake_abstained: TOTAL_STAKE / 4, // Unchanged.
    };
    "for_to_for_less_stake_decrements_stake_for"
)]
#[test_case(
    ProposalStarting {
        cooldown_active: false,
        stake_for: TOTAL_STAKE / 4, // 25% of total stake.
        stake_against: TOTAL_STAKE / 4, // 25% of total stake.
        stake_abstained: TOTAL_STAKE / 4, // 25% of total stake.
    },
    VoteSwitch {
        previous_vote_stake: TOTAL_STAKE / 20, // 5% of total stake.
        new_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        previous_election: ProposalVoteElection::For,
        new_election: ProposalVoteElection::For,
    },
    Expect::Cast {
        cooldown: false,
        stake_for: TOTAL_STAKE / 4 - TOTAL_STAKE / 20 + TOTAL_STAKE / 10, // 30% of total stake.
        stake_against: TOTAL_STAKE / 4,
        stake_abstained: TOTAL_STAKE / 4, // Unchanged.
    };
    "for_to_for_more_stake_increments_stake_for"
)]
#[test_case(
    ProposalStarting {
        cooldown_active: false,
        stake_for: TOTAL_STAKE / 4, // 25% of total stake.
        stake_against: TOTAL_STAKE / 4, // 25% of total stake.
        stake_abstained: TOTAL_STAKE / 4, // 25% of total stake.
    },
    VoteSwitch {
        previous_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        new_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        previous_election: ProposalVoteElection::For,
        new_election: ProposalVoteElection::Against,
    },
    Expect::Cast {
        cooldown: false,
        stake_for: TOTAL_STAKE / 4 - TOTAL_STAKE / 10, // 15% of total stake.
        stake_against: TOTAL_STAKE / 4 + TOTAL_STAKE / 10, // 35% of total stake.
        stake_abstained: TOTAL_STAKE / 4, // Unchanged.
    };
    "for_to_against_deducts_stake_for_increments_stake_against"
)]
#[test_case(
    ProposalStarting {
        cooldown_active: true, // Cooldown active.
        stake_for: TOTAL_STAKE / 2, // 50% of total stake.
        stake_against: TOTAL_STAKE / 4, // 25% of total stake.
        stake_abstained: TOTAL_STAKE / 4, // 25% of total stake.
    },
    VoteSwitch {
        previous_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        new_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        previous_election: ProposalVoteElection::For,
        new_election: ProposalVoteElection::Against,
    },
    Expect::Cast {
        cooldown: false, // Cooldown reset.
        stake_for: TOTAL_STAKE / 2 - TOTAL_STAKE / 10, // 40% of total stake.
        stake_against: TOTAL_STAKE / 4 + TOTAL_STAKE / 10, // 35% of total stake.
        stake_abstained: TOTAL_STAKE / 4, // Unchanged.
    };
    "for_to_against_below_for_threshold_deducts_stake_for_increments_stake_against_rests_cooldown"
)]
#[test_case(
    ProposalStarting {
        cooldown_active: false,
        stake_for: TOTAL_STAKE / 4, // 25% of total stake.
        stake_against: TOTAL_STAKE / 5 * 2, // 40% of total stake.
        stake_abstained: TOTAL_STAKE / 4, // 25% of total stake.
    },
    VoteSwitch {
        previous_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        new_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        previous_election: ProposalVoteElection::For,
        new_election: ProposalVoteElection::Against,
    },
    Expect::Terminated;
    "for_to_against_beyond_threshold_terminates"
)]
#[test_case(
    ProposalStarting {
        cooldown_active: false,
        stake_for: TOTAL_STAKE / 4, // 25% of total stake.
        stake_against: TOTAL_STAKE / 4, // 25% of total stake.
        stake_abstained: TOTAL_STAKE / 4, // 25% of total stake.
    },
    VoteSwitch {
        previous_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        new_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        previous_election: ProposalVoteElection::For,
        new_election: ProposalVoteElection::DidNotVote,
    },
    Expect::Cast {
        cooldown: false,
        stake_for: TOTAL_STAKE / 4 - TOTAL_STAKE / 10, // 15% of total stake.
        stake_against: TOTAL_STAKE / 4, // Unchanged.
        stake_abstained: TOTAL_STAKE / 4 + TOTAL_STAKE / 10, // 35% of total stake.
    };
    "for_to_dnv_deducts_stake_for_increments_stake_abstained"
)]
#[test_case(
    ProposalStarting {
        cooldown_active: false,
        stake_for: TOTAL_STAKE / 4, // 25% of total stake.
        stake_against: TOTAL_STAKE / 4, // 25% of total stake.
        stake_abstained: TOTAL_STAKE / 4, // 25% of total stake.
    },
    VoteSwitch {
        previous_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        new_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        previous_election: ProposalVoteElection::Against,
        new_election: ProposalVoteElection::Against,
    },
    Expect::Cast {
        cooldown: false,
        stake_for: TOTAL_STAKE / 4,
        stake_against: TOTAL_STAKE / 4,
        stake_abstained: TOTAL_STAKE / 4,
    };
    "against_to_against_same_stake_does_nothing"
)]
#[test_case(
    ProposalStarting {
        cooldown_active: false,
        stake_for: TOTAL_STAKE / 4, // 25% of total stake.
        stake_against: TOTAL_STAKE / 4, // 25% of total stake.
        stake_abstained: TOTAL_STAKE / 4, // 25% of total stake.
    },
    VoteSwitch {
        previous_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        new_vote_stake: TOTAL_STAKE / 20, // 5% of total stake.
        previous_election: ProposalVoteElection::Against,
        new_election: ProposalVoteElection::Against,
    },
    Expect::Cast {
        cooldown: false,
        stake_for: TOTAL_STAKE / 4,
        stake_against: TOTAL_STAKE / 4 - TOTAL_STAKE / 10 + TOTAL_STAKE / 20, // 20% of total stake.
        stake_abstained: TOTAL_STAKE / 4, // Unchanged.
    };
    "against_to_against_less_stake_decrements_stake_against"
)]
#[test_case(
    ProposalStarting {
        cooldown_active: false,
        stake_for: TOTAL_STAKE / 4, // 25% of total stake.
        stake_against: TOTAL_STAKE / 4, // 25% of total stake.
        stake_abstained: TOTAL_STAKE / 4, // 25% of total stake.
    },
    VoteSwitch {
        previous_vote_stake: TOTAL_STAKE / 20, // 5% of total stake.
        new_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        previous_election: ProposalVoteElection::Against,
        new_election: ProposalVoteElection::Against,
    },
    Expect::Cast {
        cooldown: false,
        stake_for: TOTAL_STAKE / 4,
        stake_against: TOTAL_STAKE / 4 - TOTAL_STAKE / 20 + TOTAL_STAKE / 10, // 30% of total stake.
        stake_abstained: TOTAL_STAKE / 4, // Unchanged.
    };
    "against_to_against_more_stake_increments_stake_against"
)]
#[test_case(
    ProposalStarting {
        cooldown_active: false,
        stake_for: TOTAL_STAKE / 4, // 25% of total stake.
        stake_against: TOTAL_STAKE / 4, // 25% of total stake.
        stake_abstained: TOTAL_STAKE / 4, // 25% of total stake.
    },
    VoteSwitch {
        previous_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        new_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        previous_election: ProposalVoteElection::Against,
        new_election: ProposalVoteElection::For,
    },
    Expect::Cast {
        cooldown: false,
        stake_for: TOTAL_STAKE / 4 + TOTAL_STAKE / 10, // 35% of total stake.
        stake_against: TOTAL_STAKE / 4 - TOTAL_STAKE / 10, // 15% of total stake.
        stake_abstained: TOTAL_STAKE / 4, // Unchanged.
    };
    "against_to_for_deducts_stake_against_increments_stake_for"
)]
#[test_case(
    ProposalStarting {
        cooldown_active: false,
        stake_for: TOTAL_STAKE / 5 * 2, // 40% of total stake.
        stake_against: TOTAL_STAKE / 4, // 25% of total stake.
        stake_abstained: TOTAL_STAKE / 4, // 25% of total stake.
    },
    VoteSwitch {
        previous_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        new_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        previous_election: ProposalVoteElection::Against,
        new_election: ProposalVoteElection::For,
    },
    Expect::Cast {
        cooldown: true, // Cooldown activated.
        stake_for: TOTAL_STAKE / 5 * 2 + TOTAL_STAKE / 10, // 50% of total stake.
        stake_against: TOTAL_STAKE / 4 - TOTAL_STAKE / 10, // 15% of total stake.
        stake_abstained: TOTAL_STAKE / 4, // Unchanged.
    };
    "against_to_for_beyond_threshold_deducts_stake_against_increments_stake_for_activates_cooldown"
)]
#[test_case(
    ProposalStarting {
        cooldown_active: false,
        stake_for: TOTAL_STAKE / 4, // 25% of total stake.
        stake_against: TOTAL_STAKE / 4, // 25% of total stake.
        stake_abstained: TOTAL_STAKE / 4, // 25% of total stake.
    },
    VoteSwitch {
        previous_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        new_vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        previous_election: ProposalVoteElection::Against,
        new_election: ProposalVoteElection::DidNotVote,
    },
    Expect::Cast {
        cooldown: false,
        stake_for: TOTAL_STAKE / 4, // Unchanged.
        stake_against: TOTAL_STAKE / 4 - TOTAL_STAKE / 10, // 15% of total stake.
        stake_abstained: TOTAL_STAKE / 4 + TOTAL_STAKE / 10, // 35% of total stake.
    };
    "against_to_dnv_deducts_stake_against_increments_stake_abstained"
)]
#[tokio::test]
async fn success(proposal_starting: ProposalStarting, switch: VoteSwitch, expect: Expect) {
    let stake_authority = Keypair::new();
    let validator_vote = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
    let governance = get_governance_address(&stake_config, &paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();

    setup_stake_config(&mut context, &stake_config, TOTAL_STAKE).await;
    setup_stake(
        &mut context,
        &stake,
        &stake_authority.pubkey(),
        &validator_vote,
        switch.new_vote_stake,
    )
    .await;
    setup_governance(
        &mut context,
        &governance,
        /* cooldown_period_seconds */ 100_000,
        ACCEPTANCE_THRESHOLD,
        REJECTION_THRESHOLD,
        &stake_config,
        VOTING_PERIOD_SECONDS,
    )
    .await;

    // Set up the proposal vote account with the _inverse_ vote.
    setup_proposal_vote(
        &mut context,
        &proposal_vote,
        &proposal,
        switch.previous_vote_stake,
        &stake_authority.pubkey(),
        switch.previous_election,
    )
    .await;

    // Set up the proposal with the previous vote counted.
    if proposal_starting.cooldown_active {
        setup_proposal_with_stake_and_cooldown(
            &mut context,
            &proposal,
            &stake_authority.pubkey(),
            0,
            0,
            proposal_starting.stake_for,
            proposal_starting.stake_against,
            proposal_starting.stake_abstained,
            ProposalStatus::Voting,
            /* voting_start_timestamp */ NonZeroU64::new(clock.unix_timestamp as u64),
            /* cooldown_timestamp */
            NonZeroU64::new(clock.unix_timestamp.saturating_sub(100) as u64),
        )
        .await;
    } else {
        setup_proposal_with_stake(
            &mut context,
            &proposal,
            &stake_authority.pubkey(),
            0,
            0,
            proposal_starting.stake_for,
            proposal_starting.stake_against,
            proposal_starting.stake_abstained,
            ProposalStatus::Voting,
            /* voting_start_timestamp */ NonZeroU64::new(clock.unix_timestamp as u64),
        )
        .await;
    }

    let instruction = paladin_governance_program::instruction::switch_vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        &governance,
        switch.new_election,
    );

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

    // Assert the vote was updated.
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
            switch.new_vote_stake,
            &stake_authority.pubkey(),
            switch.new_election
        )
    );

    let proposal_account = context
        .banks_client
        .get_account(proposal)
        .await
        .unwrap()
        .unwrap();
    let proposal_state = bytemuck::from_bytes::<Proposal>(&proposal_account.data);

    match expect {
        Expect::Cast {
            cooldown,
            stake_for,
            stake_against,
            stake_abstained,
        } => {
            // Assert the proposal stake matches the expected values.
            assert_eq!(proposal_state.stake_for, stake_for);
            assert_eq!(proposal_state.stake_against, stake_against);
            assert_eq!(proposal_state.stake_abstained, stake_abstained);

            if cooldown {
                // Assert the cooldown time is set.
                assert!(proposal_state.cooldown_timestamp.is_some());
            } else {
                // Assert the cooldown time is not set.
                assert!(proposal_state.cooldown_timestamp.is_none());
            }
        }
        Expect::Terminated => {
            // Assert the proposal was rejected.
            assert_eq!(proposal_state.status, ProposalStatus::Rejected);
        }
    }
}

#[test_case(true)]
#[test_case(false)]
#[tokio::test]
async fn success_voting_has_ended(was_accepted: bool) {
    let stake_authority = Keypair::new();
    let validator_vote = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
    let governance = get_governance_address(&stake_config, &paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();

    setup_stake_config(&mut context, &stake_config, TOTAL_STAKE).await;
    setup_stake(
        &mut context,
        &stake,
        &stake_authority.pubkey(),
        &validator_vote,
        /* vote_stake */ 100_000,
    )
    .await;
    setup_governance(
        &mut context,
        &governance,
        /* cooldown_period_seconds */ 1_000, // Unused here.
        ACCEPTANCE_THRESHOLD,
        REJECTION_THRESHOLD,
        &stake_config,
        /* voting_period_seconds */ 10,
    )
    .await;

    if was_accepted {
        // Set up a proposal with a cooldown timestamp and stake for above
        // threshold.
        setup_proposal_with_stake_and_cooldown(
            &mut context,
            &proposal,
            &stake_authority.pubkey(),
            /* creation_timestamp */ 0,
            /* instruction */ 0,
            /* stake_for */ TOTAL_STAKE,
            /* stake_against */ 0,
            /* stake_abstained */ 0,
            ProposalStatus::Voting,
            /* voting_start_timestamp */
            NonZeroU64::new(clock.unix_timestamp.saturating_sub(10) as u64), // Now - 10 seconds.
            /* cooldown_timestamp */ NonZeroU64::new(clock.unix_timestamp as u64),
        )
        .await;
    } else {
        // Set up a proposal without a cooldown timestamp and stake against
        // above threshold.
        setup_proposal_with_stake(
            &mut context,
            &proposal,
            &stake_authority.pubkey(),
            /* creation_timestamp */ 0,
            /* instruction */ 0,
            /* stake_for */ 0,
            /* stake_against */ TOTAL_STAKE,
            /* stake_abstained */ 0,
            ProposalStatus::Voting,
            /* voting_start_timestamp */
            NonZeroU64::new(clock.unix_timestamp.saturating_sub(10) as u64), // Now - 10 seconds.
        )
        .await;
    }

    let instruction = paladin_governance_program::instruction::switch_vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        &governance,
        ProposalVoteElection::For,
    );

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

    // Assert the proposal vote was _not_ created.
    assert!(context
        .banks_client
        .get_account(proposal_vote)
        .await
        .unwrap()
        .is_none());

    let proposal_account = context
        .banks_client
        .get_account(proposal)
        .await
        .unwrap()
        .unwrap();
    let proposal_state = bytemuck::from_bytes::<Proposal>(&proposal_account.data);

    // Assert there is no cooldown timestamp.
    assert!(proposal_state.cooldown_timestamp.is_none());

    if was_accepted {
        // Assert the proposal was accepted.
        assert_eq!(proposal_state.status, ProposalStatus::Accepted);
    } else {
        // Assert the proposal was rejected.
        assert_eq!(proposal_state.status, ProposalStatus::Rejected);
    }
}

#[tokio::test]
async fn success_cooldown_has_ended() {
    let stake_authority = Keypair::new();
    let validator_vote = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
    let governance = get_governance_address(&stake_config, &paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();

    setup_stake_config(&mut context, &stake_config, TOTAL_STAKE).await;
    setup_stake(
        &mut context,
        &stake,
        &stake_authority.pubkey(),
        &validator_vote,
        /* vote_stake */ 100_000,
    )
    .await;
    setup_governance(
        &mut context,
        &governance,
        /* cooldown_period_seconds */ 10,
        ACCEPTANCE_THRESHOLD,
        REJECTION_THRESHOLD,
        &stake_config,
        /* voting_period_seconds */ 1_000, // Unused here.
    )
    .await;

    // Set up a proposal with a cooldown timestamp and stake for above
    // threshold.
    setup_proposal_with_stake_and_cooldown(
        &mut context,
        &proposal,
        &stake_authority.pubkey(),
        /* creation_timestamp */ 0,
        /* instruction */ 0,
        /* stake_for */ TOTAL_STAKE,
        /* stake_against */ 0,
        /* stake_abstained */ 0,
        ProposalStatus::Voting,
        /* voting_start_timestamp */
        NonZeroU64::new(clock.unix_timestamp as u64),
        /* cooldown_timestamp */
        NonZeroU64::new(clock.unix_timestamp.saturating_sub(10) as u64), // Now - 10 seconds.
    )
    .await;

    let instruction = paladin_governance_program::instruction::switch_vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        &governance,
        ProposalVoteElection::For,
    );

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

    // Assert the proposal vote was _not_ created.
    assert!(context
        .banks_client
        .get_account(proposal_vote)
        .await
        .unwrap()
        .is_none());

    let proposal_account = context
        .banks_client
        .get_account(proposal)
        .await
        .unwrap()
        .unwrap();
    let proposal_state = bytemuck::from_bytes::<Proposal>(&proposal_account.data);

    // Assert there is no cooldown timestamp.
    assert!(proposal_state.cooldown_timestamp.is_none());

    // Assert the proposal was accepted.
    assert_eq!(proposal_state.status, ProposalStatus::Accepted);
}

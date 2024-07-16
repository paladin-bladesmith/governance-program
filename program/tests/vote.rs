#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_governance_program::{
        error::PaladinGovernanceError,
        state::{get_governance_address, get_vote_address, Config, Proposal, ProposalVote},
    },
    paladin_stake_program::state::{find_stake_pda, Config as StakeConfig, Stake},
    setup::{setup, setup_governance, setup_proposal, setup_stake, setup_stake_config, setup_vote},
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
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &validator.pubkey(),
        &stake_config,
        &paladin_stake_program::id(),
    )
    .0;
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
        &stake_config,
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
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &validator.pubkey(),
        &stake_config,
        &paladin_stake_program::id(),
    )
    .0;
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
        &stake_config,
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
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &validator.pubkey(),
        &stake_config,
        &paladin_stake_program::id(),
    )
    .0;
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
        &stake_config,
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
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &validator.pubkey(),
        &stake_config,
        &paladin_stake_program::id(),
    )
    .0;
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
        &stake_config,
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
async fn fail_stake_config_incorrect_account() {
    let validator = Keypair::new();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &validator.pubkey(),
        &Pubkey::new_unique(), // Incorrect stake config.
        &paladin_stake_program::id(),
    )
    .0;
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
    setup_proposal(&mut context, &proposal, &validator.pubkey(), 0, 0).await;

    let instruction = paladin_governance_program::instruction::vote(
        &validator.pubkey(),
        &stake,
        &stake_config,
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
            InstructionError::Custom(PaladinGovernanceError::IncorrectStakeConfig as u32)
        )
    );
}

#[tokio::test]
async fn fail_stake_config_incorrect_owner() {
    let validator = Keypair::new();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &validator.pubkey(),
        &stake_config,
        &paladin_stake_program::id(),
    )
    .0;
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
    setup_proposal(&mut context, &proposal, &validator.pubkey(), 0, 0).await;

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

    let instruction = paladin_governance_program::instruction::vote(
        &validator.pubkey(),
        &stake,
        &stake_config,
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
async fn fail_stake_config_not_initialized() {
    let validator = Keypair::new();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &validator.pubkey(),
        &stake_config,
        &paladin_stake_program::id(),
    )
    .0;
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
    setup_proposal(&mut context, &proposal, &validator.pubkey(), 0, 0).await;

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

    let instruction = paladin_governance_program::instruction::vote(
        &validator.pubkey(),
        &stake,
        &stake_config,
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
async fn fail_governance_incorrect_address() {
    let validator = Keypair::new();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &validator.pubkey(),
        &stake_config,
        &paladin_stake_program::id(),
    )
    .0;
    let proposal_vote = get_vote_address(
        &validator.pubkey(),
        &proposal,
        &paladin_governance_program::id(),
    );
    let governance = Pubkey::new_unique(); // Incorrect governance address.

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, 0).await;
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
        &stake_config,
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
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &validator.pubkey(),
        &stake_config,
        &paladin_stake_program::id(),
    )
    .0;
    let proposal_vote = get_vote_address(
        &validator.pubkey(),
        &proposal,
        &paladin_governance_program::id(),
    );
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, 0).await;
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
        &stake_config,
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
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &validator.pubkey(),
        &stake_config,
        &paladin_stake_program::id(),
    )
    .0;
    let proposal_vote = get_vote_address(
        &validator.pubkey(),
        &proposal,
        &paladin_governance_program::id(),
    );
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, 0).await;
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
        &stake_config,
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
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &validator.pubkey(),
        &stake_config,
        &paladin_stake_program::id(),
    )
    .0;
    let proposal_vote = get_vote_address(
        &validator.pubkey(),
        &proposal,
        &paladin_governance_program::id(),
    );
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, 0).await;
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
        &stake_config,
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
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &validator.pubkey(),
        &stake_config,
        &paladin_stake_program::id(),
    )
    .0;
    let proposal_vote = get_vote_address(
        &validator.pubkey(),
        &proposal,
        &paladin_governance_program::id(),
    );
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, 0).await;
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
        &stake_config,
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
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &validator.pubkey(),
        &stake_config,
        &paladin_stake_program::id(),
    )
    .0;
    let proposal_vote = Pubkey::new_unique(); // Incorrect vote address.
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, 0).await;
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
        &stake_config,
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
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &validator.pubkey(),
        &stake_config,
        &paladin_stake_program::id(),
    )
    .0;
    let proposal_vote = get_vote_address(
        &validator.pubkey(),
        &proposal,
        &paladin_governance_program::id(),
    );
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, 0).await;
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
        &stake_config,
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

struct ProposalSetup {
    acceptance_threshold: u64,
    rejection_threshold: u64,
    total_stake: u64,
}
struct VoteTest {
    vote_stake: u64,
    vote: bool,
    should_have_cooldown: bool,
    should_terminate: bool,
}

#[test_case(
    ProposalSetup {
        acceptance_threshold: 100_000_000, // 10%
        rejection_threshold: 100_000_000,  // 10%
        total_stake: 100_000_000,
    },
    VoteTest {
        vote_stake: 10_000_000, // 10% of total stake.
        vote: true, // For
        should_have_cooldown: true, // Cooldown should be set by this vote.
        should_terminate: false,
    };
    "10p_acceptance_threshold_met_should_begin_cooldown"
)]
#[test_case(
    ProposalSetup {
        acceptance_threshold: 100_000_000, // 10%
        rejection_threshold: 100_000_000,  // 10%
        total_stake: 100_000_000,
    },
    VoteTest {
        vote_stake: 5_000_000, // 5% of total stake.
        vote: true, // For
        should_have_cooldown: false, // Cooldown should not be set by this vote.
        should_terminate: false,
    };
    "10p_acceptance_threshold_not_met_should_not_begin_cooldown"
)]
#[test_case(
    ProposalSetup {
        acceptance_threshold: 100_000_000, // 10%
        rejection_threshold: 100_000_000,  // 10%
        total_stake: 100_000_000,
    },
    VoteTest {
        vote_stake: 10_000_000, // 10% of total stake.
        vote: false, // Against
        should_have_cooldown: false,
        should_terminate: true, // Proposal should be terminated.
    };
    "10p_rejection_threshold_met_should_terminate"
)]
#[test_case(
    ProposalSetup {
        acceptance_threshold: 100_000_000, // 10%
        rejection_threshold: 100_000_000,  // 10%
        total_stake: 100_000_000,
    },
    VoteTest {
        vote_stake: 5_000_000, // 5% of total stake.
        vote: false, // Against
        should_have_cooldown: false,
        should_terminate: false, // Proposal should not be terminated.
    };
    "10p_rejection_threshold_not_met_should_not_terminate"
)]
#[test_case(
    ProposalSetup {
        acceptance_threshold: 100_000, // .0001%
        rejection_threshold: 100_000,  // .0001%
        total_stake: 100_000_000,
    },
    VoteTest {
        vote_stake: 10_000, // .00010% of total stake.
        vote: true, // For
        should_have_cooldown: true, // Cooldown should be set by this vote.
        should_terminate: false,
    };
    "dot_0001p_acceptance_threshold_met_should_begin_cooldown"
)]
#[test_case(
    ProposalSetup {
        acceptance_threshold: 100_000, // .0001%
        rejection_threshold: 100_000,  // .0001%
        total_stake: 100_000_000,
    },
    VoteTest {
        vote_stake: 5_000, // .0005% of total stake.
        vote: true, // For
        should_have_cooldown: false, // Cooldown should not be set by this vote.
        should_terminate: false,
    };
    "dot_0001p_acceptance_threshold_not_met_should_not_begin_cooldown"
)]
#[test_case(
    ProposalSetup {
        acceptance_threshold: 100_000, // .0001%
        rejection_threshold: 100_000,  // .0001%
        total_stake: 100_000_000,
    },
    VoteTest {
        vote_stake: 10_000, // .00010% of total stake.
        vote: false, // Against
        should_have_cooldown: false,
        should_terminate: true, // Proposal should be terminated.
    };
    "dot_0001p_rejection_threshold_met_should_terminate"
)]
#[test_case(
    ProposalSetup {
    acceptance_threshold: 100_000, // .0001%
        rejection_threshold: 100_000,  // .0001%
        total_stake: 100_000_000,
    },
    VoteTest {
        vote_stake: 5_000, // .0005% of total stake.
        vote: false, // Against
        should_have_cooldown: false,
        should_terminate: false, // Proposal should not be terminated.
    };
    "dot_0001p_rejection_threshold_not_met_should_not_terminate"
)]
#[tokio::test]
async fn success(proposal_setup: ProposalSetup, vote_test: VoteTest) {
    let ProposalSetup {
        acceptance_threshold,
        rejection_threshold,
        total_stake,
    } = proposal_setup;
    let VoteTest {
        vote_stake,
        vote,
        should_have_cooldown,
        should_terminate,
    } = vote_test;

    let validator = Keypair::new();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &validator.pubkey(),
        &stake_config,
        &paladin_stake_program::id(),
    )
    .0;
    let proposal_vote = get_vote_address(
        &validator.pubkey(),
        &proposal,
        &paladin_governance_program::id(),
    );
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, total_stake).await;
    setup_stake(
        &mut context,
        &stake,
        /* authority_address */ &Pubkey::new_unique(),
        &validator.pubkey(),
        vote_stake,
    )
    .await;
    setup_governance(
        &mut context,
        &governance,
        /* cooldown_period_seconds */ 10, // Unused here.
        acceptance_threshold,
        rejection_threshold,
        total_stake,
    )
    .await;
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
        &stake_config,
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
        &ProposalVote::new(&proposal, vote_stake, &validator.pubkey(), vote)
    );

    let proposal_account = context
        .banks_client
        .get_account(proposal)
        .await
        .unwrap()
        .unwrap();

    if should_terminate {
        // Assert the proposal was terminated.
        // The proposal should be cleared and reassigned to the system program.
        assert_eq!(proposal_account.owner, solana_program::system_program::id());
        assert_eq!(proposal_account.data.len(), 0);
    } else {
        let proposal_state = bytemuck::from_bytes::<Proposal>(&proposal_account.data);

        // Assert the vote count was updated in the proposal.
        if vote {
            assert_eq!(proposal_state.stake_for, vote_stake);
        } else {
            assert_eq!(proposal_state.stake_against, vote_stake);
        }

        if should_have_cooldown {
            // Assert the cooldown time is set.
            assert!(proposal_state.cooldown_timestamp.is_some());
        } else {
            // Assert the cooldown time is not set.
            assert!(proposal_state.cooldown_timestamp.is_none());
        }
    }
}

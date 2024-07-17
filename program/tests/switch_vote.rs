#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_governance_program::{
        error::PaladinGovernanceError,
        state::{
            get_governance_address, get_proposal_vote_address, Config, Proposal, ProposalVote,
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
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &stake_authority.pubkey(),
        &stake_config,
        &paladin_stake_program::id(),
    )
    .0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;

    let mut instruction = paladin_governance_program::instruction::switch_vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        &governance,
        /* new_vote */ true,
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
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &stake_authority.pubkey(),
        &stake_config,
        &paladin_stake_program::id(),
    )
    .0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
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

    let instruction = paladin_governance_program::instruction::switch_vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        &governance,
        /* new_vote */ true,
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
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &stake_authority.pubkey(),
        &stake_config,
        &paladin_stake_program::id(),
    )
    .0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
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

    let instruction = paladin_governance_program::instruction::switch_vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        &governance,
        /* new_vote */ true,
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
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &stake_authority.pubkey(),
        &stake_config,
        &paladin_stake_program::id(),
    )
    .0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;

    // Set up a stake account with the wrong stake_authority address.
    setup_stake(
        &mut context,
        &stake,
        &Pubkey::new_unique(), // Incorrect stake_authority.
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
        /* new_vote */ true,
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
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &stake_authority.pubkey(),
        &stake_config,
        &paladin_stake_program::id(),
    )
    .0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake(&mut context, &stake, &stake_authority.pubkey(), 0).await;
    setup_proposal(&mut context, &proposal, &stake_authority.pubkey(), 0, 0).await;

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
        /* new_vote */ true,
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
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &stake_authority.pubkey(),
        &stake_config,
        &paladin_stake_program::id(),
    )
    .0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake(&mut context, &stake, &stake_authority.pubkey(), 0).await;
    setup_proposal(&mut context, &proposal, &stake_authority.pubkey(), 0, 0).await;

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
        /* new_vote */ true,
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
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &stake_authority.pubkey(),
        &stake_config,
        &paladin_stake_program::id(),
    )
    .0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
    let governance = Pubkey::new_unique(); // Incorrect governance address.

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, 0).await;
    setup_stake(&mut context, &stake, &stake_authority.pubkey(), 0).await;
    setup_proposal(&mut context, &proposal, &stake_authority.pubkey(), 0, 0).await;

    let instruction = paladin_governance_program::instruction::switch_vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        &governance,
        /* new_vote */ true,
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
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &stake_authority.pubkey(),
        &stake_config,
        &paladin_stake_program::id(),
    )
    .0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, 0).await;
    setup_stake(&mut context, &stake, &stake_authority.pubkey(), 0).await;

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
        /* new_vote */ true,
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
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &stake_authority.pubkey(),
        &stake_config,
        &paladin_stake_program::id(),
    )
    .0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, 0).await;
    setup_stake(&mut context, &stake, &stake_authority.pubkey(), 0).await;

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
        /* new_vote */ true,
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
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &stake_authority.pubkey(),
        &stake_config,
        &paladin_stake_program::id(),
    )
    .0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, 0).await;
    setup_stake(&mut context, &stake, &stake_authority.pubkey(), 0).await;
    setup_governance(&mut context, &governance, 0, 0, 0).await;

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
        /* new_vote */ true,
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
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &stake_authority.pubkey(),
        &stake_config,
        &paladin_stake_program::id(),
    )
    .0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, 0).await;
    setup_stake(&mut context, &stake, &stake_authority.pubkey(), 0).await;
    setup_governance(&mut context, &governance, 0, 0, 0).await;

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
        /* new_vote */ true,
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
async fn fail_proposal_vote_incorrect_address() {
    let stake_authority = Keypair::new();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &stake_authority.pubkey(),
        &stake_config,
        &paladin_stake_program::id(),
    )
    .0;
    let proposal_vote = Pubkey::new_unique(); // Incorrect proposal vote address.
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, 0).await;
    setup_stake(&mut context, &stake, &stake_authority.pubkey(), 0).await;
    setup_governance(&mut context, &governance, 0, 0, 0).await;
    setup_proposal(&mut context, &proposal, &stake_authority.pubkey(), 0, 0).await;

    let instruction = paladin_governance_program::instruction::switch_vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        &governance,
        /* new_vote */ true,
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
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &stake_authority.pubkey(),
        &stake_config,
        &paladin_stake_program::id(),
    )
    .0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, 0).await;
    setup_stake(&mut context, &stake, &stake_authority.pubkey(), 0).await;
    setup_governance(&mut context, &governance, 0, 0, 0).await;
    setup_proposal(&mut context, &proposal, &stake_authority.pubkey(), 0, 0).await;

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
        /* new_vote */ true,
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

struct ProposalSetup {
    acceptance_threshold: u64,
    rejection_threshold: u64,
    proposal_stake_for: u64,
    proposal_stake_against: u64,
    proposal_has_cooldown: bool,
    total_stake: u64,
}
struct SwitchVoteTest {
    vote_stake: u64,
    new_vote: bool,
    should_have_cooldown: bool,
    should_terminate: bool,
}

#[allow(clippy::arithmetic_side_effects)]
#[test_case(
    ProposalSetup {
        acceptance_threshold: 200_000_000, // 20%
        rejection_threshold: 200_000_000,  // 20%
        proposal_stake_for: 15_000_000, // 15% of total stake.
        proposal_stake_against: 15_000_000, // 15% of total stake.
        proposal_has_cooldown: false,
        total_stake: 100_000_000,
    },
    SwitchVoteTest {
        vote_stake: 10_000_000, // 10% of total stake.
        new_vote: true, // Switched from against to for.
        should_have_cooldown: true, // Cooldown should be set by this switched vote.
        should_terminate: false,
    };
    "no_cooldown_acceptance_threshold_met_should_begin_cooldown"
)]
#[test_case(
    ProposalSetup {
        acceptance_threshold: 200_000_000, // 20%
        rejection_threshold: 200_000_000,  // 20%
        proposal_stake_for: 10_000_000, // 10% of total stake.
        proposal_stake_against: 10_000_000, // 10% of total stake.
        proposal_has_cooldown: false,
        total_stake: 100_000_000,
    },
    SwitchVoteTest {
        vote_stake: 5_000_000, // 5% of total stake.
        new_vote: true, // For
        should_have_cooldown: false, // Cooldown should not be set by this switched vote.
        should_terminate: false,
    };
    "no_cooldown_acceptance_threshold_not_met_should_not_begin_cooldown"
)]
#[test_case(
    ProposalSetup {
        acceptance_threshold: 200_000_000, // 20%
        rejection_threshold: 200_000_000,  // 20%
        proposal_stake_for: 20_000_000, // 20% of total stake.
        proposal_stake_against: 10_000_000, // 10% of total stake.
        proposal_has_cooldown: true,
        total_stake: 100_000_000,
    },
    SwitchVoteTest {
        vote_stake: 5_000_000, // 5% of total stake.
        new_vote: false, // Against
        should_have_cooldown: false, // Cooldown should have been disabled by this switched vote.
        should_terminate: false,
    };
    "with_cooldown_fell_below_acceptance_threshold_should_disable_cooldown"
)]
#[test_case(
    ProposalSetup {
        acceptance_threshold: 200_000_000, // 20%
        rejection_threshold: 200_000_000,  // 20%
        proposal_stake_for: 15_000_000, // 15% of total stake.
        proposal_stake_against: 15_000_000, // 15% of total stake.
        proposal_has_cooldown: false,
        total_stake: 100_000_000,
    },
    SwitchVoteTest {
        vote_stake: 10_000_000, // 10% of total stake.
        new_vote: false, // Switched from for to against.
        should_have_cooldown: false,
        should_terminate: true, // Proposal should be terminated.
    };
    "no_cooldown_rejection_threshold_met_should_terminate"
)]
#[test_case(
    ProposalSetup {
        acceptance_threshold: 200_000_000, // 20%
        rejection_threshold: 200_000_000,  // 20%
        proposal_stake_for: 15_000_000, // 15% of total stake.
        proposal_stake_against: 15_000_000, // 15% of total stake.
        proposal_has_cooldown: true,
        total_stake: 100_000_000,
    },
    SwitchVoteTest {
        vote_stake: 10_000_000, // 10% of total stake.
        new_vote: false, // Switched from for to against.
        should_have_cooldown: false,
        should_terminate: true, // Proposal should be terminated.
    };
    "with_cooldown_rejection_threshold_met_should_terminate"
)]
#[test_case(
    ProposalSetup {
        acceptance_threshold: 400_000_000, // 40%
        rejection_threshold: 400_000_000,  // 40%
        proposal_stake_for: 15_000_000, // 15% of total stake.
        proposal_stake_against: 15_000_000, // 15% of total stake.
        proposal_has_cooldown: false,
        total_stake: 100_000_000,
    },
    SwitchVoteTest {
        vote_stake: 10_000_000, // 10% of total stake.
        new_vote: false, // Switched from for to against.
        should_have_cooldown: false,
        should_terminate: false, // Proposal should not be terminated.
    };
    "no_cooldown_rejection_threshold_not_met_should_not_terminate"
)]
#[test_case(
    ProposalSetup {
        acceptance_threshold: 400_000_000, // 40%
        rejection_threshold: 400_000_000,  // 40%
        proposal_stake_for: 15_000_000, // 15% of total stake.
        proposal_stake_against: 15_000_000, // 15% of total stake.
        proposal_has_cooldown: true,
        total_stake: 100_000_000,
    },
    SwitchVoteTest {
        vote_stake: 10_000_000, // 10% of total stake.
        new_vote: false, // Switched from for to against.
        should_have_cooldown: false,
        should_terminate: false, // Proposal should not be terminated.
    };
    "with_cooldown_rejection_threshold_not_met_should_not_terminate"
)]
#[tokio::test]
async fn success(proposal_setup: ProposalSetup, vote_test: SwitchVoteTest) {
    let ProposalSetup {
        acceptance_threshold,
        rejection_threshold,
        proposal_stake_for,
        proposal_stake_against,
        proposal_has_cooldown,
        total_stake,
    } = proposal_setup;
    let SwitchVoteTest {
        vote_stake,
        new_vote,
        should_have_cooldown,
        should_terminate,
    } = vote_test;

    let stake_authority = Keypair::new();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(
        &stake_authority.pubkey(),
        &stake_config,
        &paladin_stake_program::id(),
    )
    .0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());
    let governance = get_governance_address(&paladin_governance_program::id());

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, total_stake).await;
    setup_stake(&mut context, &stake, &stake_authority.pubkey(), vote_stake).await;
    setup_governance(
        &mut context,
        &governance,
        /* cooldown_period_seconds */ 10, // Unused here.
        acceptance_threshold,
        rejection_threshold,
    )
    .await;

    // Set up the proposal vote account with the _inverse_ vote.
    setup_proposal_vote(
        &mut context,
        &proposal_vote,
        &proposal,
        vote_stake,
        &stake_authority.pubkey(),
        !new_vote,
    )
    .await;

    // Set up the proposal with the previous vote counted.
    if proposal_has_cooldown {
        setup_proposal_with_stake_and_cooldown(
            &mut context,
            &proposal,
            &stake_authority.pubkey(),
            0,
            0,
            proposal_stake_for,
            proposal_stake_against,
            NonZeroU64::new(1), // Doesn't matter, just has to be `Some`.
        )
        .await;
    } else {
        setup_proposal_with_stake(
            &mut context,
            &proposal,
            &stake_authority.pubkey(),
            0,
            0,
            proposal_stake_for,
            proposal_stake_against,
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
        new_vote,
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
        &ProposalVote::new(&proposal, vote_stake, &stake_authority.pubkey(), new_vote)
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
        if new_vote {
            // Vote switched from against to for, so stake should have moved
            // from against to for.
            assert_eq!(
                proposal_state.stake_against,
                proposal_stake_against - vote_stake
            );
            assert_eq!(proposal_state.stake_for, proposal_stake_for + vote_stake);
        } else {
            // Vote switched from for to against, so stake should have moved
            // from for to against.
            assert_eq!(proposal_state.stake_for, proposal_stake_for - vote_stake);
            assert_eq!(
                proposal_state.stake_against,
                proposal_stake_against + vote_stake
            );
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

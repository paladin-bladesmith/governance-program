#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_governance_program::{
        error::PaladinGovernanceError,
        state::{
            get_proposal_vote_address, GovernanceConfig, Proposal, ProposalStatus, ProposalVote,
            ProposalVoteElection,
        },
    },
    paladin_stake_program::state::{find_stake_pda, Config as StakeConfig, Stake},
    setup::{
        setup, setup_proposal, setup_proposal_vote, setup_proposal_with_stake,
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
        system_program,
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

    let mut context = setup().start_with_context().await;

    let mut instruction = paladin_governance_program::instruction::vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
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
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
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
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
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

    let instruction = paladin_governance_program::instruction::vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
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
        GovernanceConfig::default(),
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

    let instruction = paladin_governance_program::instruction::vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
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
        GovernanceConfig::default(),
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

    let instruction = paladin_governance_program::instruction::vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
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
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
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
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
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

    let governance_config = GovernanceConfig::new(
        /* cooldown_period_seconds */ 0,
        /* proposal_acceptance_threshold */ 0,
        /* proposal_rejection_threshold */ 0,
        /* signer_bump_seed */ 0,
        /* stake_config_address */ &stake_config,
        /* voting_period_seconds */ 0,
    );

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
        governance_config,
        ProposalStatus::Draft, // Not voting stage.
    )
    .await;

    let instruction = paladin_governance_program::instruction::vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
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

    let governance_config = GovernanceConfig::new(
        /* cooldown_period_seconds */ 0,
        /* proposal_acceptance_threshold */ 0,
        /* proposal_rejection_threshold */ 0,
        /* signer_bump_seed */ 0,
        /* stake_config_address */ &stake_config,
        /* voting_period_seconds */ 0,
    );

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
        governance_config,
        ProposalStatus::Voting,
    )
    .await;

    let instruction = paladin_governance_program::instruction::vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
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
async fn fail_proposal_vote_already_initialized() {
    let stake_authority = Keypair::new();
    let validator_vote = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());

    let governance_config = GovernanceConfig::new(
        /* cooldown_period_seconds */ 0,
        /* proposal_acceptance_threshold */ 0,
        /* proposal_rejection_threshold */ 0,
        /* signer_bump_seed */ 0,
        /* stake_config_address */ &stake_config,
        /* voting_period_seconds */ 0,
    );

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
        governance_config,
        ProposalStatus::Voting,
    )
    .await;

    // Set up an initialized proposal vote account.
    setup_proposal_vote(
        &mut context,
        &proposal_vote,
        &proposal,
        0,
        &stake_authority.pubkey(),
        ProposalVoteElection::For,
    )
    .await;

    let instruction = paladin_governance_program::instruction::vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
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
        TransactionError::InstructionError(0, InstructionError::AccountAlreadyInitialized)
    );
}

const ACCEPTANCE_THRESHOLD: u32 = 500_000_000; // 50%
const REJECTION_THRESHOLD: u32 = 500_000_000; // 50%
const COOLDOWN_PERIOD_SECONDS: u64 = 100_000_000;
const VOTING_PERIOD_SECONDS: u64 = 100_000_000;
const TOTAL_STAKE: u64 = 100_000_000;

const PROPOSAL_STARTING_STAKE_FOR: u64 = 0;
const PROPOSAL_STARTING_STAKE_AGAINST: u64 = 0;
const PROPOSAL_STARTING_STAKE_ABSTAINED: u64 = 0;

struct Vote {
    vote_stake: u64,
    election: ProposalVoteElection,
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

#[test_case(
    Vote {
        vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        election: ProposalVoteElection::DidNotVote,
    },
    Expect::Cast {
        cooldown: false,
        stake_for: 0,
        stake_against: 0,
        stake_abstained: TOTAL_STAKE / 10,
    };
    "did_not_vote_increments_stake_abstained"
)]
#[test_case(
    Vote {
        vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        election: ProposalVoteElection::For,
    },
    Expect::Cast {
        cooldown: false,
        stake_for: TOTAL_STAKE / 10,
        stake_against: 0,
        stake_abstained: 0,
    };
    "vote_for_increments_stake_for"
)]
#[test_case(
    Vote {
        vote_stake: TOTAL_STAKE / 10, // 10% of total stake.
        election: ProposalVoteElection::Against,
    },
    Expect::Cast {
        cooldown: false,
        stake_for: 0,
        stake_against: TOTAL_STAKE / 10,
        stake_abstained: 0,
    };
    "vote_against_increments_stake_against"
)]
#[test_case(
    Vote {
        vote_stake: TOTAL_STAKE / 2, // 50% of total stake.
        election: ProposalVoteElection::For,
    },
    Expect::Cast {
        cooldown: true, // Cooldown should be set.
        stake_for: TOTAL_STAKE / 2,
        stake_against: 0,
        stake_abstained: 0,
    };
    "vote_for_beyond_threshold_increments_stake_for_and_activates_cooldown"
)]
#[test_case(
    Vote {
        vote_stake: TOTAL_STAKE / 2, // 50% of total stake.
        election: ProposalVoteElection::Against,
    },
    Expect::Terminated;
    "vote_against_beyond_threshold_terminates"
)]
#[tokio::test]
async fn success(vote: Vote, expect: Expect) {
    let Vote {
        vote_stake,
        election,
    } = vote;

    let stake_authority = Keypair::new();
    let validator_vote = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());

    let governance_config = GovernanceConfig::new(
        COOLDOWN_PERIOD_SECONDS,
        ACCEPTANCE_THRESHOLD,
        REJECTION_THRESHOLD,
        /* signer_bump_seed */ 0,
        &stake_config,
        VOTING_PERIOD_SECONDS,
    );

    let mut context = setup().start_with_context().await;
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();

    setup_stake_config(&mut context, &stake_config, TOTAL_STAKE).await;
    setup_stake(
        &mut context,
        &stake,
        &stake_authority.pubkey(),
        &validator_vote,
        vote_stake,
    )
    .await;
    setup_proposal_with_stake(
        &mut context,
        &proposal,
        &stake_authority.pubkey(),
        0,
        governance_config,
        PROPOSAL_STARTING_STAKE_FOR,
        PROPOSAL_STARTING_STAKE_AGAINST,
        PROPOSAL_STARTING_STAKE_ABSTAINED,
        ProposalStatus::Voting,
        /* voting_start_timestamp */ NonZeroU64::new(clock.unix_timestamp as u64),
    )
    .await;

    // Fund the proposal vote account.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let lamports = rent.minimum_balance(std::mem::size_of::<ProposalVote>());
        context.set_account(
            &proposal_vote,
            &AccountSharedData::new(lamports, 0, &system_program::id()),
        );
    }

    let instruction = paladin_governance_program::instruction::vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        election,
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

    // Assert the proposal vote was created.
    let proposal_vote_account = context
        .banks_client
        .get_account(proposal_vote)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        bytemuck::from_bytes::<ProposalVote>(&proposal_vote_account.data),
        &ProposalVote::new(&proposal, vote_stake, &stake_authority.pubkey(), election)
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

#[tokio::test]
async fn success_voting_closed() {
    let stake_authority = Keypair::new();
    let validator_vote = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());

    let vote_stake = TOTAL_STAKE / 10;
    let election = ProposalVoteElection::Against;

    let governance_config = GovernanceConfig::new(
        /* cooldown_period_seconds */ 10,
        ACCEPTANCE_THRESHOLD,
        REJECTION_THRESHOLD,
        /* signer_bump_seed */ 0,
        &stake_config,
        /* voting_period_seconds */ 10,
    );

    let mut context = setup().start_with_context().await;

    setup_stake_config(&mut context, &stake_config, TOTAL_STAKE).await;
    setup_stake(
        &mut context,
        &stake,
        &stake_authority.pubkey(),
        &validator_vote,
        vote_stake,
    )
    .await;

    // Set up a proposal with stake against > threshold and a voting period
    // that began very long ago (expired).
    setup_proposal_with_stake(
        &mut context,
        &proposal,
        &stake_authority.pubkey(),
        /* creation_timestamp */ 0,
        governance_config,
        /* stake_for */ 0,
        /* stake_against */ TOTAL_STAKE,
        /* stake_abstained */ 0,
        ProposalStatus::Voting,
        /* voting_start_timestamp */ NonZeroU64::new(1), // Wayyy earlier.
    )
    .await;

    // Fund the proposal vote account.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let lamports = rent.minimum_balance(std::mem::size_of::<ProposalVote>());
        context.set_account(
            &proposal_vote,
            &AccountSharedData::new(lamports, 0, &system_program::id()),
        );
    }

    let instruction = paladin_governance_program::instruction::vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        election,
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

    let proposal_account = context
        .banks_client
        .get_account(proposal)
        .await
        .unwrap()
        .unwrap();

    // Assert the proposal was marked as rejected.
    let proposal_state = bytemuck::from_bytes::<Proposal>(&proposal_account.data);
    assert_eq!(proposal_state.status, ProposalStatus::Rejected);

    let proposal_vote_account = context
        .banks_client
        .get_account(proposal_vote)
        .await
        .unwrap()
        .unwrap();

    // Assert the proposal vote was _not_ initialized.
    assert_eq!(proposal_vote_account.data.len(), 0);
    assert_eq!(proposal_vote_account.owner, system_program::id());
}

#[tokio::test]
async fn success_voting_closed_but_cooldown_active() {
    // Here we're testing the case where a proposal's voting period has
    // expired, but since there's an active cooldown period, it doesn'
    // actually disable voting, and instead lets the cooldown period expire.
    let stake_authority = Keypair::new();
    let validator_vote = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake = find_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());

    let vote_stake = TOTAL_STAKE / 10;
    let election = ProposalVoteElection::Against;

    let governance_config = GovernanceConfig::new(
        /* cooldown_period_seconds */ 1_000,
        ACCEPTANCE_THRESHOLD,
        REJECTION_THRESHOLD,
        /* signer_bump_seed */ 0,
        &stake_config,
        /* voting_period_seconds */ 10,
    );

    let mut context = setup().start_with_context().await;
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();

    setup_stake_config(&mut context, &stake_config, TOTAL_STAKE).await;
    setup_stake(
        &mut context,
        &stake,
        &stake_authority.pubkey(),
        &validator_vote,
        vote_stake,
    )
    .await;

    // Set up a proposal with stake for > threshold and an active cooldown
    // period.
    // The cooldown period is scheduled to expire _after_ the voting period
    // expires.
    setup_proposal_with_stake_and_cooldown(
        &mut context,
        &proposal,
        &stake_authority.pubkey(),
        /* creation_timestamp */ 0,
        governance_config,
        /* stake_for */ TOTAL_STAKE,
        /* stake_against */ 0,
        /* stake_abstained */ 0,
        ProposalStatus::Voting,
        /* voting_start_timestamp */ NonZeroU64::new(1), // Wayyy earlier.
        /* cooldown_timestamp */
        NonZeroU64::new(clock.unix_timestamp.saturating_sub(10) as u64), // Still active.
    )
    .await;

    // Fund the proposal vote account.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let lamports = rent.minimum_balance(std::mem::size_of::<ProposalVote>());
        context.set_account(
            &proposal_vote,
            &AccountSharedData::new(lamports, 0, &system_program::id()),
        );
    }

    let instruction = paladin_governance_program::instruction::vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        election,
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

    let proposal_account = context
        .banks_client
        .get_account(proposal)
        .await
        .unwrap()
        .unwrap();

    // Assert the proposal is still in the voting stage.
    let proposal_state = bytemuck::from_bytes::<Proposal>(&proposal_account.data);
    assert_eq!(proposal_state.status, ProposalStatus::Voting);

    // Assert the proposal vote was created.
    let proposal_vote_account = context
        .banks_client
        .get_account(proposal_vote)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        bytemuck::from_bytes::<ProposalVote>(&proposal_vote_account.data),
        &ProposalVote::new(&proposal, vote_stake, &stake_authority.pubkey(), election)
    );
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

    let vote_stake = TOTAL_STAKE / 10;
    let election = ProposalVoteElection::Against;

    let governance_config = GovernanceConfig::new(
        /* cooldown_period_seconds */ 10,
        ACCEPTANCE_THRESHOLD,
        REJECTION_THRESHOLD,
        /* signer_bump_seed */ 0,
        &stake_config,
        /* voting_period_seconds */ 1_000,
    );

    let mut context = setup().start_with_context().await;
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();

    setup_stake_config(&mut context, &stake_config, TOTAL_STAKE).await;
    setup_stake(
        &mut context,
        &stake,
        &stake_authority.pubkey(),
        &validator_vote,
        vote_stake,
    )
    .await;

    // Set up a proposal with a cooldown timestamp and stake for above
    // threshold.
    setup_proposal_with_stake_and_cooldown(
        &mut context,
        &proposal,
        &stake_authority.pubkey(),
        /* creation_timestamp */ 0,
        governance_config,
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

    // Fund the proposal vote account.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let lamports = rent.minimum_balance(std::mem::size_of::<ProposalVote>());
        context.set_account(
            &proposal_vote,
            &AccountSharedData::new(lamports, 0, &system_program::id()),
        );
    }

    let instruction = paladin_governance_program::instruction::vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        election,
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

    let proposal_account = context
        .banks_client
        .get_account(proposal)
        .await
        .unwrap()
        .unwrap();
    let proposal_state = bytemuck::from_bytes::<Proposal>(&proposal_account.data);

    // Assert the proposal was accepted.
    assert_eq!(proposal_state.status, ProposalStatus::Accepted);

    let proposal_vote_account = context
        .banks_client
        .get_account(proposal_vote)
        .await
        .unwrap()
        .unwrap();

    // Assert the proposal vote was _not_ initialized.
    assert_eq!(proposal_vote_account.data.len(), 0);
    assert_eq!(proposal_vote_account.owner, system_program::id());
}

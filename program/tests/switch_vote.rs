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
    paladin_stake_program::state::{
        find_validator_stake_pda, Config as StakeConfig, ValidatorStake,
    },
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

    let stake =
        find_validator_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());

    let mut context = setup().start_with_context().await;

    let mut instruction = paladin_governance_program::instruction::switch_vote(
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

    let stake =
        find_validator_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());

    let mut context = setup().start_with_context().await;

    // Set up the stake account with the incorrect owner.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let space = std::mem::size_of::<ValidatorStake>();
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

    let stake =
        find_validator_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());

    let mut context = setup().start_with_context().await;

    // Set up an uninitialized stake account.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let space = std::mem::size_of::<ValidatorStake>();
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

    let stake =
        find_validator_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
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

    let instruction = paladin_governance_program::instruction::switch_vote(
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

    let stake =
        find_validator_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
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

    let instruction = paladin_governance_program::instruction::switch_vote(
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

    let stake =
        find_validator_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
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

    let instruction = paladin_governance_program::instruction::switch_vote(
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

    let stake =
        find_validator_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
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

    let instruction = paladin_governance_program::instruction::switch_vote(
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

    let stake =
        find_validator_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
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

    let instruction = paladin_governance_program::instruction::switch_vote(
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

    let stake =
        find_validator_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());

    let governance_config = GovernanceConfig {
        cooldown_period_seconds: 0,
        proposal_acceptance_threshold: 0,
        proposal_rejection_threshold: 0,
        signer_bump_seed: 0,
        _padding: [0; 7],
        stake_config_address: stake_config,
        voting_period_seconds: 0,
    };

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
        ProposalStatus::Draft, // Not in voting stage.
    )
    .await;

    let instruction = paladin_governance_program::instruction::switch_vote(
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

    let stake =
        find_validator_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote = Pubkey::new_unique(); // Incorrect proposal vote address.

    let governance_config = GovernanceConfig {
        cooldown_period_seconds: 0,
        proposal_acceptance_threshold: 0,
        proposal_rejection_threshold: 0,
        signer_bump_seed: 0,
        _padding: [0; 7],
        stake_config_address: stake_config,
        voting_period_seconds: 0,
    };

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

    let instruction = paladin_governance_program::instruction::switch_vote(
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
async fn fail_proposal_vote_not_initialized() {
    let stake_authority = Keypair::new();
    let validator_vote = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake =
        find_validator_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());

    let governance_config = GovernanceConfig {
        cooldown_period_seconds: 0,
        proposal_acceptance_threshold: 0,
        proposal_rejection_threshold: 0,
        signer_bump_seed: 0,
        _padding: [0; 7],
        stake_config_address: stake_config,
        voting_period_seconds: 0,
    };

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
const COOLDOWN_PERIOD_SECONDS: u64 = 100_000_000;
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
        cooldown: true, // Cooldown unchanged.
        stake_for: TOTAL_STAKE / 2 - TOTAL_STAKE / 10, // 40% of total stake.
        stake_against: TOTAL_STAKE / 4 + TOTAL_STAKE / 10, // 35% of total stake.
        stake_abstained: TOTAL_STAKE / 4, // Unchanged.
    };
    "for_to_against_below_for_threshold_deducts_stake_for_increments_stake_against"
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

    let stake =
        find_validator_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());

    let governance_config = GovernanceConfig {
        cooldown_period_seconds: COOLDOWN_PERIOD_SECONDS,
        proposal_acceptance_threshold: ACCEPTANCE_THRESHOLD,
        proposal_rejection_threshold: REJECTION_THRESHOLD,
        signer_bump_seed: 0,
        _padding: [0; 7],
        stake_config_address: stake_config,
        voting_period_seconds: VOTING_PERIOD_SECONDS,
    };

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
            governance_config,
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
            governance_config,
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

#[tokio::test]
async fn success_voting_closed() {
    let stake_authority = Keypair::new();
    let validator_vote = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake =
        find_validator_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());

    let prev_vote_stake = TOTAL_STAKE / 10;
    let prev_election = ProposalVoteElection::Against;

    let new_vote_stake = TOTAL_STAKE / 10 + 100;
    let new_election = ProposalVoteElection::For;

    let governance_config = GovernanceConfig {
        cooldown_period_seconds: 10,
        proposal_acceptance_threshold: ACCEPTANCE_THRESHOLD,
        proposal_rejection_threshold: REJECTION_THRESHOLD,
        signer_bump_seed: 0,
        _padding: [0; 7],
        stake_config_address: stake_config,
        voting_period_seconds: 10,
    };

    let mut context = setup().start_with_context().await;

    setup_stake_config(&mut context, &stake_config, TOTAL_STAKE).await;
    setup_stake(
        &mut context,
        &stake,
        &stake_authority.pubkey(),
        &validator_vote,
        new_vote_stake,
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

    setup_proposal_vote(
        &mut context,
        &proposal_vote,
        &proposal,
        prev_vote_stake,
        &stake_authority.pubkey(),
        prev_election,
    )
    .await;

    let instruction = paladin_governance_program::instruction::switch_vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        new_election,
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

    // Assert the proposal vote was _not_ updated.
    let proposal_vote_state = bytemuck::from_bytes::<ProposalVote>(&proposal_vote_account.data);
    assert_eq!(proposal_vote_state.stake, prev_vote_stake);
    assert_eq!(proposal_vote_state.election, prev_election);
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

    let stake =
        find_validator_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());

    let prev_vote_stake = TOTAL_STAKE / 10;
    let prev_election = ProposalVoteElection::For;

    let new_vote_stake = TOTAL_STAKE / 10 + 100;
    let new_election = ProposalVoteElection::Against;

    let governance_config = GovernanceConfig {
        cooldown_period_seconds: 1_000,
        proposal_acceptance_threshold: ACCEPTANCE_THRESHOLD,
        proposal_rejection_threshold: REJECTION_THRESHOLD,
        signer_bump_seed: 0,
        _padding: [0; 7],
        stake_config_address: stake_config,
        voting_period_seconds: 10,
    };

    let mut context = setup().start_with_context().await;
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();

    setup_stake_config(&mut context, &stake_config, TOTAL_STAKE).await;
    setup_stake(
        &mut context,
        &stake,
        &stake_authority.pubkey(),
        &validator_vote,
        new_vote_stake,
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

    setup_proposal_vote(
        &mut context,
        &proposal_vote,
        &proposal,
        prev_vote_stake,
        &stake_authority.pubkey(),
        prev_election,
    )
    .await;

    let instruction = paladin_governance_program::instruction::switch_vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        new_election,
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

    let proposal_vote_account = context
        .banks_client
        .get_account(proposal_vote)
        .await
        .unwrap()
        .unwrap();

    // Assert the proposal vote was updated.
    let proposal_vote_state = bytemuck::from_bytes::<ProposalVote>(&proposal_vote_account.data);
    assert_eq!(proposal_vote_state.stake, new_vote_stake);
    assert_eq!(proposal_vote_state.election, new_election);
}

#[test_case(true, ProposalStatus::Accepted; "threshold_met")]
#[test_case(false, ProposalStatus::Rejected; "threshold_not_met")]
#[tokio::test]
async fn success_cooldown_has_ended(threshold_met: bool, expected_status: ProposalStatus) {
    let stake_authority = Keypair::new();
    let validator_vote = Pubkey::new_unique();
    let stake_config = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let stake =
        find_validator_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_vote =
        get_proposal_vote_address(&stake, &proposal, &paladin_governance_program::id());

    let prev_vote_stake = TOTAL_STAKE / 10;
    let prev_election = ProposalVoteElection::Against;

    let new_vote_stake = TOTAL_STAKE / 10 + 100;
    let new_election = ProposalVoteElection::For;

    let governance_config = GovernanceConfig {
        cooldown_period_seconds: 10,
        proposal_acceptance_threshold: ACCEPTANCE_THRESHOLD,
        proposal_rejection_threshold: REJECTION_THRESHOLD,
        signer_bump_seed: 0,
        _padding: [0; 7],
        stake_config_address: stake_config,
        voting_period_seconds: 1000,
    };

    // We'll set up a proposal whose cooldown period has ended.
    // If `threshold_met` is true, the proposal's stake_for will be set to the
    // exact amount needed to meet the threshold.
    // If `threshold_met` is false, the proposal's stake_for will be set to
    // just below the threshold.
    // The vote doesn't matter, since once cooldown is over, no more votes can
    // be tallied, so this invocation is basically just a crank.
    let accepance_threshold_stake_amount = TOTAL_STAKE / 2; // 50%
    let proposal_stake_for = if threshold_met {
        accepance_threshold_stake_amount
    } else {
        accepance_threshold_stake_amount - 1
    };

    let mut context = setup().start_with_context().await;
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();

    setup_stake_config(&mut context, &stake_config, TOTAL_STAKE).await;
    setup_stake(
        &mut context,
        &stake,
        &stake_authority.pubkey(),
        &validator_vote,
        new_vote_stake,
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
        proposal_stake_for,
        /* stake_against */ 0,
        /* stake_abstained */ 0,
        ProposalStatus::Voting,
        /* voting_start_timestamp */
        NonZeroU64::new(clock.unix_timestamp as u64),
        /* cooldown_timestamp */
        NonZeroU64::new(clock.unix_timestamp.saturating_sub(10) as u64), // Now - 10 seconds.
    )
    .await;

    setup_proposal_vote(
        &mut context,
        &proposal_vote,
        &proposal,
        prev_vote_stake,
        &stake_authority.pubkey(),
        prev_election,
    )
    .await;

    let instruction = paladin_governance_program::instruction::switch_vote(
        &stake_authority.pubkey(),
        &stake,
        &stake_config,
        &proposal_vote,
        &proposal,
        new_election,
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

    // Assert the proposal has the expected status.
    assert_eq!(proposal_state.status, expected_status);

    let proposal_vote_account = context
        .banks_client
        .get_account(proposal_vote)
        .await
        .unwrap()
        .unwrap();

    // Assert the proposal vote was _not_ updated.
    let proposal_vote_state = bytemuck::from_bytes::<ProposalVote>(&proposal_vote_account.data);
    assert_eq!(proposal_vote_state.stake, prev_vote_stake);
    assert_eq!(proposal_vote_state.election, prev_election);
}

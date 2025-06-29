#![cfg(feature = "test-sbf")]

mod setup;

use {
    borsh::BorshDeserialize,
    paladin_governance_program::{
        error::PaladinGovernanceError,
        instruction::create_proposal,
        state::{
            get_proposal_transaction_address, GovernanceConfig, Proposal, ProposalStatus,
            ProposalTransaction,
        },
    },
    paladin_stake_program::state::{find_validator_stake_pda, ValidatorStake},
    setup::{
        setup, setup_author, setup_governance, setup_proposal, setup_proposal_transaction,
        setup_stake, setup_stake_config,
    },
    solana_program_test::*,
    solana_sdk::{
        account::AccountSharedData,
        borsh1::get_instance_packed_len,
        clock::Clock,
        instruction::InstructionError,
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
        system_program,
        transaction::{Transaction, TransactionError},
    },
};

#[tokio::test]
async fn fail_stake_authority_not_signer() {
    let stake_authority = Keypair::new();
    let stake = Pubkey::new_unique(); // PDA doesn't matter here.
    let proposal = Pubkey::new_unique();
    let proposal_transaction =
        get_proposal_transaction_address(&proposal, &paladin_governance_program::id());
    let governance = Pubkey::new_unique(); // PDA doesn't matter here.

    let context = setup().start_with_context().await;

    let mut instruction = create_proposal(
        &stake_authority.pubkey(),
        &stake,
        &proposal,
        &proposal_transaction,
        &governance,
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
    let validator_vote = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let stake =
        find_validator_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_transaction =
        get_proposal_transaction_address(&proposal, &paladin_governance_program::id());
    let governance = Pubkey::new_unique(); // PDA doesn't matter here.

    let mut context = setup().start_with_context().await;
    setup_author(&mut context, &stake_authority.pubkey(), 0).await;

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

    let instruction = create_proposal(
        &stake_authority.pubkey(),
        &stake,
        &proposal,
        &proposal_transaction,
        &governance,
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
    let validator_vote = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let stake =
        find_validator_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_transaction =
        get_proposal_transaction_address(&proposal, &paladin_governance_program::id());
    let governance = Pubkey::new_unique(); // PDA doesn't matter here.

    let mut context = setup().start_with_context().await;
    setup_author(&mut context, &stake_authority.pubkey(), 0).await;

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

    let instruction = create_proposal(
        &stake_authority.pubkey(),
        &stake,
        &proposal,
        &proposal_transaction,
        &governance,
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
    let validator_vote = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let stake =
        find_validator_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_transaction =
        get_proposal_transaction_address(&proposal, &paladin_governance_program::id());
    let governance = Pubkey::new_unique(); // PDA doesn't matter here.

    let mut context = setup().start_with_context().await;
    setup_author(&mut context, &stake_authority.pubkey(), 0).await;

    // Set up a stake account with the wrong stake authority address.
    setup_stake(
        &mut context,
        &stake,
        Pubkey::new_unique(), // Incorrect stake authority.
        validator_vote,
        0,
    )
    .await;

    let instruction = create_proposal(
        &stake_authority.pubkey(),
        &stake,
        &proposal,
        &proposal_transaction,
        &governance,
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
async fn fail_governance_incorrect_owner() {
    let stake_authority = Keypair::new();
    let stake_config = Pubkey::new_unique();
    let validator_vote = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let stake =
        find_validator_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_transaction =
        get_proposal_transaction_address(&proposal, &paladin_governance_program::id());
    let governance = Pubkey::new_unique(); // PDA doesn't matter here.

    let mut context = setup().start_with_context().await;
    setup_author(&mut context, &stake_authority.pubkey(), 0).await;
    setup_stake(
        &mut context,
        &stake,
        stake_authority.pubkey(),
        validator_vote,
        0,
    )
    .await;

    // Set up the governance account with the incorrect owner.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let space = std::mem::size_of::<GovernanceConfig>();
        let lamports = rent.minimum_balance(space);
        context.set_account(
            &governance,
            &AccountSharedData::new(lamports, space, &Pubkey::new_unique()), // Incorrect owner.
        );
    }

    let instruction = create_proposal(
        &stake_authority.pubkey(),
        &stake,
        &proposal,
        &proposal_transaction,
        &governance,
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
    let validator_vote = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let stake =
        find_validator_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_transaction =
        get_proposal_transaction_address(&proposal, &paladin_governance_program::id());
    let governance = Pubkey::new_unique(); // PDA doesn't matter here.

    let mut context = setup().start_with_context().await;
    setup_author(&mut context, &stake_authority.pubkey(), 0).await;
    setup_stake(
        &mut context,
        &stake,
        stake_authority.pubkey(),
        validator_vote,
        0,
    )
    .await;

    // Set up the governance account uninitialized.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let lamports = rent.minimum_balance(std::mem::size_of::<GovernanceConfig>());
        context.set_account(
            &governance,
            &AccountSharedData::new(lamports, 0, &paladin_governance_program::id()),
        );
    }

    let instruction = create_proposal(
        &stake_authority.pubkey(),
        &stake,
        &proposal,
        &proposal_transaction,
        &governance,
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
    let validator_vote = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let stake =
        find_validator_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_transaction =
        get_proposal_transaction_address(&proposal, &paladin_governance_program::id());
    let governance = Pubkey::new_unique(); // PDA doesn't matter here.
    let governance_config = GovernanceConfig {
        stake_config_address: stake_config,
        ..GovernanceConfig::default()
    };

    let mut context = setup().start_with_context().await;
    setup_author(&mut context, &stake_authority.pubkey(), 0).await;
    setup_stake(
        &mut context,
        &stake,
        stake_authority.pubkey(),
        validator_vote,
        0,
    )
    .await;
    setup_governance(&mut context, &governance, &governance_config).await;

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

    let instruction = create_proposal(
        &stake_authority.pubkey(),
        &stake,
        &proposal,
        &proposal_transaction,
        &governance,
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
async fn fail_proposal_not_enough_space() {
    let stake_authority = Keypair::new();
    let stake = Pubkey::new_unique(); // PDA doesn't matter here.
    let proposal = Pubkey::new_unique();
    let proposal_transaction =
        get_proposal_transaction_address(&proposal, &paladin_governance_program::id());
    let governance = Pubkey::new_unique(); // PDA doesn't matter here.

    let mut context = setup().start_with_context().await;
    setup_stake(
        &mut context,
        &stake,
        stake_authority.pubkey(),
        /* validator_vote_address */ Pubkey::new_unique(), // Unused here.
        0,
    )
    .await;
    setup_governance(&mut context, &governance, &GovernanceConfig::default()).await;

    // Set up the proposal account with not enough space.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let space = std::mem::size_of::<Proposal>() - 1; // Not enough space.
        let lamports = rent.minimum_balance(space);
        context.set_account(
            &proposal,
            &AccountSharedData::new(lamports, space, &paladin_governance_program::id()),
        );
    }

    let instruction = create_proposal(
        &stake_authority.pubkey(),
        &stake,
        &proposal,
        &proposal_transaction,
        &governance,
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
        TransactionError::InstructionError(0, InstructionError::InvalidAccountData)
    );
}

#[tokio::test]
async fn fail_proposal_already_initialized() {
    let stake_authority = Keypair::new();
    let stake_config = Pubkey::new_unique();
    let validator_vote = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let stake =
        find_validator_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_transaction =
        get_proposal_transaction_address(&proposal, &paladin_governance_program::id());
    let governance = Pubkey::new_unique(); // PDA doesn't matter here.
    let governance_config = GovernanceConfig {
        stake_config_address: stake_config,
        ..GovernanceConfig::default()
    };

    let mut context = setup().start_with_context().await;
    setup_stake_config(&mut context, &stake_config, 0).await;
    setup_stake(
        &mut context,
        &stake,
        stake_authority.pubkey(),
        validator_vote,
        0,
    )
    .await;
    setup_governance(&mut context, &governance, &governance_config).await;

    // Set up an initialized proposal account.
    setup_author(&mut context, &stake_authority.pubkey(), 0).await;
    setup_proposal(
        &mut context,
        &proposal,
        &stake_authority.pubkey(),
        0,
        governance_config,
        ProposalStatus::Draft,
    )
    .await;

    let instruction = create_proposal(
        &stake_authority.pubkey(),
        &stake,
        &proposal,
        &proposal_transaction,
        &governance,
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

#[tokio::test]
async fn fail_proposal_transaction_incorrect_address() {
    let stake_authority = Keypair::new();
    let stake_config = Pubkey::new_unique();
    let validator_vote = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let stake =
        find_validator_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_transaction = Pubkey::new_unique(); // Intentionally not correct.
    let governance = Pubkey::new_unique(); // PDA doesn't matter here.
    let governance_config = GovernanceConfig {
        stake_config_address: stake_config,
        ..Default::default()
    };

    let mut context = setup().start_with_context().await;
    setup_author(&mut context, &stake_authority.pubkey(), 0).await;
    setup_stake(
        &mut context,
        &stake,
        stake_authority.pubkey(),
        validator_vote,
        0,
    )
    .await;
    setup_governance(&mut context, &governance, &governance_config).await;

    // Fund the proposal account.
    {
        let rent = context.banks_client.get_rent().await.unwrap();

        let space = std::mem::size_of::<Proposal>();
        let lamports = rent.minimum_balance(space);
        context.set_account(
            &proposal,
            &AccountSharedData::new(lamports, space, &paladin_governance_program::id()),
        );
    }

    let instruction = create_proposal(
        &stake_authority.pubkey(),
        &stake,
        &proposal,
        &proposal_transaction,
        &governance,
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
                PaladinGovernanceError::IncorrectProposalTransactionAddress as u32
            )
        )
    );
}

#[tokio::test]
async fn fail_proposal_transaction_already_initialized() {
    let stake_authority = Keypair::new();
    let stake_config = Pubkey::new_unique();
    let validator_vote = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let stake =
        find_validator_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_transaction =
        get_proposal_transaction_address(&proposal, &paladin_governance_program::id());
    let governance = Pubkey::new_unique(); // PDA doesn't matter here.
    let governance_config = GovernanceConfig {
        cooldown_period_seconds: 100_000_000,
        proposal_minimum_quorum: 5 * 10u32.pow(8), // 50%
        proposal_pass_threshold: 5 * 10u32.pow(8), // 50%
        stake_config_address: stake_config,
        voting_period_seconds: 100_000_000,
        stake_per_proposal: 0,
        governance_config: governance,
        cooldown_expires: 0, 
    };

    let mut context = setup().start_with_context().await;
    setup_author(&mut context, &stake_authority.pubkey(), 0).await;
    setup_stake(
        &mut context,
        &stake,
        stake_authority.pubkey(),
        validator_vote,
        0,
    )
    .await;
    setup_governance(&mut context, &governance, &governance_config).await;

    // Set up a proposal transaction account already initialized.
    setup_proposal_transaction(
        &mut context,
        &proposal_transaction,
        ProposalTransaction::default(),
    )
    .await;

    // Fund the proposal account.
    {
        let rent = context.banks_client.get_rent().await.unwrap();

        let space = std::mem::size_of::<Proposal>();
        let lamports = rent.minimum_balance(space);
        context.set_account(
            &proposal,
            &AccountSharedData::new(lamports, space, &paladin_governance_program::id()),
        );
    }

    let instruction = create_proposal(
        &stake_authority.pubkey(),
        &stake,
        &proposal,
        &proposal_transaction,
        &governance,
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

#[tokio::test]
async fn fail_proposal_too_many_active_proposals() {
    let stake_authority = Keypair::new();
    let stake_config = Pubkey::new_unique();
    let validator_vote = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let stake =
        find_validator_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_transaction =
        get_proposal_transaction_address(&proposal, &paladin_governance_program::id());
    let governance = Pubkey::new_unique(); // PDA doesn't matter here.
    let governance_config = GovernanceConfig {
        cooldown_period_seconds: 100_000_000,
        proposal_minimum_quorum: 5 * 10u32.pow(8), // 50%
        proposal_pass_threshold: 5 * 10u32.pow(8), // 50%
        stake_config_address: stake_config,
        voting_period_seconds: 100_000_000,
        stake_per_proposal: 1,
        governance_config: governance,
        cooldown_expires: 0, 
    };

    let mut context = setup().start_with_context().await;
    setup_author(&mut context, &stake_authority.pubkey(), 0).await;
    setup_stake(
        &mut context,
        &stake,
        stake_authority.pubkey(),
        validator_vote,
        0,
    )
    .await;
    setup_governance(&mut context, &governance, &governance_config).await;

    // Set up a proposal transaction account already initialized.
    setup_proposal_transaction(
        &mut context,
        &proposal_transaction,
        ProposalTransaction::default(),
    )
    .await;

    // Fund the proposal account.
    {
        let rent = context.banks_client.get_rent().await.unwrap();

        let space = std::mem::size_of::<Proposal>();
        let lamports = rent.minimum_balance(space);
        context.set_account(
            &proposal,
            &AccountSharedData::new(lamports, space, &paladin_governance_program::id()),
        );
    }

    let instruction = create_proposal(
        &stake_authority.pubkey(),
        &stake,
        &proposal,
        &proposal_transaction,
        &governance,
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
            InstructionError::Custom(PaladinGovernanceError::TooManyActiveProposals as u32)
        )
    );
}

#[tokio::test]
async fn success() {
    let stake_authority = Keypair::new();
    let stake_config = Pubkey::new_unique();
    let validator_vote = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let stake =
        find_validator_stake_pda(&validator_vote, &stake_config, &paladin_stake_program::id()).0;
    let proposal_transaction =
        get_proposal_transaction_address(&proposal, &paladin_governance_program::id());
    let governance = Pubkey::new_unique(); // PDA doesn't matter here.
    let governance_config = GovernanceConfig {
        cooldown_period_seconds: 100_000_000,
        proposal_minimum_quorum: 5 * 10u32.pow(8), // 50%
        proposal_pass_threshold: 5 * 10u32.pow(8), // 50%
        stake_config_address: stake_config,
        voting_period_seconds: 100_000_000,
        stake_per_proposal: 0,
        governance_config: governance,
        cooldown_expires: 0,
    };

    let mut context = setup().start_with_context().await;
    setup_author(&mut context, &stake_authority.pubkey(), 0).await;
    setup_stake(
        &mut context,
        &stake,
        stake_authority.pubkey(),
        validator_vote,
        0,
    )
    .await;
    setup_governance(&mut context, &governance, &governance_config).await;

    // Fund the proposal and proposal transaction accounts.
    {
        let rent = context.banks_client.get_rent().await.unwrap();

        let space = std::mem::size_of::<Proposal>();
        let lamports = rent.minimum_balance(space);
        context.set_account(
            &proposal,
            &AccountSharedData::new(lamports, space, &paladin_governance_program::id()),
        );

        let space = get_instance_packed_len(&ProposalTransaction::default()).unwrap();
        let lamports = rent.minimum_balance(space);
        context.set_account(
            &proposal_transaction,
            &AccountSharedData::new(lamports, 0, &system_program::id()),
        );
    }

    // For checks later.
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    let timestamp = clock.unix_timestamp;

    let instruction = create_proposal(
        &stake_authority.pubkey(),
        &stake,
        &proposal,
        &proposal_transaction,
        &governance,
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

    // Assert the proposal was created.
    let proposal_account = context
        .banks_client
        .get_account(proposal)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        bytemuck::from_bytes::<Proposal>(&proposal_account.data),
        &Proposal::new(&stake_authority.pubkey(), timestamp, governance_config)
    );

    // Assert the proposal transaction was created.
    let proposal_transaction_account = context
        .banks_client
        .get_account(proposal_transaction)
        .await
        .unwrap()
        .unwrap();
    let state = ProposalTransaction::try_from_slice(&proposal_transaction_account.data).unwrap();
    assert_eq!(state, ProposalTransaction::default());
}

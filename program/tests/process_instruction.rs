#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_governance_program::{
        error::PaladinGovernanceError,
        instruction::process_instruction,
        state::{
            get_proposal_transaction_address, get_treasury_address, GovernanceConfig, Proposal,
            ProposalStatus, ProposalTransaction,
        },
    },
    setup::{create_mock_proposal_transaction, setup, setup_proposal, setup_proposal_transaction},
    solana_program_test::*,
    solana_sdk::{
        account::AccountSharedData,
        borsh1::get_instance_packed_len,
        instruction::{AccountMeta, InstructionError},
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
        system_instruction, system_program,
        transaction::{Transaction, TransactionError},
    },
};

#[tokio::test]
async fn fail_proposal_incorrect_owner() {
    let proposal_address = Pubkey::new_unique();
    let proposal_transaction_address =
        get_proposal_transaction_address(&proposal_address, &paladin_governance_program::id());

    let instruction_index = 0u32;

    let mut context = setup().start_with_context().await;

    // Set up the proposal account with the incorrect owner.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let space = std::mem::size_of::<Proposal>();
        let lamports = rent.minimum_balance(space);
        context.set_account(
            &proposal_address,
            &AccountSharedData::new(lamports, space, &Pubkey::new_unique()), // Incorrect owner.
        );
    }

    let instruction = process_instruction(
        &proposal_address,
        &proposal_transaction_address,
        &[],
        instruction_index,
    );

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
        TransactionError::InstructionError(0, InstructionError::InvalidAccountOwner)
    );
}

#[tokio::test]
async fn fail_proposal_not_initialized() {
    let proposal_address = Pubkey::new_unique();
    let proposal_transaction_address =
        get_proposal_transaction_address(&proposal_address, &paladin_governance_program::id());

    let instruction_index = 0u32;

    let mut context = setup().start_with_context().await;

    // Set up the proposal account uninitialized.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let space = std::mem::size_of::<Proposal>();
        let lamports = rent.minimum_balance(space);
        context.set_account(
            &proposal_address,
            &AccountSharedData::new(lamports, space, &paladin_governance_program::id()),
        );
    }

    let instruction = process_instruction(
        &proposal_address,
        &proposal_transaction_address,
        &[],
        instruction_index,
    );

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
        TransactionError::InstructionError(0, InstructionError::UninitializedAccount)
    );
}

#[tokio::test]
async fn fail_proposal_not_accepted() {
    let proposal_address = Pubkey::new_unique();
    let proposal_transaction_address =
        get_proposal_transaction_address(&proposal_address, &paladin_governance_program::id());

    let instruction_index = 0u32;

    let mut context = setup().start_with_context().await;
    setup_proposal(
        &mut context,
        &proposal_address,
        &Pubkey::new_unique(),
        0,
        GovernanceConfig::default(),
        ProposalStatus::Voting, // Not accepted.
    )
    .await;

    let instruction = process_instruction(
        &proposal_address,
        &proposal_transaction_address,
        &[],
        instruction_index,
    );

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
            InstructionError::Custom(PaladinGovernanceError::ProposalNotAccepted as u32)
        )
    );
}

#[tokio::test]
async fn fail_proposal_transaction_incorrect_address() {
    let proposal_address = Pubkey::new_unique();
    let proposal_transaction_address = Pubkey::new_unique(); // Incorrect proposal transaction address.

    let instruction_index = 0u32;

    let mut context = setup().start_with_context().await;
    setup_proposal(
        &mut context,
        &proposal_address,
        &Pubkey::new_unique(),
        0,
        GovernanceConfig::default(),
        ProposalStatus::Accepted,
    )
    .await;

    let instruction = process_instruction(
        &proposal_address,
        &proposal_transaction_address,
        &[],
        instruction_index,
    );

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
            InstructionError::Custom(
                PaladinGovernanceError::IncorrectProposalTransactionAddress as u32
            )
        )
    );
}

#[tokio::test]
async fn fail_proposal_transaction_incorrect_owner() {
    let proposal_address = Pubkey::new_unique();
    let proposal_transaction_address =
        get_proposal_transaction_address(&proposal_address, &paladin_governance_program::id());

    let instruction_index = 0u32;

    let mut context = setup().start_with_context().await;
    setup_proposal(
        &mut context,
        &proposal_address,
        &Pubkey::new_unique(),
        0,
        GovernanceConfig::default(),
        ProposalStatus::Accepted,
    )
    .await;

    // Set up a proposal transaction account with incorrect owner.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let lamports =
            rent.minimum_balance(get_instance_packed_len(&ProposalTransaction::default()).unwrap());
        context.set_account(
            &proposal_transaction_address,
            &AccountSharedData::new(lamports, 0, &Pubkey::new_unique()), // Incorrect owner.
        );
    }

    let instruction = process_instruction(
        &proposal_address,
        &proposal_transaction_address,
        &[],
        instruction_index,
    );

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
        TransactionError::InstructionError(0, InstructionError::InvalidAccountOwner)
    );
}

#[tokio::test]
async fn fail_proposal_transaction_not_initialized() {
    let proposal_address = Pubkey::new_unique();
    let proposal_transaction_address =
        get_proposal_transaction_address(&proposal_address, &paladin_governance_program::id());

    let instruction_index = 0u32;

    let mut context = setup().start_with_context().await;
    setup_proposal(
        &mut context,
        &proposal_address,
        &Pubkey::new_unique(),
        0,
        GovernanceConfig::default(),
        ProposalStatus::Accepted,
    )
    .await;

    // Set up a proposal transaction account uninitialized.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let lamports =
            rent.minimum_balance(get_instance_packed_len(&ProposalTransaction::default()).unwrap());
        context.set_account(
            &proposal_transaction_address,
            &AccountSharedData::new(lamports, 0, &paladin_governance_program::id()),
        );
    }

    let instruction = process_instruction(
        &proposal_address,
        &proposal_transaction_address,
        &[],
        instruction_index,
    );

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
        TransactionError::InstructionError(0, InstructionError::UninitializedAccount)
    );
}

#[tokio::test]
async fn fail_invalid_instruction_index() {
    let proposal_address = Pubkey::new_unique();
    let proposal_transaction_address =
        get_proposal_transaction_address(&proposal_address, &paladin_governance_program::id());

    let proposal_transaction = create_mock_proposal_transaction(&[
        &Pubkey::new_unique(), // One instruction.
    ]);
    let instruction_index = 300u32; // Invalid instruction index.

    let mut context = setup().start_with_context().await;
    setup_proposal(
        &mut context,
        &proposal_address,
        &Pubkey::new_unique(),
        0,
        GovernanceConfig::default(),
        ProposalStatus::Accepted,
    )
    .await;
    setup_proposal_transaction(
        &mut context,
        &proposal_transaction_address,
        proposal_transaction,
    )
    .await;

    let instruction = process_instruction(
        &proposal_address,
        &proposal_transaction_address,
        &[],
        instruction_index,
    );

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
            InstructionError::Custom(PaladinGovernanceError::InvalidTransactionIndex as u32)
        )
    );
}

#[tokio::test]
async fn fail_instruction_already_executed() {
    let proposal_address = Pubkey::new_unique();
    let proposal_transaction_address =
        get_proposal_transaction_address(&proposal_address, &paladin_governance_program::id());

    let mut proposal_transaction =
        create_mock_proposal_transaction(&[&Pubkey::new_unique(), &Pubkey::new_unique()]);
    proposal_transaction.instructions[0].executed = true; // Instruction already executed.
    let instruction_index = 0u32; // Instruction already executed.

    let mut context = setup().start_with_context().await;
    setup_proposal(
        &mut context,
        &proposal_address,
        &Pubkey::new_unique(),
        0,
        GovernanceConfig::default(),
        ProposalStatus::Accepted,
    )
    .await;
    setup_proposal_transaction(
        &mut context,
        &proposal_transaction_address,
        proposal_transaction,
    )
    .await;

    let instruction = process_instruction(
        &proposal_address,
        &proposal_transaction_address,
        &[],
        instruction_index,
    );

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
            InstructionError::Custom(PaladinGovernanceError::InstructionAlreadyExecuted as u32)
        )
    );
}

#[tokio::test]
async fn fail_previous_instruction_not_executed() {
    let proposal_address = Pubkey::new_unique();
    let proposal_transaction_address =
        get_proposal_transaction_address(&proposal_address, &paladin_governance_program::id());

    let proposal_transaction =
        create_mock_proposal_transaction(&[&Pubkey::new_unique(), &Pubkey::new_unique()]);
    let instruction_index = 1u32; // Instruction 0 was not executed yet.

    let mut context = setup().start_with_context().await;
    setup_proposal(
        &mut context,
        &proposal_address,
        &Pubkey::new_unique(),
        0,
        GovernanceConfig::default(),
        ProposalStatus::Accepted,
    )
    .await;
    setup_proposal_transaction(
        &mut context,
        &proposal_transaction_address,
        proposal_transaction,
    )
    .await;

    let instruction = process_instruction(
        &proposal_address,
        &proposal_transaction_address,
        &[],
        instruction_index,
    );

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
            InstructionError::Custom(
                PaladinGovernanceError::PreviousInstructionHasNotBeenExecuted as u32
            )
        )
    );
}

#[tokio::test]
async fn fail_instruction_two_back_not_executed() {
    let proposal_address = Pubkey::new_unique();
    let proposal_transaction_address =
        get_proposal_transaction_address(&proposal_address, &paladin_governance_program::id());

    let proposal_transaction = create_mock_proposal_transaction(&[
        &Pubkey::new_unique(),
        &Pubkey::new_unique(),
        &Pubkey::new_unique(),
    ]);

    let mut context = setup().start_with_context().await;
    setup_proposal(
        &mut context,
        &proposal_address,
        &Pubkey::new_unique(),
        0,
        GovernanceConfig::default(),
        ProposalStatus::Accepted,
    )
    .await;
    setup_proposal_transaction(
        &mut context,
        &proposal_transaction_address,
        proposal_transaction,
    )
    .await;

    let instruction = process_instruction(&proposal_address, &proposal_transaction_address, &[], 2);

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
            InstructionError::Custom(
                PaladinGovernanceError::PreviousInstructionHasNotBeenExecuted as u32
            )
        )
    );
}

#[allow(clippy::arithmetic_side_effects)]
#[tokio::test]
async fn success() {
    let proposal_address = Pubkey::new_unique();
    let proposal_transaction_address =
        get_proposal_transaction_address(&proposal_address, &paladin_governance_program::id());

    let stake_config_address = Pubkey::new_unique();
    let governance_config = GovernanceConfig {
        cooldown_period_seconds: 0,
        proposal_minimum_quorum: 0,
        proposal_pass_threshold: 0,
        stake_config_address: stake_config_address,
        voting_period_seconds: 0,
        stake_per_proposal: 0,
    };

    let treasury = get_treasury_address(&stake_config_address, &paladin_governance_program::id());
    let alice = Keypair::new();

    let treasury_starting_lamports = 500_000_000;
    let alice_starting_lamports = 350_000_000;

    // Transfer amounts.
    let treasury_to_alice_lamports = 100_000_000;
    let alice_to_treasury_lamports = 50_000_000;

    let proposal_transaction = ProposalTransaction {
        instructions: vec![
            (&system_instruction::transfer(&treasury, &alice.pubkey(), treasury_to_alice_lamports))
                .into(),
            (&system_instruction::transfer(&alice.pubkey(), &treasury, alice_to_treasury_lamports))
                .into(),
        ],
    };

    let mut context = setup().start_with_context().await;
    setup_proposal(
        &mut context,
        &proposal_address,
        &Pubkey::new_unique(),
        0,
        governance_config,
        ProposalStatus::Accepted,
    )
    .await;
    setup_proposal_transaction(
        &mut context,
        &proposal_transaction_address,
        proposal_transaction,
    )
    .await;

    // Set up treasury and alice with some lamports for transferring.
    {
        context.set_account(
            &treasury,
            &AccountSharedData::new(treasury_starting_lamports, 0, &system_program::id()), // System-owned.
        );
        context.set_account(
            &alice.pubkey(),
            &AccountSharedData::new(alice_starting_lamports, 0, &system_program::id()),
        );
    }

    // Execute the first instruction.
    {
        let instruction = process_instruction(
            &proposal_address,
            &proposal_transaction_address,
            &[
                AccountMeta::new(treasury, false),
                AccountMeta::new(alice.pubkey(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            0, // First instruction.
        );

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&context.payer.pubkey()),
            &[&context.payer], // Note treasury not signer (PDA).
            context.last_blockhash,
        );

        context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap();

        // Assert lamports were transferred from Alice to Bob.
        assert_eq!(
            context
                .banks_client
                .get_account(treasury)
                .await
                .unwrap()
                .unwrap()
                .lamports,
            treasury_starting_lamports - treasury_to_alice_lamports
        );
        assert_eq!(
            context
                .banks_client
                .get_account(alice.pubkey())
                .await
                .unwrap()
                .unwrap()
                .lamports,
            alice_starting_lamports + treasury_to_alice_lamports
        );
    }

    // Execute the second instruction.
    {
        let blockhash = context.banks_client.get_latest_blockhash().await.unwrap();

        let instruction = process_instruction(
            &proposal_address,
            &proposal_transaction_address,
            &[
                AccountMeta::new(alice.pubkey(), true),
                AccountMeta::new(treasury, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            1, // Second instruction.
        );

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&context.payer.pubkey()),
            &[&context.payer, &alice],
            blockhash,
        );

        context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap();

        // Assert lamports were transferred from Bob to Alice.
        assert_eq!(
            context
                .banks_client
                .get_account(treasury)
                .await
                .unwrap()
                .unwrap()
                .lamports,
            treasury_starting_lamports - treasury_to_alice_lamports + alice_to_treasury_lamports
        );
        assert_eq!(
            context
                .banks_client
                .get_account(alice.pubkey())
                .await
                .unwrap()
                .unwrap()
                .lamports,
            alice_starting_lamports + treasury_to_alice_lamports - alice_to_treasury_lamports
        );
    }

    // Assert - The proposal has been marked as processed.
    let proposal = context
        .banks_client
        .get_account(proposal_address)
        .await
        .unwrap()
        .unwrap();
    let proposal = bytemuck::from_bytes::<Proposal>(&proposal.data);
    assert_eq!(proposal.status, ProposalStatus::Processed);
}

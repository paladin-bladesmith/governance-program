#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_governance_program::{
        error::PaladinGovernanceError,
        instruction::process_instruction,
        state::{
            get_proposal_transaction_address, Config, Proposal, ProposalStatus, ProposalTransaction,
        },
    },
    setup::{create_mock_proposal_transaction, setup, setup_proposal, setup_proposal_transaction},
    solana_program_test::*,
    solana_sdk::{
        account::AccountSharedData,
        borsh1::get_instance_packed_len,
        instruction::InstructionError,
        pubkey::Pubkey,
        signer::Signer,
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
        Config::default(),
        ProposalStatus::Voting, // Not accepted.
    )
    .await;

    let instruction = process_instruction(
        &proposal_address,
        &proposal_transaction_address,
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
        Config::default(),
        ProposalStatus::Accepted,
    )
    .await;

    let instruction = process_instruction(
        &proposal_address,
        &proposal_transaction_address,
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
        Config::default(),
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
        Config::default(),
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
        Config::default(),
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
        Config::default(),
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
        Config::default(),
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

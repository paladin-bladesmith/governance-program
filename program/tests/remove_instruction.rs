#![cfg(feature = "test-sbf")]

mod setup;

use {
    borsh::BorshDeserialize,
    paladin_governance_program::{
        error::PaladinGovernanceError,
        instruction::remove_instruction,
        state::{
            get_proposal_transaction_address, GovernanceConfig, Proposal, ProposalStatus,
            ProposalTransaction,
        },
    },
    setup::{create_mock_proposal_transaction, setup, setup_proposal, setup_proposal_transaction},
    solana_program_test::*,
    solana_sdk::{
        account::AccountSharedData,
        borsh1::get_instance_packed_len,
        instruction::InstructionError,
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
        transaction::{Transaction, TransactionError},
    },
};

#[tokio::test]
async fn fail_stake_authority_not_signer() {
    let stake_authority = Keypair::new();
    let proposal_address = Pubkey::new_unique();

    let proposal_transaction_address =
        get_proposal_transaction_address(&proposal_address, &paladin_governance_program::id());

    let instruction_index = 0u32;

    let context = setup().start_with_context().await;

    let mut instruction = remove_instruction(
        &stake_authority.pubkey(),
        &proposal_address,
        &proposal_transaction_address,
        instruction_index,
    );
    instruction.accounts[0].is_signer = false; // Stake authority not signer.

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer], // Missing stake authority.
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
async fn fail_proposal_incorrect_owner() {
    let stake_authority = Keypair::new();
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

    let instruction = remove_instruction(
        &stake_authority.pubkey(),
        &proposal_address,
        &proposal_transaction_address,
        instruction_index,
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

    let instruction = remove_instruction(
        &stake_authority.pubkey(),
        &proposal_address,
        &proposal_transaction_address,
        instruction_index,
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
async fn fail_stake_authority_not_author() {
    let stake_authority = Keypair::new();
    let proposal_address = Pubkey::new_unique();

    let proposal_transaction_address =
        get_proposal_transaction_address(&proposal_address, &paladin_governance_program::id());

    let instruction_index = 0u32;

    let mut context = setup().start_with_context().await;
    setup_proposal(
        &mut context,
        &proposal_address,
        &Pubkey::new_unique(), // Stake authority not author.
        0,
        GovernanceConfig::default(),
        ProposalStatus::Draft,
    )
    .await;

    let instruction = remove_instruction(
        &stake_authority.pubkey(),
        &proposal_address,
        &proposal_transaction_address,
        instruction_index,
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
async fn fail_proposal_not_in_draft_stage() {
    let stake_authority = Keypair::new();
    let proposal_address = Pubkey::new_unique();

    let proposal_transaction_address =
        get_proposal_transaction_address(&proposal_address, &paladin_governance_program::id());

    let instruction_index = 0u32;

    let mut context = setup().start_with_context().await;
    setup_proposal(
        &mut context,
        &proposal_address,
        &stake_authority.pubkey(),
        0,
        GovernanceConfig::default(),
        ProposalStatus::Voting, // Not in draft stage.
    )
    .await;

    let instruction = remove_instruction(
        &stake_authority.pubkey(),
        &proposal_address,
        &proposal_transaction_address,
        instruction_index,
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
            InstructionError::Custom(PaladinGovernanceError::ProposalIsImmutable as u32)
        )
    );
}

#[tokio::test]
async fn fail_proposal_transaction_incorrect_address() {
    let stake_authority = Keypair::new();
    let proposal_address = Pubkey::new_unique();

    let proposal_transaction_address = Pubkey::new_unique(); // Incorrect proposal transaction address.

    let instruction_index = 0u32;

    let mut context = setup().start_with_context().await;
    setup_proposal(
        &mut context,
        &proposal_address,
        &stake_authority.pubkey(),
        0,
        GovernanceConfig::default(),
        ProposalStatus::Draft,
    )
    .await;

    let instruction = remove_instruction(
        &stake_authority.pubkey(),
        &proposal_address,
        &proposal_transaction_address,
        instruction_index,
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
async fn fail_proposal_transaction_incorrect_owner() {
    let stake_authority = Keypair::new();
    let proposal_address = Pubkey::new_unique();

    let proposal_transaction_address =
        get_proposal_transaction_address(&proposal_address, &paladin_governance_program::id());

    let instruction_index = 0u32;

    let mut context = setup().start_with_context().await;
    setup_proposal(
        &mut context,
        &proposal_address,
        &stake_authority.pubkey(),
        0,
        GovernanceConfig::default(),
        ProposalStatus::Draft,
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

    let instruction = remove_instruction(
        &stake_authority.pubkey(),
        &proposal_address,
        &proposal_transaction_address,
        instruction_index,
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
async fn fail_proposal_transaction_not_initialized() {
    let stake_authority = Keypair::new();
    let proposal_address = Pubkey::new_unique();

    let proposal_transaction_address =
        get_proposal_transaction_address(&proposal_address, &paladin_governance_program::id());

    let instruction_index = 0u32;

    let mut context = setup().start_with_context().await;
    setup_proposal(
        &mut context,
        &proposal_address,
        &stake_authority.pubkey(),
        0,
        GovernanceConfig::default(),
        ProposalStatus::Draft,
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

    let instruction = remove_instruction(
        &stake_authority.pubkey(),
        &proposal_address,
        &proposal_transaction_address,
        instruction_index,
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
async fn fail_invalid_instruction_index() {
    let stake_authority = Keypair::new();
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
        &stake_authority.pubkey(),
        0,
        GovernanceConfig::default(),
        ProposalStatus::Draft,
    )
    .await;
    setup_proposal_transaction(
        &mut context,
        &proposal_transaction_address,
        proposal_transaction,
    )
    .await;

    let instruction = remove_instruction(
        &stake_authority.pubkey(),
        &proposal_address,
        &proposal_transaction_address,
        instruction_index,
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
            InstructionError::Custom(PaladinGovernanceError::InvalidTransactionIndex as u32)
        )
    );
}

#[tokio::test]
async fn success() {
    let stake_authority = Keypair::new();
    let proposal_address = Pubkey::new_unique();

    let proposal_transaction_address =
        get_proposal_transaction_address(&proposal_address, &paladin_governance_program::id());

    let instruction_program_id_0 = Pubkey::new_unique();
    let instruction_program_id_1 = Pubkey::new_unique();
    let instruction_program_id_2 = Pubkey::new_unique();
    let instruction_program_id_3 = Pubkey::new_unique();
    let instruction_program_id_4 = Pubkey::new_unique();

    let proposal_transaction = create_mock_proposal_transaction(&[
        &instruction_program_id_0,
        &instruction_program_id_1,
        &instruction_program_id_2,
        &instruction_program_id_3,
        &instruction_program_id_4,
    ]);

    let mut context = setup().start_with_context().await;
    setup_proposal(
        &mut context,
        &proposal_address,
        &stake_authority.pubkey(),
        0,
        GovernanceConfig::default(),
        ProposalStatus::Draft,
    )
    .await;
    setup_proposal_transaction(
        &mut context,
        &proposal_transaction_address,
        proposal_transaction,
    )
    .await;

    // Remove instruction zero.
    {
        let instruction = remove_instruction(
            &stake_authority.pubkey(),
            &proposal_address,
            &proposal_transaction_address,
            0,
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

        // Assert the proposal transaction account was updated.
        let proposal_transaction_account = context
            .banks_client
            .get_account(proposal_transaction_address)
            .await
            .unwrap()
            .unwrap();
        let proposal_transaction_state =
            ProposalTransaction::try_from_slice(&proposal_transaction_account.data).unwrap();

        assert_eq!(proposal_transaction_state.instructions.len(), 4);

        // Assert program ID 0 was removed.
        assert_eq!(
            proposal_transaction_state
                .instructions
                .iter()
                .map(|i| i.program_id)
                .collect::<Vec<_>>(),
            vec![
                instruction_program_id_1,
                instruction_program_id_2,
                instruction_program_id_3,
                instruction_program_id_4
            ]
        );
    }

    // Remove instruction three.
    {
        let instruction = remove_instruction(
            &stake_authority.pubkey(),
            &proposal_address,
            &proposal_transaction_address,
            3,
        );

        let blockhash = context.banks_client.get_latest_blockhash().await.unwrap();

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&context.payer.pubkey()),
            &[&context.payer, &stake_authority],
            blockhash,
        );

        context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap();

        // Assert the proposal transaction account was updated.
        let proposal_transaction_account = context
            .banks_client
            .get_account(proposal_transaction_address)
            .await
            .unwrap()
            .unwrap();
        let proposal_transaction_state =
            ProposalTransaction::try_from_slice(&proposal_transaction_account.data).unwrap();

        assert_eq!(proposal_transaction_state.instructions.len(), 3);

        // Assert program ID 4 was removed from index 3.
        assert_eq!(
            proposal_transaction_state
                .instructions
                .iter()
                .map(|i| i.program_id)
                .collect::<Vec<_>>(),
            vec![
                instruction_program_id_1,
                instruction_program_id_2,
                instruction_program_id_3
            ]
        );
    }

    // Remove instruction zero again (different instruction now).
    {
        let instruction = remove_instruction(
            &stake_authority.pubkey(),
            &proposal_address,
            &proposal_transaction_address,
            0,
        );

        let blockhash = context.banks_client.get_latest_blockhash().await.unwrap();

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&context.payer.pubkey()),
            &[&context.payer, &stake_authority],
            blockhash,
        );

        context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap();

        // Assert the proposal transaction account was updated.
        let proposal_transaction_account = context
            .banks_client
            .get_account(proposal_transaction_address)
            .await
            .unwrap()
            .unwrap();
        let proposal_transaction_state =
            ProposalTransaction::try_from_slice(&proposal_transaction_account.data).unwrap();

        assert_eq!(proposal_transaction_state.instructions.len(), 2);

        // Assert program ID 1 was removed from index 0.
        assert_eq!(
            proposal_transaction_state
                .instructions
                .iter()
                .map(|i| i.program_id)
                .collect::<Vec<_>>(),
            vec![instruction_program_id_2, instruction_program_id_3]
        );
    }
}

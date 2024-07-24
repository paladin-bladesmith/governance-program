#![cfg(feature = "test-sbf")]

mod setup;

use {
    borsh::BorshDeserialize,
    paladin_governance_program::{
        error::PaladinGovernanceError,
        instruction::insert_instruction,
        state::{
            get_proposal_transaction_address, Config, Proposal, ProposalAccountMeta,
            ProposalInstruction, ProposalStatus, ProposalTransaction,
        },
    },
    setup::{setup, setup_proposal, setup_proposal_transaction},
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

    let mut context = setup().start_with_context().await;

    let mut instruction = insert_instruction(
        &stake_authority.pubkey(),
        &proposal_address,
        &proposal_transaction_address,
        &Pubkey::new_unique(),
        vec![],
        vec![],
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

    let instruction = insert_instruction(
        &stake_authority.pubkey(),
        &proposal_address,
        &proposal_transaction_address,
        &Pubkey::new_unique(),
        vec![],
        vec![],
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

    let instruction = insert_instruction(
        &stake_authority.pubkey(),
        &proposal_address,
        &proposal_transaction_address,
        &Pubkey::new_unique(),
        vec![],
        vec![],
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

    let mut context = setup().start_with_context().await;
    setup_proposal(
        &mut context,
        &proposal_address,
        &Pubkey::new_unique(), // Stake authority not author.
        0,
        Config::default(),
        ProposalStatus::Draft,
    )
    .await;

    let instruction = insert_instruction(
        &stake_authority.pubkey(),
        &proposal_address,
        &proposal_transaction_address,
        &Pubkey::new_unique(),
        vec![],
        vec![],
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

    let mut context = setup().start_with_context().await;
    setup_proposal(
        &mut context,
        &proposal_address,
        &stake_authority.pubkey(),
        0,
        Config::default(),
        ProposalStatus::Voting, // Not in draft stage.
    )
    .await;

    let instruction = insert_instruction(
        &stake_authority.pubkey(),
        &proposal_address,
        &proposal_transaction_address,
        &Pubkey::new_unique(),
        vec![],
        vec![],
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

    let mut context = setup().start_with_context().await;
    setup_proposal(
        &mut context,
        &proposal_address,
        &stake_authority.pubkey(),
        0,
        Config::default(),
        ProposalStatus::Draft,
    )
    .await;

    let instruction = insert_instruction(
        &stake_authority.pubkey(),
        &proposal_address,
        &proposal_transaction_address,
        &Pubkey::new_unique(),
        vec![],
        vec![],
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

    let mut context = setup().start_with_context().await;
    setup_proposal(
        &mut context,
        &proposal_address,
        &stake_authority.pubkey(),
        0,
        Config::default(),
        ProposalStatus::Draft,
    )
    .await;

    // Set up the proposal transaction account with the incorrect owner.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let space = std::mem::size_of::<ProposalTransaction>();
        let lamports = rent.minimum_balance(space);
        context.set_account(
            &proposal_transaction_address,
            &AccountSharedData::new(lamports, space, &Pubkey::new_unique()), // Incorrect owner.
        );
    }

    let instruction = insert_instruction(
        &stake_authority.pubkey(),
        &proposal_address,
        &proposal_transaction_address,
        &Pubkey::new_unique(),
        vec![],
        vec![],
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

    let mut context = setup().start_with_context().await;
    setup_proposal(
        &mut context,
        &proposal_address,
        &stake_authority.pubkey(),
        0,
        Config::default(),
        ProposalStatus::Draft,
    )
    .await;

    // Set up the proposal transaction account uninitialized.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let space = std::mem::size_of::<ProposalTransaction>();
        let lamports = rent.minimum_balance(space);
        context.set_account(
            &proposal_transaction_address,
            &AccountSharedData::new(lamports, 0, &paladin_governance_program::id()),
        );
    }

    let instruction = insert_instruction(
        &stake_authority.pubkey(),
        &proposal_address,
        &proposal_transaction_address,
        &Pubkey::new_unique(),
        vec![],
        vec![],
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
async fn success() {
    let stake_authority = Keypair::new();
    let proposal_address = Pubkey::new_unique();

    let proposal_transaction_address =
        get_proposal_transaction_address(&proposal_address, &paladin_governance_program::id());

    let instruction_program_id = Pubkey::new_unique();
    let instruction_account_metas = vec![
        ProposalAccountMeta {
            pubkey: Pubkey::new_unique(),
            is_signer: false,
            is_writable: true,
        },
        ProposalAccountMeta {
            pubkey: Pubkey::new_unique(),
            is_signer: false,
            is_writable: false,
        },
    ];
    let instruction_data = vec![1, 2, 3];

    let mut context = setup().start_with_context().await;
    setup_proposal(
        &mut context,
        &proposal_address,
        &stake_authority.pubkey(),
        0,
        Config::default(),
        ProposalStatus::Draft,
    )
    .await;
    setup_proposal_transaction(
        &mut context,
        &proposal_transaction_address,
        ProposalTransaction::default(),
    )
    .await;

    // Fund the proposal transaction account to cover the new rent-exemption.
    #[allow(clippy::arithmetic_side_effects)]
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let new_instruction_len = get_instance_packed_len(&instruction_program_id).unwrap()
            + get_instance_packed_len(&instruction_account_metas).unwrap()
            + get_instance_packed_len(&instruction_data).unwrap();
        let additional_lamports = rent.minimum_balance(new_instruction_len);

        let mut proposal_transaction_account = context
            .banks_client
            .get_account(proposal_transaction_address)
            .await
            .unwrap()
            .unwrap();
        proposal_transaction_account.lamports += additional_lamports;
        context.set_account(
            &proposal_transaction_address,
            &proposal_transaction_account.into(),
        );
    }

    let instruction = insert_instruction(
        &stake_authority.pubkey(),
        &proposal_address,
        &proposal_transaction_address,
        &instruction_program_id,
        instruction_account_metas.clone(),
        instruction_data.clone(),
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
    assert_eq!(proposal_transaction_state.instructions.len(), 1);
    assert_eq!(
        proposal_transaction_state.instructions[0],
        ProposalInstruction {
            program_id: instruction_program_id,
            accounts: instruction_account_metas,
            data: instruction_data,
            executed: false,
        }
    );

    // Add another instruction.

    let instruction_program_id = Pubkey::new_unique();
    let instruction_account_metas = vec![
        ProposalAccountMeta {
            pubkey: Pubkey::new_unique(),
            is_signer: false,
            is_writable: false,
        },
        ProposalAccountMeta {
            pubkey: Pubkey::new_unique(),
            is_signer: false,
            is_writable: true,
        },
    ];
    let instruction_data = vec![4, 5, 6];

    // Fund the proposal transaction account to cover the new rent-exemption.
    #[allow(clippy::arithmetic_side_effects)]
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let new_instruction_len = get_instance_packed_len(&instruction_program_id).unwrap()
            + get_instance_packed_len(&instruction_account_metas).unwrap()
            + get_instance_packed_len(&instruction_data).unwrap();
        let additional_lamports = rent.minimum_balance(new_instruction_len);

        let mut proposal_transaction_account = context
            .banks_client
            .get_account(proposal_transaction_address)
            .await
            .unwrap()
            .unwrap();
        proposal_transaction_account.lamports += additional_lamports;
        context.set_account(
            &proposal_transaction_address,
            &proposal_transaction_account.into(),
        );
    }

    let instruction = insert_instruction(
        &stake_authority.pubkey(),
        &proposal_address,
        &proposal_transaction_address,
        &instruction_program_id,
        instruction_account_metas.clone(),
        instruction_data.clone(),
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
    assert_eq!(proposal_transaction_state.instructions.len(), 2);
    assert_eq!(
        proposal_transaction_state.instructions[1],
        ProposalInstruction {
            program_id: instruction_program_id,
            accounts: instruction_account_metas,
            data: instruction_data,
            executed: false,
        }
    );
}

#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_governance_program::{
        error::PaladinGovernanceError,
        instruction::{process_instruction, transfer_from_treasury},
        state::{
            get_governance_address, get_proposal_transaction_address, get_treasury_address, Config,
            ProposalStatus, ProposalTransaction,
        },
    },
    setup::{setup, setup_governance, setup_proposal, setup_proposal_transaction},
    solana_program_test::*,
    solana_sdk::{
        account::AccountSharedData,
        instruction::{AccountMeta, InstructionError},
        pubkey::Pubkey,
        signer::Signer,
        system_program,
        transaction::{Transaction, TransactionError},
    },
};

fn proposal_transaction_with_transfer_from_treasury_instruction(
    governance_config_address: &Pubkey,
    treasury_address: &Pubkey,
    destination_address: &Pubkey,
    amount: u64,
) -> ProposalTransaction {
    ProposalTransaction {
        instructions: vec![
            (&paladin_governance_program::instruction::transfer_from_treasury(
                governance_config_address,
                treasury_address,
                destination_address,
                amount,
            ))
                .into(),
        ],
    }
}

#[tokio::test]
async fn fail_governance_not_signer() {
    let stake_config_address = Pubkey::new_unique();

    let governance =
        get_governance_address(&stake_config_address, &paladin_governance_program::id());
    let treasury = get_treasury_address(&stake_config_address, &paladin_governance_program::id());

    let destination = Pubkey::new_unique();

    let amount = 100_000_000;

    let mut context = setup().start_with_context().await;

    // Try just invoking the instruction directly.
    let mut instruction = transfer_from_treasury(&governance, &treasury, &destination, amount);
    instruction.accounts[0].is_signer = false; // Governance not signer.

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
        TransactionError::InstructionError(0, InstructionError::MissingRequiredSignature)
    );
}

#[tokio::test]
async fn fail_governance_incorrect_owner() {
    let stake_config_address = Pubkey::new_unique();

    let governance =
        get_governance_address(&stake_config_address, &paladin_governance_program::id());
    let treasury = Pubkey::new_unique(); // Incorrect treasury address.

    let destination = Pubkey::new_unique();

    let proposal_address = Pubkey::new_unique();
    let proposal_transaction_address =
        get_proposal_transaction_address(&proposal_address, &paladin_governance_program::id());

    let amount = 100_000_000;

    let governance_config = Config::new(
        /* cooldown_period_seconds */ 0,
        /* proposal_acceptance_threshold */ 0,
        /* proposal_rejection_threshold */ 0,
        /* signer_bump_seed */ 0,
        /* stake_config_address */ &stake_config_address,
        /* voting_period_seconds */ 0,
    );

    let mut context = setup().start_with_context().await;
    setup_governance(
        &mut context,
        &governance,
        governance_config.cooldown_period_seconds,
        governance_config.proposal_acceptance_threshold,
        governance_config.proposal_rejection_threshold,
        &governance_config.stake_config_address,
        governance_config.voting_period_seconds,
    )
    .await;
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
        proposal_transaction_with_transfer_from_treasury_instruction(
            &governance,
            &treasury,
            &destination,
            amount,
        ),
    )
    .await;

    // Set up a governance account with an incorrect owner.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let space = std::mem::size_of::<Config>();
        let lamports = rent.minimum_balance(space);
        context.set_account(
            &governance,
            &AccountSharedData::new(lamports, space, &Pubkey::new_unique()), // Incorrect owner.
        );
    }

    let instruction = process_instruction(
        &proposal_address,
        &proposal_transaction_address,
        &[
            AccountMeta::new(governance, false),
            AccountMeta::new(treasury, false),
            AccountMeta::new(destination, false),
            AccountMeta::new_readonly(paladin_governance_program::id(), false),
        ],
        0,
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
async fn fail_governance_not_initialized() {
    let stake_config_address = Pubkey::new_unique();

    let governance =
        get_governance_address(&stake_config_address, &paladin_governance_program::id());
    let treasury = Pubkey::new_unique(); // Incorrect treasury address.

    let destination = Pubkey::new_unique();

    let proposal_address = Pubkey::new_unique();
    let proposal_transaction_address =
        get_proposal_transaction_address(&proposal_address, &paladin_governance_program::id());

    let amount = 100_000_000;

    let governance_config = Config::new(
        /* cooldown_period_seconds */ 0,
        /* proposal_acceptance_threshold */ 0,
        /* proposal_rejection_threshold */ 0,
        /* signer_bump_seed */ 0,
        /* stake_config_address */ &stake_config_address,
        /* voting_period_seconds */ 0,
    );

    let mut context = setup().start_with_context().await;
    setup_governance(
        &mut context,
        &governance,
        governance_config.cooldown_period_seconds,
        governance_config.proposal_acceptance_threshold,
        governance_config.proposal_rejection_threshold,
        &governance_config.stake_config_address,
        governance_config.voting_period_seconds,
    )
    .await;
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
        proposal_transaction_with_transfer_from_treasury_instruction(
            &governance,
            &treasury,
            &destination,
            amount,
        ),
    )
    .await;

    // Set up an uninitialized governance account.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let lamports = rent.minimum_balance(0);
        context.set_account(
            &governance,
            &AccountSharedData::new(lamports, 0, &paladin_governance_program::id()),
        );
    }

    let instruction = process_instruction(
        &proposal_address,
        &proposal_transaction_address,
        &[
            AccountMeta::new(governance, false),
            AccountMeta::new(treasury, false),
            AccountMeta::new(destination, false),
            AccountMeta::new_readonly(paladin_governance_program::id(), false),
        ],
        0,
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
async fn fail_governance_incorrect_address() {
    let stake_config_address = Pubkey::new_unique();

    let governance = Pubkey::new_unique(); // Incorrect governance address.
    let treasury = get_treasury_address(&stake_config_address, &paladin_governance_program::id());

    let destination = Pubkey::new_unique();

    let proposal_address = Pubkey::new_unique();
    let proposal_transaction_address =
        get_proposal_transaction_address(&proposal_address, &paladin_governance_program::id());

    let amount = 100_000_000;

    let governance_config = Config::new(
        /* cooldown_period_seconds */ 0,
        /* proposal_acceptance_threshold */ 0,
        /* proposal_rejection_threshold */ 0,
        /* signer_bump_seed */ 0,
        /* stake_config_address */ &stake_config_address,
        /* voting_period_seconds */ 0,
    );

    let mut context = setup().start_with_context().await;
    setup_governance(
        &mut context,
        &governance,
        governance_config.cooldown_period_seconds,
        governance_config.proposal_acceptance_threshold,
        governance_config.proposal_rejection_threshold,
        &governance_config.stake_config_address,
        governance_config.voting_period_seconds,
    )
    .await;
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
        proposal_transaction_with_transfer_from_treasury_instruction(
            &governance,
            &treasury,
            &destination,
            amount,
        ),
    )
    .await;

    let instruction = process_instruction(
        &proposal_address,
        &proposal_transaction_address,
        &[
            AccountMeta::new(governance, false),
            AccountMeta::new(treasury, false),
            AccountMeta::new(destination, false),
            AccountMeta::new_readonly(paladin_governance_program::id(), false),
        ],
        0,
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
            InstructionError::PrivilegeEscalation, /* Can't form the governance PDA signature
                                                    * with the wrong address. */
        )
    );
}

#[tokio::test]
async fn fail_treasury_incorrect_address() {
    let stake_config_address = Pubkey::new_unique();

    let governance =
        get_governance_address(&stake_config_address, &paladin_governance_program::id());
    let treasury = Pubkey::new_unique(); // Incorrect treasury address.

    let destination = Pubkey::new_unique();

    let proposal_address = Pubkey::new_unique();
    let proposal_transaction_address =
        get_proposal_transaction_address(&proposal_address, &paladin_governance_program::id());

    let amount = 100_000_000;

    let governance_config = Config::new(
        /* cooldown_period_seconds */ 0,
        /* proposal_acceptance_threshold */ 0,
        /* proposal_rejection_threshold */ 0,
        /* signer_bump_seed */ 0,
        /* stake_config_address */ &stake_config_address,
        /* voting_period_seconds */ 0,
    );

    let mut context = setup().start_with_context().await;
    setup_governance(
        &mut context,
        &governance,
        governance_config.cooldown_period_seconds,
        governance_config.proposal_acceptance_threshold,
        governance_config.proposal_rejection_threshold,
        &governance_config.stake_config_address,
        governance_config.voting_period_seconds,
    )
    .await;
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
        proposal_transaction_with_transfer_from_treasury_instruction(
            &governance,
            &treasury,
            &destination,
            amount,
        ),
    )
    .await;

    let instruction = process_instruction(
        &proposal_address,
        &proposal_transaction_address,
        &[
            AccountMeta::new(governance, false),
            AccountMeta::new(treasury, false),
            AccountMeta::new(destination, false),
            AccountMeta::new_readonly(paladin_governance_program::id(), false),
        ],
        0,
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
            InstructionError::Custom(PaladinGovernanceError::IncorrectTreasuryAddress as u32)
        )
    );
}

#[tokio::test]
async fn success() {
    let stake_config_address = Pubkey::new_unique();

    let governance =
        get_governance_address(&stake_config_address, &paladin_governance_program::id());
    let treasury = get_treasury_address(&stake_config_address, &paladin_governance_program::id());

    let destination = Pubkey::new_unique();

    let proposal_address = Pubkey::new_unique();
    let proposal_transaction_address =
        get_proposal_transaction_address(&proposal_address, &paladin_governance_program::id());

    let treasury_starting_lamports = 500_000_000;
    let destination_starting_lamports = 350_000_000;
    let amount = 100_000_000;

    let governance_config = Config::new(
        /* cooldown_period_seconds */ 0,
        /* proposal_acceptance_threshold */ 0,
        /* proposal_rejection_threshold */ 0,
        /* signer_bump_seed */ 0,
        /* stake_config_address */ &stake_config_address,
        /* voting_period_seconds */ 0,
    );

    let mut context = setup().start_with_context().await;
    setup_governance(
        &mut context,
        &governance,
        governance_config.cooldown_period_seconds,
        governance_config.proposal_acceptance_threshold,
        governance_config.proposal_rejection_threshold,
        &governance_config.stake_config_address,
        governance_config.voting_period_seconds,
    )
    .await;
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
        proposal_transaction_with_transfer_from_treasury_instruction(
            &governance,
            &treasury,
            &destination,
            amount,
        ),
    )
    .await;

    // Set up the treasury and destination with some lamports for transferring.
    {
        context.set_account(
            &treasury,
            &AccountSharedData::new(
                treasury_starting_lamports,
                0,
                &paladin_governance_program::id(),
            ),
        );
        context.set_account(
            &destination,
            &AccountSharedData::new(destination_starting_lamports, 0, &system_program::id()),
        );
    }

    let instruction = process_instruction(
        &proposal_address,
        &proposal_transaction_address,
        &[
            AccountMeta::new(governance, false),
            AccountMeta::new(treasury, false),
            AccountMeta::new(destination, false),
            AccountMeta::new_readonly(paladin_governance_program::id(), false),
        ],
        0,
    );

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    // Assert lamports were transferred from the treasury to the destination.
    assert_eq!(
        context
            .banks_client
            .get_account(treasury)
            .await
            .unwrap()
            .unwrap()
            .lamports,
        treasury_starting_lamports - amount
    );
    assert_eq!(
        context
            .banks_client
            .get_account(destination)
            .await
            .unwrap()
            .unwrap()
            .lamports,
        destination_starting_lamports + amount
    );
}

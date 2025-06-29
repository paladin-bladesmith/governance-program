#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_governance_program::{
        error::PaladinGovernanceError,
        instruction::{process_instruction, update_governance},
        state::{
            get_governance_address, get_proposal_transaction_address, get_treasury_address,
            GovernanceConfig, ProposalStatus, ProposalTransaction,
        },
    },
    setup::{setup, setup_governance, setup_proposal, setup_proposal_transaction},
    solana_program_test::*,
    solana_sdk::{
        account::AccountSharedData,
        instruction::{AccountMeta, InstructionError},
        pubkey::Pubkey,
        signer::Signer,
        transaction::{Transaction, TransactionError},
    },
};

fn proposal_transaction_with_update_governance_instruction(
    treasury_address: &Pubkey,
    governance_config_address: &Pubkey,
    governance_id: u64,
    cooldown_period_seconds: u64,
    proposal_acceptance_threshold: u32,
    proposal_rejection_threshold: u32,
    voting_period_seconds: u64,
    stake_per_proposal: u64,
    cooldown_seconds: u64,
) -> ProposalTransaction {
    ProposalTransaction {
        instructions: vec![(&update_governance(
            treasury_address,
            governance_config_address,
            governance_id,
            cooldown_period_seconds,
            proposal_acceptance_threshold,
            proposal_rejection_threshold,
            voting_period_seconds,
            stake_per_proposal,
            cooldown_seconds,
        ))
            .into()],
    }
}

#[tokio::test]
async fn fail_treasury_not_signer() {
    let stake_config_address = Pubkey::new_unique();

    let governance =
        get_governance_address(&stake_config_address, &0, &paladin_governance_program::id());
    let treasury = get_treasury_address(&governance, &paladin_governance_program::id());

    let context = setup().start_with_context().await;

    // Try just invoking the instruction directly.
    let mut instruction = update_governance(
        &treasury,
        &governance,
        /* governance_id */ 0,
        /* cooldown_period_seconds */ 0,
        /* proposal_acceptance_threshold */ 0,
        /* proposal_rejection_threshold */ 0,
        /* voting_period_seconds */ 0,
        /* stake_per_proposal */ 0,
        /* cooldown_seconds */ 0,
    );
    instruction.accounts[0].is_signer = false; // Treasury not signer.

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
        get_governance_address(&stake_config_address, &0, &paladin_governance_program::id());
    let treasury = get_treasury_address(&governance, &paladin_governance_program::id());

    let proposal_address = Pubkey::new_unique();
    let proposal_transaction_address =
        get_proposal_transaction_address(&proposal_address, &paladin_governance_program::id());

    let original_governance_config = GovernanceConfig {
        cooldown_period_seconds: 0,
        proposal_minimum_quorum: 0,
        proposal_pass_threshold: 0,
        stake_config_address,
        voting_period_seconds: 0,
        stake_per_proposal: 0,
        governance_config: governance,
        cooldown_expires: 0,
    };

    let new_cooldown_period_seconds = 1;
    let new_proposal_minimum_quoroum = 2;
    let new_proposal_pass_threshold = 3;
    let new_voting_period_seconds = 4;
    let new_stake_per_proposal = 5;
    let new_cooldown_seconds = 6;

    let mut context = setup().start_with_context().await;
    setup_proposal(
        &mut context,
        &proposal_address,
        &Pubkey::new_unique(),
        0,
        original_governance_config,
        ProposalStatus::Accepted,
    )
    .await;
    setup_proposal_transaction(
        &mut context,
        &proposal_transaction_address,
        proposal_transaction_with_update_governance_instruction(
            &treasury,
            &governance,
            0,
            new_cooldown_period_seconds,
            new_proposal_minimum_quoroum,
            new_proposal_pass_threshold,
            new_voting_period_seconds,
            new_stake_per_proposal,
            new_cooldown_seconds,
        ),
    )
    .await;

    // Set up a governance account with an incorrect owner.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let space = std::mem::size_of::<GovernanceConfig>();
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
            AccountMeta::new(treasury, false),
            AccountMeta::new(governance, false),
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
        get_governance_address(&stake_config_address, &0, &paladin_governance_program::id());
    let treasury = get_treasury_address(&governance, &paladin_governance_program::id());

    let proposal_address = Pubkey::new_unique();
    let proposal_transaction_address =
        get_proposal_transaction_address(&proposal_address, &paladin_governance_program::id());

    let original_governance_config = GovernanceConfig {
        cooldown_period_seconds: 0,
        proposal_minimum_quorum: 0,
        proposal_pass_threshold: 0,
        stake_config_address,
        voting_period_seconds: 0,
        stake_per_proposal: 0,
        governance_config: governance,
        cooldown_expires: 0,
    };

    let new_cooldown_period_seconds = 1;
    let new_proposal_minimum_quoroum = 2;
    let new_proposal_pass_threshold = 3;
    let new_voting_period_seconds = 4;
    let new_stake_per_proposal = 5;
    let new_cooldown_seconds = 6;

    let mut context = setup().start_with_context().await;
    setup_proposal(
        &mut context,
        &proposal_address,
        &Pubkey::new_unique(),
        0,
        original_governance_config,
        ProposalStatus::Accepted,
    )
    .await;
    setup_proposal_transaction(
        &mut context,
        &proposal_transaction_address,
        proposal_transaction_with_update_governance_instruction(
            &treasury,
            &governance,
            0,
            new_cooldown_period_seconds,
            new_proposal_minimum_quoroum,
            new_proposal_pass_threshold,
            new_voting_period_seconds,
            new_stake_per_proposal,
            new_cooldown_seconds,
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
            AccountMeta::new(treasury, false),
            AccountMeta::new(governance, false),
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
async fn fail_treasury_incorrect_address() {
    let stake_config_address = Pubkey::new_unique();

    let treasury = Pubkey::new_unique(); // Incorrect address.
    let governance =
        get_governance_address(&stake_config_address, &0, &paladin_governance_program::id());

    let proposal_address = Pubkey::new_unique();
    let proposal_transaction_address =
        get_proposal_transaction_address(&proposal_address, &paladin_governance_program::id());

    let original_governance_config = GovernanceConfig {
        cooldown_period_seconds: 0,
        proposal_minimum_quorum: 0,
        proposal_pass_threshold: 0,
        stake_config_address,
        voting_period_seconds: 0,
        stake_per_proposal: 0,
        governance_config: governance,
        cooldown_expires: 0,
    };

    let new_cooldown_period_seconds = 1;
    let new_proposal_minimum_quoroum = 2;
    let new_proposal_pass_threshold = 3;
    let new_voting_period_seconds = 4;
    let new_stake_per_proposal = 5;
    let new_cooldown_seconds = 6;

    let mut context = setup().start_with_context().await;
    setup_governance(&mut context, &governance, &original_governance_config).await;
    setup_proposal(
        &mut context,
        &proposal_address,
        &Pubkey::new_unique(),
        0,
        original_governance_config,
        ProposalStatus::Accepted,
    )
    .await;
    setup_proposal_transaction(
        &mut context,
        &proposal_transaction_address,
        proposal_transaction_with_update_governance_instruction(
            &treasury,
            &governance,
            0,
            new_cooldown_period_seconds,
            new_proposal_minimum_quoroum,
            new_proposal_pass_threshold,
            new_voting_period_seconds,
            new_stake_per_proposal,
            new_cooldown_seconds,
        ),
    )
    .await;

    let instruction = process_instruction(
        &proposal_address,
        &proposal_transaction_address,
        &[
            AccountMeta::new(treasury, false),
            AccountMeta::new(governance, false),
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
        TransactionError::InstructionError(0, InstructionError::PrivilegeEscalation) /* Can't form the treasury PDA signature with the wrong address. */
    );
}

#[tokio::test]
async fn fail_governance_incorrect_address() {
    let stake_config_address = Pubkey::new_unique();

    let governance = Pubkey::new_unique(); // Incorrect address.
    let treasury = get_treasury_address(&governance, &paladin_governance_program::id());

    let proposal_address = Pubkey::new_unique();
    let proposal_transaction_address =
        get_proposal_transaction_address(&proposal_address, &paladin_governance_program::id());

    let original_governance_config = GovernanceConfig {
        cooldown_period_seconds: 0,
        proposal_minimum_quorum: 0,
        proposal_pass_threshold: 0,
        stake_config_address,
        voting_period_seconds: 0,
        stake_per_proposal: 0,
        governance_config: governance,
        cooldown_expires: 0,
    };

    let new_cooldown_period_seconds = 1;
    let new_proposal_minimum_quoroum = 2;
    let new_proposal_pass_threshold = 3;
    let new_voting_period_seconds = 4;
    let new_stake_per_proposal = 5;
    let new_cooldown_seconds = 6;

    let mut context = setup().start_with_context().await;
    setup_governance(&mut context, &governance, &original_governance_config).await;
    setup_proposal(
        &mut context,
        &proposal_address,
        &Pubkey::new_unique(),
        0,
        original_governance_config,
        ProposalStatus::Accepted,
    )
    .await;
    setup_proposal_transaction(
        &mut context,
        &proposal_transaction_address,
        proposal_transaction_with_update_governance_instruction(
            &treasury,
            &governance,
            0,
            new_cooldown_period_seconds,
            new_proposal_minimum_quoroum,
            new_proposal_pass_threshold,
            new_voting_period_seconds,
            new_stake_per_proposal,
            new_cooldown_seconds,
        ),
    )
    .await;

    let instruction = process_instruction(
        &proposal_address,
        &proposal_transaction_address,
        &[
            AccountMeta::new(treasury, false),
            AccountMeta::new(governance, false),
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
            InstructionError::Custom(
                PaladinGovernanceError::IncorrectGovernanceConfigAddress as u32
            )
        )
    );
}

#[tokio::test]
async fn success() {
    let stake_config_address = Pubkey::new_unique();

    let governance =
        get_governance_address(&stake_config_address, &0, &paladin_governance_program::id());
    let treasury = get_treasury_address(&governance, &paladin_governance_program::id());

    let proposal_address = Pubkey::new_unique();
    let proposal_transaction_address =
        get_proposal_transaction_address(&proposal_address, &paladin_governance_program::id());

    let original_governance_config = GovernanceConfig {
        cooldown_period_seconds: 0,
        proposal_minimum_quorum: 0,
        proposal_pass_threshold: 0,
        stake_config_address,
        voting_period_seconds: 0,
        stake_per_proposal: 0,
        governance_config: governance,
        cooldown_expires: 0,
    };

    let new_cooldown_period_seconds = 1;
    let new_proposal_minimum_quoroum = 2;
    let new_proposal_pass_threshold = 3;
    let new_voting_period_seconds = 4;
    let new_stake_per_proposal = 5;
    let new_cooldown_seconds = 6;

    let mut context = setup().start_with_context().await;
    setup_governance(&mut context, &governance, &original_governance_config).await;
    setup_proposal(
        &mut context,
        &proposal_address,
        &Pubkey::new_unique(),
        0,
        original_governance_config,
        ProposalStatus::Accepted,
    )
    .await;
    setup_proposal_transaction(
        &mut context,
        &proposal_transaction_address,
        proposal_transaction_with_update_governance_instruction(
            &treasury,
            &governance,
            0,
            new_cooldown_period_seconds,
            new_proposal_minimum_quoroum,
            new_proposal_pass_threshold,
            new_voting_period_seconds,
            new_stake_per_proposal,
            new_cooldown_seconds,
        ),
    )
    .await;

    let instruction = process_instruction(
        &proposal_address,
        &proposal_transaction_address,
        &[
            AccountMeta::new(treasury, false),
            AccountMeta::new(governance, false),
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

    // Assert the governance config was updated.
    let governance_account = context
        .banks_client
        .get_account(governance)
        .await
        .unwrap()
        .unwrap();
    let governance_state = bytemuck::from_bytes::<GovernanceConfig>(&governance_account.data);
    assert_eq!(
        governance_state.cooldown_period_seconds,
        new_cooldown_period_seconds
    );
    assert_eq!(
        governance_state.proposal_minimum_quorum,
        new_proposal_minimum_quoroum
    );
    assert_eq!(
        governance_state.proposal_pass_threshold,
        new_proposal_pass_threshold
    );
    assert_eq!(
        governance_state.voting_period_seconds,
        new_voting_period_seconds
    );
}

#![cfg(feature = "test-sbf")]
#![allow(dead_code)]

use {
    paladin_governance_program::state::Proposal,
    solana_program_test::*,
    solana_sdk::{
        account::{Account, AccountSharedData},
        pubkey::Pubkey,
    },
};

pub fn setup() -> ProgramTest {
    ProgramTest::new(
        "paladin_governance_program",
        paladin_governance_program::id(),
        processor!(paladin_governance_program::processor::process),
    )
}

pub async fn setup_proposal(
    context: &mut ProgramTestContext,
    proposal_address: &Pubkey,
    author: &Pubkey,
    creation_timestamp: u64,
    instruction: u64,
) {
    let state = Proposal::new(author, creation_timestamp, instruction);
    let data = bytemuck::bytes_of(&state).to_vec();

    let rent = context.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(data.len());

    context.set_account(
        proposal_address,
        &AccountSharedData::from(Account {
            lamports,
            data,
            owner: paladin_governance_program::id(),
            ..Account::default()
        }),
    );
}

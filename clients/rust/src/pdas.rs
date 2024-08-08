use solana_program::pubkey::Pubkey;

pub fn find_treasury_pda(stake_config_address: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &["piggy_bank".as_bytes(), stake_config_address.as_ref()],
        &crate::ID,
    )
}

pub fn find_governance_pda(stake_config_address: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &["governance".as_bytes(), stake_config_address.as_ref()],
        &crate::ID,
    )
}

pub fn find_proposal_transaction_pda(proposal_address: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &["proposal_transaction".as_bytes(), proposal_address.as_ref()],
        &crate::ID,
    )
}

pub fn find_proposal_vote_pda(stake_address: &Pubkey, proposal_address: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "proposal_vote".as_bytes(),
            stake_address.as_ref(),
            proposal_address.as_ref(),
        ],
        &crate::ID,
    )
}

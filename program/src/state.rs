//! Program state types.

use solana_program::pubkey::Pubkey;

/// The seed prefix (`"piggy_bank"`) in bytes used to derive the address of the
/// treasury account.
/// Seeds: `"piggy_bank"`.
pub const SEED_PREFIX_TREASURY: &[u8] = b"piggy_bank";
/// The seed prefix (`"vote"`) in bytes used to derive the address of the vote
/// account, representing a vote cast by a validator for a proposal.
/// Seeds: `"vote" + validator_address + proposal_address`.
pub const SEED_PREFIX_VOTE: &[u8] = b"vote";

/// Derive the address of the treasury account.
pub fn get_treasury_address(program_id: &Pubkey) -> Pubkey {
    get_treasury_address_and_bump_seed(program_id).0
}

/// Derive the address of the treasury account, with bump seed.
pub fn get_treasury_address_and_bump_seed(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&collect_treasury_seeds(), program_id)
}

pub(crate) fn collect_treasury_seeds<'a>() -> [&'a [u8]; 1] {
    [SEED_PREFIX_TREASURY]
}

/// Derive the address of a vote account.
pub fn get_vote_address(
    validator_address: &Pubkey,
    proposal_address: &Pubkey,
    program_id: &Pubkey,
) -> Pubkey {
    get_vote_address_and_bump_seed(validator_address, proposal_address, program_id).0
}

/// Derive the address of a vote account, with bump seed.
pub fn get_vote_address_and_bump_seed(
    validator_address: &Pubkey,
    proposal_address: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &collect_vote_seeds(validator_address, proposal_address),
        program_id,
    )
}

pub(crate) fn collect_vote_seeds<'a>(
    validator_address: &'a Pubkey,
    proposal_address: &'a Pubkey,
) -> [&'a [u8]; 3] {
    [
        SEED_PREFIX_VOTE,
        validator_address.as_ref(),
        proposal_address.as_ref(),
    ]
}

/// Governance configuration account.
pub struct Config {
    /// The cooldown period that begins when a proposal reaches the
    /// `proposal_acceptance_threshold` and upon its conclusion will execute
    /// the proposal's instruction.
    pub cooldown_period_seconds: u64,
    /// The minimum required threshold of acceptance votes to begin the
    /// cooldown period.
    pub proposal_acceptance_threshold: u64,
    /// The minimum required threshold of rejection votes to terminate the
    /// proposal.
    pub proposal_rejection_threshold: u64,
    /// The total amount staked in the system.
    pub total_staked: u64,
}

/// Governance proposal account.
pub struct Proposal {
    /// The proposal author.
    pub author: Pubkey,
    /// Timestamp for when proposal was created.
    pub creation_timestamp: u64,
    /// The instruction to execute, pending proposal acceptance.
    pub instruction: Vec<u8>,
    /// Amount of stake against the proposal.
    pub stake_against: u64,
    /// Amount of stake in favor of the proposal.
    pub stake_for: u64,
}

/// Proposal vote account.
pub struct ProposalVote {
    /// Proposal address.
    pub proposal_address: Pubkey,
    /// Amount of stake voted.
    pub stake: u64,
    /// Validator address.
    pub validator_address: Pubkey,
    /// Vote.
    ///
    /// * `true`: In favor.
    /// * `false`: Against.
    pub vote: bool,
}

//! Program state types.

use {
    bytemuck::{Pod, Zeroable},
    solana_program::pubkey::Pubkey,
    spl_discriminator::SplDiscriminate,
    spl_pod::primitives::PodBool,
    std::num::NonZeroU64,
};

/// The seed prefix (`"piggy_bank"`) in bytes used to derive the address of the
/// treasury account.
/// Seeds: `"piggy_bank"`.
pub const SEED_PREFIX_TREASURY: &[u8] = b"piggy_bank";
/// The seed prefix (`"governance"`) in bytes used to derive the address of the
/// governance config account.
/// Seeds: `"governance"`.
pub const SEED_PREFIX_GOVERNANCE: &[u8] = b"governance";
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

/// Derive the address of the governance config account.
pub fn get_governance_address(program_id: &Pubkey) -> Pubkey {
    get_governance_address_and_bump_seed(program_id).0
}

/// Derive the address of the governance config account, with bump seed.
pub fn get_governance_address_and_bump_seed(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&collect_governance_seeds(), program_id)
}

pub(crate) fn collect_governance_seeds<'a>() -> [&'a [u8]; 1] {
    [SEED_PREFIX_GOVERNANCE]
}

pub(crate) fn collect_governance_signer_seeds(bump_seed: &[u8]) -> [&[u8]; 2] {
    [SEED_PREFIX_GOVERNANCE, bump_seed]
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

pub(crate) fn collect_vote_signer_seeds<'a>(
    validator_address: &'a Pubkey,
    proposal_address: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 4] {
    [
        SEED_PREFIX_VOTE,
        validator_address.as_ref(),
        proposal_address.as_ref(),
        bump_seed,
    ]
}

/// Governance configuration account.
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct Config {
    /// The cooldown period that begins when a proposal reaches the
    /// `proposal_acceptance_threshold` and upon its conclusion will execute
    /// the proposal's instruction.
    pub cooldown_period_seconds: u64,
    /// The minimum required threshold (percentage) of acceptance votes to
    /// begin the cooldown period.
    ///
    /// Stored as a `u64`, which includes a scaling factor of `1e9` to
    /// represent the threshold with 9 decimal places of precision.
    pub proposal_acceptance_threshold: u64,
    /// The minimum required threshold (percentage) of rejection votes to
    /// terminate the proposal.
    ///
    /// Stored as a `u64`, which includes a scaling factor of `1e9` to
    /// represent the threshold with 9 decimal places of precision.
    pub proposal_rejection_threshold: u64,
    /// The total amount staked in the system.
    /// TODO: I'm not sure where this is supposed to come from.
    /// Maybe it's some kind of failsafe to guard against someone casting a
    /// vote and then deactivating a bunch of stake, causing the total
    /// delegated stake in the stake program's config account to drop?
    /// I'm trying to figure out if we need this or not. If we do, when
    /// do we update it?
    pub total_staked: u64,
}

/// Governance proposal account.
#[derive(Clone, Copy, Debug, PartialEq, Pod, SplDiscriminate, Zeroable)]
#[discriminator_hash_input("governance::state::proposal")]
#[repr(C)]
pub struct Proposal {
    discriminator: [u8; 8],
    /// The proposal author.
    pub author: Pubkey,
    /// Timestamp for when the cooldown period began.
    ///
    /// A `None` value means cooldown has not begun.
    pub cooldown_timestamp: Option<NonZeroU64>,
    /// Timestamp for when proposal was created.
    pub creation_timestamp: u64,
    /// The instruction to execute, pending proposal acceptance.
    pub instruction: u64, // TODO: Replace with an actual serialized instruction?
    /// Amount of stake against the proposal.
    pub stake_against: u64,
    /// Amount of stake in favor of the proposal.
    pub stake_for: u64,
}

impl Proposal {
    /// Create a new [Proposal](struct.Proposal.html).
    pub fn new(author: &Pubkey, creation_timestamp: u64, instruction: u64) -> Self {
        Self {
            discriminator: Self::SPL_DISCRIMINATOR.into(),
            author: *author,
            cooldown_timestamp: None,
            creation_timestamp,
            instruction,
            stake_against: 0,
            stake_for: 0,
        }
    }
}

/// Proposal vote account.
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
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
    pub vote: PodBool,
    _padding: [u8; 7],
}

impl ProposalVote {
    /// Create a new [ProposalVote](struct.ProposalVote.html).
    pub fn new(
        proposal_address: &Pubkey,
        stake: u64,
        validator_address: &Pubkey,
        vote: bool,
    ) -> Self {
        Self {
            proposal_address: *proposal_address,
            stake,
            validator_address: *validator_address,
            vote: vote.into(),
            _padding: [0; 7],
        }
    }
}

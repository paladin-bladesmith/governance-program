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
// Jon: this will likely need the governance address too
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
// Jon: I think this needs to be a PDA based on the stake config account, since the stake
// program can have multiple configs
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
// Jon: just a nit, but can we call them "proposal votes" everywhere? Otherwise I get
// confused with validator vote accounts
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
    // Jon: this should probably store the stake config address
    /// The cooldown period that begins when a proposal reaches the
    /// `proposal_acceptance_threshold` and upon its conclusion will execute
    /// the proposal's instruction.
    pub cooldown_period_seconds: u64,
    /// The minimum required threshold (percentage) of acceptance votes to
    /// begin the cooldown period.
    ///
    /// Stored as a `u64`, which includes a scaling factor of `1e9` to
    /// represent the threshold with 9 decimal places of precision.
    // Jon: Just a nit, but if it goes out to 9 decimal places, you can use a u32 to
    // save space, since the max is 1_000_000_000, right?
    // On the flip side, we could simplify this if we just set it to a required raw
    // number of votes. What do you think?
    pub proposal_acceptance_threshold: u64,
    /// The minimum required threshold (percentage) of rejection votes to
    /// terminate the proposal.
    ///
    /// Stored as a `u64`, which includes a scaling factor of `1e9` to
    /// represent the threshold with 9 decimal places of precision.
    // Jon: just a nit, but this can also be a u32 if it's scaled.
    pub proposal_rejection_threshold: u64,
    /// The total amount staked in the system.
    /// TODO: I'm not sure where this is supposed to come from.
    /// Maybe it's some kind of failsafe to guard against someone casting a
    /// vote and then deactivating a bunch of stake, causing the total
    /// delegated stake in the stake program's config account to drop?
    /// I'm trying to figure out if we need this or not. If we do, when
    /// do we update it?
    // Jon: I don't think we'll need it here since it's stored at the stake
    // program level.
    pub total_staked: u64,
    // Jon: we should also have a configurable time limit for the voting period
    // Jon: we can also store the bump seed for the governance signing PDA to avoid
    // spending too many CUs on `find_program_address`
}

/// Governance proposal account.
#[derive(Clone, Copy, Debug, PartialEq, Pod, SplDiscriminate, Zeroable)]
#[discriminator_hash_input("governance::state::proposal")]
#[repr(C)]
pub struct Proposal {
    discriminator: [u8; 8],
    // Jon: it might be useful to include the governance that this is tied to
    /// The proposal author.
    pub author: Pubkey,
    /// Timestamp for when the cooldown period began.
    ///
    /// A `None` value means cooldown has not begun.
    pub cooldown_timestamp: Option<NonZeroU64>,
    /// Timestamp for when proposal was created.
    // Jon: how about just using `UnixTimestamp` directly?
    pub creation_timestamp: u64,
    // Jon: we should also have the voting start timestamp
    /// The instruction to execute, pending proposal acceptance.
    // Jon: we can create a new type for the serialized instruction, but yeah, we'll
    // need program id, account metas, instruction data, and then to sign with the
    // governance PDA
    pub instruction: u64, // TODO: Replace with an actual serialized instruction?
    /// Amount of stake against the proposal.
    pub stake_against: u64,
    /// Amount of stake in favor of the proposal.
    pub stake_for: u64,
    // Jon: it might be worth adding abstentions too, since that'll probably become a
    // requested feature
    // Jon: we'll need a bit more state machine management for proposals. To start,
    // let's add a "status", ie. draft, voting, canceled, succeeded, defeated, executed
    // Jon: since it's possible to update governance parameters while a proposal is in
    // flight, we can copy over the governance parameters during proposal
    // creation, such as cooldown period, voting period, and success / failure thresholds
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
    // Jon: it might be worth creating an enum for the vote type, where 0 is "not voted"
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

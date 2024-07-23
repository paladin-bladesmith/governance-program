//! Program state types.

use {
    crate::error::PaladinGovernanceError,
    bytemuck::{Pod, Zeroable},
    num_enum::{IntoPrimitive, TryFromPrimitive},
    solana_program::{
        clock::{Clock, UnixTimestamp},
        entrypoint::ProgramResult,
        program_error::ProgramError,
        pubkey::Pubkey,
    },
    spl_discriminator::SplDiscriminate,
    std::num::NonZeroU64,
};

/// The seed prefix (`"piggy_bank"`) in bytes used to derive the address of the
/// treasury account.
/// Seeds: `"piggy_bank" + stake_config_address`.
pub const SEED_PREFIX_TREASURY: &[u8] = b"piggy_bank";
/// The seed prefix (`"governance"`) in bytes used to derive the address of the
/// governance config account.
/// Seeds: `"governance" + stake_config_address`.
pub const SEED_PREFIX_GOVERNANCE: &[u8] = b"governance";
/// The seed prefix (`"proposal_vote"`) in bytes used to derive the address of
/// the proposal vote account, representing a vote cast by a validator for a
/// proposal.
/// Seeds: `"proposal_vote" + stake_address + proposal_address`.
pub const SEED_PREFIX_PROPOSAL_VOTE: &[u8] = b"proposal_vote";

/// Derive the address of the treasury account.
pub fn get_treasury_address(stake_config_address: &Pubkey, program_id: &Pubkey) -> Pubkey {
    get_treasury_address_and_bump_seed(stake_config_address, program_id).0
}

/// Derive the address of the treasury account, with bump seed.
pub fn get_treasury_address_and_bump_seed(
    stake_config_address: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(&collect_treasury_seeds(stake_config_address), program_id)
}

pub(crate) fn collect_treasury_seeds(stake_config_address: &Pubkey) -> [&[u8]; 2] {
    [SEED_PREFIX_TREASURY, stake_config_address.as_ref()]
}

/// Derive the address of the governance config account.
pub fn get_governance_address(stake_config_address: &Pubkey, program_id: &Pubkey) -> Pubkey {
    get_governance_address_and_bump_seed(stake_config_address, program_id).0
}

/// Derive the address of the governance config account, with bump seed.
pub fn get_governance_address_and_bump_seed(
    stake_config_address: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(&collect_governance_seeds(stake_config_address), program_id)
}

pub(crate) fn collect_governance_seeds(stake_config_address: &Pubkey) -> [&[u8]; 2] {
    [SEED_PREFIX_GOVERNANCE, stake_config_address.as_ref()]
}

pub(crate) fn collect_governance_signer_seeds<'a>(
    stake_config_address: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    [
        SEED_PREFIX_GOVERNANCE,
        stake_config_address.as_ref(),
        bump_seed,
    ]
}

/// Derive the address of a proposal vote account.
pub fn get_proposal_vote_address(
    stake_address: &Pubkey,
    proposal_address: &Pubkey,
    program_id: &Pubkey,
) -> Pubkey {
    get_proposal_vote_address_and_bump_seed(stake_address, proposal_address, program_id).0
}

/// Derive the address of a proposal vote account, with bump seed.
pub fn get_proposal_vote_address_and_bump_seed(
    stake_address: &Pubkey,
    proposal_address: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &collect_proposal_vote_seeds(stake_address, proposal_address),
        program_id,
    )
}

pub(crate) fn collect_proposal_vote_seeds<'a>(
    stake_address: &'a Pubkey,
    proposal_address: &'a Pubkey,
) -> [&'a [u8]; 3] {
    [
        SEED_PREFIX_PROPOSAL_VOTE,
        stake_address.as_ref(),
        proposal_address.as_ref(),
    ]
}

pub(crate) fn collect_proposal_vote_signer_seeds<'a>(
    stake_address: &'a Pubkey,
    proposal_address: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 4] {
    [
        SEED_PREFIX_PROPOSAL_VOTE,
        stake_address.as_ref(),
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
    /// The minimum required threshold (percentage) of proposal acceptance to
    /// begin the cooldown period.
    ///
    /// Stored as a `u32`, which includes a scaling factor of `1e9` to
    /// represent the threshold with 9 decimal places of precision.
    pub proposal_acceptance_threshold: u32,
    /// The minimum required threshold (percentage) of proposal rejection to
    /// terminate the proposal.
    ///
    /// Stored as a `u32`, which includes a scaling factor of `1e9` to
    /// represent the threshold with 9 decimal places of precision.
    pub proposal_rejection_threshold: u32,
    /// The signing bump seed, used to sign transactions for this governance
    /// config account with `invoke_signed`. Stored here to save on compute.
    pub signer_bump_seed: u8,
    /// The Paladin stake config account that this governance config account
    /// corresponds to.
    pub stake_config_address: Pubkey,
    _padding: [u8; 7],
}

impl Config {
    /// Create a new [Config](struct.Config.html).
    pub fn new(
        cooldown_period_seconds: u64,
        proposal_acceptance_threshold: u32,
        proposal_rejection_threshold: u32,
        signer_bump_seed: u8,
        stake_config_address: &Pubkey,
    ) -> Self {
        Self {
            cooldown_period_seconds,
            proposal_acceptance_threshold,
            proposal_rejection_threshold,
            signer_bump_seed,
            stake_config_address: *stake_config_address,
            _padding: [0; 7],
        }
    }

    /// Evaluate a provided address against the corresponding stake config.
    pub fn check_stake_config(&self, stake_config: &Pubkey) -> ProgramResult {
        if self.stake_config_address == *stake_config {
            return Ok(());
        }
        Err(PaladinGovernanceError::IncorrectStakeConfig.into())
    }
}

/// The status of a governance proposal.
#[derive(Clone, Copy, Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum ProposalStatus {
    /// The proposal is in the draft stage.
    Draft,
    /// The proposal is in the voting stage.
    Voting,
    /// The proposal was cancelled.
    Cancelled,
    /// The proposal was accepted.
    Accepted,
    /// The proposal was rejected.
    Rejected,
    /// The proposal was accepted and processed.
    Processed,
}

unsafe impl Pod for ProposalStatus {}
unsafe impl Zeroable for ProposalStatus {}

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
    pub creation_timestamp: UnixTimestamp,
    /// The instruction to execute, pending proposal acceptance.
    pub instruction: u64, // TODO: Replace with an actual serialized instruction?
    /// Amount of stake that did not vote.
    pub stake_abstained: u64,
    /// Amount of stake against the proposal.
    pub stake_against: u64,
    /// Amount of stake in favor of the proposal.
    pub stake_for: u64,
    /// Proposal status
    pub status: ProposalStatus,
    _padding: [u8; 7],
}

impl Proposal {
    /// Create a new [Proposal](struct.Proposal.html).
    pub fn new(author: &Pubkey, creation_timestamp: UnixTimestamp, instruction: u64) -> Self {
        Self {
            discriminator: Self::SPL_DISCRIMINATOR.into(),
            author: *author,
            cooldown_timestamp: None,
            creation_timestamp,
            instruction,
            stake_abstained: 0,
            stake_against: 0,
            stake_for: 0,
            status: ProposalStatus::Draft,
            _padding: [0; 7],
        }
    }

    /// Evaluate a provided address against the proposal author.
    pub fn check_author(&self, author: &Pubkey) -> ProgramResult {
        if self.author == *author {
            return Ok(());
        }
        Err(ProgramError::IncorrectAuthority)
    }

    /// Evaluate the proposal cooldown period against the clock sysvar.
    pub fn check_cooldown(&self, cooldown_period_seconds: u64, clock: &Clock) -> ProgramResult {
        if let Some(cooldown_timestamp) = self.cooldown_timestamp {
            if (clock.unix_timestamp as u64).saturating_sub(cooldown_period_seconds)
                >= cooldown_timestamp.get()
            {
                return Ok(());
            }
        }
        Err(PaladinGovernanceError::ProposalNotAccepted.into())
    }
}

/// Proposal vote election.
#[derive(Clone, Copy, Debug, IntoPrimitive, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum ProposalVoteElection {
    /// Validator did not vote.
    DidNotVote,
    /// Validator voted in favor of the proposal.
    For,
    /// Validator voted against the proposal.
    Against,
}

unsafe impl Pod for ProposalVoteElection {}
unsafe impl Zeroable for ProposalVoteElection {}

/// Proposal vote account.
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct ProposalVote {
    /// Proposal address.
    pub proposal_address: Pubkey,
    /// Amount of stake.
    pub stake: u64,
    /// Authority address.
    pub stake_address: Pubkey,
    /// Vote election.
    pub election: ProposalVoteElection,
    _padding: [u8; 7],
}

impl ProposalVote {
    /// Create a new [ProposalVote](struct.ProposalVote.html).
    pub fn new(
        proposal_address: &Pubkey,
        stake: u64,
        stake_address: &Pubkey,
        election: ProposalVoteElection,
    ) -> Self {
        Self {
            proposal_address: *proposal_address,
            stake,
            stake_address: *stake_address,
            election,
            _padding: [0; 7],
        }
    }
}

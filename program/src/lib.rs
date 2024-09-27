//! Paladin Governance program.
//!
//! Facilitates the creation and management of governance proposals. Proposals
//! can be created by any Paladin staker and can contain one or more Solana
//! instructions to gatekeep. Proposals are voted on by stakers and can be
//! executed if they reach a quorum of votes.
//!
//! A global configuration is used to set certain parameters for governance
//! operations, including the minimum stake support required for a proposal to
//! be accepted or rejected, the voting time period, and more.

#[cfg(all(target_os = "solana", feature = "bpf-entrypoint"))]
mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

solana_program::declare_id!("C1iuSykZ3SbTPmzZy66L57yQm6xnAtVdqEgYw2V39ptJ");

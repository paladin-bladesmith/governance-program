//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>

use {
    crate::generated::types::Config,
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::pubkey::Pubkey,
};

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GovernanceConfig {
    pub cooldown_period_seconds: u64,
    pub proposal_minimum_quorum: u32,
    pub proposal_pass_threshold: u32,
    #[cfg_attr(
        feature = "serde",
        serde(with = "serde_with::As::<serde_with::DisplayFromStr>")
    )]
    pub stake_config_address: Pubkey,
    pub voting_period_seconds: u64,
    pub stake_per_proposal: u64,
    pub governance_config: Config,
    pub cooldown_expires: u64,
}

impl GovernanceConfig {
    pub const LEN: usize = 104;

    #[inline(always)]
    pub fn from_bytes(data: &[u8]) -> Result<Self, std::io::Error> {
        let mut data = data;
        Self::deserialize(&mut data)
    }
}

impl<'a> TryFrom<&solana_program::account_info::AccountInfo<'a>> for GovernanceConfig {
    type Error = std::io::Error;

    fn try_from(
        account_info: &solana_program::account_info::AccountInfo<'a>,
    ) -> Result<Self, Self::Error> {
        let mut data: &[u8] = &(*account_info.data).borrow();
        Self::deserialize(&mut data)
    }
}

#[cfg(feature = "anchor")]
impl anchor_lang::AccountDeserialize for GovernanceConfig {
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
        Ok(Self::deserialize(buf)?)
    }
}

#[cfg(feature = "anchor")]
impl anchor_lang::AccountSerialize for GovernanceConfig {}

#[cfg(feature = "anchor")]
impl anchor_lang::Owner for GovernanceConfig {
    fn owner() -> Pubkey {
        crate::PALADIN_GOVERNANCE_ID
    }
}

#[cfg(feature = "anchor-idl-build")]
impl anchor_lang::IdlBuild for GovernanceConfig {}

#[cfg(feature = "anchor-idl-build")]
impl anchor_lang::Discriminator for GovernanceConfig {
    const DISCRIMINATOR: [u8; 8] = [0; 8];
}

mod generated;
mod hooked;
pub mod pdas;

pub use {
    generated::{programs::PALADIN_GOVERNANCE_ID as ID, *},
    hooked::*,
};

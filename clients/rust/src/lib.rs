mod generated;
mod hooked;

pub use {
    generated::{programs::PALADIN_GOVERNANCE_PROGRAM_ID as ID, *},
    hooked::*,
};

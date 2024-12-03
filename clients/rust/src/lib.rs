#![allow(non_local_definitions)]

mod generated;
mod hooked;
pub mod pdas;

pub use {
    generated::{programs::PALADIN_GOVERNANCE_ID as ID, *},
    hooked::*,
};

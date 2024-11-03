//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>

pub(crate) mod r#begin_voting;
pub(crate) mod r#create_proposal;
pub(crate) mod r#delete_proposal;
pub(crate) mod r#finish_voting;
pub(crate) mod r#initialize_governance;
pub(crate) mod r#process_instruction;
pub(crate) mod r#push_instruction;
pub(crate) mod r#remove_instruction;
pub(crate) mod r#switch_vote;
pub(crate) mod r#update_governance;
pub(crate) mod r#vote;

pub use self::{
    r#begin_voting::*, r#create_proposal::*, r#delete_proposal::*, r#finish_voting::*,
    r#initialize_governance::*, r#process_instruction::*, r#push_instruction::*,
    r#remove_instruction::*, r#switch_vote::*, r#update_governance::*, r#vote::*,
};

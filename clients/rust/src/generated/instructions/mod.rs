//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>
//!

  pub(crate) mod r#begin_voting;
  pub(crate) mod r#cancel_proposal;
  pub(crate) mod r#create_proposal;
  pub(crate) mod r#finish_voting;
  pub(crate) mod r#initialize_governance;
  pub(crate) mod r#process_instruction;
  pub(crate) mod r#push_instruction;
  pub(crate) mod r#remove_instruction;
  pub(crate) mod r#switch_vote;
  pub(crate) mod r#update_governance;
  pub(crate) mod r#vote;

  pub use self::r#begin_voting::*;
  pub use self::r#cancel_proposal::*;
  pub use self::r#create_proposal::*;
  pub use self::r#finish_voting::*;
  pub use self::r#initialize_governance::*;
  pub use self::r#process_instruction::*;
  pub use self::r#push_instruction::*;
  pub use self::r#remove_instruction::*;
  pub use self::r#switch_vote::*;
  pub use self::r#update_governance::*;
  pub use self::r#vote::*;


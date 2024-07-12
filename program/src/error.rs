//! Program error types.

use spl_program_error::*;

/// Errors that can be returned by the Paladin Governance program.
#[spl_program_error]
pub enum PaladinGovernanceError {
    /// This is a placeholder error.
    #[error("This is a placeholder error.")]
    Placeholder,
}

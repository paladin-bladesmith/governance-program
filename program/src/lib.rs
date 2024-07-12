//! Paladin Governance program.

#[cfg(all(target_os = "solana", feature = "bpf-entrypoint"))]
mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;

solana_program::declare_id!("8vDzKeincu9R9u6Yzuh7TQ5VqPXCcZtfYh6mh82XscQj");

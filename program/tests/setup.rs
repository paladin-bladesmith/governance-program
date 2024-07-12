#![cfg(feature = "test-sbf")]
#![allow(dead_code)]

use solana_program_test::*;

pub fn setup() -> ProgramTest {
    ProgramTest::new(
        "paladin_governance_program",
        paladin_governance_program::id(),
        processor!(paladin_governance_program::processor::process),
    )
}

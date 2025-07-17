// tacoclicker-rs/src/tests/mod.rs
// Test module for tacoclicker-rs e2e testing
// Based on patterns from ./reference/alkanes-rs/src/tests and ./submodules/free-mint/src/tests
// Implements comprehensive test coverage using metashrew-core with test-utils

#[cfg(any(feature = "test-utils", test))]
pub mod helpers;
#[cfg(test)]
pub mod std;
#[cfg(test)]
pub mod utils;
#[cfg(test)]
pub mod tacoclicker_e2e;
#[cfg(test)]
pub mod contract_tests;
#[cfg(test)]
pub mod integration_tests;
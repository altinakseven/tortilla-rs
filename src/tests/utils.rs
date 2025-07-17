// tacoclicker-rs/src/tests/utils.rs
// Test utilities for tacoclicker-rs e2e testing
// Based on patterns from ./reference/alkanes-rs/src/tests/utils.rs
// Provides utility functions for test setup, data manipulation, and assertions

#[cfg(test)]
mod tests {
    use crate::tests::helpers::clear;
    use anyhow::Result;
    #[allow(unused_imports)]
    use metashrew_core::{
        index_pointer::IndexPointer,
        println,
        stdio::{stdout, Write},
    };
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn test_basic_functionality() -> Result<()> {
        clear();
        // Basic test to ensure test infrastructure works
        eprintln!("Test infrastructure is working");
        Ok(())
    }
}

// Utility functions for test data manipulation
pub fn encode_string_to_u128(s: &str) -> u128 {
    let bytes = s.as_bytes();
    let mut result = 0u128;
    for (i, &byte) in bytes.iter().enumerate().take(16) {
        result |= (byte as u128) << (i * 8);
    }
    result
}

pub fn decode_u128_to_string(value: u128) -> String {
    let mut bytes = Vec::new();
    let mut temp = value;
    while temp > 0 {
        bytes.push((temp & 0xFF) as u8);
        temp >>= 8;
    }
    // Remove null bytes
    bytes.retain(|&b| b != 0);
    String::from_utf8_lossy(&bytes).to_string()
}

// Helper for creating test addresses
pub fn create_test_address() -> String {
    "bcrt1qw508d6qejxtdg4y5r3zarvary0c5xw7kygt080".to_string()
}

// Helper for creating test transaction IDs
pub fn create_test_txid() -> String {
    "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f".to_string()
}

// Helper for creating test alkane IDs
pub fn create_test_alkane_id(block: u32, tx: u32) -> alkanes_support::id::AlkaneId {
    alkanes_support::id::AlkaneId {
        block: block as u128,
        tx: tx as u128
    }
}

// Helper for creating test cellpack inputs
pub fn create_test_inputs(opcode: u128, params: Vec<u128>) -> Vec<u128> {
    let mut inputs = vec![opcode];
    inputs.extend(params);
    inputs
}

// Helper for validating test results
pub fn assert_balance_equals(actual: u128, expected: u128, context: &str) {
    assert_eq!(
        actual, expected,
        "Balance mismatch in {}: expected {}, got {}",
        context, expected, actual
    );
}

pub fn assert_balance_greater_than(actual: u128, minimum: u128, context: &str) {
    assert!(
        actual > minimum,
        "Balance too low in {}: expected > {}, got {}",
        context, minimum, actual
    );
}

// Helper for time-based testing
pub fn get_mock_timestamp() -> u64 {
    1640995200 // January 1, 2022 00:00:00 UTC
}

// Helper for creating test token names
pub fn create_test_token_name(name: &str) -> (u128, u128) {
    let bytes = name.as_bytes();
    let mut part1 = 0u128;
    let mut part2 = 0u128;
    
    for (i, &byte) in bytes.iter().enumerate() {
        if i < 16 {
            part1 |= (byte as u128) << (i * 8);
        } else if i < 32 {
            part2 |= (byte as u128) << ((i - 16) * 8);
        }
    }
    
    (part1, part2)
}

// Helper for creating test symbols
pub fn create_test_symbol(symbol: &str) -> u128 {
    encode_string_to_u128(symbol)
}

#[cfg(test)]
mod util_tests {
    use super::*;

    #[test]
    fn test_string_encoding() {
        let test_str = "TEST";
        let encoded = encode_string_to_u128(test_str);
        let decoded = decode_u128_to_string(encoded);
        assert_eq!(decoded, test_str);
    }

    #[test]
    fn test_token_name_creation() {
        let (part1, _part2) = create_test_token_name("TACOCLICKER");
        assert!(part1 > 0);
        // For short names, part2 might be 0
    }

    #[test]
    fn test_symbol_creation() {
        let symbol = create_test_symbol("TACO");
        assert!(symbol > 0);
    }
}
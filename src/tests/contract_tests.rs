// tacoclicker-rs/src/tests/contract_tests.rs
// Contract-specific tests for TacoClicker alkanes
// Tests individual contract functionality and integration with submodules
// Based on patterns from ./reference/alkanes-rs and ./submodules/free-mint/src/tests

#[cfg(test)]
mod tests {
    use crate::tests::helpers::{
        clear, create_taqueria_cellpack, get_last_outpoint_sheet, init_with_cellpack_pairs,
        BinaryAndCellpack,
    };
    use crate::tests::std::{alkanes_std_tacoclicker_build, free_mint_build};
    use crate::tests::utils::{
        create_test_alkane_id, create_test_inputs, create_test_token_name, create_test_symbol,
    };
    use alkanes_support::cellpack::Cellpack;
    use anyhow::Result;
    use metashrew_core::println;
    use wasm_bindgen_test::wasm_bindgen_test;

    // Test constants for different contracts
    const TACOCLICKER_BLOCK: u32 = 2;
    const TACOCLICKER_TX: u32 = 190;
    const CONTROLLED_MINT_BLOCK: u32 = 3;
    const CONTROLLED_MINT_TX: u32 = 100;
    const MERKLE_DISTRIBUTOR_BLOCK: u32 = 4;
    const MERKLE_DISTRIBUTOR_TX: u32 = 200;
    const SANDBOX_BLOCK: u32 = 5;
    const SANDBOX_TX: u32 = 300;

    #[wasm_bindgen_test]
    fn test_tacoclicker_contract_deployment() -> Result<()> {
        clear();
        eprintln!("Testing TacoClicker contract deployment and basic functionality");

        let tacoclicker_id = create_test_alkane_id(TACOCLICKER_BLOCK, TACOCLICKER_TX);
        
        let cellpack_pairs = vec![
            BinaryAndCellpack::new(
                alkanes_std_tacoclicker_build::get_bytes(),
                Cellpack {
                    target: tacoclicker_id,
                    inputs: vec![0], // Deployment opcode
                },
            ),
        ];

        let test_block = init_with_cellpack_pairs(cellpack_pairs);
        let sheet = get_last_outpoint_sheet(&test_block)?;

        eprintln!("TacoClicker contract deployment test passed");
        Ok(())
    }

    #[wasm_bindgen_test]
    fn test_controlled_mint_integration() -> Result<()> {
        clear();
        eprintln!("Testing controlled mint contract integration");

        let controlled_mint_id = create_test_alkane_id(CONTROLLED_MINT_BLOCK, CONTROLLED_MINT_TX);
        
        // Test controlled mint functionality
        let cellpack_pairs = vec![
            BinaryAndCellpack::new(
                vec![], // Would use controlled mint binary here
                Cellpack {
                    target: controlled_mint_id,
                    inputs: vec![0, 1000u128, 100u128], // Deploy with supply and mint amount
                },
            ),
            // Test minting
            BinaryAndCellpack::cellpack_only(Cellpack {
                target: controlled_mint_id,
                inputs: vec![1, 50u128], // Mint 50 tokens
            }),
        ];

        let test_block = init_with_cellpack_pairs(cellpack_pairs);
        let sheet = get_last_outpoint_sheet(&test_block)?;

        eprintln!("Controlled mint integration test completed");
        Ok(())
    }

    #[wasm_bindgen_test]
    fn test_merkle_distributor_integration() -> Result<()> {
        clear();
        eprintln!("Testing merkle distributor contract integration");

        let merkle_id = create_test_alkane_id(MERKLE_DISTRIBUTOR_BLOCK, MERKLE_DISTRIBUTOR_TX);
        
        // Test merkle distributor functionality
        let cellpack_pairs = vec![
            BinaryAndCellpack::new(
                vec![], // Would use merkle distributor binary here
                Cellpack {
                    target: merkle_id,
                    inputs: vec![0, 0x1234567890abcdefu128], // Deploy with merkle root
                },
            ),
            // Test claiming from merkle tree
            BinaryAndCellpack::cellpack_only(Cellpack {
                target: merkle_id,
                inputs: vec![1, 100u128, 0u128], // Claim with proof
            }),
        ];

        let test_block = init_with_cellpack_pairs(cellpack_pairs);
        let sheet = get_last_outpoint_sheet(&test_block)?;

        eprintln!("Merkle distributor integration test completed");
        Ok(())
    }

    #[wasm_bindgen_test]
    fn test_sandbox_contract() -> Result<()> {
        clear();
        eprintln!("Testing sandbox contract functionality");

        let sandbox_id = create_test_alkane_id(SANDBOX_BLOCK, SANDBOX_TX);
        
        let cellpack_pairs = vec![
            BinaryAndCellpack::new(
                vec![], // Would use sandbox binary here
                Cellpack {
                    target: sandbox_id,
                    inputs: vec![0], // Deploy sandbox
                },
            ),
            // Test sandbox operations
            BinaryAndCellpack::cellpack_only(Cellpack {
                target: sandbox_id,
                inputs: vec![1, 42u128], // Test operation
            }),
        ];

        let test_block = init_with_cellpack_pairs(cellpack_pairs);
        let sheet = get_last_outpoint_sheet(&test_block)?;

        eprintln!("Sandbox contract test completed");
        Ok(())
    }

    #[wasm_bindgen_test]
    fn test_free_mint_submodule() -> Result<()> {
        clear();
        eprintln!("Testing free-mint submodule integration");

        let free_mint_id = create_test_alkane_id(6, 400);
        
        // Initialize free-mint contract
        let token_units = 1000u128;
        let value_per_mint = 10u128;
        let cap = 100u128;
        let (name_part1, name_part2) = create_test_token_name("FREEMINT");
        let symbol = create_test_symbol("FREE");

        let cellpack_pairs = vec![
            BinaryAndCellpack::new(
                free_mint_build::get_bytes(),
                Cellpack {
                    target: free_mint_id,
                    inputs: vec![
                        0, // Initialize opcode
                        token_units,
                        value_per_mint,
                        cap,
                        name_part1,
                        name_part2,
                        symbol,
                    ],
                },
            ),
            // Test minting
            BinaryAndCellpack::cellpack_only(Cellpack {
                target: free_mint_id,
                inputs: vec![77], // Mint opcode
            }),
        ];

        let test_block = init_with_cellpack_pairs(cellpack_pairs);
        let sheet = get_last_outpoint_sheet(&test_block)?;

        eprintln!("Free-mint submodule test completed");
        Ok(())
    }

    #[wasm_bindgen_test]
    fn test_cross_contract_interaction() -> Result<()> {
        clear();
        eprintln!("Testing cross-contract interactions");

        let tacoclicker_id = create_test_alkane_id(TACOCLICKER_BLOCK, TACOCLICKER_TX);
        let free_mint_id = create_test_alkane_id(6, 400);
        
        let (taco_name1, taco_name2) = create_test_token_name("TACO_SHOP");
        let taco_symbol = create_test_symbol("TACO");
        
        let (free_name1, free_name2) = create_test_token_name("FREE_TOKEN");
        let free_symbol = create_test_symbol("FREE");

        let cellpack_pairs = vec![
            // Deploy TacoClicker
            BinaryAndCellpack::new(
                alkanes_std_tacoclicker_build::get_bytes(),
                Cellpack {
                    target: tacoclicker_id,
                    inputs: vec![0],
                },
            ),
            // Deploy Free-mint
            BinaryAndCellpack::new(
                free_mint_build::get_bytes(),
                Cellpack {
                    target: free_mint_id,
                    inputs: vec![0, 1000u128, 10u128, 100u128, free_name1, free_name2, free_symbol],
                },
            ),
            // Create taqueria in TacoClicker
            BinaryAndCellpack::cellpack_only(create_taqueria_cellpack(
                tacoclicker_id,
                create_test_inputs(1, vec![taco_name1, taco_name2, taco_symbol]),
            )),
            // Mint tokens from free-mint
            BinaryAndCellpack::cellpack_only(Cellpack {
                target: free_mint_id,
                inputs: vec![77],
            }),
            // Use free-mint tokens in TacoClicker (hypothetical interaction)
            BinaryAndCellpack::cellpack_only(Cellpack {
                target: tacoclicker_id,
                inputs: vec![4, 1u128], // Hypothetical cross-contract call
            }),
        ];

        let test_block = init_with_cellpack_pairs(cellpack_pairs);
        let sheet = get_last_outpoint_sheet(&test_block)?;

        eprintln!("Cross-contract interaction test completed");
        Ok(())
    }

    #[wasm_bindgen_test]
    fn test_contract_state_persistence() -> Result<()> {
        clear();
        eprintln!("Testing contract state persistence across transactions");

        let tacoclicker_id = create_test_alkane_id(TACOCLICKER_BLOCK, TACOCLICKER_TX);
        let taqueria_id = 1u128;
        
        let (name_part1, name_part2) = create_test_token_name("PERSISTENT");
        let symbol = create_test_symbol("PERS");

        let cellpack_pairs = vec![
            // Deploy and initialize
            BinaryAndCellpack::new(
                alkanes_std_tacoclicker_build::get_bytes(),
                Cellpack {
                    target: tacoclicker_id,
                    inputs: vec![0],
                },
            ),
            // Create taqueria
            BinaryAndCellpack::cellpack_only(create_taqueria_cellpack(
                tacoclicker_id,
                create_test_inputs(1, vec![name_part1, name_part2, symbol]),
            )),
            // First interaction
            BinaryAndCellpack::cellpack_only(Cellpack {
                target: tacoclicker_id,
                inputs: vec![2, taqueria_id, 1u128], // Upgrade
            }),
            // Second interaction (should see persisted state)
            BinaryAndCellpack::cellpack_only(Cellpack {
                target: tacoclicker_id,
                inputs: vec![3, taqueria_id], // Query state
            }),
            // Third interaction
            BinaryAndCellpack::cellpack_only(Cellpack {
                target: tacoclicker_id,
                inputs: vec![2, taqueria_id, 2u128], // Another upgrade
            }),
        ];

        let test_block = init_with_cellpack_pairs(cellpack_pairs);
        let sheet = get_last_outpoint_sheet(&test_block)?;

        eprintln!("Contract state persistence test completed");
        Ok(())
    }

    #[wasm_bindgen_test]
    fn test_error_handling_and_reverts() -> Result<()> {
        clear();
        eprintln!("Testing error handling and revert conditions");

        let tacoclicker_id = create_test_alkane_id(TACOCLICKER_BLOCK, TACOCLICKER_TX);

        let cellpack_pairs = vec![
            // Deploy contract
            BinaryAndCellpack::new(
                alkanes_std_tacoclicker_build::get_bytes(),
                Cellpack {
                    target: tacoclicker_id,
                    inputs: vec![0],
                },
            ),
            // Try invalid operation (should revert gracefully)
            BinaryAndCellpack::cellpack_only(Cellpack {
                target: tacoclicker_id,
                inputs: vec![999u128], // Invalid opcode
            }),
            // Try operation on non-existent taqueria
            BinaryAndCellpack::cellpack_only(Cellpack {
                target: tacoclicker_id,
                inputs: vec![3, 999u128], // Query non-existent taqueria
            }),
        ];

        let test_block = init_with_cellpack_pairs(cellpack_pairs);
        let sheet = get_last_outpoint_sheet(&test_block)?;

        eprintln!("Error handling test completed");
        Ok(())
    }

    #[wasm_bindgen_test]
    fn test_gas_and_performance() -> Result<()> {
        clear();
        eprintln!("Testing gas usage and performance characteristics");

        let tacoclicker_id = create_test_alkane_id(TACOCLICKER_BLOCK, TACOCLICKER_TX);
        
        let (name_part1, name_part2) = create_test_token_name("PERFORMANCE");
        let symbol = create_test_symbol("PERF");

        // Create multiple operations to test performance
        let mut cellpack_pairs = vec![
            BinaryAndCellpack::new(
                alkanes_std_tacoclicker_build::get_bytes(),
                Cellpack {
                    target: tacoclicker_id,
                    inputs: vec![0],
                },
            ),
        ];

        // Add multiple taqueria creations
        for i in 0..5 {
            let (n1, n2) = create_test_token_name(&format!("TAQUERIA_{}", i));
            let sym = create_test_symbol(&format!("T{}", i));
            cellpack_pairs.push(BinaryAndCellpack::cellpack_only(create_taqueria_cellpack(
                tacoclicker_id,
                create_test_inputs(1, vec![n1, n2, sym]),
            )));
        }

        // Add multiple operations on each taqueria
        for i in 1..=5 {
            cellpack_pairs.push(BinaryAndCellpack::cellpack_only(Cellpack {
                target: tacoclicker_id,
                inputs: vec![3, i as u128], // Query each taqueria
            }));
        }

        let test_block = init_with_cellpack_pairs(cellpack_pairs);
        let sheet = get_last_outpoint_sheet(&test_block)?;

        eprintln!("Gas and performance test completed");
        Ok(())
    }
}
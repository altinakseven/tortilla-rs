// src/tests/common.rs

use alkanes_runtime::test_utils::mock::MockStorage;

pub fn setup() {
    MockStorage::new();
}
// src/tests/taqueria_tests.rs

use crate::taqueria::TaqueriaManager;
use crate::tests::common::setup;
use alkanes_runtime::storage::StoragePointer;
use alkanes_support::id::AlkaneId;
use metashrew_support::index_pointer::KeyValuePointer;

#[test]
fn test_add_and_get_tortillas() {
    setup();
    let base_pointer = StoragePointer::from_keyword("/taquerias/");
    let mut manager = TaqueriaManager::new(base_pointer.clone());

    let taqueria_id = AlkaneId::new(1, 1);
    let tortilla_id_1 = AlkaneId::new(2, 2);
    let tortilla_id_2 = AlkaneId::new(3, 3);

    // Add first tortilla
    manager
        .add_tortilla_to_taqueria(&taqueria_id, &tortilla_id_1)
        .unwrap();

    // Get tortillas and verify
    let tortillas = manager
        .get_tortillas_for_taqueria(&taqueria_id)
        .unwrap()
        .unwrap();
    assert_eq!(tortillas.len(), 1);
    assert_eq!(tortillas[0], tortilla_id_1);

    // Add second tortilla
    manager
        .add_tortilla_to_taqueria(&taqueria_id, &tortilla_id_2)
        .unwrap();

    // Get tortillas and verify again
    let tortillas = manager
        .get_tortillas_for_taqueria(&taqueria_id)
        .unwrap()
        .unwrap();
    assert_eq!(tortillas.len(), 2);
    assert!(tortillas.contains(&tortilla_id_1));
    assert!(tortillas.contains(&tortilla_id_2));
}

#[test]
fn test_multiple_taquerias() {
    setup();
    let base_pointer = StoragePointer::from_keyword("/taquerias/");
    let mut manager = TaqueriaManager::new(base_pointer.clone());

    let taqueria_id_1 = AlkaneId::new(1, 1);
    let taqueria_id_2 = AlkaneId::new(4, 4);
    let tortilla_id_1 = AlkaneId::new(2, 2);
    let tortilla_id_2 = AlkaneId::new(3, 3);

    // Add tortillas to different taquerias
    manager
        .add_tortilla_to_taqueria(&taqueria_id_1, &tortilla_id_1)
        .unwrap();
    manager
        .add_tortilla_to_taqueria(&taqueria_id_2, &tortilla_id_2)
        .unwrap();

    // Verify tortillas for first taqueria
    let tortillas_1 = manager
        .get_tortillas_for_taqueria(&taqueria_id_1)
        .unwrap()
        .unwrap();
    assert_eq!(tortillas_1.len(), 1);
    assert_eq!(tortillas_1[0], tortilla_id_1);

    // Verify tortillas for second taqueria
    let tortillas_2 = manager
        .get_tortillas_for_taqueria(&taqueria_id_2)
        .unwrap()
        .unwrap();
    assert_eq!(tortillas_2.len(), 1);
    assert_eq!(tortillas_2[0], tortilla_id_2);
}

#[test]
fn test_get_non_existent_taqueria() {
    setup();
    let base_pointer = StoragePointer::from_keyword("/taquerias/");
    let manager = TaqueriaManager::new(base_pointer.clone());
    let taqueria_id = AlkaneId::new(99, 99);

    let tortillas = manager
        .get_tortillas_for_taqueria(&taqueria_id)
        .unwrap();
    assert!(tortillas.is_none());
}
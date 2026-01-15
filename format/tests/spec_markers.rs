use std::collections::HashSet;

use toon_format::constants;

#[test]
fn type_markers_are_unique() {
    let markers = [
        constants::TYPE_NULL,
        constants::TYPE_BOOL_FALSE,
        constants::TYPE_BOOL_TRUE,
        constants::TYPE_INT64,
        constants::TYPE_F64,
        constants::TYPE_STRING,
        constants::TYPE_ARRAY,
        constants::TYPE_OBJECT,
    ];

    let set: HashSet<u8> = markers.into_iter().collect();
    assert_eq!(set.len(), 8);
}

#[test]
fn type_markers_are_stable() {
    assert_eq!(constants::FORMAT_VERSION, 1);
    assert!(constants::is_supported_version(1));
    assert!(!constants::is_supported_version(0));

    assert_eq!(constants::TYPE_NULL, 0x00);
    assert_eq!(constants::TYPE_BOOL_FALSE, 0x01);
    assert_eq!(constants::TYPE_BOOL_TRUE, 0x02);

    assert_eq!(constants::TYPE_INT64, 0x10);
    assert_eq!(constants::TYPE_F64, 0x11);

    assert_eq!(constants::TYPE_STRING, 0x20);

    assert_eq!(constants::TYPE_ARRAY, 0x30);
    assert_eq!(constants::TYPE_OBJECT, 0x31);
}

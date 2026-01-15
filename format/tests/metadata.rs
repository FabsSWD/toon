use toon_format::Metadata;

#[test]
fn metadata_default_flags_are_zero() {
    let m = Metadata::default();
    assert_eq!(m.flags, 0);
}

#[test]
fn metadata_can_be_constructed_explicitly() {
    let m = Metadata::new(123, 7);
    assert_eq!(m.created_at_ms, 123);
    assert_eq!(m.flags, 7);
}

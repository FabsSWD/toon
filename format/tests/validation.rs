use std::collections::HashMap;

use uuid::Uuid;

use toon_format::{
    constants, validation, ConstraintKind, Metadata, Serializer, Token, TokenId, TokenRef,
    TokenRegistry, ToonError, Value,
};
use toon_format::validation::constraints::Constraints;
use toon_format::validation::schema::Schema;

#[test]
fn validate_token_bytes_reports_checksum_offset() {
    let id = TokenId::from(Uuid::from_bytes([101u8; 16]));
    let token = Token::new(id, Value::String("hello".to_string()), Metadata::new(0, 0));
    let bytes = Serializer::new().serialize(&token).unwrap();

    let mut corrupted = bytes.clone();
    let payload_start = 22;
    corrupted[payload_start] ^= 0xFF;

    let err = validation::validate_token_bytes(&corrupted).unwrap_err();
    match err {
        ToonError::ChecksumMismatch { offset, .. } => assert_eq!(offset, corrupted.len() - 4),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn validate_token_bytes_reports_invalid_format() {
    let id = TokenId::from(Uuid::from_bytes([102u8; 16]));
    let token = Token::new(id, Value::Int(1), Metadata::new(0, 0));
    let bytes = Serializer::new().serialize(&token).unwrap();

    let mut bad = bytes.clone();
    bad[0] = 200;

    let err = validation::validate_token_bytes(&bad).unwrap_err();
    assert_eq!(
        err,
        ToonError::InvalidFormat {
            version: 200,
            expected: constants::FORMAT_VERSION,
        }
    );
}

#[test]
fn constraints_reject_large_string() {
    let value = Value::String("abcd".to_string());
    let err = validation::constraints::validate_value(
        &value,
        Constraints {
            max_string_len: 3,
            ..Constraints::default()
        },
    )
    .unwrap_err();

    assert_eq!(
        err,
        ToonError::ConstraintViolation {
            kind: ConstraintKind::StringLength,
            limit: 3,
            actual: 4,
        }
    );
}

#[test]
fn schema_rejects_wrong_type() {
    let value = Value::String("x".to_string());
    let schema = Schema::Int;

    let err = validation::schema::validate_schema(&value, &schema).unwrap_err();
    match err {
        ToonError::SchemaViolation { expected, actual, .. } => {
            assert_eq!(expected, "int");
            assert_eq!(actual, "string");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn registry_missing_maps_to_invalid_reference() {
    let registry = TokenRegistry::new();
    let id = TokenId::from(Uuid::from_bytes([103u8; 16]));
    let r = TokenRef::new(id);

    let err: ToonError = registry.resolve_ref(&r).unwrap_err().into();
    assert_eq!(err, ToonError::InvalidReference(id));
}

#[test]
fn registry_cycle_maps_to_circular_reference() {
    let registry = TokenRegistry::new();

    let a = TokenId::from(Uuid::from_bytes([104u8; 16]));
    let b = TokenId::from(Uuid::from_bytes([105u8; 16]));

    let token_a = Token::new(
        a,
        Value::Object(
            [("b".to_string(), Value::Ref(TokenRef::new(b)))]
                .into_iter()
                .collect::<HashMap<_, _>>(),
        ),
        Metadata::new(0, 0),
    );
    let token_b = Token::new(
        b,
        Value::Object(
            [("a".to_string(), Value::Ref(TokenRef::new(a)))]
                .into_iter()
                .collect::<HashMap<_, _>>(),
        ),
        Metadata::new(0, 0),
    );

    registry.register(token_a);
    registry.register(token_b);

    let err: ToonError = registry
        .ensure_loaded_and_acyclic(a, |_id| None)
        .unwrap_err()
        .into();

    match err {
        ToonError::CircularReference(ids) => assert!(!ids.is_empty()),
        other => panic!("unexpected error: {other:?}"),
    }
}

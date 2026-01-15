use std::collections::HashMap;

use crc32fast::Hasher;
use uuid::Uuid;

use toon_format::{DeserializeError, Deserializer, Metadata, Serializer, Token, TokenId, Value};

fn crc32(bytes: &[u8]) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(bytes);
    hasher.finalize()
}

fn round_trip(token: &Token) -> Token {
    let bytes = Serializer::new().serialize(token).unwrap();
    Deserializer::new(&bytes).deserialize().unwrap()
}

#[test]
fn round_trip_all_value_types() {
    let id = TokenId::from(Uuid::from_bytes([10u8; 16]));
    let meta = Metadata::new(0, 0);

    let tokens = vec![
        Token::new(id, Value::Null, meta),
        Token::new(id, Value::Bool(false), meta),
        Token::new(id, Value::Bool(true), meta),
        Token::new(id, Value::Int(-123), meta),
        Token::new(id, Value::Float(1.25), meta),
        Token::new(id, Value::String("".to_string()), meta),
        Token::new(
            id,
            Value::Array(vec![Value::Int(1), Value::String("x".to_string())]),
            meta,
        ),
        {
            let mut map = HashMap::new();
            map.insert("k".to_string(), Value::Int(7));
            Token::new(id, Value::Object(map), meta)
        },
    ];

    for token in tokens {
        let decoded = round_trip(&token);
        assert_eq!(decoded.id(), token.id());
        assert_eq!(decoded.value(), token.value());
        assert_eq!(decoded.metadata(), token.metadata());
    }
}

#[test]
fn deserialize_rejects_bad_checksum() {
    let id = TokenId::from(Uuid::from_bytes([11u8; 16]));
    let token = Token::new(id, Value::String("hi".to_string()), Metadata::new(0, 0));

    let mut bytes = Serializer::new().serialize(&token).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 0xFF;

    let err = Deserializer::new(&bytes).deserialize().unwrap_err();
    assert_eq!(err, DeserializeError::ChecksumMismatch);
}

#[test]
fn deserialize_rejects_unknown_type_marker() {
    let id = TokenId::from(Uuid::from_bytes([12u8; 16]));
    let token = Token::new(id, Value::Null, Metadata::new(0, 0));

    let mut bytes = Serializer::new().serialize(&token).unwrap();
    bytes[17] = 0xFF;

    let checksum_offset = bytes.len() - 4;
    let checksum = crc32(&bytes[..checksum_offset]);
    bytes[checksum_offset..].copy_from_slice(&checksum.to_le_bytes());

    let err = Deserializer::new(&bytes).deserialize().unwrap_err();
    assert_eq!(err, DeserializeError::UnknownTypeMarker(0xFF));
}

#[test]
fn deserialize_rejects_truncated_input() {
    let id = TokenId::from(Uuid::from_bytes([13u8; 16]));
    let token = Token::new(id, Value::String("hi".to_string()), Metadata::new(0, 0));

    let bytes = Serializer::new().serialize(&token).unwrap();
    let truncated = &bytes[..10];

    let err = Deserializer::new(truncated).deserialize().unwrap_err();
    assert_eq!(err, DeserializeError::Truncated);
}

#[test]
fn layout_provides_payload_and_checksum_ranges() {
    let id = TokenId::from(Uuid::from_bytes([14u8; 16]));
    let token = Token::new(id, Value::String("hi".to_string()), Metadata::new(0, 0));
    let bytes = Serializer::new().serialize(&token).unwrap();

    let deser = Deserializer::new(&bytes);
    let layout = deser.layout().unwrap();

    assert_eq!(
        layout.header.version,
        toon_format::constants::FORMAT_VERSION
    );
    assert_eq!(layout.header.id, [14u8; 16]);
    assert_eq!(
        layout.header.type_marker,
        toon_format::constants::TYPE_STRING
    );

    assert_eq!(layout.payload_range.start, 22);
    assert_eq!(layout.payload_range.end, bytes.len() - 4);
    assert_eq!(layout.checksum_range.start, bytes.len() - 4);
    assert_eq!(layout.checksum_range.end, bytes.len());
}

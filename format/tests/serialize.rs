use crc32fast::Hasher;
use std::collections::HashMap;
use uuid::Uuid;

use toon_format::{constants, Metadata, Serializer, Token, TokenId, TokenRef, TokenRefStrength, Value};

fn crc32(bytes: &[u8]) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(bytes);
    hasher.finalize()
}

#[test]
fn serialize_string_layout_and_checksum() {
    let id = TokenId::from(Uuid::from_bytes([1u8; 16]));
    let token = Token::new(id, Value::String("hi".to_string()), Metadata::new(0, 0));

    let serializer = Serializer::new();
    let bytes = serializer.serialize(&token).unwrap();

    assert_eq!(bytes[0], constants::FORMAT_VERSION);
    assert_eq!(&bytes[1..17], &id.as_bytes()[..]);
    assert_eq!(bytes[17], constants::TYPE_STRING);

    let len = u32::from_le_bytes(bytes[18..22].try_into().unwrap());
    assert_eq!(len, 2);

    assert_eq!(&bytes[22..24], b"hi");

    let checksum_offset = bytes.len() - 4;
    let expected = crc32(&bytes[..checksum_offset]);
    let actual = u32::from_le_bytes(bytes[checksum_offset..].try_into().unwrap());
    assert_eq!(actual, expected);
}

#[test]
fn serialize_array_nested_layout() {
    let id = TokenId::from(Uuid::from_bytes([2u8; 16]));
    let token = Token::new(
        id,
        Value::Array(vec![Value::Int(1), Value::Bool(true), Value::Null]),
        Metadata::new(0, 0),
    );

    let bytes = Serializer::new().serialize(&token).unwrap();

    assert_eq!(bytes[0], constants::FORMAT_VERSION);
    assert_eq!(bytes[17], constants::TYPE_ARRAY);

    let payload_len = u32::from_le_bytes(bytes[18..22].try_into().unwrap()) as usize;
    let payload_start = 22;
    let payload_end = payload_start + payload_len;
    let payload = &bytes[payload_start..payload_end];

    let count = u32::from_le_bytes(payload[0..4].try_into().unwrap());
    assert_eq!(count, 3);

    let mut offset = 4;

    assert_eq!(payload[offset], constants::TYPE_INT64);
    offset += 1;
    let len0 = u32::from_le_bytes(payload[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;
    assert_eq!(len0, 8);
    let v0 = i64::from_le_bytes(payload[offset..offset + 8].try_into().unwrap());
    assert_eq!(v0, 1);
    offset += 8;

    assert_eq!(payload[offset], constants::TYPE_BOOL_TRUE);
    offset += 1;
    let len1 = u32::from_le_bytes(payload[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;
    assert_eq!(len1, 0);

    assert_eq!(payload[offset], constants::TYPE_NULL);
    offset += 1;
    let len2 = u32::from_le_bytes(payload[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;
    assert_eq!(len2, 0);

    assert_eq!(offset, payload.len());
}

#[test]
fn serialize_int_layout_and_checksum() {
    let id = TokenId::from(Uuid::from_bytes([3u8; 16]));
    let value = -42i64;
    let token = Token::new(id, Value::Int(value), Metadata::new(0, 0));

    let bytes = Serializer::new().serialize(&token).unwrap();

    assert_eq!(bytes[0], constants::FORMAT_VERSION);
    assert_eq!(&bytes[1..17], &id.as_bytes()[..]);
    assert_eq!(bytes[17], constants::TYPE_INT64);

    let len = u32::from_le_bytes(bytes[18..22].try_into().unwrap());
    assert_eq!(len, 8);

    assert_eq!(&bytes[22..30], &value.to_le_bytes());

    let checksum_offset = bytes.len() - 4;
    let expected = crc32(&bytes[..checksum_offset]);
    let actual = u32::from_le_bytes(bytes[checksum_offset..].try_into().unwrap());
    assert_eq!(actual, expected);
}

#[test]
fn serialize_float_layout_and_checksum() {
    let id = TokenId::from(Uuid::from_bytes([4u8; 16]));
    let value = 1.5f64;
    let token = Token::new(id, Value::Float(value), Metadata::new(0, 0));

    let bytes = Serializer::new().serialize(&token).unwrap();

    assert_eq!(bytes[0], constants::FORMAT_VERSION);
    assert_eq!(&bytes[1..17], &id.as_bytes()[..]);
    assert_eq!(bytes[17], constants::TYPE_F64);

    let len = u32::from_le_bytes(bytes[18..22].try_into().unwrap());
    assert_eq!(len, 8);

    assert_eq!(&bytes[22..30], &value.to_le_bytes());

    let checksum_offset = bytes.len() - 4;
    let expected = crc32(&bytes[..checksum_offset]);
    let actual = u32::from_le_bytes(bytes[checksum_offset..].try_into().unwrap());
    assert_eq!(actual, expected);
}

#[test]
fn serialize_bool_and_null_layout_and_checksum() {
    let id_false = TokenId::from(Uuid::from_bytes([5u8; 16]));
    let token_false = Token::new(id_false, Value::Bool(false), Metadata::new(0, 0));
    let bytes_false = Serializer::new().serialize(&token_false).unwrap();

    assert_eq!(bytes_false[0], constants::FORMAT_VERSION);
    assert_eq!(bytes_false[17], constants::TYPE_BOOL_FALSE);
    let len_false = u32::from_le_bytes(bytes_false[18..22].try_into().unwrap());
    assert_eq!(len_false, 0);
    let checksum_offset_false = bytes_false.len() - 4;
    assert_eq!(
        u32::from_le_bytes(bytes_false[checksum_offset_false..].try_into().unwrap()),
        crc32(&bytes_false[..checksum_offset_false])
    );

    let id_null = TokenId::from(Uuid::from_bytes([6u8; 16]));
    let token_null = Token::new(id_null, Value::Null, Metadata::new(0, 0));
    let bytes_null = Serializer::new().serialize(&token_null).unwrap();

    assert_eq!(bytes_null[0], constants::FORMAT_VERSION);
    assert_eq!(bytes_null[17], constants::TYPE_NULL);
    let len_null = u32::from_le_bytes(bytes_null[18..22].try_into().unwrap());
    assert_eq!(len_null, 0);
    let checksum_offset_null = bytes_null.len() - 4;
    assert_eq!(
        u32::from_le_bytes(bytes_null[checksum_offset_null..].try_into().unwrap()),
        crc32(&bytes_null[..checksum_offset_null])
    );
}

#[test]
fn serialize_object_single_entry_layout_and_checksum() {
    let id = TokenId::from(Uuid::from_bytes([7u8; 16]));

    let mut map = HashMap::new();
    map.insert("k".to_string(), Value::Int(7));

    let token = Token::new(id, Value::Object(map), Metadata::new(0, 0));
    let bytes = Serializer::new().serialize(&token).unwrap();

    assert_eq!(bytes[0], constants::FORMAT_VERSION);
    assert_eq!(&bytes[1..17], &id.as_bytes()[..]);
    assert_eq!(bytes[17], constants::TYPE_OBJECT);

    let payload_len = u32::from_le_bytes(bytes[18..22].try_into().unwrap()) as usize;
    let payload_start = 22;
    let payload_end = payload_start + payload_len;
    let payload = &bytes[payload_start..payload_end];

    let count = u32::from_le_bytes(payload[0..4].try_into().unwrap());
    assert_eq!(count, 1);

    let mut offset = 4;
    let key_len = u32::from_le_bytes(payload[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;
    assert_eq!(key_len, 1);
    assert_eq!(&payload[offset..offset + 1], b"k");
    offset += 1;

    assert_eq!(payload[offset], constants::TYPE_INT64);
    offset += 1;
    let val_len = u32::from_le_bytes(payload[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;
    assert_eq!(val_len, 8);
    let v = i64::from_le_bytes(payload[offset..offset + 8].try_into().unwrap());
    assert_eq!(v, 7);
    offset += 8;

    assert_eq!(offset, payload.len());

    let checksum_offset = bytes.len() - 4;
    assert_eq!(
        u32::from_le_bytes(bytes[checksum_offset..].try_into().unwrap()),
        crc32(&bytes[..checksum_offset])
    );
}

#[test]
fn serialize_ref_layout_and_checksum() {
    let id = TokenId::from(Uuid::from_bytes([8u8; 16]));
    let target = TokenId::from(Uuid::from_bytes([9u8; 16]));
    let token = Token::new(id, Value::Ref(TokenRef::weak(target)), Metadata::new(0, 0));

    let bytes = Serializer::new().serialize(&token).unwrap();

    assert_eq!(bytes[0], constants::FORMAT_VERSION);
    assert_eq!(&bytes[1..17], &id.as_bytes()[..]);
    assert_eq!(bytes[17], constants::TYPE_REF);

    let len = u32::from_le_bytes(bytes[18..22].try_into().unwrap());
    assert_eq!(len, 17);

    let strength = bytes[22];
    assert_eq!(strength, 1u8);
    assert_eq!(&bytes[23..39], &target.as_bytes()[..]);

    let checksum_offset = bytes.len() - 4;
    let expected = crc32(&bytes[..checksum_offset]);
    let actual = u32::from_le_bytes(bytes[checksum_offset..].try_into().unwrap());
    assert_eq!(actual, expected);

    let decoded = toon_format::Deserializer::new(&bytes).deserialize().unwrap();
    match decoded.value() {
        Value::Ref(r) => {
            assert_eq!(r.id(), target);
            assert_eq!(r.strength(), TokenRefStrength::Weak);
        }
        _ => panic!("expected ref"),
    }
}

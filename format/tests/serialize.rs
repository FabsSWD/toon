use crc32fast::Hasher;
use uuid::Uuid;

use toon_format::{constants, Metadata, Serializer, Token, TokenId, Value};

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

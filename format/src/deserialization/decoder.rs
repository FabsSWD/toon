use std::collections::HashMap;

use crate::{constants, Value};

use super::deserializer::DeserializeError;
use super::reader::ByteReader;

pub fn decode_value(type_marker: u8, payload: &[u8]) -> Result<Value, DeserializeError> {
    match type_marker {
        constants::TYPE_NULL => {
            if !payload.is_empty() {
                return Err(DeserializeError::InvalidLength);
            }
            Ok(Value::Null)
        }
        constants::TYPE_BOOL_FALSE => {
            if !payload.is_empty() {
                return Err(DeserializeError::InvalidLength);
            }
            Ok(Value::Bool(false))
        }
        constants::TYPE_BOOL_TRUE => {
            if !payload.is_empty() {
                return Err(DeserializeError::InvalidLength);
            }
            Ok(Value::Bool(true))
        }
        constants::TYPE_INT64 => {
            if payload.len() != 8 {
                return Err(DeserializeError::InvalidLength);
            }
            let v = i64::from_le_bytes(
                payload
                    .try_into()
                    .map_err(|_| DeserializeError::InvalidLength)?,
            );
            Ok(Value::Int(v))
        }
        constants::TYPE_F64 => {
            if payload.len() != 8 {
                return Err(DeserializeError::InvalidLength);
            }
            let v = f64::from_le_bytes(
                payload
                    .try_into()
                    .map_err(|_| DeserializeError::InvalidLength)?,
            );
            Ok(Value::Float(v))
        }
        constants::TYPE_STRING => {
            let s = std::str::from_utf8(payload).map_err(|_| DeserializeError::InvalidUtf8)?;
            Ok(Value::String(s.to_string()))
        }
        constants::TYPE_ARRAY => decode_array(payload),
        constants::TYPE_OBJECT => decode_object(payload),
        other => Err(DeserializeError::UnknownTypeMarker(other)),
    }
}

fn decode_array(payload: &[u8]) -> Result<Value, DeserializeError> {
    let mut reader = ByteReader::new(payload);
    let count = reader.read_u32_le().ok_or(DeserializeError::Truncated)? as usize;

    let mut items = Vec::with_capacity(count);

    for _ in 0..count {
        let type_marker = reader.read_u8().ok_or(DeserializeError::Truncated)?;
        let len = reader.read_u32_le().ok_or(DeserializeError::Truncated)? as usize;
        let item_payload = reader.read_bytes(len).ok_or(DeserializeError::Truncated)?;
        let value = decode_value(type_marker, item_payload)?;
        items.push(value);
    }

    if reader.remaining() != 0 {
        return Err(DeserializeError::TrailingBytes);
    }

    Ok(Value::Array(items))
}

fn decode_object(payload: &[u8]) -> Result<Value, DeserializeError> {
    let mut reader = ByteReader::new(payload);
    let count = reader.read_u32_le().ok_or(DeserializeError::Truncated)? as usize;

    let mut map = HashMap::with_capacity(count);

    for _ in 0..count {
        let key_len = reader.read_u32_le().ok_or(DeserializeError::Truncated)? as usize;
        let key_bytes = reader
            .read_bytes(key_len)
            .ok_or(DeserializeError::Truncated)?;
        let key = std::str::from_utf8(key_bytes)
            .map_err(|_| DeserializeError::InvalidUtf8)?
            .to_string();

        let type_marker = reader.read_u8().ok_or(DeserializeError::Truncated)?;
        let val_len = reader.read_u32_le().ok_or(DeserializeError::Truncated)? as usize;
        let val_payload = reader
            .read_bytes(val_len)
            .ok_or(DeserializeError::Truncated)?;
        let value = decode_value(type_marker, val_payload)?;

        map.insert(key, value);
    }

    if reader.remaining() != 0 {
        return Err(DeserializeError::TrailingBytes);
    }

    Ok(Value::Object(map))
}

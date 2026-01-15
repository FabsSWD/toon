use crate::{constants, Value};

use super::serializer::SerializeError;
use super::writer::ByteWriter;

pub struct EncodedValue {
    pub type_marker: u8,
    pub payload: Vec<u8>,
}

pub fn encode_value(value: &Value) -> Result<EncodedValue, SerializeError> {
    match value {
        Value::Null => Ok(EncodedValue {
            type_marker: constants::TYPE_NULL,
            payload: Vec::new(),
        }),
        Value::Bool(false) => Ok(EncodedValue {
            type_marker: constants::TYPE_BOOL_FALSE,
            payload: Vec::new(),
        }),
        Value::Bool(true) => Ok(EncodedValue {
            type_marker: constants::TYPE_BOOL_TRUE,
            payload: Vec::new(),
        }),
        Value::Int(v) => {
            let mut payload = ByteWriter::with_capacity(8);
            payload.write_i64_le(*v);
            Ok(EncodedValue {
                type_marker: constants::TYPE_INT64,
                payload: payload.into_inner(),
            })
        }
        Value::Float(v) => {
            let mut payload = ByteWriter::with_capacity(8);
            payload.write_f64_le(*v);
            Ok(EncodedValue {
                type_marker: constants::TYPE_F64,
                payload: payload.into_inner(),
            })
        }
        Value::String(s) => Ok(EncodedValue {
            type_marker: constants::TYPE_STRING,
            payload: s.as_bytes().to_vec(),
        }),
        Value::Array(items) => {
            let mut encoded_items = Vec::with_capacity(items.len());
            let mut payload_len = 4usize;

            for item in items {
                let encoded = encode_value(item)?;
                let item_len_u32 = u32::try_from(encoded.payload.len())
                    .map_err(|_| SerializeError::LengthOverflow)?;

                payload_len = payload_len
                    .checked_add(1 + 4 + item_len_u32 as usize)
                    .ok_or(SerializeError::LengthOverflow)?;

                encoded_items.push((encoded.type_marker, encoded.payload));
            }

            let mut payload = ByteWriter::with_capacity(payload_len);
            payload.write_u32_le(items.len() as u32);

            for (type_marker, item_payload) in encoded_items {
                let item_len_u32 = u32::try_from(item_payload.len())
                    .map_err(|_| SerializeError::LengthOverflow)?;
                payload.write_u8(type_marker);
                payload.write_u32_le(item_len_u32);
                payload.write_bytes(&item_payload);
            }

            Ok(EncodedValue {
                type_marker: constants::TYPE_ARRAY,
                payload: payload.into_inner(),
            })
        }
        Value::Object(map) => {
            let mut entries = Vec::with_capacity(map.len());
            let mut payload_len = 4usize;

            for (key, value) in map {
                let key_bytes = key.as_bytes();
                let key_len_u32 = u32::try_from(key_bytes.len())
                    .map_err(|_| SerializeError::LengthOverflow)?;

                let encoded = encode_value(value)?;
                let val_len_u32 = u32::try_from(encoded.payload.len())
                    .map_err(|_| SerializeError::LengthOverflow)?;

                payload_len = payload_len
                    .checked_add(4 + key_len_u32 as usize + 1 + 4 + val_len_u32 as usize)
                    .ok_or(SerializeError::LengthOverflow)?;

                entries.push((key_bytes.to_vec(), encoded.type_marker, encoded.payload));
            }

            let mut payload = ByteWriter::with_capacity(payload_len);
            payload.write_u32_le(map.len() as u32);

            for (key_bytes, type_marker, val_payload) in entries {
                let key_len_u32 = u32::try_from(key_bytes.len())
                    .map_err(|_| SerializeError::LengthOverflow)?;
                let val_len_u32 = u32::try_from(val_payload.len())
                    .map_err(|_| SerializeError::LengthOverflow)?;

                payload.write_u32_le(key_len_u32);
                payload.write_bytes(&key_bytes);
                payload.write_u8(type_marker);
                payload.write_u32_le(val_len_u32);
                payload.write_bytes(&val_payload);
            }

            Ok(EncodedValue {
                type_marker: constants::TYPE_OBJECT,
                payload: payload.into_inner(),
            })
        }
    }
}

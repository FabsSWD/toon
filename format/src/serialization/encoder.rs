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
            let mut payload = Vec::new();
            payload.extend_from_slice(&(items.len() as u32).to_le_bytes());

            for item in items {
                let encoded = encode_value(item)?;
                let len_u32 = u32::try_from(encoded.payload.len())
                    .map_err(|_| SerializeError::LengthOverflow)?;

                payload.push(encoded.type_marker);
                payload.extend_from_slice(&len_u32.to_le_bytes());
                payload.extend_from_slice(&encoded.payload);
            }

            Ok(EncodedValue {
                type_marker: constants::TYPE_ARRAY,
                payload,
            })
        }
        Value::Object(map) => {
            let mut payload = Vec::new();
            payload.extend_from_slice(&(map.len() as u32).to_le_bytes());

            for (key, value) in map {
                let key_bytes = key.as_bytes();
                let key_len_u32 = u32::try_from(key_bytes.len())
                    .map_err(|_| SerializeError::LengthOverflow)?;

                let encoded = encode_value(value)?;
                let val_len_u32 = u32::try_from(encoded.payload.len())
                    .map_err(|_| SerializeError::LengthOverflow)?;

                payload.extend_from_slice(&key_len_u32.to_le_bytes());
                payload.extend_from_slice(key_bytes);

                payload.push(encoded.type_marker);
                payload.extend_from_slice(&val_len_u32.to_le_bytes());
                payload.extend_from_slice(&encoded.payload);
            }

            Ok(EncodedValue {
                type_marker: constants::TYPE_OBJECT,
                payload,
            })
        }
    }
}

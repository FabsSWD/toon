use crc32fast::Hasher;
use thiserror::Error;

use crate::{constants, Token};

use super::encoder::encode_value;
use super::writer::ByteWriter;

#[derive(Debug, Error)]
pub enum SerializeError {
    #[error("payload length does not fit in u32")]
    LengthOverflow,
}

pub struct Serializer;

impl Default for Serializer {
    fn default() -> Self {
        Self::new()
    }
}

impl Serializer {
    pub fn new() -> Self {
        Self
    }

    pub fn serialize(&self, token: &Token) -> Result<Vec<u8>, SerializeError> {
        let encoded = encode_value(token.value())?;
        let payload_len_u32 =
            u32::try_from(encoded.payload.len()).map_err(|_| SerializeError::LengthOverflow)?;

        let total_len = 1usize + 16 + 1 + 4 + encoded.payload.len() + 4;
        let mut writer = ByteWriter::with_capacity(total_len);

        writer.write_u8(constants::FORMAT_VERSION);
        writer.write_bytes(token.id().as_bytes());
        writer.write_u8(encoded.type_marker);
        writer.write_u32_le(payload_len_u32);
        writer.write_bytes(&encoded.payload);

        let checksum = crc32(writer.as_slice());
        writer.write_u32_le(checksum);

        Ok(writer.into_inner())
    }
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(bytes);
    hasher.finalize()
}

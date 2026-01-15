pub mod checksum;
pub mod constraints;
pub mod schema;

use crate::{constants, Deserializer, ToonError};

pub fn validate_token_bytes(bytes: &[u8]) -> Result<(), ToonError> {
    if bytes.len() < 1 + 16 + 1 + 4 + 4 {
        return Err(ToonError::Deserialization(crate::DeserializeError::Truncated));
    }

    let header = Deserializer::new(bytes).header()?;
    if !constants::is_supported_version(header.version) {
        return Err(ToonError::InvalidFormat {
            version: header.version,
            expected: constants::FORMAT_VERSION,
        });
    }

    let checksum_start = bytes
        .len()
        .checked_sub(4)
        .ok_or(crate::DeserializeError::Truncated)?;

    let payload_start: usize = 22;
    let payload_end = payload_start
        .checked_add(header.payload_len as usize)
        .ok_or(crate::DeserializeError::Truncated)?;

    if payload_end > checksum_start {
        return Err(ToonError::Deserialization(crate::DeserializeError::Truncated));
    }

    if payload_end != checksum_start {
        return Err(ToonError::Deserialization(crate::DeserializeError::TrailingBytes));
    }

    let actual = u32::from_le_bytes(
        bytes[checksum_start..]
            .try_into()
            .map_err(|_| crate::DeserializeError::Truncated)?,
    );
    let expected = checksum::crc32(&bytes[..checksum_start]);

    if expected != actual {
        return Err(ToonError::ChecksumMismatch {
            expected,
            actual,
            offset: checksum_start,
        });
    }

    Ok(())
}

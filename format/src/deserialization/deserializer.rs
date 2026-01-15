use crc32fast::Hasher;
use std::ops::Range;
use thiserror::Error;
use uuid::Uuid;

use crate::{constants, Metadata, Token, TokenId, Value};

use super::decoder::decode_value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenHeader {
    pub version: u8,
    pub id: [u8; 16],
    pub type_marker: u8,
    pub payload_len: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenLayout {
    pub header: TokenHeader,
    pub payload_range: Range<usize>,
    pub checksum_range: Range<usize>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DeserializeError {
    #[error("input is truncated")]
    Truncated,

    #[error("unsupported format version")]
    UnsupportedVersion,

    #[error("checksum mismatch")]
    ChecksumMismatch,

    #[error("unknown type marker")]
    UnknownTypeMarker(u8),

    #[error("invalid length for type")]
    InvalidLength,

    #[error("invalid utf-8")]
    InvalidUtf8,

    #[error("trailing bytes in payload")]
    TrailingBytes,

    #[error("invalid reference strength")]
    InvalidReferenceStrength,
}

pub struct Deserializer<'a> {
    bytes: &'a [u8],
}

impl<'a> Deserializer<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    pub fn header(&self) -> Result<TokenHeader, DeserializeError> {
        if self.bytes.len() < 1 + 16 + 1 + 4 + 4 {
            return Err(DeserializeError::Truncated);
        }

        let version = self.bytes[0];
        let id: [u8; 16] = self.bytes[1..17]
            .try_into()
            .map_err(|_| DeserializeError::Truncated)?;
        let type_marker = self.bytes[17];
        let payload_len = u32::from_le_bytes(
            self.bytes[18..22]
                .try_into()
                .map_err(|_| DeserializeError::Truncated)?,
        );

        Ok(TokenHeader {
            version,
            id,
            type_marker,
            payload_len,
        })
    }

    pub fn layout(&self) -> Result<TokenLayout, DeserializeError> {
        let header = self.header()?;

        if !constants::is_supported_version(header.version) {
            return Err(DeserializeError::UnsupportedVersion);
        }

        let checksum_start = self
            .bytes
            .len()
            .checked_sub(4)
            .ok_or(DeserializeError::Truncated)?;

        let payload_start: usize = 22;
        let payload_len_usize = header.payload_len as usize;
        let payload_end = payload_start
            .checked_add(payload_len_usize)
            .ok_or(DeserializeError::Truncated)?;

        if payload_end > checksum_start {
            return Err(DeserializeError::Truncated);
        }

        if payload_end != checksum_start {
            return Err(DeserializeError::TrailingBytes);
        }

        Ok(TokenLayout {
            header,
            payload_range: payload_start..payload_end,
            checksum_range: checksum_start..self.bytes.len(),
        })
    }

    pub fn deserialize(&self) -> Result<Token, DeserializeError> {
        let layout = self.layout()?;
        let header = layout.header;
        let checksum_offset = layout.checksum_range.start;
        let payload_start = layout.payload_range.start;
        let payload_end = layout.payload_range.end;

        let actual = u32::from_le_bytes(
            self.bytes[checksum_offset..]
                .try_into()
                .map_err(|_| DeserializeError::Truncated)?,
        );
        let expected = crc32(&self.bytes[..checksum_offset]);

        if expected != actual {
            return Err(DeserializeError::ChecksumMismatch);
        }

        let payload = &self.bytes[payload_start..payload_end];
        let value: Value = decode_value(header.type_marker, payload)?;

        let id = TokenId::from(Uuid::from_bytes(header.id));
        Ok(Token::new(id, value, Metadata::new(0, 0)))
    }
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(bytes);
    hasher.finalize()
}

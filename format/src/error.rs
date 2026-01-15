use thiserror::Error;

use crate::{DeserializeError, RegistryError, SerializeError, TokenId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintKind {
    StringLength,
    ArrayLength,
    ObjectLength,
    Depth,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ToonError {
    #[error("serialization error")]
    Serialization(SerializeError),

    #[error("deserialization error")]
    Deserialization(DeserializeError),

    #[error("checksum mismatch")]
    ChecksumMismatch {
        expected: u32,
        actual: u32,
        offset: usize,
    },

    #[error("invalid format version")]
    InvalidFormat { version: u8, expected: u8 },

    #[error("invalid reference")]
    InvalidReference(TokenId),

    #[error("circular reference detected")]
    CircularReference(Vec<TokenId>),

    #[error("constraint violation")]
    ConstraintViolation {
        kind: ConstraintKind,
        limit: usize,
        actual: usize,
    },

    #[error("schema mismatch")]
    SchemaViolation {
        path: String,
        expected: &'static str,
        actual: &'static str,
    },
}

impl From<SerializeError> for ToonError {
    fn from(value: SerializeError) -> Self {
        Self::Serialization(value)
    }
}

impl From<DeserializeError> for ToonError {
    fn from(value: DeserializeError) -> Self {
        Self::Deserialization(value)
    }
}

impl From<RegistryError> for ToonError {
    fn from(value: RegistryError) -> Self {
        match value {
            RegistryError::NotFound(id) => Self::InvalidReference(id),
            RegistryError::CircularReference(ids) => Self::CircularReference(ids),
        }
    }
}

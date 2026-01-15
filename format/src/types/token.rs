use uuid::Uuid;

use super::{Metadata, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TokenId(Uuid);

impl TokenId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn as_bytes(&self) -> &[u8; 16] {
        self.0.as_bytes()
    }
}

impl From<Uuid> for TokenId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl From<TokenId> for Uuid {
    fn from(value: TokenId) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    id: TokenId,
    value: Value,
    metadata: Metadata,
}

impl Token {
    pub fn new(id: TokenId, value: Value, metadata: Metadata) -> Self {
        Self { id, value, metadata }
    }

    pub fn id(&self) -> TokenId {
        self.id
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }
}

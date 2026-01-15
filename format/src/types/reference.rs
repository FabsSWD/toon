use super::TokenId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TokenRef {
    id: TokenId,
}

impl TokenRef {
    pub fn new(id: TokenId) -> Self {
        Self { id }
    }

    pub fn id(&self) -> TokenId {
        self.id
    }
}

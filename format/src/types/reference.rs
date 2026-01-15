use super::TokenId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenRefStrength {
    Strong,
    Weak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TokenRef {
    id: TokenId,
    strength: TokenRefStrength,
}

impl TokenRef {
    pub fn new(id: TokenId) -> Self {
        Self {
            id,
            strength: TokenRefStrength::Strong,
        }
    }

    pub fn strong(id: TokenId) -> Self {
        Self {
            id,
            strength: TokenRefStrength::Strong,
        }
    }

    pub fn weak(id: TokenId) -> Self {
        Self {
            id,
            strength: TokenRefStrength::Weak,
        }
    }

    pub fn id(&self) -> TokenId {
        self.id
    }

    pub fn strength(&self) -> TokenRefStrength {
        self.strength
    }
}

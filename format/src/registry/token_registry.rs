use std::sync::Arc;

use parking_lot::RwLock;
use thiserror::Error;

use crate::{Token, TokenId, TokenRef, TokenRefStrength};

use super::cache::LruCache;
use super::resolver;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RegistryError {
    #[error("token not found")]
    NotFound(TokenId),

    #[error("circular reference detected")]
    CircularReference(Vec<TokenId>),
}

pub struct TokenRegistry {
    cache: RwLock<LruCache<TokenId, Arc<Token>>>,
}

impl TokenRegistry {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(LruCache::new_unbounded()),
        }
    }

    pub fn with_max_entries(max_entries: usize) -> Self {
        Self {
            cache: RwLock::new(LruCache::with_max_entries(max_entries)),
        }
    }

    pub fn register(&self, token: Token) {
        let _ = self.insert(token);
    }

    pub fn get(&self, id: TokenId) -> Option<Arc<Token>> {
        self.cache.write().get_cloned(&id)
    }

    pub fn resolve_ref(&self, reference: &TokenRef) -> Result<Arc<Token>, RegistryError> {
        self.get(reference.id())
            .ok_or(RegistryError::NotFound(reference.id()))
    }

    pub fn resolve_ref_or_load<F>(
        &self,
        reference: &TokenRef,
        loader: F,
    ) -> Result<Option<Arc<Token>>, RegistryError>
    where
        F: FnOnce(TokenId) -> Option<Token>,
    {
        if let Some(token) = self.get(reference.id()) {
            return Ok(Some(token));
        }

        if reference.strength() == TokenRefStrength::Weak {
            return Ok(None);
        }

        let loaded = loader(reference.id()).ok_or(RegistryError::NotFound(reference.id()))?;
        Ok(Some(self.insert(loaded)))
    }

    pub fn ensure_loaded_and_acyclic<F>(&self, root: TokenId, loader: F) -> Result<(), RegistryError>
    where
        F: FnMut(TokenId) -> Option<Token>,
    {
        resolver::ensure_loaded_and_acyclic(
            root,
            loader,
            |id| self.get(id),
            |token| self.insert(token),
        )
    }

    fn insert(&self, token: Token) -> Arc<Token> {
        let token = Arc::new(token);
        let id = token.id();
        self.cache.write().insert(id, Arc::clone(&token));
        token
    }
}

impl Default for TokenRegistry {
    fn default() -> Self {
        Self::new()
    }
}

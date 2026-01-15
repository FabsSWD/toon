use std::collections::HashSet;
use std::sync::Arc;

use crate::{Token, TokenId, TokenRef, TokenRefStrength, Value};

use super::token_registry::RegistryError;

pub(crate) fn ensure_loaded_and_acyclic<F, Get, Insert>(
    root: TokenId,
    mut loader: F,
    mut get: Get,
    mut insert: Insert,
) -> Result<(), RegistryError>
where
    F: FnMut(TokenId) -> Option<Token>,
    Get: FnMut(TokenId) -> Option<Arc<Token>>,
    Insert: FnMut(Token) -> Arc<Token>,
{
    if get(root).is_none() {
        let loaded = loader(root).ok_or(RegistryError::NotFound(root))?;
        let _ = insert(loaded);
    }

    let mut visiting: HashSet<TokenId> = HashSet::new();
    let mut visited: HashSet<TokenId> = HashSet::new();
    let mut stack: Vec<TokenId> = Vec::new();

    visit_token(root, &mut loader, &mut get, &mut insert, &mut visiting, &mut visited, &mut stack)
}

fn visit_token<F, Get, Insert>(
    id: TokenId,
    loader: &mut F,
    get: &mut Get,
    insert: &mut Insert,
    visiting: &mut HashSet<TokenId>,
    visited: &mut HashSet<TokenId>,
    stack: &mut Vec<TokenId>,
) -> Result<(), RegistryError>
where
    F: FnMut(TokenId) -> Option<Token>,
    Get: FnMut(TokenId) -> Option<Arc<Token>>,
    Insert: FnMut(Token) -> Arc<Token>,
{
    if visited.contains(&id) {
        return Ok(());
    }

    if visiting.contains(&id) {
        let mut cycle = stack.clone();
        cycle.push(id);
        return Err(RegistryError::CircularReference(cycle));
    }

    visiting.insert(id);
    stack.push(id);

    let token = get(id).ok_or(RegistryError::NotFound(id))?;
    let mut refs: Vec<TokenRef> = Vec::new();
    collect_refs(token.value(), &mut refs);

    for r in refs {
        match r.strength() {
            TokenRefStrength::Strong => {
                if get(r.id()).is_none() {
                    let loaded = loader(r.id()).ok_or(RegistryError::NotFound(r.id()))?;
                    let _ = insert(loaded);
                }
                visit_token(r.id(), loader, get, insert, visiting, visited, stack)?;
            }
            TokenRefStrength::Weak => {
                if get(r.id()).is_some() {
                    visit_token(r.id(), loader, get, insert, visiting, visited, stack)?;
                }
            }
        }
    }

    stack.pop();
    visiting.remove(&id);
    visited.insert(id);
    Ok(())
}

fn collect_refs(value: &Value, out: &mut Vec<TokenRef>) {
    match value {
        Value::Ref(r) => out.push(r.clone()),
        Value::Array(items) => {
            for item in items {
                collect_refs(item, out);
            }
        }
        Value::Object(map) => {
            for v in map.values() {
                collect_refs(v, out);
            }
        }
        _ => {}
    }
}

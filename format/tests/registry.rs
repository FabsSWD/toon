use uuid::Uuid;

use toon_format::{Metadata, RegistryError, Token, TokenId, TokenRef, TokenRefStrength, TokenRegistry, Value};

#[test]
fn can_register_and_resolve() {
    let registry = TokenRegistry::new();

    let id = TokenId::from(Uuid::from_bytes([20u8; 16]));
    let token = Token::new(id, Value::String("A".to_string()), Metadata::new(0, 0));
    registry.register(token.clone());

    let r = TokenRef::new(id);
    let resolved = registry.resolve_ref(&r).unwrap();
    assert_eq!(&*resolved, &token);
}

#[test]
fn resolve_missing_returns_error() {
    let registry = TokenRegistry::new();

    let id = TokenId::from(Uuid::from_bytes([21u8; 16]));
    let r = TokenRef::new(id);

    let err = registry.resolve_ref(&r).unwrap_err();
    assert_eq!(err, RegistryError::NotFound(id));
}

#[test]
fn resolve_or_load_inserts_loaded_token() {
    let registry = TokenRegistry::new();

    let id = TokenId::from(Uuid::from_bytes([22u8; 16]));
    let r = TokenRef::new(id);

    let token = registry
        .resolve_ref_or_load(&r, |token_id| {
            Some(Token::new(
                token_id,
                Value::Int(42),
                Metadata::new(0, 0),
            ))
        })
        .unwrap()
        .expect("strong ref should load");

    assert_eq!(token.id(), id);
    assert_eq!(registry.get(id).unwrap().id(), token.id());
}

#[test]
fn weak_ref_missing_does_not_load() {
    let registry = TokenRegistry::new();

    let id = TokenId::from(Uuid::from_bytes([24u8; 16]));
    let r = TokenRef::weak(id);

    let loaded = registry
        .resolve_ref_or_load(&r, |_token_id| panic!("loader should not be called for weak refs"))
        .unwrap();

    assert!(loaded.is_none());
}

#[test]
fn ensure_loaded_and_acyclic_skips_missing_weak_refs() {
    let registry = TokenRegistry::new();

    let a = TokenId::from(Uuid::from_bytes([25u8; 16]));
    let b = TokenId::from(Uuid::from_bytes([26u8; 16]));

    let token_a = Token::new(
        a,
        Value::Object(
            [("b".to_string(), Value::Ref(TokenRef::weak(b)))]
                .into_iter()
                .collect(),
        ),
        Metadata::new(0, 0),
    );
    registry.register(token_a);

    registry
        .ensure_loaded_and_acyclic(a, |_id| None)
        .expect("missing weak refs should not fail");
}

#[test]
fn ensure_loaded_and_acyclic_detects_cycles() {
    let registry = TokenRegistry::new();

    let a = TokenId::from(Uuid::from_bytes([27u8; 16]));
    let b = TokenId::from(Uuid::from_bytes([28u8; 16]));

    let token_a = Token::new(
        a,
        Value::Object(
            [("b".to_string(), Value::Ref(TokenRef::new(b)))]
                .into_iter()
                .collect(),
        ),
        Metadata::new(0, 0),
    );
    let token_b = Token::new(
        b,
        Value::Object(
            [("a".to_string(), Value::Ref(TokenRef::new(a)))]
                .into_iter()
                .collect(),
        ),
        Metadata::new(0, 0),
    );

    registry.register(token_a);
    registry.register(token_b);

    let err = registry.ensure_loaded_and_acyclic(a, |_id| None).unwrap_err();
    match err {
        RegistryError::CircularReference(ids) => {
            assert!(!ids.is_empty());
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn lru_eviction_respects_max_entries() {
    let registry = TokenRegistry::with_max_entries(1);

    let a = TokenId::from(Uuid::from_bytes([29u8; 16]));
    let b = TokenId::from(Uuid::from_bytes([30u8; 16]));

    registry.register(Token::new(a, Value::Int(1), Metadata::new(0, 0)));
    registry.register(Token::new(b, Value::Int(2), Metadata::new(0, 0)));

    assert!(registry.get(a).is_none());
    assert!(registry.get(b).is_some());
}

#[test]
fn ensure_loaded_and_acyclic_loads_strong_refs_only() {
    let registry = TokenRegistry::new();

    let a = TokenId::from(Uuid::from_bytes([31u8; 16]));
    let b = TokenId::from(Uuid::from_bytes([32u8; 16]));
    let c = TokenId::from(Uuid::from_bytes([33u8; 16]));
    let d = TokenId::from(Uuid::from_bytes([34u8; 16]));

    registry.register(Token::new(
        a,
        Value::Array(vec![
            Value::Ref(TokenRef::new(b)),
            Value::Ref(TokenRef::weak(c)),
        ]),
        Metadata::new(0, 0),
    ));

    registry
        .ensure_loaded_and_acyclic(a, |id| {
            if id == b {
                Some(Token::new(
                    b,
                    Value::Object(
                        [("d".to_string(), Value::Ref(TokenRef::new(d)))]
                            .into_iter()
                            .collect(),
                    ),
                    Metadata::new(0, 0),
                ))
            } else if id == d {
                Some(Token::new(d, Value::Int(7), Metadata::new(0, 0)))
            } else {
                None
            }
        })
        .unwrap();

    assert!(registry.get(b).is_some());
    assert!(registry.get(d).is_some());
    assert!(registry.get(c).is_none());
}

#[test]
fn token_ref_strength_defaults_to_strong() {
    let id = TokenId::from(Uuid::from_bytes([23u8; 16]));
    let r = TokenRef::new(id);
    assert_eq!(r.strength(), TokenRefStrength::Strong);

    let weak = TokenRef::weak(id);
    assert_eq!(weak.strength(), TokenRefStrength::Weak);
}

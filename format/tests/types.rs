use std::collections::HashMap;

use toon_format::{Metadata, Token, TokenId, TokenRef, Value};

#[test]
fn token_id_is_unique() {
    let a = TokenId::new();
    let b = TokenId::new();
    assert_ne!(a, b);
}

#[test]
fn can_construct_token_and_ref() {
    let id = TokenId::new();
    let meta = Metadata::default();

    let token = Token::new(id, Value::String("Hello".to_string()), meta);
    assert_eq!(token.id(), id);

    let reference = TokenRef::new(token.id());
    assert_eq!(reference.id(), id);
}

#[test]
fn value_object_construction() {
    let mut obj = HashMap::new();
    obj.insert("answer".to_string(), Value::Int(42));

    let v = Value::Object(obj);
    match v {
        Value::Object(map) => assert_eq!(map.get("answer"), Some(&Value::Int(42))),
        _ => panic!("expected object"),
    }
}

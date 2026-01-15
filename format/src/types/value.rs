use std::collections::HashMap;

use super::TokenRef;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    Ref(TokenRef),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

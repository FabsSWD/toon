use std::collections::HashMap;
use std::f64::consts::PI;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use uuid::Uuid;

use toon_format::{Metadata, Serializer, Token, TokenId, Value};

fn build_sample_value() -> Value {
    let mut obj = HashMap::new();
    obj.insert("name".to_string(), Value::String("strand".to_string()));
    obj.insert("count".to_string(), Value::Int(123456));
    obj.insert("enabled".to_string(), Value::Bool(true));
    obj.insert(
        "items".to_string(),
        Value::Array(vec![
            Value::Null,
            Value::Float(PI),
            Value::String("hello".to_string()),
            Value::Array(vec![Value::Int(-1), Value::Int(0), Value::Int(1)]),
        ]),
    );
    Value::Object(obj)
}

fn build_token() -> Token {
    let id = TokenId::from(Uuid::from_bytes([9u8; 16]));
    Token::new(id, build_sample_value(), Metadata::new(0, 0))
}

fn build_json_value() -> serde_json::Value {
    serde_json::json!({
        "name": "strand",
        "count": 123456,
        "enabled": true,
        "items": [null, PI, "hello", [-1, 0, 1]]
    })
}

fn bench_serialize(c: &mut Criterion) {
    let token = build_token();
    let json_value = build_json_value();

    let serializer = Serializer::new();

    c.bench_function("toon_format::serialize", |b| {
        b.iter(|| serializer.serialize(black_box(&token)).unwrap())
    });

    c.bench_function("serde_json::to_vec", |b| {
        b.iter(|| serde_json::to_vec(black_box(&json_value)).unwrap())
    });
}

criterion_group!(benches, bench_serialize);
criterion_main!(benches);

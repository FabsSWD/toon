use std::collections::HashMap;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde::Serialize;
use uuid::Uuid;

use toon_format::{Metadata, Serializer, Token, TokenId, Value};

#[derive(Clone, Serialize)]
enum BenchValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    Array(Vec<BenchValue>),
    Object(HashMap<String, BenchValue>),
}

#[derive(Clone, Serialize)]
struct BenchMetadata {
    created_at_ms: u64,
    flags: u32,
}

#[derive(Clone, Serialize)]
struct BenchToken {
    version: u8,
    id: [u8; 16],
    value: BenchValue,
    metadata: BenchMetadata,
}

fn build_sample_value() -> Value {
    let mut obj = HashMap::new();
    obj.insert("name".to_string(), Value::String("strand".to_string()));
    obj.insert("count".to_string(), Value::Int(123456));
    obj.insert("enabled".to_string(), Value::Bool(true));
    obj.insert(
        "items".to_string(),
        Value::Array(vec![
            Value::Null,
            Value::Float(3.141592653589793),
            Value::String("hello".to_string()),
            Value::Array(vec![Value::Int(-1), Value::Int(0), Value::Int(1)]),
        ]),
    );
    Value::Object(obj)
}

fn build_sample_bench_value() -> BenchValue {
    let mut obj = HashMap::new();
    obj.insert("name".to_string(), BenchValue::String("strand".to_string()));
    obj.insert("count".to_string(), BenchValue::Int(123456));
    obj.insert("enabled".to_string(), BenchValue::Bool(true));
    obj.insert(
        "items".to_string(),
        BenchValue::Array(vec![
            BenchValue::Null,
            BenchValue::Float(3.141592653589793),
            BenchValue::String("hello".to_string()),
            BenchValue::Array(vec![
                BenchValue::Int(-1),
                BenchValue::Int(0),
                BenchValue::Int(1),
            ]),
        ]),
    );
    BenchValue::Object(obj)
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
        "items": [null, 3.141592653589793, "hello", [-1, 0, 1]]
    })
}

fn build_bincode_token() -> BenchToken {
    BenchToken {
        version: 1,
        id: [9u8; 16],
        value: build_sample_bench_value(),
        metadata: BenchMetadata {
            created_at_ms: 0,
            flags: 0,
        },
    }
}

fn bench_serialize(c: &mut Criterion) {
    let token = build_token();
    let json_value = build_json_value();
    let bincode_token = build_bincode_token();

    let serializer = Serializer::new();

    c.bench_function("toon_format::serialize", |b| {
        b.iter(|| serializer.serialize(black_box(&token)).unwrap())
    });

    c.bench_function("serde_json::to_vec", |b| {
        b.iter(|| serde_json::to_vec(black_box(&json_value)).unwrap())
    });

    c.bench_function("bincode::serialize", |b| {
        b.iter(|| bincode::serialize(black_box(&bincode_token)).unwrap())
    });
}

criterion_group!(benches, bench_serialize);
criterion_main!(benches);

use std::collections::HashMap;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use uuid::Uuid;

use toon_format::{Metadata, Token, TokenId, TokenRef, TokenRegistry, Value};

fn build_registry(count: usize) -> (TokenRegistry, Vec<TokenId>) {
    let registry = TokenRegistry::with_max_entries(count);
    let mut ids = Vec::with_capacity(count);

    for i in 0..count {
        let mut b = [0u8; 16];
        b[0] = (i & 0xFF) as u8;
        b[1] = ((i >> 8) & 0xFF) as u8;
        b[2] = ((i >> 16) & 0xFF) as u8;
        b[3] = ((i >> 24) & 0xFF) as u8;

        let id = TokenId::from(Uuid::from_bytes(b));
        ids.push(id);
        registry.register(Token::new(id, Value::Int(i as i64), Metadata::new(0, 0)));
    }

    (registry, ids)
}

fn bench_registry_get_10k(c: &mut Criterion) {
    let (registry, ids) = build_registry(10_000);

    c.bench_function("registry_get_hit_10k", |b| {
        let mut idx: usize = 0;
        b.iter(|| {
            idx = (idx + 1) % ids.len();
            let id = black_box(ids[idx]);
            black_box(registry.get(id).unwrap());
        })
    });
}

fn bench_registry_resolve_ref_10k(c: &mut Criterion) {
    let (registry, ids) = build_registry(10_000);

    c.bench_function("registry_resolve_ref_hit_10k", |b| {
        let mut idx: usize = 0;
        b.iter(|| {
            idx = (idx + 1) % ids.len();
            let id = black_box(ids[idx]);
            let r = TokenRef::new(id);
            black_box(registry.resolve_ref(&r).unwrap());
        })
    });
}

fn bench_registry_acyclic_traversal(c: &mut Criterion) {
    let registry = TokenRegistry::new();

    let a = TokenId::from(Uuid::from_bytes([1u8; 16]));
    let b = TokenId::from(Uuid::from_bytes([2u8; 16]));
    let c_id = TokenId::from(Uuid::from_bytes([3u8; 16]));

    let token_a = Token::new(
        a,
        Value::Object(
            [(
                "b".to_string(),
                Value::Ref(TokenRef::new(b)),
            )]
            .into_iter()
            .collect::<HashMap<_, _>>(),
        ),
        Metadata::new(0, 0),
    );

    let token_b = Token::new(
        b,
        Value::Object(
            [(
                "c".to_string(),
                Value::Ref(TokenRef::new(c_id)),
            )]
            .into_iter()
            .collect::<HashMap<_, _>>(),
        ),
        Metadata::new(0, 0),
    );

    let token_c = Token::new(c_id, Value::Int(123), Metadata::new(0, 0));

    registry.register(token_a);
    registry.register(token_b);
    registry.register(token_c);

    c.bench_function("registry_ensure_loaded_and_acyclic_small", |b| {
        b.iter(|| registry.ensure_loaded_and_acyclic(black_box(a), |_id| None).unwrap())
    });
}

criterion_group!(benches, bench_registry_get_10k, bench_registry_resolve_ref_10k, bench_registry_acyclic_traversal);
criterion_main!(benches);

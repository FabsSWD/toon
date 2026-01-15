use proptest::prelude::*;
use uuid::Uuid;

use toon_format::{Deserializer, Metadata, Serializer, Token, TokenId, Value};

fn value_strategy() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(Value::Int),
        any::<f64>()
            .prop_filter("finite and not NaN", |v| v.is_finite() && !v.is_nan())
            .prop_map(Value::Float),
        proptest::string::string_regex(r"[ -~]{0,32}")
            .unwrap()
            .prop_map(Value::String),
    ];

    leaf.prop_recursive(4, 64, 8, |inner| {
        prop_oneof![
            proptest::collection::vec(inner.clone(), 0..8).prop_map(Value::Array),
            proptest::collection::hash_map(
                proptest::string::string_regex(r"[a-zA-Z0-9_]{0,16}").unwrap(),
                inner,
                0..8,
            )
            .prop_map(Value::Object),
        ]
    })
}

proptest! {
    #[test]
    fn proptest_round_trip_value(id_bytes in any::<[u8;16]>(), value in value_strategy()) {
        let id = TokenId::from(Uuid::from_bytes(id_bytes));
        let meta = Metadata::new(0, 0);
        let token = Token::new(id, value.clone(), meta);

        let bytes = Serializer::new().serialize(&token).unwrap();
        let decoded = Deserializer::new(&bytes).deserialize().unwrap();

        prop_assert_eq!(decoded.id(), token.id());
        prop_assert_eq!(decoded.value(), &value);
    }

    #[test]
    fn proptest_deserialize_never_panics(input in proptest::collection::vec(any::<u8>(), 0..256)) {
        let bytes = input;

        let result = std::panic::catch_unwind(|| {
            let _ = Deserializer::new(&bytes).deserialize();
        });

        prop_assert!(result.is_ok());
    }
}

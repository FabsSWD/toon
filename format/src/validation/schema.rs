use std::collections::HashMap;

use crate::{ToonError, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Schema {
    Any,
    Null,
    Bool,
    Int,
    Float,
    String,
    Ref,
    Array(Box<Schema>),
    Object {
        fields: HashMap<String, Schema>,
        allow_extra: bool,
    },
}

pub fn validate_schema(value: &Value, schema: &Schema) -> Result<(), ToonError> {
    validate_schema_at(value, schema, "$")
}

fn validate_schema_at(value: &Value, schema: &Schema, path: &str) -> Result<(), ToonError> {
    match schema {
        Schema::Any => Ok(()),
        Schema::Null => match value {
            Value::Null => Ok(()),
            other => Err(mismatch(path, "null", actual_name(other))),
        },
        Schema::Bool => match value {
            Value::Bool(_) => Ok(()),
            other => Err(mismatch(path, "bool", actual_name(other))),
        },
        Schema::Int => match value {
            Value::Int(_) => Ok(()),
            other => Err(mismatch(path, "int", actual_name(other))),
        },
        Schema::Float => match value {
            Value::Float(_) => Ok(()),
            other => Err(mismatch(path, "float", actual_name(other))),
        },
        Schema::String => match value {
            Value::String(_) => Ok(()),
            other => Err(mismatch(path, "string", actual_name(other))),
        },
        Schema::Ref => match value {
            Value::Ref(_) => Ok(()),
            other => Err(mismatch(path, "ref", actual_name(other))),
        },
        Schema::Array(item_schema) => match value {
            Value::Array(items) => {
                for (idx, item) in items.iter().enumerate() {
                    let child_path = format!("{path}[{idx}]");
                    validate_schema_at(item, item_schema, &child_path)?;
                }
                Ok(())
            }
            other => Err(mismatch(path, "array", actual_name(other))),
        },
        Schema::Object { fields, allow_extra } => match value {
            Value::Object(map) => {
                for (k, s) in fields {
                    let Some(v) = map.get(k) else {
                        return Err(ToonError::SchemaViolation {
                            path: format!("{path}.{k}"),
                            expected: "present",
                            actual: "missing",
                        });
                    };
                    let child_path = format!("{path}.{k}");
                    validate_schema_at(v, s, &child_path)?;
                }

                if !*allow_extra {
                    for k in map.keys() {
                        if !fields.contains_key(k) {
                            return Err(ToonError::SchemaViolation {
                                path: format!("{path}.{k}"),
                                expected: "no extra fields",
                                actual: "extra field",
                            });
                        }
                    }
                }

                Ok(())
            }
            other => Err(mismatch(path, "object", actual_name(other))),
        },
    }
}

fn mismatch(path: &str, expected: &'static str, actual: &'static str) -> ToonError {
    ToonError::SchemaViolation {
        path: path.to_string(),
        expected,
        actual,
    }
}

fn actual_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Int(_) => "int",
        Value::Float(_) => "float",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
        Value::Ref(_) => "ref",
    }
}

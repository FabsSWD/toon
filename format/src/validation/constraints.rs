use crate::{ConstraintKind, ToonError, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Constraints {
    pub max_string_len: usize,
    pub max_array_len: usize,
    pub max_object_len: usize,
    pub max_depth: usize,
}

impl Default for Constraints {
    fn default() -> Self {
        Self {
            max_string_len: 1024 * 1024,
            max_array_len: 1_000_000,
            max_object_len: 1_000_000,
            max_depth: 256,
        }
    }
}

pub fn validate_value(value: &Value, constraints: Constraints) -> Result<(), ToonError> {
    validate_value_inner(value, constraints, 0)
}

fn validate_value_inner(value: &Value, constraints: Constraints, depth: usize) -> Result<(), ToonError> {
    if depth > constraints.max_depth {
        return Err(ToonError::ConstraintViolation {
            kind: ConstraintKind::Depth,
            limit: constraints.max_depth,
            actual: depth,
        });
    }

    match value {
        Value::String(s) => {
            if s.len() > constraints.max_string_len {
                return Err(ToonError::ConstraintViolation {
                    kind: ConstraintKind::StringLength,
                    limit: constraints.max_string_len,
                    actual: s.len(),
                });
            }
        }
        Value::Array(items) => {
            if items.len() > constraints.max_array_len {
                return Err(ToonError::ConstraintViolation {
                    kind: ConstraintKind::ArrayLength,
                    limit: constraints.max_array_len,
                    actual: items.len(),
                });
            }
            for item in items {
                validate_value_inner(item, constraints, depth + 1)?;
            }
        }
        Value::Object(map) => {
            if map.len() > constraints.max_object_len {
                return Err(ToonError::ConstraintViolation {
                    kind: ConstraintKind::ObjectLength,
                    limit: constraints.max_object_len,
                    actual: map.len(),
                });
            }
            for v in map.values() {
                validate_value_inner(v, constraints, depth + 1)?;
            }
        }
        Value::Ref(_) | Value::Null | Value::Bool(_) | Value::Int(_) | Value::Float(_) => {}
    }

    Ok(())
}

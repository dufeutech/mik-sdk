//! Constructor functions for building JSON values.

use super::value::JsonValue;
use miniserde::json::{Array, Number, Object, Value};

/// Create an empty object `{}`.
#[must_use]
pub fn obj() -> JsonValue {
    JsonValue::new(Value::Object(Object::new()))
}

/// Create an empty array `[]`.
#[must_use]
pub fn arr() -> JsonValue {
    JsonValue::new(Value::Array(Array::new()))
}

/// Create a string value.
#[must_use]
pub fn str<S: AsRef<str>>(value: S) -> JsonValue {
    JsonValue::new(Value::String(value.as_ref().to_string()))
}

/// Create an integer value.
#[must_use]
pub fn int(value: i64) -> JsonValue {
    JsonValue::new(Value::Number(Number::I64(value)))
}

/// Create a float value.
///
/// # Precision Note
///
/// JSON numbers are typically parsed as f64 by JavaScript, which has limited
/// precision for integers > 2^53. If you're building JSON for JavaScript consumption,
/// consider using string values for large integers to preserve precision.
#[must_use]
pub fn float(value: f64) -> JsonValue {
    JsonValue::new(Value::Number(Number::F64(value)))
}

/// Create a boolean value.
#[must_use]
pub fn bool(value: bool) -> JsonValue {
    JsonValue::new(Value::Bool(value))
}

/// Create a null value.
#[must_use]
pub fn null() -> JsonValue {
    JsonValue::null()
}

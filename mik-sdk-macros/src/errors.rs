//! Error helper utilities for consistent, informative compile-time errors.
//!
//! This module provides utilities for building user-friendly error messages
//! in proc-macros with examples and context.
//!
//! These helpers are available for use in improving error messages throughout
//! the macro implementations.

#![allow(dead_code)] // Helpers provided for future use

use proc_macro2::Span;
use syn::Error;

/// Build a formatted error with examples.
///
/// # Examples
///
/// ```ignore
/// use crate::errors::parse_error;
///
/// return Err(parse_error(
///     span,
///     "Invalid field type",
///     &["str(expr)", "int(expr)", "float(expr)", "bool(expr)"]
/// ));
/// ```
pub fn parse_error(span: Span, message: &str, examples: &[&str]) -> Error {
    let examples_str = examples
        .iter()
        .map(|e| format!("  {e}"))
        .collect::<Vec<_>>()
        .join("\n");

    Error::new(span, format!("{message}\n\nExamples:\n{examples_str}"))
}

/// Build an error for an unknown identifier with valid options.
///
/// # Examples
///
/// ```ignore
/// use crate::errors::unknown_error;
///
/// return Err(unknown_error(
///     span,
///     "operator",
///     "$invalid",
///     &["$eq", "$ne", "$gt", "$lt"]
/// ));
/// ```
pub fn unknown_error(span: Span, kind: &str, got: &str, valid: &[&str]) -> Error {
    let valid_str = valid.join(", ");
    Error::new(
        span,
        format!("Unknown {kind} '{got}'.\n\nValid options: {valid_str}"),
    )
}

/// Build an error for a missing required field.
///
/// # Examples
///
/// ```ignore
/// use crate::errors::missing_field_error;
///
/// return Err(missing_field_error(
///     span,
///     "status",
///     "error! { status: 404, title: \"Not Found\" }"
/// ));
/// ```
pub fn missing_field_error(span: Span, field: &str, example: &str) -> Error {
    Error::new(
        span,
        format!("Missing required field '{field}'.\n\nExample:\n  {example}"),
    )
}

/// Build an error for a duplicate field.
pub fn duplicate_field_error(span: Span, field: &str) -> Error {
    Error::new(
        span,
        format!("Duplicate '{field}' field. Each field can only appear once."),
    )
}

/// Build an error for an invalid field attribute value.
///
/// # Examples
///
/// ```ignore
/// use crate::errors::invalid_attr;
///
/// return Err(invalid_attr(
///     span,
///     "min",
///     "a number",
///     "#[field(min = 1)]"
/// ));
/// ```
pub fn invalid_attr(span: Span, attr: &str, expected: &str, example: &str) -> Error {
    Error::new(
        span,
        format!("'{attr}' expects {expected}.\n\n\u{2705} Correct: {example}",),
    )
}

/// Build an error for expected syntax.
///
/// # Examples
///
/// ```ignore
/// use crate::errors::expected_syntax;
///
/// return Err(expected_syntax(
///     span,
///     "=>",
///     "after route path",
///     "GET \"/users\" => list_users"
/// ));
/// ```
pub fn expected_syntax(span: Span, expected: &str, context: &str, example: &str) -> Error {
    Error::new(
        span,
        format!("Expected {expected} {context}.\n\nExample: {example}"),
    )
}

/// Build an error for type mismatch.
pub fn type_mismatch(span: Span, field: &str, expected: &str, got: &str) -> Error {
    Error::new(
        span,
        format!("Type mismatch for '{field}'.\n\nExpected: {expected}\nGot: {got}"),
    )
}

/// Build an error for unsupported construct.
pub fn unsupported(span: Span, what: &str, suggestion: &str) -> Error {
    Error::new(span, format!("{what}\n\n\u{2705} Try: {suggestion}"))
}

/// Extension trait to add context to errors.
pub trait ErrorContext {
    /// Wrap an error with additional context.
    fn with_context(self, context: &str) -> Error;
}

impl ErrorContext for Error {
    fn with_context(self, context: &str) -> Self {
        Self::new(self.span(), format!("{context}\n\nCaused by: {self}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_formats_correctly() {
        let err = parse_error(Span::call_site(), "Invalid value", &["str(x)", "int(y)"]);
        let msg = err.to_string();
        assert!(msg.contains("Invalid value"));
        assert!(msg.contains("str(x)"));
        assert!(msg.contains("int(y)"));
    }

    #[test]
    fn test_unknown_error_formats_correctly() {
        let err = unknown_error(Span::call_site(), "operator", "$bad", &["$eq", "$ne"]);
        let msg = err.to_string();
        assert!(msg.contains("Unknown operator"));
        assert!(msg.contains("$bad"));
        assert!(msg.contains("$eq, $ne"));
    }
}

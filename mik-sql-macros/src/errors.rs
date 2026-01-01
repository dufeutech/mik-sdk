//! Error helper utilities for consistent, informative compile-time errors.
//!
//! This module provides utilities for building user-friendly error messages
//! in proc-macros with examples and context.

#![allow(dead_code)] // Helpers provided for future use

use proc_macro2::Span;
use syn::Error;

/// Build an error for an unknown identifier with valid options.
pub fn unknown_error(span: Span, kind: &str, got: &str, valid: &[&str]) -> Error {
    let valid_str = valid.join(", ");
    Error::new(
        span,
        format!("Unknown {kind} '{got}'.\n\nValid options: {valid_str}"),
    )
}

/// Build an error for a missing required field.
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

/// Build an error for expected syntax.
pub fn expected_syntax(span: Span, expected: &str, context: &str, example: &str) -> Error {
    Error::new(
        span,
        format!("Expected {expected} {context}.\n\nExample: {example}"),
    )
}

/// Build an error for an empty block that requires content.
pub fn empty_block_error(span: Span, block_type: &str, example: &str) -> Error {
    Error::new(
        span,
        format!("Empty {block_type} block. At least one item required.\n\nExample:\n  {example}"),
    )
}

/// Build an error for invalid operator.
pub fn invalid_operator(span: Span, op: &str) -> Error {
    Error::new(
        span,
        format!(
            "Unknown operator '${op}'.\n\n\
             Valid operators:\n\
             \u{2022} Comparison: $eq, $ne, $gt, $gte, $lt, $lte\n\
             \u{2022} Collection: $in, $nin\n\
             \u{2022} String: $like, $ilike, $starts_with, $ends_with, $contains\n\
             \u{2022} Range: $between\n\
             \u{2022} Logical: $and, $or, $not"
        ),
    )
}

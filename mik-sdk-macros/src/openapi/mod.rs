//! OpenAPI schema and specification generation.
//!
//! This module consolidates all OpenAPI-related functionality:
//! - `utoipa`: Low-level schema builders using utoipa types
//! - `routes`: OpenAPI specification generation for routes

pub mod routes;
pub mod utoipa;

// Re-export commonly used items
pub use routes::generate_openapi_json;
pub use utoipa::{
    array_schema, make_nullable_json, ref_or_schema_to_json, rust_type_to_schema, schema_to_json,
};

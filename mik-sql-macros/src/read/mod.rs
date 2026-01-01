//! `sql_read!` macro implementation for SELECT queries.
//!
//! This module is split into submodules for maintainability:
//! - `input`: `SqlInput` struct and Parse implementation
//! - `codegen`: Token generation for the query builder

mod codegen;
mod input;

use proc_macro::TokenStream;
use syn::parse_macro_input;

use codegen::generate_sql_tokens;
use input::SqlInput;

/// Build a SELECT query using the query builder (CRUD: Read).
pub fn sql_read_impl(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as SqlInput);
    TokenStream::from(generate_sql_tokens(parsed))
}

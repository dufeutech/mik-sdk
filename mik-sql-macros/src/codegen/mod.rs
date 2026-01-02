//! Code generation utilities for SQL CRUD macros.
//!
//! This module contains functions that convert parsed AST types into
//! `TokenStream` output for the generated Rust code.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

mod filter;
mod value;

pub use filter::sql_filter_expr_to_tokens;
pub use value::{compute_expr_to_sql, sql_aggregate_to_tokens, sql_value_to_tokens};

/// Generate the `.returning(&[...])` method chain for CRUD operations.
pub fn generate_returning_chain(returning: &[syn::Ident]) -> TokenStream2 {
    if returning.is_empty() {
        quote! {}
    } else {
        let ret_strs: Vec<String> = returning
            .iter()
            .map(std::string::ToString::to_string)
            .collect();
        quote! { .returning(&[#(#ret_strs),*]) }
    }
}

//! #[derive(Type)] implementation for JSON body/response types.
//!
//! This module is split into submodules for maintainability:
//! - `case`: PascalCase to snake_case conversion
//! - `enum_impl`: Enum derive implementation
//! - `struct_impl`: Struct derive implementation
//! - `validation`: Validation code generation

mod case;
mod enum_impl;
mod struct_impl;
mod validation;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{Data, DeriveInput, parse_macro_input};

/// Entry point for #[derive(Type)] macro.
#[allow(clippy::too_many_lines)]
#[allow(clippy::cognitive_complexity)]
pub fn derive_type_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match &input.data {
        Data::Enum(data_enum) => enum_impl::derive_enum_type_impl(&input, data_enum),
        Data::Struct(data_struct) => struct_impl::derive_struct_type_impl(&input, data_struct),
        Data::Union(_) => syn::Error::new_spanned(
            &input,
            "Oops! #[derive(Type)] only works on structs and enums, not unions.\n\
             \n\
             \u{2705} What you need:\n\
               #[derive(Type)]\n\
               struct MyType { field: String }\n\
             \n\
               // Or for enums:\n\
               #[derive(Type)]\n\
               enum Status { Active, Inactive }",
        )
        .to_compile_error()
        .into(),
    }
}

/// Inner implementation for potential future refactoring.
#[allow(dead_code)]
pub fn derive_type_impl_inner(input: proc_macro2::TokenStream) -> TokenStream2 {
    let input = match syn::parse2::<DeriveInput>(input) {
        Ok(i) => i,
        Err(e) => return e.to_compile_error(),
    };

    match &input.data {
        Data::Enum(data_enum) => {
            TokenStream2::from(enum_impl::derive_enum_type_impl(&input, data_enum))
        },
        Data::Struct(data_struct) => {
            TokenStream2::from(struct_impl::derive_struct_type_impl(&input, data_struct))
        },
        Data::Union(_) => syn::Error::new_spanned(
            &input,
            "Oops! #[derive(Type)] only works on structs and enums, not unions.\n\
             \n\
             \u{2705} What you need:\n\
               #[derive(Type)]\n\
               struct MyType { field: String }\n\
             \n\
               // Or for enums:\n\
               #[derive(Type)]\n\
               enum Status { Active, Inactive }",
        )
        .to_compile_error(),
    }
}

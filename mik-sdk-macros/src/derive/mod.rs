//! Derive macros for typed inputs: Type, Query, Path.
//!
//! These generate implementations for FromJson, FromQuery, FromPath traits,
//! along with OpenAPI schema generation and optional validation.

mod path_derive;
mod query_derive;
mod type_derive;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Expr, Fields, Lit, Type};

// Re-export the public entry points
pub use path_derive::derive_path_impl;
pub use query_derive::derive_query_impl;
pub use type_derive::derive_type_impl;

// ============================================================================
// JSON STRING ESCAPING
// ============================================================================

/// Escape a string for use in JSON output.
/// Handles all JSON control characters per RFC 8259.
pub(crate) fn escape_json_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            // Control characters (U+0000 to U+001F)
            c if c.is_control() => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            },
            c => result.push(c),
        }
    }
    result
}

// ============================================================================
// FIELD ATTRIBUTE PARSING
// ============================================================================

#[derive(Default, Clone)]
pub(crate) struct FieldAttrs {
    pub(crate) min: Option<i64>,
    pub(crate) max: Option<i64>,
    pub(crate) format: Option<String>,
    pub(crate) pattern: Option<String>,
    pub(crate) default: Option<String>,
    pub(crate) rename: Option<String>,
    pub(crate) docs: Option<String>,
}

pub(crate) fn parse_field_attrs(attrs: &[Attribute]) -> Result<FieldAttrs, syn::Error> {
    let mut result = FieldAttrs::default();

    for attr in attrs {
        if !attr.path().is_ident("field") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("min") {
                let value: Lit = meta.value()?.parse()?;
                if let Lit::Int(lit) = value {
                    result.min = lit.base10_parse().ok();
                }
            } else if meta.path.is_ident("max") {
                let value: Lit = meta.value()?.parse()?;
                if let Lit::Int(lit) = value {
                    result.max = lit.base10_parse().ok();
                }
            } else if meta.path.is_ident("format") {
                let value: Lit = meta.value()?.parse()?;
                if let Lit::Str(lit) = value {
                    result.format = Some(lit.value());
                }
            } else if meta.path.is_ident("pattern") {
                let value: Lit = meta.value()?.parse()?;
                if let Lit::Str(lit) = value {
                    result.pattern = Some(lit.value());
                }
            } else if meta.path.is_ident("default") {
                let value: Expr = meta.value()?.parse()?;
                result.default = Some(quote!(#value).to_string());
            } else if meta.path.is_ident("rename") {
                let value: Lit = meta.value()?.parse()?;
                if let Lit::Str(lit) = value {
                    result.rename = Some(lit.value());
                }
            } else if meta.path.is_ident("docs") {
                let value: Lit = meta.value()?.parse()?;
                if let Lit::Str(lit) = value {
                    result.docs = Some(lit.value());
                }
            }
            Ok(())
        })?;
    }

    Ok(result)
}

// ============================================================================
// TYPE HELPERS
// ============================================================================

pub(crate) fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "Option";
    }
    false
}

pub(crate) fn get_inner_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
    {
        return Some(inner);
    }
    None
}

pub(crate) fn type_to_openapi(ty: &Type) -> String {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        let name = segment.ident.to_string();
        return match name.as_str() {
            "String" | "str" => r#"{"type":"string"}"#.to_string(),
            "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "isize" | "usize" => {
                r#"{"type":"integer"}"#.to_string()
            },
            "f32" | "f64" => r#"{"type":"number"}"#.to_string(),
            "bool" => r#"{"type":"boolean"}"#.to_string(),
            "Option" => {
                if let Some(inner) = get_inner_type(ty) {
                    let inner_schema = type_to_openapi(inner);
                    // Remove the outer braces and add nullable (safe extraction)
                    let inner_content = inner_schema
                        .strip_prefix('{')
                        .and_then(|s| s.strip_suffix('}'))
                        .unwrap_or(&inner_schema);
                    format!("{{{},\"nullable\":true}}", inner_content)
                } else {
                    r#"{"type":"object","nullable":true}"#.to_string()
                }
            },
            "Vec" => {
                if let Some(inner) = get_inner_type(ty) {
                    let inner_schema = type_to_openapi(inner);
                    format!(r#"{{"type":"array","items":{}}}"#, inner_schema)
                } else {
                    r#"{"type":"array"}"#.to_string()
                }
            },
            _ => {
                // Assume it's a reference to another schema
                format!("{{\"$ref\":\"#/components/schemas/{}\"}}", name)
            },
        };
    }
    r#"{"type":"object"}"#.to_string()
}

pub(crate) fn rust_type_to_json_getter(ty: &Type) -> Option<TokenStream2> {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        let name = segment.ident.to_string();
        return match name.as_str() {
            "String" => Some(quote! { .str() }),
            "i8" | "i16" | "i32" => Some(quote! { .int().map(|n| n as _) }),
            "i64" => Some(quote! { .int() }),
            "u8" | "u16" | "u32" => Some(quote! { .int().map(|n| n as _) }),
            "u64" | "usize" => Some(quote! { .int().map(|n| n as _) }),
            "f32" => Some(quote! { .float().map(|n| n as f32) }),
            "f64" => Some(quote! { .float() }),
            "bool" => Some(quote! { .bool() }),
            _ => None, // Complex type - use FromJson trait
        };
    }
    None
}

/// Get a human-readable type name for error messages
pub(crate) fn rust_type_to_name(ty: &Type) -> &'static str {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        let name = segment.ident.to_string();
        return match name.as_str() {
            "String" => "string",
            "i8" | "i16" | "i32" | "i64" => "integer",
            "u8" | "u16" | "u32" | "u64" | "usize" => "integer",
            "f32" | "f64" => "number",
            "bool" => "boolean",
            "Vec" => "array",
            "Option" => "value",
            _ => "object",
        };
    }
    "value"
}

// ============================================================================
// STRUCT FIELD EXTRACTION HELPER
// ============================================================================

/// Context for derive macro error messages
pub(crate) enum DeriveContext {
    Type,
    Query,
    Path,
}

impl DeriveContext {
    fn name(&self) -> &'static str {
        match self {
            DeriveContext::Type => "Type",
            DeriveContext::Query => "Query",
            DeriveContext::Path => "Path",
        }
    }

    fn example(&self) -> &'static str {
        match self {
            DeriveContext::Type => "struct MyType { field: String }",
            DeriveContext::Query => "struct MyQuery { page: u32, limit: u32 }",
            DeriveContext::Path => "struct UserPath { org_id: String, id: String }",
        }
    }

    fn purpose(&self) -> &'static str {
        match self {
            DeriveContext::Type => "for JSON body/response types",
            DeriveContext::Query => "for query parameters",
            DeriveContext::Path => "for URL path parameters",
        }
    }
}

/// Extract named fields from a DeriveInput, returning an error TokenStream if invalid.
pub(crate) fn extract_named_fields(
    input: &DeriveInput,
    ctx: DeriveContext,
) -> Result<&syn::punctuated::Punctuated<syn::Field, syn::token::Comma>, TokenStream> {
    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => Ok(&fields.named),
            _ => Err(syn::Error::new_spanned(
                input,
                format!(
                    "{} derive only supports structs with named fields. \
                     Example: `{}`",
                    ctx.name(),
                    ctx.example()
                ),
            )
            .to_compile_error()
            .into()),
        },
        _ => Err(syn::Error::new_spanned(
            input,
            format!(
                "{} derive only supports structs. \
                 Hint: Use `#[derive({})]` on a struct {}.",
                ctx.name(),
                ctx.name(),
                ctx.purpose()
            ),
        )
        .to_compile_error()
        .into()),
    }
}

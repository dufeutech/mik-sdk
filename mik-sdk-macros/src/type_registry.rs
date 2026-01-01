//! Centralized type registry for Rust type mappings.
//!
//! This module provides a single source of truth for mapping Rust types to:
//! - JSON getter methods (for parsing)
//! - OpenAPI schemas (for documentation)
//! - Human-readable names (for error messages)
//!
//! Adding support for a new type requires only updating this registry.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Type;

/// Information about a Rust type for macro code generation.
pub struct TypeInfo {
    /// Rust type names that match this entry (e.g., `["String", "&str"]`)
    pub rust_names: &'static [&'static str],
    /// JSON getter method to use (e.g., `"str"`, `"int"`, `"float"`, `"bool"`)
    pub json_getter: &'static str,
    /// Human-readable name for error messages
    pub display_name: &'static str,
}

/// The type registry - single source of truth for type mappings.
/// OpenAPI schema generation is now handled by the openapi module using utoipa.
pub static TYPE_REGISTRY: &[TypeInfo] = &[
    TypeInfo {
        rust_names: &["String"],
        json_getter: "str",
        display_name: "string",
    },
    TypeInfo {
        rust_names: &["i8", "i16", "i32", "u8", "u16", "u32", "u64", "usize"],
        json_getter: "int_cast",
        display_name: "integer",
    },
    TypeInfo {
        rust_names: &["i64"],
        json_getter: "int",
        display_name: "integer",
    },
    TypeInfo {
        rust_names: &["f32"],
        json_getter: "float_f32",
        display_name: "number",
    },
    TypeInfo {
        rust_names: &["f64"],
        json_getter: "float",
        display_name: "number",
    },
    TypeInfo {
        rust_names: &["bool"],
        json_getter: "bool",
        display_name: "boolean",
    },
];

/// Look up type info by Rust type name.
pub fn lookup_type(type_name: &str) -> Option<&'static TypeInfo> {
    TYPE_REGISTRY
        .iter()
        .find(|t| t.rust_names.contains(&type_name))
}

/// Get the JSON getter TokenStream for a type.
pub fn get_json_getter(ty: &Type) -> Option<TokenStream2> {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        let name = segment.ident.to_string();
        if let Some(info) = lookup_type(&name) {
            return Some(match info.json_getter {
                "str" => quote! { .str() },
                "int" => quote! { .int() },
                "int_cast" => quote! { .int().map(|n| n as _) },
                "float" => quote! { .float() },
                "float_f32" => quote! { .float().map(|n| n as f32) },
                "bool" => quote! { .bool() },
                _ => return None,
            });
        }
    }
    None
}

/// Get the OpenAPI schema for a type (handles Option, Vec, and references).
/// Uses utoipa for type-safe schema generation.
pub fn get_openapi_schema(ty: &Type) -> String {
    use crate::openapi::{
        array_schema, make_nullable_json, ref_or_schema_to_json, rust_type_to_schema,
    };

    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        let name = segment.ident.to_string();

        // Handle wrapper types
        if name == "Option" {
            if let Some(inner) = get_inner_type(ty) {
                let inner_schema = get_openapi_schema(inner);
                return make_nullable_json(&inner_schema);
            }
            return r#"{"nullable":true,"type":"object"}"#.to_string();
        }

        if name == "Vec" {
            if let Some(inner) = get_inner_type(ty) {
                let inner_schema_ref = rust_type_to_schema(&get_type_name_from_type(inner));
                let arr = array_schema(inner_schema_ref);
                return crate::openapi::schema_to_json(&arr);
            }
            return r#"{"type":"array"}"#.to_string();
        }

        // Use utoipa for basic types
        let schema = rust_type_to_schema(&name);
        return ref_or_schema_to_json(&schema);
    }

    r#"{"type":"object"}"#.to_string()
}

/// Helper to get type name string from a Type.
fn get_type_name_from_type(ty: &Type) -> String {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident.to_string();
    }
    "object".to_string()
}

/// Get a human-readable type name for error messages.
pub fn get_type_name(ty: &Type) -> &'static str {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        let name = segment.ident.to_string();

        // Handle wrapper types
        if name == "Option" {
            return "value";
        }
        if name == "Vec" {
            return "array";
        }

        // Look up in registry
        if let Some(info) = lookup_type(&name) {
            return info.display_name;
        }

        return "object";
    }
    "value"
}

/// Check if a type is Option<T>.
pub fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "Option";
    }
    false
}

/// Get the inner type from Option<T> or Vec<T>.
pub fn get_inner_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
    {
        return Some(inner);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_string() {
        let info = lookup_type("String").unwrap();
        assert_eq!(info.json_getter, "str");
        assert_eq!(info.display_name, "string");
    }

    #[test]
    fn test_lookup_i64() {
        let info = lookup_type("i64").unwrap();
        assert_eq!(info.json_getter, "int");
        assert_eq!(info.display_name, "integer");
    }

    #[test]
    fn test_lookup_unknown() {
        assert!(lookup_type("MyCustomType").is_none());
    }
}

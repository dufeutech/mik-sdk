//! Validation code generation for #[derive(Type)].

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::derive::FieldAttrs;

/// Generate validation check code for a field.
pub fn generate_validation_checks(
    attrs: &FieldAttrs,
    field_name: &syn::Ident,
    is_optional: bool,
    base_schema: &str,
    validation_checks: &mut Vec<TokenStream2>,
) {
    if let Some(min) = attrs.min {
        let field_name_str = field_name.to_string();
        if is_optional {
            // Validate optional fields when Some
            if base_schema.contains("string") {
                validation_checks.push(quote! {
                    if let Some(ref __val) = self.#field_name {
                        if __val.len() < #min as usize {
                            return Err(mik_sdk::typed::ValidationError::min(#field_name_str, #min));
                        }
                    }
                });
            } else {
                // Use i128 for safe comparison across all integer types (avoids u64 -> i64 overflow)
                validation_checks.push(quote! {
                    if let Some(__val) = self.#field_name {
                        if (__val as i128) < (#min as i128) {
                            return Err(mik_sdk::typed::ValidationError::min(#field_name_str, #min));
                        }
                    }
                });
            }
        } else if base_schema.contains("string") {
            validation_checks.push(quote! {
                if self.#field_name.len() < #min as usize {
                    return Err(mik_sdk::typed::ValidationError::min(#field_name_str, #min));
                }
            });
        } else {
            // Use i128 for safe comparison across all integer types (avoids u64 -> i64 overflow)
            validation_checks.push(quote! {
                if (self.#field_name as i128) < (#min as i128) {
                    return Err(mik_sdk::typed::ValidationError::min(#field_name_str, #min));
                }
            });
        }
    }
    if let Some(max) = attrs.max {
        let field_name_str = field_name.to_string();
        if is_optional {
            // Validate optional fields when Some
            if base_schema.contains("string") {
                validation_checks.push(quote! {
                    if let Some(ref __val) = self.#field_name {
                        if __val.len() > #max as usize {
                            return Err(mik_sdk::typed::ValidationError::max(#field_name_str, #max));
                        }
                    }
                });
            } else {
                // Use i128 for safe comparison across all integer types (avoids u64 -> i64 overflow)
                validation_checks.push(quote! {
                    if let Some(__val) = self.#field_name {
                        if (__val as i128) > (#max as i128) {
                            return Err(mik_sdk::typed::ValidationError::max(#field_name_str, #max));
                        }
                    }
                });
            }
        } else if base_schema.contains("string") {
            validation_checks.push(quote! {
                if self.#field_name.len() > #max as usize {
                    return Err(mik_sdk::typed::ValidationError::max(#field_name_str, #max));
                }
            });
        } else {
            // Use i128 for safe comparison across all integer types (avoids u64 -> i64 overflow)
            validation_checks.push(quote! {
                if (self.#field_name as i128) > (#max as i128) {
                    return Err(mik_sdk::typed::ValidationError::max(#field_name_str, #max));
                }
            });
        }
    }
}

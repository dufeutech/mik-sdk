//! Struct implementation for #[derive(Type)].

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Fields};

use super::validation::generate_validation_checks;
use crate::derive::{
    escape_json_string, get_inner_type, is_option_type, parse_field_attrs,
    rust_type_to_json_getter, rust_type_to_name, type_to_openapi,
};

/// Generate FromJson, Validate, and OpenApiSchema implementations for structs.
#[allow(clippy::too_many_lines)]
pub fn derive_struct_type_impl(input: &DeriveInput, data_struct: &syn::DataStruct) -> TokenStream {
    let name = &input.ident;
    let name_str = name.to_string();

    let fields = match &data_struct.fields {
        Fields::Named(fields) => &fields.named,
        Fields::Unnamed(_) => {
            return syn::Error::new_spanned(
                input,
                "Oops! #[derive(Type)] needs named fields, not tuple fields.\n\
                 \n\
                 \u{274C} What you have (tuple struct):\n\
                   struct MyStruct(String, i32);\n\
                 \n\
                 \u{2705} What you need (named fields):\n\
                   #[derive(Type)]\n\
                   struct MyType { field: String }\n\
                 \n\
                 Named fields have names like 'id', 'name', 'email' - they make\n\
                 your code easier to read and work with JSON/query params.",
            )
            .to_compile_error()
            .into();
        },
        Fields::Unit => {
            return syn::Error::new_spanned(
                input,
                "Oops! #[derive(Type)] needs a struct with fields.\n\
                 \n\
                 \u{274C} What you have (empty struct):\n\
                   struct MyStruct;\n\
                 \n\
                 \u{2705} What you need:\n\
                   #[derive(Type)]\n\
                   struct MyType { field: String }\n\
                 \n\
                 Add some fields to hold your data!",
            )
            .to_compile_error()
            .into();
        },
    };

    // Generate from_json implementation
    let mut from_json_fields = Vec::new();
    let mut required_fields = Vec::new();
    let mut openapi_properties = Vec::new();
    let mut validation_checks: Vec<TokenStream2> = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        let attrs = match parse_field_attrs(&field.attrs) {
            Ok(attrs) => attrs,
            Err(e) => return e.to_compile_error().into(),
        };

        let json_key = attrs
            .rename
            .clone()
            .unwrap_or_else(|| field_name.to_string());
        let is_optional = is_option_type(field_ty);

        // Generate from_json field extraction
        if is_optional {
            let inner_ty = get_inner_type(field_ty);
            let inner_getter = inner_ty.and_then(rust_type_to_json_getter);

            if let Some(getter) = inner_getter {
                // Simple type inside Option - use getter method
                let type_name = inner_ty.map_or("value", rust_type_to_name);
                from_json_fields.push(quote! {
                    #field_name: {
                        let v = __value.get(#json_key);
                        if v.is_null() {
                            None
                        } else {
                            Some(v #getter .ok_or_else(|| mik_sdk::typed::ParseError::type_mismatch(#json_key, #type_name))?)
                        }
                    }
                });
            } else if let Some(inner) = inner_ty {
                // Complex type inside Option - use FromJson trait
                from_json_fields.push(quote! {
                    #field_name: {
                        let v = __value.get(#json_key);
                        if v.is_null() {
                            None
                        } else {
                            Some(<#inner as mik_sdk::typed::FromJson>::from_json(&v)?)
                        }
                    }
                });
            } else {
                // Could not extract inner type from Option - emit compile error
                let field_name_str = field_name.to_string();
                return syn::Error::new_spanned(
                    field_ty,
                    format!(
                        "Can't figure out what type is inside Option for '{field_name_str}'.\n\
                         \n\
                         \u{2705} Use a concrete type:\n\
                         pub {field_name_str}: Option<String>,     // text\n\
                         pub {field_name_str}: Option<i32>,        // number\n\
                         pub {field_name_str}: Option<bool>,       // true/false\n\
                         pub {field_name_str}: Option<MyStruct>,   // your own type\n\
                         \n\
                         \u{274C} These won't work:\n\
                         pub {field_name_str}: Option<impl Trait>, // too abstract\n\
                         pub {field_name_str}: Option<_>,          // can't infer type"
                    ),
                )
                .to_compile_error()
                .into();
            }
        } else {
            let getter = rust_type_to_json_getter(field_ty);
            if let Some(getter) = getter {
                // Simple type - use getter method
                from_json_fields.push(quote! {
                    #field_name: __value.get(#json_key) #getter
                        .ok_or_else(|| mik_sdk::typed::ParseError::missing(#json_key))?
                });
            } else {
                // Complex type (Vec, custom struct, etc.) - use FromJson trait
                from_json_fields.push(quote! {
                    #field_name: <#field_ty as mik_sdk::typed::FromJson>::from_json(&__value.get(#json_key))?
                });
            }
            required_fields.push(json_key.clone());
        }

        // Generate OpenAPI property
        let mut base_schema = type_to_openapi(field_ty);

        // Add constraints to schema
        // Note: Check array BEFORE string because array schemas contain "string" in items
        let mut extra_props = Vec::new();
        if let Some(min) = attrs.min {
            if base_schema.contains("array") {
                extra_props.push(format!(r#""minItems":{min}"#));
            } else if base_schema.contains("string") {
                extra_props.push(format!(r#""minLength":{min}"#));
            } else if base_schema.contains("integer") || base_schema.contains("number") {
                extra_props.push(format!(r#""minimum":{min}"#));
            }
        }
        if let Some(max) = attrs.max {
            if base_schema.contains("array") {
                extra_props.push(format!(r#""maxItems":{max}"#));
            } else if base_schema.contains("string") {
                extra_props.push(format!(r#""maxLength":{max}"#));
            } else if base_schema.contains("integer") || base_schema.contains("number") {
                extra_props.push(format!(r#""maximum":{max}"#));
            }
        }
        if let Some(ref fmt) = attrs.format {
            extra_props.push(format!(r#""format":"{}""#, escape_json_string(fmt)));
        }
        if let Some(ref pattern) = attrs.pattern {
            extra_props.push(format!(r#""pattern":"{}""#, escape_json_string(pattern)));
        }
        if let Some(ref docs) = attrs.docs {
            extra_props.push(format!(r#""description":"{}""#, escape_json_string(docs)));
        }

        if !extra_props.is_empty() {
            // Merge extra props into base schema (safe extraction)
            let base_inner = base_schema
                .strip_prefix('{')
                .and_then(|s| s.strip_suffix('}'))
                .unwrap_or(&base_schema);
            base_schema = format!("{{{},{}}}", base_inner, extra_props.join(","));
        }

        openapi_properties.push(format!(
            r#""{}":{}"#,
            escape_json_string(&json_key),
            base_schema
        ));

        // Generate validation checks
        generate_validation_checks(
            &attrs,
            field_name,
            is_optional,
            &base_schema,
            &mut validation_checks,
        );
    }

    // Build OpenAPI schema
    let required_json = if required_fields.is_empty() {
        String::new()
    } else {
        format!(
            r#","required":[{}]"#,
            required_fields
                .iter()
                .map(|f| format!(r#""{}""#, escape_json_string(f)))
                .collect::<Vec<_>>()
                .join(",")
        )
    };

    let openapi_schema = format!(
        r#"{{"type":"object","properties":{{{}}}{}}}  "#,
        openapi_properties.join(","),
        required_json
    );

    let tokens = quote! {
        impl mik_sdk::typed::FromJson for #name {
            fn from_json(__value: &mik_sdk::json::JsonValue) -> Result<Self, mik_sdk::typed::ParseError> {
                Ok(Self {
                    #(#from_json_fields),*
                })
            }
        }

        impl mik_sdk::typed::Validate for #name {
            fn validate(&self) -> Result<(), mik_sdk::typed::ValidationError> {
                #(#validation_checks)*
                Ok(())
            }
        }

        impl mik_sdk::typed::OpenApiSchema for #name {
            fn openapi_schema() -> &'static str {
                #openapi_schema
            }

            fn schema_name() -> &'static str {
                #name_str
            }
        }
    };

    TokenStream::from(tokens)
}

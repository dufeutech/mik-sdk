//! OpenAPI specification generation for routes.
//!
//! Generates OpenAPI 3.0 specifications with full type schemas.
//!
//! Strategy: The paths/parameters are computed at macro expansion time,
//! but schemas are resolved at runtime by calling the `OpenApiSchema` trait
//! methods on each type. This allows full schema information to be included.

use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;

use crate::schema::codegen::extract_param_names;
use crate::schema::types::{InputSource, RouteDef};

// =============================================================================
// OPENAPI GENERATION
// =============================================================================

/// Generate a static OpenAPI method entry for a route.
fn generate_method_entry(route: &RouteDef) -> String {
    let method_name = route.method.as_str();
    let path = route
        .patterns
        .first()
        .map_or("/", std::string::String::as_str);

    let mut parts: Vec<String> = Vec::new();

    // Request body reference
    if let Some(body_input) = route
        .inputs
        .iter()
        .find(|i| matches!(i.source, InputSource::Body))
    {
        let type_name = body_input.type_name.to_string();
        parts.push(format!(
            "\"requestBody\":{{\"required\":true,\"content\":{{\"application/json\":{{\"schema\":{{\"$ref\":\"#/components/schemas/{type_name}\"}}}}}}}}"
        ));
    }

    // Parameters (path + query)
    let mut params: Vec<String> = Vec::new();

    // Path parameters from URL pattern
    for name in extract_param_names(path) {
        params.push(format!(
            "{{\"name\":\"{name}\",\"in\":\"path\",\"required\":true,\"schema\":{{\"type\":\"string\"}}}}"
        ));
    }

    // Query parameters - reference the type for schema lookup
    if let Some(query_input) = route
        .inputs
        .iter()
        .find(|i| matches!(i.source, InputSource::Query))
    {
        let type_name = query_input.type_name.to_string();
        // Add a reference note - full query params are in the schema
        params.push(format!(
            "{{\"name\":\"(see {type_name})\",\"in\":\"query\",\"required\":false,\"schema\":{{\"$ref\":\"#/components/schemas/{type_name}\"}}}}"
        ));
    }

    if !params.is_empty() {
        parts.push(format!("\"parameters\":[{}]", params.join(",")));
    }

    // Response
    if let Some(ref output_type) = route.output_type {
        parts.push(format!(
            "\"responses\":{{\"200\":{{\"description\":\"Success\",\"content\":{{\"application/json\":{{\"schema\":{{\"$ref\":\"#/components/schemas/{output_type}\"}}}}}}}}}}"
        ));
    } else {
        parts.push("\"responses\":{\"200\":{\"description\":\"Success\"}}".to_string());
    }

    format!("\"{}\":{{{}}}", method_name, parts.join(","))
}

/// Collect unique type names from routes for schema generation.
fn collect_type_names(routes: &[RouteDef]) -> Vec<Ident> {
    use std::collections::HashSet;

    let mut type_names: Vec<Ident> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for route in routes {
        for input in &route.inputs {
            let name = input.type_name.to_string();
            if seen.insert(name) {
                type_names.push(input.type_name.clone());
            }
        }
        if let Some(ref output) = route.output_type {
            let name = output.to_string();
            if seen.insert(name) {
                type_names.push(output.clone());
            }
        }
    }

    type_names
}

/// Generate the static paths JSON portion of the OpenAPI spec.
fn generate_paths_json(routes: &[RouteDef]) -> String {
    use std::collections::HashMap;

    // Group routes by path
    let mut paths: HashMap<String, Vec<&RouteDef>> = HashMap::new();
    for route in routes {
        let path = route
            .patterns
            .first()
            .map_or("/", std::string::String::as_str);
        paths.entry(path.to_string()).or_default().push(route);
    }

    // Build paths JSON
    let path_entries: Vec<String> = paths
        .iter()
        .map(|(path, methods)| {
            let method_entries: Vec<String> =
                methods.iter().map(|r| generate_method_entry(r)).collect();
            format!("\"{}\":{{{}}}", path, method_entries.join(","))
        })
        .collect();

    path_entries.join(",")
}

/// Generate the complete OpenAPI JSON with full type schemas.
///
/// The paths/parameters are computed at macro expansion time as static strings.
/// The schemas are resolved at runtime by calling `OpenApiSchema::openapi_schema()`
/// on each type, allowing full schema information to be included.
pub fn generate_openapi_json(routes: &[RouteDef]) -> TokenStream2 {
    let paths_json = generate_paths_json(routes);
    let type_names = collect_type_names(routes);

    // Generate code to build schema entries by calling trait methods
    // Use super:: prefix because this runs inside __mik_schema module
    let schema_builders: Vec<TokenStream2> = type_names
        .iter()
        .map(|type_name| {
            let type_name_str = type_name.to_string();
            quote! {
                schema_parts.push(::std::format!(
                    "\"{}\":{}",
                    #type_name_str,
                    <super::#type_name as mik_sdk::typed::OpenApiSchema>::openapi_schema()
                ));
            }
        })
        .collect();

    // Return code that builds the OpenAPI JSON at runtime
    quote! {
        {
            let mut schema_parts: ::std::vec::Vec<::std::string::String> = ::std::vec::Vec::new();
            #(#schema_builders)*
            let schemas_json = schema_parts.join(",");
            ::std::format!(
                r#"{{"openapi":"3.0.0","info":{{"title":"API","version":"1.0.0"}},"paths":{{{}}},"components":{{"schemas":{{{}}}}}}}"#,
                #paths_json,
                schemas_json
            )
        }
    }
}

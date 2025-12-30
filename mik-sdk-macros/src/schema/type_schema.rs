//! Types and parsing for the routes macro.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Ident, LitStr, Result, Token,
    parse::{Parse, ParseStream},
};

// =============================================================================
// TYPES
// =============================================================================

#[derive(Clone)]
pub(crate) enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

impl HttpMethod {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "get",
            HttpMethod::Post => "post",
            HttpMethod::Put => "put",
            HttpMethod::Patch => "patch",
            HttpMethod::Delete => "delete",
            HttpMethod::Head => "head",
            HttpMethod::Options => "options",
        }
    }

    pub(crate) fn to_method_check(&self) -> TokenStream2 {
        match self {
            HttpMethod::Get => quote! { mik_sdk::Method::Get },
            HttpMethod::Post => quote! { mik_sdk::Method::Post },
            HttpMethod::Put => quote! { mik_sdk::Method::Put },
            HttpMethod::Patch => quote! { mik_sdk::Method::Patch },
            HttpMethod::Delete => quote! { mik_sdk::Method::Delete },
            HttpMethod::Head => quote! { mik_sdk::Method::Head },
            HttpMethod::Options => quote! { mik_sdk::Method::Options },
        }
    }
}

/// Input source for typed parameters
#[derive(Clone)]
pub(crate) enum InputSource {
    Path,  // from URL path params
    Body,  // from JSON body
    Query, // from query string
}

/// A typed input parameter for a handler
#[derive(Clone)]
pub(crate) struct TypedInput {
    pub(crate) source: InputSource,
    pub(crate) type_name: Ident,
}

/// A route definition
pub(crate) struct RouteDef {
    pub(crate) method: HttpMethod,
    pub(crate) patterns: Vec<String>,
    pub(crate) handler: Ident,
    pub(crate) inputs: Vec<TypedInput>,
    pub(crate) output_type: Option<Ident>,
}

/// All routes in the macro
pub(crate) struct RoutesDef {
    pub(crate) routes: Vec<RouteDef>,
}

// =============================================================================
// PARSING
// =============================================================================

impl Parse for RoutesDef {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut routes = Vec::new();

        while !input.is_empty() {
            let route = parse_route(input)?;
            routes.push(route);

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(RoutesDef { routes })
    }
}

fn parse_route(input: ParseStream<'_>) -> Result<RouteDef> {
    // Parse method: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS
    let method_ident: Ident = input.parse().map_err(|e| {
        syn::Error::new(
            e.span(),
            format!(
                "Expected HTTP method at start of route definition.\n\
                 \n\
                 Valid methods: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS\n\
                 \n\
                 Example:\n\
                 routes! {{\n\
                     GET \"/users\" => list_users,\n\
                     POST \"/users\" => create_user(body: CreateUser) -> User,\n\
                 }}\n\
                 \n\
                 Original error: {e}"
            ),
        )
    })?;

    let method_str = method_ident.to_string().to_uppercase();
    let method = match method_str.as_str() {
        "GET" => HttpMethod::Get,
        "POST" => HttpMethod::Post,
        "PUT" => HttpMethod::Put,
        "PATCH" => HttpMethod::Patch,
        "DELETE" => HttpMethod::Delete,
        "HEAD" => HttpMethod::Head,
        "OPTIONS" => HttpMethod::Options,
        _ => {
            return Err(syn::Error::new_spanned(
                &method_ident,
                format!(
                    "Invalid HTTP method '{}'.\n\
                     \n\
                     Valid methods: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS\n\
                     \n\
                     Example: GET \"/users\" => list_users",
                    method_ident
                ),
            ));
        },
    };

    // Parse pattern(s): "/path" or "/path" | "/other"
    let mut patterns = Vec::new();
    let first_pattern: LitStr = input.parse().map_err(|e| {
        syn::Error::new(
            e.span(),
            format!(
                "Expected route path (string literal) after HTTP method '{}'.\n\
                 \n\
                 Correct syntax: {} \"/path\" => handler\n\
                 \n\
                 Common mistakes:\n\
                 - Path must be a string literal: {} \"/users\" ✓ not {} /users ✗\n\
                 - Path should start with /: {} \"/users\" ✓ not {} \"users\" ✗\n\
                 \n\
                 Original error: {e}",
                method_str, method_str, method_str, method_str, method_str, method_str
            ),
        )
    })?;
    patterns.push(first_pattern.value());

    while input.peek(Token![|]) {
        input.parse::<Token![|]>()?;
        let alt_pattern: LitStr = input.parse().map_err(|e| {
            syn::Error::new(
                e.span(),
                format!(
                    "Expected alternative route path after '|'.\n\
                     \n\
                     Correct syntax: {} \"/path\" | \"/alt-path\" => handler\n\
                     \n\
                     Original error: {e}",
                    method_str
                ),
            )
        })?;
        patterns.push(alt_pattern.value());
    }

    // Parse =>
    input.parse::<Token![=>]>().map_err(|e| {
        syn::Error::new(
            e.span(),
            format!(
                "Expected '=>' after route path.\n\
                 \n\
                 Correct syntax: {} \"{}\" => handler_name\n\
                 \n\
                 Common mistakes:\n\
                 - Use => not -> for route arrow: {} \"{}\" => handler ✓\n\
                 - Use => not : for route arrow: {} \"{}\" => handler ✓\n\
                 \n\
                 Original error: {e}",
                method_str,
                patterns.first().unwrap_or(&String::from("/path")),
                method_str,
                patterns.first().unwrap_or(&String::from("/path")),
                method_str,
                patterns.first().unwrap_or(&String::from("/path")),
            ),
        )
    })?;

    // Parse handler name
    let handler: Ident = input.parse().map_err(|e| {
        syn::Error::new(
            e.span(),
            format!(
                "Expected handler function name after '=>'.\n\
                 \n\
                 Correct syntax: {} \"{}\" => handler_name\n\
                 \n\
                 The handler must be an identifier (function name), not a string.\n\
                 \n\
                 Example:\n\
                 fn list_users(_req: &Request) -> Response {{ ... }}\n\
                 \n\
                 routes! {{\n\
                     {} \"{}\" => list_users,\n\
                 }}\n\
                 \n\
                 Original error: {e}",
                method_str,
                patterns.first().unwrap_or(&String::from("/path")),
                method_str,
                patterns.first().unwrap_or(&String::from("/path")),
            ),
        )
    })?;

    // Parse optional typed inputs: (path: Id, body: CreateUser, query: ListQuery)
    let inputs = if input.peek(syn::token::Paren) {
        let content;
        syn::parenthesized!(content in input);
        parse_typed_inputs(&content, &method_str, &patterns, &handler)?
    } else {
        Vec::new()
    };

    // Parse optional output type: -> User or -> Vec<User>
    let output_type = if input.peek(Token![->]) {
        input.parse::<Token![->]>()?;
        // For now just parse as Ident, handle Vec<T> later if needed
        let type_ident: Ident = input.parse().map_err(|e| {
            syn::Error::new(
                e.span(),
                format!(
                    "Expected response type name after '->'.\n\
                     \n\
                     Correct syntax: {} \"{}\" => {}(...) -> ResponseType\n\
                     \n\
                     The response type should be an identifier like User, Vec<User>, etc.\n\
                     \n\
                     Example:\n\
                     {} \"{}\" => {} -> User,\n\
                     \n\
                     Original error: {e}",
                    method_str,
                    patterns.first().unwrap_or(&String::from("/path")),
                    handler,
                    method_str,
                    patterns.first().unwrap_or(&String::from("/path")),
                    handler,
                ),
            )
        })?;
        Some(type_ident)
    } else {
        None
    };

    Ok(RouteDef {
        method,
        patterns,
        handler,
        inputs,
        output_type,
    })
}

fn parse_typed_inputs(
    input: ParseStream<'_>,
    method_str: &str,
    patterns: &[String],
    handler: &Ident,
) -> Result<Vec<TypedInput>> {
    let mut inputs = Vec::new();
    let path = patterns.first().map(|s| s.as_str()).unwrap_or("/path");

    while !input.is_empty() {
        // Parse source: path, body, or query
        let source_ident: Ident = input.parse().map_err(|e| {
            syn::Error::new(
                e.span(),
                format!(
                    "Expected input source in handler parameters.\n\
                     \n\
                     Valid sources:\n\
                     - path: Type   - URL path parameters (e.g., /users/{{id}})\n\
                     - body: Type   - JSON request body\n\
                     - query: Type  - Query string parameters\n\
                     \n\
                     Example:\n\
                     {} \"{}\" => {}(path: UserId, body: CreateUser, query: Pagination) -> User\n\
                     \n\
                     Original error: {e}",
                    method_str, path, handler
                ),
            )
        })?;

        let source = match source_ident.to_string().as_str() {
            "path" => InputSource::Path,
            "body" => InputSource::Body,
            "query" => InputSource::Query,
            other => {
                return Err(syn::Error::new_spanned(
                    &source_ident,
                    format!(
                        "Invalid input source '{}'.\n\
                         \n\
                         Valid sources:\n\
                         - path  - URL path parameters (e.g., /users/{{id}})\n\
                         - body  - JSON request body\n\
                         - query - Query string parameters\n\
                         \n\
                         Example:\n\
                         {} \"{}\" => {}(path: Id, body: CreateUser) -> User",
                        other, method_str, path, handler
                    ),
                ));
            },
        };

        // Parse colon
        input.parse::<Token![:]>().map_err(|e| {
            syn::Error::new(
                e.span(),
                format!(
                    "Expected ':' after input source '{}'.\n\
                     \n\
                     Correct syntax: {}: TypeName\n\
                     \n\
                     Example:\n\
                     {} \"{}\" => {}({}: UserId)\n\
                     \n\
                     Original error: {e}",
                    source_ident, source_ident, method_str, path, handler, source_ident
                ),
            )
        })?;

        // Parse type name
        let type_name: Ident = input.parse().map_err(|e| {
            syn::Error::new(
                e.span(),
                format!(
                    "Expected type name after '{}: '.\n\
                     \n\
                     The type must be a struct that derives the appropriate trait:\n\
                     - path: Type   - Type must derive Path\n\
                     - body: Type   - Type must derive Type (for JSON parsing)\n\
                     - query: Type  - Type must derive Query\n\
                     \n\
                     Example:\n\
                     #[derive(Path)]\n\
                     struct UserId {{ id: String }}\n\
                     \n\
                     {} \"{}\" => {}({}: UserId)\n\
                     \n\
                     Original error: {e}",
                    source_ident, method_str, path, handler, source_ident
                ),
            )
        })?;

        inputs.push(TypedInput { source, type_name });

        // Optional comma
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
    }

    Ok(inputs)
}

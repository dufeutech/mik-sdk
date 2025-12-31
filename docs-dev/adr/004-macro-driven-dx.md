# ADR-004: Macro-Driven Developer Experience

## Status

Accepted

## Context

HTTP handler code has significant boilerplate:
- Route matching and parameter extraction
- JSON serialization/deserialization
- Error response formatting
- Input validation and type conversion

Without macros, a simple handler looks like:

```rust
fn handle(req: Request) -> Response {
    if req.method() != Method::Get || !req.path().starts_with("/users/") {
        return Response::new(404, vec![], None);
    }
    let id = req.path().strip_prefix("/users/").unwrap();
    let body = format!(r#"{{"id":"{}"}}"#, id);
    Response::new(200, vec![("content-type".into(), "application/json".into())], Some(body.into()))
}
```

This is verbose, error-prone, and obscures intent.

## Decision

Provide **procedural macros** that reduce boilerplate while remaining transparent:

### 1. Route definition (`routes!`)

```rust
routes! {
    GET "/" => home,
    GET "/users/{id}" => get_user(path: UserId),
    POST "/users" => create_user(body: CreateUser) -> User,
}
```

Generates: Match tree, parameter extraction, type conversion, error handling.

### 2. JSON responses (`ok!`, `error!`)

```rust
ok!({ "name": user.name, "age": user.age })

error! {
    status: 404,
    title: "Not Found",
    detail: format!("User {} not found", id)
}
```

Generates: JSON serialization, content-type header, status code.

### 3. Type derivation (`#[derive(Type)]`, `#[derive(Path)]`, `#[derive(Query)]`)

```rust
#[derive(Type)]
struct User {
    name: String,
    age: u32,
}

#[derive(Path)]
struct UserId {
    id: String,
}

#[derive(Query)]
struct Pagination {
    #[field(default = 1)]
    page: u32,
    #[field(default = 10)]
    limit: u32,
}
```

Generates: JSON parsing, URL parameter extraction, validation, error messages.

### Design principles

1. **Transparent** - Macro expansion is inspectable (`cargo expand`)
2. **Type-safe** - Compile-time route/type checking
3. **Minimal magic** - Clear 1:1 mapping from macro to generated code
4. **Escape hatches** - Can always drop to manual code

## Consequences

### Positive

- **Concise handlers** - Focus on business logic, not plumbing
- **Compile-time safety** - Type mismatches caught at build time
- **Consistent patterns** - Error responses always RFC 7807 format
- **Self-documenting** - Route definitions serve as API documentation
- **Type inference** - `ok!({ "name": name })` infers String → JSON string

### Negative

- **Macro complexity** - proc-macro crate is non-trivial (~3000 LOC)
- **Compile errors** - Macro errors can be cryptic
- **Learning curve** - Developers must learn macro syntax
- **IDE support** - Some IDEs struggle with macro expansion

### Neutral

- Macros hide generated code (mitigated by `cargo expand`)
- Testing macros requires snapshot testing (we use `insta`)

## Alternatives Considered

### No macros - manual code only

Rejected: Too much boilerplate. Every handler repeats the same patterns. Inconsistency between developers.

### Attribute macros on functions

```rust
#[get("/users/{id}")]
fn get_user(id: String) -> User { ... }
```

Rejected: Requires specific function signatures. Less flexible than `routes!` which separates routing from handlers.

### Builder pattern

```rust
Router::new()
    .get("/users/{id}", get_user)
    .post("/users", create_user)
    .build()
```

Rejected: Runtime cost. Can't do compile-time route validation. Still needs separate type extraction.

### Code generation (build.rs)

Rejected: Separate file for routes, harder to maintain, no IDE support for generated code.

## References

- [Rust proc-macro documentation](https://doc.rust-lang.org/reference/procedural-macros.html)
- [syn crate](https://docs.rs/syn) for parsing
- [quote crate](https://docs.rs/quote) for code generation
- Testing: `trybuild` for compile-fail tests, `insta` for snapshot testing

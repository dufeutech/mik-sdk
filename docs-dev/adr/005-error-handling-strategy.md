# ADR-005: Error Handling Strategy

## Status

Accepted

## Context

HTTP APIs need consistent, informative error handling:
- Input validation failures (missing fields, wrong types)
- Business logic errors (not found, conflict, forbidden)
- System errors (external service failures)

Poor error handling leads to:
- Unhelpful "500 Internal Server Error" responses
- Inconsistent error formats across endpoints
- Security leaks (stack traces in production)
- Difficult debugging

## Decision

### 1. Custom error types with context

```rust
pub enum ParseError {
    MissingField { field: String },
    InvalidFormat { field: String, value: String },
    TypeMismatch { field: String, expected: String },
    Custom { field: String, message: String },
}

pub enum ValidationError {
    Min { field: String, min: String },
    Max { field: String, max: String },
    Pattern { field: String, pattern: String },
    Format { field: String, expected: String },
    Custom { field: String, constraint: String, message: String },
}
```

Both support **nested field context**:

```rust
let err = ParseError::missing("city").with_path("address");
// err.field() => "address.city"
```

### 2. RFC 7807 Problem Details for HTTP errors

```rust
error! {
    status: 404,
    title: "Resource Not Found",
    detail: format!("User {} does not exist", id),
    instance: req.path()
}
```

Generates:

```json
{
    "type": "about:blank",
    "status": 404,
    "title": "Resource Not Found",
    "detail": "User abc123 does not exist",
    "instance": "/users/abc123"
}
```

### 3. DX macros for common patterns

```rust
// Early return on condition
guard!(user.is_admin(), 403, "Admin access required");

// Unwrap Option/Result or return error
let user = ensure!(find_user(id), 404, "User not found");

// Shorthand error responses
not_found!("User not found")
bad_request!("Invalid email format")
forbidden!("Insufficient permissions")
conflict!("Username already taken")
```

### 4. Non-exhaustive enums

```rust
#[non_exhaustive]
pub enum ParseError { ... }
```

Allows adding variants without breaking downstream code.

## Consequences

### Positive

- **Consistent format** - All errors follow RFC 7807
- **Rich context** - Nested field paths, specific error types
- **Type-safe** - Compiler ensures error handling
- **Debuggable** - Clear error messages with field context
- **Secure** - No stack traces or internal details leak

### Negative

- **More code** - Custom error types vs simple strings
- **Learning curve** - Developers must understand error type hierarchy
- **Overhead** - String allocations for error messages (negligible)

### Neutral

- RFC 7807 format may be unfamiliar to some developers
- Error types are SDK-specific (not standard Rust errors)

## Alternatives Considered

### Use `anyhow` or `thiserror`

Rejected for `anyhow`: Erases error types, can't match on specific errors.

`thiserror` is used internally but not exposed. Our error types are simpler and HTTP-focused.

### String-only errors

```rust
Err("User not found".to_string())
```

Rejected: No structure, can't programmatically distinguish error types, inconsistent formatting.

### HTTP status codes only

```rust
Response::new(404, ...)
```

Rejected: Status code alone doesn't explain *why*. "404" could mean route not found, resource not found, or soft-deleted resource.

### Nested Result types

```rust
Result<Result<User, BusinessError>, SystemError>
```

Rejected: Awkward to handle. Single error type with variants is cleaner.

## References

- [RFC 7807: Problem Details for HTTP APIs](https://tools.ietf.org/html/rfc7807)
- [Rust Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [thiserror crate](https://docs.rs/thiserror) (used internally)

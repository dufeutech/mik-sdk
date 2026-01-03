<p align="center">
  <img src="docs/src/assets/logo.png" alt="mik" width="180" />
</p>

<h1 align="center">mik-sdk</h1>

<p align="center">
  <strong>Portable WASI HTTP SDK using Component Composition</strong><br>
  <span>Write handlers once, run on wasmtime or any HTTP WASI-compliant runtime.</span>
</p>

<p align="center">
  <a href="https://github.com/dufeutech/mik-sdk/actions/workflows/ci.yml"><img src="https://github.com/dufeutech/mik-sdk/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://codecov.io/gh/dufeutech/mik-sdk"><img src="https://codecov.io/gh/dufeutech/mik-sdk/graph/badge.svg" alt="codecov"></a>
  <a href="https://crates.io/crates/mik-sdk"><img src="https://img.shields.io/crates/v/mik-sdk.svg" alt="Crates.io"></a>
  <a href="https://docs.rs/mik-sdk"><img src="https://docs.rs/mik-sdk/badge.svg" alt="docs.rs"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

<p align="center">
  <a href="https://dufeutech.github.io/mik-sdk/">Documentation</a> ·
  <a href="https://github.com/dufeutech/mik-sdk/tree/main/examples">Examples</a>
</p>

---

## Why mik-sdk?

- **Write once, run anywhere** — Your handlers work on wasmtime and any WASI P2 runtime without code changes.
- **Type-safe by default** — Path params, query strings, and JSON bodies are parsed and validated at compile time.
- **Tiny footprint** — Composed components are ~250KB. No bloat, no unnecessary dependencies.

## Quick Example

```rust
use mik_sdk::prelude::*;

#[derive(Path)]
struct HelloPath { name: String }

#[derive(Type)]
struct HelloResponse { greeting: String }

routes! {
    GET "/" => home,
    GET "/hello/{name}" => hello(path: HelloPath) -> HelloResponse,
}

fn home(_req: &Request) -> Response {
    ok!({ "message": "Welcome!" })
}

fn hello(path: HelloPath, _req: &Request) -> Response {
    ok!({ "greeting": format!("Hello, {}!", path.name) })
}
```

## Installation

```toml
[dependencies]
mik-sdk = "0.1"
```

Requires Rust 1.85+ with `wasm32-wasip2` target and [cargo-component](https://github.com/bytecodealliance/cargo-component).

## Build & Run

```bash
# Get the bridge and WIT interface
curl -LO https://github.com/dufeutech/mik-sdk/releases/latest/download/mik-bridge.wasm
mkdir -p wit/deps/core
curl -L https://github.com/dufeutech/mik-sdk/releases/latest/download/core.wit \
  -o wit/deps/core/core.wit

# Build your handler
cargo component build --release

# Compose with the bridge component
wac plug mik-bridge.wasm --plug target/wasm32-wasip2/release/my_handler.wasm -o service.wasm

# Run on wasmtime
wasmtime serve -S cli=y service.wasm
```

**OCI Registry** (alternative):

```bash
oras pull ghcr.io/dufeutech/mik-sdk-bridge:latest    # mik_bridge.wasm
oras pull ghcr.io/dufeutech/mik-sdk-wit:latest       # core.wit
```

## Features

**Routing & Inputs**
- `routes!` macro with typed path, query, and body extraction
- `#[derive(Path)]`, `#[derive(Query)]`, `#[derive(Type)]` for input types
- Automatic 400 errors for invalid inputs

**Responses**
- `ok!`, `created!`, `no_content!`, `redirect!` for common responses
- `error!` for RFC 7807 Problem Details
- `guard!`, `ensure!` for early returns

**Built-in Utilities**
- Pure Rust JSON (no external calls)
- `time::now()`, `time::now_iso()` via WASI clocks
- `random::uuid()`, `random::hex()` via WASI random
- Structured logging with `log!`

**Included by Default**
- `http-client` — Outbound HTTP with `fetch!` macro and SSRF protection
- `sql` — Query builder with Mongo-style filters and cursor pagination

Use `default-features = false` for a minimal build.

## Examples

| Example                                 | Description                                |
| --------------------------------------- | ------------------------------------------ |
| [hello-world](examples/hello-world)     | Minimal handler with path and query params |
| [crud-api](examples/crud-api)           | REST API with SQL builder and pagination   |
| [auth-api](examples/auth-api)           | Authentication patterns                    |
| [external-api](examples/external-api)   | Outbound HTTP with `fetch!`                |
| [resilient-api](examples/resilient-api) | Retry, fallback, rate limiting             |

## Architecture

mik-sdk uses a two-component architecture for maximum portability:

```
┌─────────────────────────────────────────────────────────┐
│  Your Handler                                           │
│  exports mik:core/handler                               │
└─────────────────────────────────────────────────────────┘
                          ↓ compose
┌─────────────────────────────────────────────────────────┐
│  mik-bridge                                             │
│  WASI HTTP ↔ mik:core/handler translation               │
└─────────────────────────────────────────────────────────┘
                          ↓ runs on
┌─────────────────────────────────────────────────────────┐
│  wasmtime · any WASI HTTP runtime                       │
└─────────────────────────────────────────────────────────┘
```

## Crates

| Crate                                       | Description                                     |
| ------------------------------------------- | ----------------------------------------------- |
| [mik-sdk](https://crates.io/crates/mik-sdk) | Core SDK — routing, JSON, time, random, logging |
| [mik-sql](https://crates.io/crates/mik-sql) | SQL query builder with Mongo-style filters      |

## Resources

- [Documentation](https://dufeutech.github.io/mik-sdk/) — Guides, reference, and best practices
- [API Reference (docs.rs)](https://docs.rs/mik-sdk) — Rust API documentation
- [Examples](https://github.com/dufeutech/mik-sdk/tree/main/examples) — Complete working examples
- [Contributing](CONTRIBUTING.md) — How to contribute

## License

MIT

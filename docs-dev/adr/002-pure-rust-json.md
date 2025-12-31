# ADR-002: Pure Rust JSON (No WASI Dependency)

## Status

Accepted

## Context

JSON handling is fundamental to HTTP APIs. Options for WASI components:

1. **Pure Rust library** - Compiled into the component
2. **WASI capability** - Import from host (doesn't exist yet)
3. **External service** - Call out to parse JSON (absurd overhead)

WASI P2 has no JSON interface. Even if one existed, the overhead of crossing the component boundary for every parse/serialize operation would be prohibitive.

## Decision

Use **miniserde** as a pure Rust JSON library with a custom lazy parsing layer on top.

### Implementation

```rust
// Pure Rust - no WASI imports for JSON
use mik_sdk::json;

let value = json::obj()
    .set("name", json::str("Alice"))
    .set("age", json::int(30));

// Lazy parsing - doesn't build tree until accessed
let parsed = json::try_parse(bytes)?;
let name = parsed.path_str(&["user", "name"]); // Scans directly
```

### Why miniserde over serde

| Aspect | serde | miniserde |
|--------|-------|-----------|
| Binary size | 50-100KB | ~15KB |
| Compile time | Slow (proc macros) | Fast |
| Features | Everything | JSON only |
| Streaming | No | No |
| Zero-copy | Limited | No |

We don't need serde's format-agnostic abstraction. We only do JSON.

### Custom lazy parsing layer

miniserde builds a full tree on parse. For large payloads where you only need a few fields:

- **Tree traversal**: Parse all → allocate tree → traverse → extract
- **Lazy scanning**: Scan bytes → find path → extract (no tree)

Lazy scanning is **~33x faster** for partial field extraction.

## Consequences

### Positive

- **Small binary** - 15KB vs 100KB for serde
- **Fast partial reads** - Lazy scanning avoids full parse
- **No WASI dependency** - Works on any WASI runtime, present and future
- **Predictable** - No host-specific JSON quirks

### Negative

- **No serde ecosystem** - Can't use `#[serde(rename)]` etc.
- **Custom derive macros** - We maintain `#[derive(Type)]` instead
- **No streaming** - Full payload must be in memory (acceptable for typical HTTP bodies)

### Neutral

- Learning curve for developers used to serde (mitigated by similar API design)

## Alternatives Considered

### Use serde + serde_json

Rejected: 50-100KB binary size overhead. The flexibility of serde (multiple formats) is unused - we only need JSON.

### Use simd-json

Rejected: Requires SIMD instructions not available in WASM. Also larger binary.

### Wait for WASI JSON interface

Rejected: No such interface exists or is planned. Even if it did, crossing the component boundary for JSON operations would add latency.

### Hand-roll JSON parser

Rejected: Reinventing the wheel. miniserde is battle-tested and minimal. Our lazy layer on top adds value without reimplementing core parsing.

## References

- [miniserde](https://github.com/dtolnay/miniserde) - David Tolnay's minimal JSON library
- Benchmarks: `cargo bench -p mik-sdk -- json`

# ADR-001: Binary Size Optimization Strategy

## Status

Accepted

## Context

WASI components run in resource-constrained environments. Binary size directly impacts:
- Cold start latency (larger = slower to load)
- Memory consumption during instantiation
- Network transfer time for deployment
- Edge/serverless costs (often billed by memory)

The target is **<300KB** for a composed service (bridge + handler), with an ideal of ~200-250KB.

### Current Measurements (December 2025)

| Component | Size |
|-----------|------|
| mik-bridge | 85 KB |
| hello-world handler | 155 KB |
| **Composed service** | **246 KB** |

## Decision

Optimize for binary size through:

1. **Release profile settings**
   ```toml
   [profile.release]
   lto = true          # Link-time optimization - eliminates dead code across crates
   opt-level = "z"     # Optimize for size over speed
   codegen-units = 1   # Better optimization (slower compile)
   strip = true        # Remove symbols
   panic = "abort"     # No unwinding machinery
   ```

2. **Minimal dependencies**
   - Use `miniserde` (pure Rust, ~15KB contribution) instead of `serde` (~50-100KB)
   - No heavy frameworks - each dependency audited for size impact
   - Feature flags for optional functionality (`http-client` adds ~78KB)

3. **Two-component architecture**
   - Bridge component is shared across all handlers (85KB one-time cost)
   - Handler components contain only business logic
   - Composition deduplicates shared code

4. **No std where unnecessary**
   - WASI bindings are `no_std` compatible
   - Core SDK carefully uses std features

## Consequences

### Positive

- 246KB composed binary competitive with hand-written WASI components
- Fast cold starts (<50ms typical)
- Lower memory footprint than alternatives
- Predictable size growth (handler logic, not framework overhead)

### Negative

- `opt-level = "z"` may sacrifice ~5-10% runtime performance vs `"3"`
- `codegen-units = 1` significantly slower compile times in release
- `lto = true` increases link time
- Limited to dependencies that are size-conscious

### Neutral

- Strip removes debug symbols - stack traces less readable in production (acceptable for WASM)

## Alternatives Considered

### Use `opt-level = "3"` for speed

Rejected: Testing showed <5% performance difference but 15-20% size increase. For HTTP handlers, I/O dominates - raw CPU performance rarely matters.

### Use `serde` for JSON

Rejected: serde adds 50-100KB to binary size. miniserde is 15KB and sufficient for our JSON needs. The derive macro compatibility loss is offset by our own `#[derive(Type)]` macro.

### Single monolithic component

Rejected: Every handler would include the full WASI HTTP translation layer. Two-component architecture means the bridge is shared, and handler components stay small.

## References

- [Rust WASM Size Optimization](https://rustwasm.github.io/docs/book/reference/code-size.html)
- [min-sized-rust](https://github.com/johnthagen/min-sized-rust)
- Initial measurements: `wasm-tools strip` and `twiggy` analysis

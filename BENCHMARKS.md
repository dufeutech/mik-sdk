# Benchmarks

Performance benchmarks for mik-sdk and mik-sql using [Criterion.rs](https://github.com/bheisler/criterion.rs).

## Running Benchmarks

### Quick Start

```bash
# Run all benchmarks
cargo bench -p mik-sdk
cargo bench -p mik-sql

# Run specific benchmark suite
cargo bench -p mik-sdk -- json
cargo bench -p mik-sdk -- request
cargo bench -p mik-sdk -- parsing
cargo bench -p mik-sql -- sql_builder
```

### Advanced Usage

```bash
# Run specific benchmark group
cargo bench -p mik-sdk -- json_parse
cargo bench -p mik-sql -- cursor

# Compare against baseline
cargo bench -- --save-baseline main
cargo bench -- --baseline main

# Generate HTML report (opens in browser)
cargo bench -- --open
```

## Benchmark Suites

### mik-sdk Benchmarks

#### JSON Operations (`benches/json.rs`)

| Benchmark              | What It Measures                                    |
| ---------------------- | --------------------------------------------------- |
| `json_parse`           | Parsing speed at different sizes (50B to 50KB)      |
| `json_obj_builder`     | Object building with 3-20 fields, nested structures |
| `json_arr_builder`     | Array building with 5-50 items                      |
| `json_path_extraction` | Lazy `path_str()` vs chained `.get()` calls         |
| `json_serialization`   | `to_string()` and `to_bytes()` performance          |

**Key insight:** Lazy path extraction (`path_str()`) is significantly faster than building a full tree and traversing it.

#### Request Operations (`benches/request.rs`)

| Benchmark             | What It Measures                                        |
| --------------------- | ------------------------------------------------------- |
| `query_parsing`       | URL query string parsing (1-10 params, encoded, arrays) |
| `query_access`        | Query parameter lookup (cached vs uncached)             |
| `header_lookup`       | Header access (case-insensitive matching)               |
| `header_all`          | Multiple values for same header (e.g., Set-Cookie)      |
| `path_params`         | Path parameter extraction                               |
| `request_creation`    | Request struct initialization overhead                  |
| `content_type_checks` | `is_json()`, `is_form()`, `accepts()` checks            |
| `body_access`         | Body retrieval patterns                                 |

#### Parsing Operations (`benches/parsing.rs`)

| Benchmark          | What It Measures                             |
| ------------------ | -------------------------------------------- |
| `request_creation` | Request creation with various configurations |
| `request_access`   | Access patterns (query, headers, params)     |
| `json_parsing`     | Parsing request bodies (simple to complex)   |
| `json_building`    | Building response objects                    |
| `json_access`      | Field access patterns                        |
| `form_parsing`     | Form-encoded body parsing                    |
| `url_decoding`     | URL decoding performance                     |

### mik-sql Benchmarks

#### SQL Builder (`benches/sql_builder.rs`)

| Benchmark           | What It Measures                               |
| ------------------- | ---------------------------------------------- |
| `sql_validation`    | Identifier and expression validation           |
| `query_builder`     | SELECT query building (simple to complex)      |
| `dialects`          | PostgreSQL vs SQLite dialect comparison        |
| `cursor`            | Cursor creation, encoding, decoding            |
| `filter_validation` | Filter operator validation                     |
| `string_escaping`   | SQL string escaping                            |
| `complex_queries`   | Real-world query patterns (pagination, search) |

## Performance Tips

Based on benchmark results:

1. **Use lazy path extraction** - `path_str(&["user", "name"])` is faster than `.get("user").get("name").str()`

2. **Minimize JSON parsing** - Parse once, extract multiple fields from the same `JsonValue`

3. **Batch SQL queries** - Use `WHERE id IN (...)` instead of per-row queries

4. **Reuse request data** - Query parameters and headers are lazily parsed and cached

## CI Integration

Benchmarks are compiled (but not run) in CI to catch build regressions. To run full benchmarks locally:

```bash
# Full benchmark suite (~5-10 minutes)
cargo bench --all

# Quick smoke test
cargo bench -p mik-sdk -- --sample-size 10
```

## Adding New Benchmarks

1. Add benchmark function to appropriate file in `benches/`
2. Register with Criterion: `criterion_group!(benches, your_benchmark);`
3. Run locally to verify: `cargo bench -p <crate> -- <name>`

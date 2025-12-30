# Parallel Execution Plan for File Splitting

## Dependency Analysis

```
mik-sdk-macros/     (independent crate)
├── lib.rs ─────────┐
├── schema.rs ──────┼── can run in parallel (different files)
├── derive.rs ──────┘
└── response.rs

mik-sql-macros/     (independent crate)
└── lib.rs          (single file, standalone)

mik-sql/            (independent crate)
├── validate.rs ◄── used by builder.rs & pagination.rs
├── builder.rs
└── pagination.rs

mik-sdk/            (independent crate)
├── json.rs ◄────── used by typed.rs
├── typed.rs
├── http_client/mod.rs
└── log.rs
```

## Parallel Streams (4 agents)

### Stream A: `mik-sql-macros` (1 file)
**Agent can start immediately**

| Order | File | Lines | Split Into |
|:-----:|------|------:|------------|
| 1 | `mik-sql-macros/src/lib.rs` | 1,641 | `read.rs`, `create.rs`, `update.rs`, `delete.rs`, `common.rs` |

**Instructions:**
1. Create `mik-sql-macros/src/` submodules
2. Extract each CRUD macro to its own file
3. Keep shared parsing logic in `common.rs`
4. Update `lib.rs` to re-export from submodules
5. Run `cargo test -p mik-sql-macros`

---

### Stream B: `mik-sdk-macros` (3 files)
**Agent can start immediately**

| Order | File | Lines | Split Into |
|:-----:|------|------:|------------|
| 1 | `schema.rs` | 996 | `schema/type_schema.rs`, `schema/field_schema.rs`, `schema/openapi.rs` |
| 2 | `derive.rs` | 829 | `derive/type_derive.rs`, `derive/query_derive.rs`, `derive/path_derive.rs` |
| 3 | `response.rs` | 752 | Only if time permits (lower priority) |

**Instructions:**
1. Create `mik-sdk-macros/src/schema/` directory with `mod.rs`
2. Create `mik-sdk-macros/src/derive/` directory with `mod.rs`
3. Split by logical concern (each derive macro gets its own file)
4. Run `cargo test -p mik-sdk-macros`

---

### Stream C: `mik-sql` (3 files - sequential due to dependencies)
**Agent can start immediately**

| Order | File | Lines | Split Into | Notes |
|:-----:|------|------:|------------|-------|
| 1 | `validate.rs` | 1,556 | `validate/filter.rs`, `validate/column.rs`, `validate/order.rs` | **DO FIRST** - others depend on it |
| 2 | `builder.rs` | 1,415 | `builder/select.rs`, `builder/insert.rs`, `builder/update.rs`, `builder/delete.rs` | After validate.rs |
| 3 | `pagination.rs` | 1,252 | `pagination/cursor.rs`, `pagination/page_info.rs`, `pagination/encoding.rs` | After validate.rs |

**Instructions:**
1. Start with `validate.rs` - others import from it
2. Create `mik-sql/src/validate/` with `mod.rs`
3. After validate.rs is done, builder.rs and pagination.rs can be done in any order
4. Run `cargo test -p mik-sql` after each file

---

### Stream D: `mik-sdk` (4 files - json.rs first due to dependency)
**Agent can start immediately**

| Order | File | Lines | Split Into | Notes |
|:-----:|------|------:|------------|-------|
| 1 | `json.rs` | 2,494 | `json/lazy.rs`, `json/value.rs`, `json/builder.rs`, `json/to_json.rs`, `json/tests.rs` | **DO FIRST** - typed.rs uses it |
| 2 | `typed.rs` | 1,550 | `typed/parse_error.rs`, `typed/validation_error.rs` | After json.rs |
| 3 | `http_client/mod.rs` | 967 | `http_client/client.rs`, `http_client/builder.rs` (ssrf.rs exists) | Independent |
| 4 | `log.rs` | 704 | Only if time permits | Independent, lower priority |

**Instructions:**
1. Start with `json.rs` - largest file, typed.rs imports from it
2. Create `mik-sdk/src/json/` with `mod.rs`
3. Keep lazy scanner in `lazy.rs`, builder API in `builder.rs`, ToJson in `to_json.rs`
4. Run `cargo test -p mik-sdk` after each file

---

## Execution Timeline

```
Time ──────────────────────────────────────────────────────►

Stream A (mik-sql-macros):
[═══ lib.rs ═══] ✓ DONE

Stream B (mik-sdk-macros):
[═══ schema.rs ═══][═══ derive.rs ═══][response.rs?]

Stream C (mik-sql):
[═══ validate.rs ═══][═══ builder.rs ═══]
                     [═══ pagination.rs ═══]  ← can parallel after validate

Stream D (mik-sdk):
[═══════ json.rs ═══════][═══ typed.rs ═══][http_client][log?]
```

## Verification Commands

After all streams complete:

```bash
# Full test suite
cargo test --all

# Clippy check
cargo clippy --all

# Format check
cargo fmt --check
```

## Summary

| Stream | Crate | Files | Est. Complexity |
|--------|-------|------:|-----------------|
| A | mik-sql-macros | 1 | Medium |
| B | mik-sdk-macros | 2-3 | Medium |
| C | mik-sql | 3 | High (dependencies) |
| D | mik-sdk | 3-4 | High (json.rs is large) |

**Total: 4 parallel agents, 9-11 files**

---

## Agent Prompts (copy-paste ready)

### Agent A Prompt:
```
Refactor mik-sql-macros/src/lib.rs (1,641 lines) by splitting into submodules:
- read.rs: sql_read! macro
- create.rs: sql_create! macro
- update.rs: sql_update! macro
- delete.rs: sql_delete! macro
- common.rs: shared parsing/validation logic

Keep lib.rs as the entry point that re-exports. Run cargo test -p mik-sql-macros when done.
```

### Agent B Prompt:
```
Refactor mik-sdk-macros by splitting these files into submodules:

1. src/schema.rs (996 lines) → src/schema/mod.rs with:
   - type_schema.rs: Type schema generation
   - field_schema.rs: Field schema logic
   - openapi.rs: OpenAPI spec generation

2. src/derive.rs (829 lines) → src/derive/mod.rs with:
   - type_derive.rs: #[derive(Type)] implementation
   - query_derive.rs: #[derive(Query)] implementation
   - path_derive.rs: #[derive(Path)] implementation

Run cargo test -p mik-sdk-macros when done.
```

### Agent C Prompt:
```
Refactor mik-sql crate by splitting these files into submodules.
ORDER MATTERS - validate.rs must be done first as others depend on it.

1. src/validate.rs (1,556 lines) → src/validate/mod.rs with:
   - filter.rs: Filter validation
   - column.rs: Column validation
   - order.rs: Order clause validation

2. src/builder.rs (1,415 lines) → src/builder/mod.rs with:
   - select.rs: SELECT query building
   - insert.rs: INSERT query building
   - update.rs: UPDATE query building
   - delete.rs: DELETE query building

3. src/pagination.rs (1,252 lines) → src/pagination/mod.rs with:
   - cursor.rs: Cursor encoding/decoding
   - page_info.rs: PageInfo struct
   - encoding.rs: Base64/serialization

Run cargo test -p mik-sql after each file.
```

### Agent D Prompt:
```
Refactor mik-sdk crate by splitting these files into submodules.
ORDER MATTERS - json.rs must be done first as typed.rs depends on it.

1. src/json.rs (2,494 lines) → src/json/mod.rs with:
   - lazy.rs: Lazy JSON scanner (lines 53-527)
   - value.rs: JsonValue struct and methods (lines 532-1106)
   - builder.rs: Constructors obj(), arr(), str(), etc. (lines 1107-1155)
   - to_json.rs: ToJson trait + implementations (lines 1157-1422)
   - tests.rs: All test code (lines 1580-2494)

2. src/typed.rs (1,550 lines) → src/typed/mod.rs with:
   - parse_error.rs: ParseError enum
   - validation_error.rs: ValidationError enum

3. src/http_client/mod.rs (967 lines) - split further if needed

Run cargo test -p mik-sdk after each file.
```

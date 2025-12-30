# File Splitting Refactoring Order

Files with 600+ LOC sorted by priority for splitting into submodules.

## Phase 1: HIGH Priority

| Order | File                        | Lines | Suggested Split                                     |
| :---: | --------------------------- | ----: | --------------------------------------------------- |
|   1   | `mik-sdk/src/json.rs`       | 2,494 | builder.rs, parser.rs, value.rs, traits.rs          |
|   2   | `mik-sql-macros/src/lib.rs` | 1,641 | read.rs, create.rs, update.rs, delete.rs, common.rs |
|   3   | `mik-sql/src/validate.rs`   | 1,556 | filter.rs, column.rs, order.rs, pagination.rs       |

## Phase 2: MEDIUM Priority

| Order | File                             | Lines | Suggested Split                                       |
| :---: | -------------------------------- | ----: | ----------------------------------------------------- |
|   4   | `mik-sdk/src/typed.rs`           | 1,550 | parse_error.rs, validation_error.rs, common.rs        |
|   5   | `mik-sql/src/builder.rs`         | 1,415 | select.rs, insert.rs, update.rs, delete.rs, clause.rs |
|   6   | `mik-sql/src/pagination.rs`      | 1,252 | cursor.rs, page_info.rs, encoding.rs                  |
|   7   | `mik-sdk-macros/src/schema.rs`   |   996 | type_schema.rs, field_schema.rs, openapi.rs           |
|   8   | `mik-sdk/src/http_client/mod.rs` |   967 | client.rs, builder.rs, ssrf.rs (already exists)       |
|   9   | `mik-sdk-macros/src/derive.rs`   |   829 | type_derive.rs, query_derive.rs, path_derive.rs       |

## Phase 3: LOW Priority

| Order | File                                              | Lines | Notes                                         |
| :---: | ------------------------------------------------- | ----: | --------------------------------------------- |
|  10   | `mik-sdk/src/request/tests.rs`                    | 2,265 | Test file - split by test category if desired |
|  11   | `mik-sql/tests/sql_macro_test.rs`                 | 1,648 | Test file - split by CRUD operation           |
|  12   | `mik-sdk-macros/tests/routes_integration_test.rs` | 1,134 | Test file - split by route pattern            |
|  13   | `mik-sdk-macros/src/response.rs`                  |   752 | Cohesive, split only if needed                |
|  14   | `mik-sdk-macros/tests/schema_macro_test.rs`       |   734 | Test file                                     |
|  15   | `mik-sdk/src/log.rs`                              |   704 | Single concern, split only if needed          |
|  16   | `mik-sdk/benches/request.rs`                      |   680 | Benchmark file                                |

## Total

- **Phase 1:** 3 files, 5,691 LOC
- **Phase 2:** 6 files, 7,009 LOC
- **Phase 3:** 7 files, 6,917 LOC (optional)

---
Generated: 2025-12-30

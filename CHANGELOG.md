# Changelog

All notable changes to this project will be documented in this file.
## [0.1.2] - 2026-01-02

### Bug Fixes

- Remove __schema endpoint tests
- Ignore example compile tests (require cargo-component)
- Ignore json_macro_test that requires cargo-component
- Ignore remaining tests that require cargo-component
- Exclude examples from fmt check (require cargo-component)
- Correct cargo fmt syntax for workspace packages

### Documentation

- Update READMEs for v0.1.x release
- Add parse_filter_bytes example and E2E tests
- Add beginner-friendly README for snippets
- Add READMEs for macro crates (crates.io)

### Features

- Add "did you mean?" suggestions to error messages
- Add runtime filter parsing with parse_filter
- Add VS Code snippets for all mik-sdk macros
- Static schema generation excluded from WASM builds
- Add deploy-github template for CI/CD

### Miscellaneous

- Update examples with static OpenAPI schema
- Gitignore generated files (bindings, openapi schemas)
- Relax inter-crate version constraints (0.1.0 -> 0.1)
- Normalize line endings in READMEs

### Refactor

- Consolidate and modularize macro crates
- Extract shared logic to __write_simple_log function
- Improve maintainability with debug features and modular structure
- Apply error helpers to remaining call sites
- Add "did you mean?" to HTTP methods and input sources
- Use duplicate_field_error helper
- Extract shared helpers for DRY compliance
- Rename parse_returning_fields to parse_ident_list
- Remove parse_filter_bytes (use parse_filter with req.text())

### Testing

- Add SQLite integration tests for generated SQL validation
- Add expansion snapshots and remove dead code

### Ci

- Cache tools for faster releases

## [0.1.1] - 2026-01-01

### Bug Fixes

- Improve test coverage and add E2E test harness
- Improve error messages, add helpers, polish API
- Install Playwright for mermaid rendering in docs
- Use correct wash-cli version 0.39.0
- Exclude wasmcloud from CI - requires NATS setup
- Use bump type dropdown (patch/minor/major)

### Documentation

- Convert ignored doctests to runnable examples
- Add SDK enforcement checklist for contributors
- Add documentation for bridge component functions
- Restructure README for better front page experience
- Align configuration tables in lib.rs and constants.rs

### Features

- Auto-bump versions with cargo-workspaces
- Add MIK_MAX_JSON_SIZE env var for configurable JSON size limit

### Miscellaneous

- Consolidate CI workflows and add project badges
- Add architecture decision records (ADRs)
- Stop tracking .notes folder
- Fix clippy warnings in macros crate
- Allow unsafe_code in WASM components for WIT bindings
- Improve code quality and API consistency
- Replace release-please with git-cliff, add WIT to releases
- Release v0.1.1

### Refactor

- Split large files into modular submodules
- Improve API guidelines compliance (92% → ~98%)
- Add #[non_exhaustive] and strict clippy lints
- Resolve clippy warnings and add pre-commit hook
- Use Self in Value enum (clippy use_self)
- Align pre-commit and CI clippy flags
- Add multi-runtime E2E tests and move getrandom to dev-deps
- Add enum support for #[derive(Type)] and improve error messages
- Reject trailing content after JSON value (security)
- Split large test files into organized modules

### Testing

- Improve code coverage across all crates
- Add filter.rs coverage tests
- Expand macro and E2E test coverage
- Handle connection reset in 413 payload test
- Add wasmCloud to CI and expand E2E test coverage

### Ci

- Cache WASM tooling for faster E2E runs
- Add spin to E2E test matrix
- Use correct fermyon/actions version, ignore expand test

## [mik-sdk-v0.1.0] - 2025-12-28

### Bug Fixes

- Use releases_created for cargo-workspace merged releases
- Add crates.io link to documentation

### Features

- Initial release of mik-sdk

### Miscellaneous

- Release

### Ci

- Add manual release workflow
- Use explicit wasm32-wasip2 target for bridge builds
- Fix wasm path - cargo-component outputs to wasm32-wasip1



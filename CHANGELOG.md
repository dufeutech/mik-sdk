# Changelog

All notable changes to this project will be documented in this file.
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



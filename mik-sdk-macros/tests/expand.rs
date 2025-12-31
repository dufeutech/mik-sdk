//! Macro expansion snapshot tests.
//!
//! These tests capture the expanded output of macros and compare against
//! saved snapshots. This catches regressions in macro expansion.
//!
//! To update snapshots after intentional changes:
//! ```bash
//! MACROTEST=overwrite cargo test --test expand
//! ```

/// Requires `cargo-expand` (install via `cargo install cargo-expand`).
/// Run with: `cargo test --test expand -- --ignored`
#[test]
#[ignore = "requires cargo-expand which is not installed in CI"]
fn expand_macros() {
    macrotest::expand("tests/expand/*.rs");
}

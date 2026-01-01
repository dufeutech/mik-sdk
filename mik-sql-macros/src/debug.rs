//! Debug utilities for macro development.
//!
//! Enable with `--features debug-expand` to see pretty-printed macro output.

// These utilities are available for macro debugging but not always used
#![allow(dead_code)]

use proc_macro2::TokenStream;

/// Print pretty-formatted macro expansion to stderr.
///
/// Only active when `debug-expand` feature is enabled.
/// Usage: `cargo build --features debug-expand`
#[cfg(feature = "debug-expand")]
pub fn debug_tokens(name: &str, tokens: &TokenStream) {
    eprintln!("\n=== MACRO EXPAND: {name} ===");
    if let Ok(file) = syn::parse2::<syn::File>(tokens.clone()) {
        eprintln!("{}", prettyplease::unparse(&file));
    } else {
        // Fallback to raw token stream if it's not a valid file
        eprintln!("{tokens}");
    }
    eprintln!("=== END {name} ===\n");
}

/// No-op when debug-expand feature is disabled.
#[cfg(not(feature = "debug-expand"))]
#[inline]
pub const fn debug_tokens(_name: &str, _tokens: &TokenStream) {}

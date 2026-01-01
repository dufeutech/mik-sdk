//! Tracing utilities for macro development.
//!
//! Enable with `--features macro-trace` to see parsing/codegen steps.

// These utilities are available for macro debugging but not always used
#![allow(unused_macros)]
#![allow(unused_imports)]

/// Trace macro execution phases to stderr.
///
/// Only active when `macro-trace` feature is enabled.
/// Usage: `cargo build --features macro-trace 2>&1 | grep MACRO`
#[cfg(feature = "macro-trace")]
macro_rules! trace {
    ($phase:expr, $msg:expr) => {
        eprintln!("[MACRO:{}] {}", $phase, $msg);
    };
    ($phase:expr, $msg:expr, $val:expr) => {
        eprintln!("[MACRO:{}] {}: {:?}", $phase, $msg, $val);
    };
}

/// No-op when macro-trace feature is disabled.
#[cfg(not(feature = "macro-trace"))]
macro_rules! trace {
    ($phase:expr, $msg:expr) => {};
    ($phase:expr, $msg:expr, $val:expr) => {};
}

pub(crate) use trace;

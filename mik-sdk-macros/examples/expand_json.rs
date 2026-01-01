//! Example to test json and ok macro expansion.

#![allow(dead_code, missing_docs)]

use mik_sdk::prelude::*;

fn json_example() {
    let name = "Alice";
    let age = 30;

    let value = json!({
        "name": str(name),
        "age": int(age),
        "active": true,
        "tags": ["rust", "wasm"]
    });

    println!("{value}");
}

fn main() {
    json_example();
    println!("Run with: cargo expand --example expand_json -p mik-sdk-macros");
}

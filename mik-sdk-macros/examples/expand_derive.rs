//! Example to test derive macro expansion.

#![allow(dead_code, missing_docs)]

use mik_sdk::prelude::*;

#[derive(Type)]
struct User {
    name: String,
    age: u32,
    #[field(format = "email")]
    email: Option<String>,
}

#[derive(Type)]
enum Status {
    Active,
    Inactive,
    Pending,
}

#[derive(Query)]
struct ListParams {
    #[field(default = 1)]
    page: u32,
    #[field(default = 20)]
    limit: u32,
    search: Option<String>,
}

#[derive(Path)]
struct UserPath {
    id: String,
}

fn main() {
    println!("Run with: cargo expand --example expand_derive -p mik-sdk-macros");
}

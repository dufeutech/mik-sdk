// Test #[derive(Type)] on struct
use mik_sdk::prelude::*;

#[derive(Type)]
pub struct User {
    pub name: String,
    pub age: u32,
    pub email: Option<String>,
}

fn main() {}

// Test #[derive(Type)] on enum
use mik_sdk::prelude::*;

#[derive(Type)]
pub enum Status {
    Active,
    Inactive,
    Pending,
}

fn main() {}

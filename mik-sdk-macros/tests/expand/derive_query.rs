// Test #[derive(Query)] on struct
use mik_sdk::prelude::*;

#[derive(Query)]
pub struct ListParams {
    #[field(default = 1)]
    pub page: u32,
    #[field(default = 20)]
    pub limit: u32,
    pub search: Option<String>,
}

fn main() {}

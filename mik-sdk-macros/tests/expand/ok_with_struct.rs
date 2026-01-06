// Test ok! macro with struct expressions
use mik_sdk::prelude::*;

mod bindings {
    pub mod exports {
        pub mod mik {
            pub mod core {
                pub mod handler {
                    pub struct Response {
                        pub status: u16,
                        pub headers: Vec<(String, String)>,
                        pub body: Option<Vec<u8>>,
                    }
                }
            }
        }
    }
}

use bindings::exports::mik::core::handler;

#[derive(Type)]
pub struct HelloResponse {
    pub greeting: String,
    pub count: i32,
}

// Test ok! with struct literal expression
fn with_struct_literal() -> handler::Response {
    ok!(HelloResponse {
        greeting: "Hello".to_string(),
        count: 42
    })
}

// Test ok! with variable
fn with_variable() -> handler::Response {
    let response = HelloResponse {
        greeting: "World".to_string(),
        count: 100,
    };
    ok!(response)
}

fn main() {
    let _ = with_struct_literal();
    let _ = with_variable();
}

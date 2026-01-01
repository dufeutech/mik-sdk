// Test ok! macro expansion
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

fn example() -> handler::Response {
    ok!({ "message": "Hello, World!" })
}

fn main() {
    let _ = example();
}

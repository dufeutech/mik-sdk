// Test ok! macro expansion with headers
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

fn example_with_headers() -> handler::Response {
    let trace_id = "abc-123";
    ok!({ "message": "Hello, World!" }, headers: {
        "X-Request-Id": trace_id,
        "Cache-Control": "no-cache"
    })
}

fn main() {
    let response = example_with_headers();
    // Should have Content-Type + 2 custom headers
    assert!(response.headers.len() >= 3);
}

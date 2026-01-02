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
    handler::Response {
        status: 200,
        headers: <[_]>::into_vec(
            ::alloc::boxed::box_new([
                (
                    ::mik_sdk::constants::HEADER_CONTENT_TYPE.to_string(),
                    ::mik_sdk::constants::MIME_JSON.to_string(),
                ),
            ]),
        ),
        body: Some(json::obj().set("message", json::str("Hello, World!")).to_bytes()),
    }
}
fn main() {
    let _ = example();
}

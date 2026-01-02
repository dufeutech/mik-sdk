// Test routes! macro expansion
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

                    pub trait Guest {
                        fn handle(req: super::super::super::super::RequestData) -> Response;
                    }
                }
            }
        }
    }

    pub struct RequestData {
        pub method: Method,
        pub path: String,
        pub headers: Vec<(String, String)>,
        pub body: Option<Vec<u8>>,
    }

    pub enum Method {
        Get,
        Post,
        Put,
        Patch,
        Delete,
        Head,
        Options,
    }
}

use bindings::exports::mik::core::handler::{self, Response};

#[derive(Path)]
pub struct HelloPath {
    pub name: String,
}

#[derive(Query)]
pub struct SearchQuery {
    pub q: Option<String>,
    #[field(default = 1)]
    pub page: u32,
}

#[derive(Type)]
pub struct EchoBody {
    pub message: String,
}

// Routes with all input types
routes! {
    GET "/" => home,
    GET "/hello/{name}" => hello(path: HelloPath),
    GET "/search" => search(query: SearchQuery),
    POST "/echo" => echo(body: EchoBody),
}

fn home(_req: &Request) -> Response {
    handler::Response {
        status: 200,
        headers: vec![],
        body: None,
    }
}

fn hello(_path: HelloPath, _req: &Request) -> Response {
    handler::Response {
        status: 200,
        headers: vec![],
        body: None,
    }
}

fn search(_query: SearchQuery, _req: &Request) -> Response {
    handler::Response {
        status: 200,
        headers: vec![],
        body: None,
    }
}

fn echo(_body: EchoBody, _req: &Request) -> Response {
    handler::Response {
        status: 200,
        headers: vec![],
        body: None,
    }
}

fn main() {}

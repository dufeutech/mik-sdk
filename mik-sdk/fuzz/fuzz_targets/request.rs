#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use mik_sdk::{Method, Request};
use std::collections::HashMap;

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    method: u8,
    path: String,
    headers: Vec<(String, String)>,
    body: Option<Vec<u8>>,
    params: Vec<(String, String)>,
}

fuzz_target!(|input: FuzzInput| {
    // Map arbitrary u8 to Method enum
    let method = match input.method % 7 {
        0 => Method::Get,
        1 => Method::Post,
        2 => Method::Put,
        3 => Method::Patch,
        4 => Method::Delete,
        5 => Method::Head,
        _ => Method::Options,
    };

    let params: HashMap<String, String> = input.params.into_iter().collect();

    let req = Request::new(method, input.path, input.headers, input.body, params);

    // Exercise all Request methods - none should panic
    let _ = req.method();
    let _ = req.path();
    let _ = req.body();
    let _ = req.text();

    // Query parameter extraction
    let _ = req.query_or("test", "");
    let _ = req.query_or("foo", "");
    let _ = req.query_all("bar");

    // Header extraction
    let _ = req.header_or("content-type", "");
    let _ = req.header_or("authorization", "");
    let _ = req.header_all("set-cookie");
    let _ = req.trace_id_or("");

    // Content type checks
    let _ = req.is_json();
    let _ = req.is_html();
    let _ = req.is_form();
    let _ = req.accepts("json");
    let _ = req.accepts("html");

    // Form body parsing
    let _ = req.form_or("field", "");
    let _ = req.form_all("fields");

    // Path parameter extraction
    let _ = req.param_or("id", "");
    let _ = req.param_or("name", "");
});

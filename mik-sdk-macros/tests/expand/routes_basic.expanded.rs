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
                        fn handle(
                            req: super::super::super::super::RequestData,
                        ) -> Response;
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
pub struct HelloPath {
    pub name: String,
}
impl mik_sdk::typed::FromPath for HelloPath {
    fn from_params(
        __params: &::std::collections::HashMap<String, String>,
    ) -> Result<Self, mik_sdk::typed::ParseError> {
        Ok(Self {
            name: __params
                .get("name")
                .ok_or_else(|| mik_sdk::typed::ParseError::missing("name"))?
                .clone(),
        })
    }
}
impl mik_sdk::typed::OpenApiSchema for HelloPath {
    fn openapi_schema() -> &'static str {
        "{\"type\":\"object\",\"properties\":{\"name\":{\"type\":\"string\"}},\"required\":[\"name\"]}"
    }
    fn schema_name() -> &'static str {
        "HelloPath"
    }
}
pub struct SearchQuery {
    pub q: Option<String>,
    #[field(default = 1)]
    pub page: u32,
}
impl mik_sdk::typed::FromQuery for SearchQuery {
    fn from_query(
        __params: &[(String, String)],
    ) -> Result<Self, mik_sdk::typed::ParseError> {
        let mut q: Option<String> = None;
        let mut page: u32 = 1;
        for (__k, __v) in __params {
            match __k.as_str() {
                "q" => {
                    q = Some(
                        __v
                            .parse()
                            .map_err(|_| mik_sdk::typed::ParseError::type_mismatch(
                                "q",
                                "string",
                            ))?,
                    );
                }
                "page" => {
                    page = __v
                        .parse()
                        .map_err(|_| mik_sdk::typed::ParseError::type_mismatch(
                            "page",
                            "integer",
                        ))?;
                }
                _ => {}
            }
        }
        Ok(Self { q, page })
    }
}
impl mik_sdk::typed::OpenApiSchema for SearchQuery {
    fn openapi_schema() -> &'static str {
        "{\"type\":\"object\",\"properties\":{\"q\":{\"type\":\"string\"},\"page\":{\"type\":\"integer\"}}}"
    }
    fn schema_name() -> &'static str {
        "SearchQuery"
    }
    fn openapi_query_params() -> &'static str {
        "[{\"name\":\"q\",\"in\":\"query\",\"required\":false,\"schema\":{\"type\":\"string\"}},{\"name\":\"page\",\"in\":\"query\",\"required\":false,\"schema\":{\"type\":\"integer\"}}]"
    }
}
pub struct EchoBody {
    pub message: String,
}
impl mik_sdk::typed::FromJson for EchoBody {
    fn from_json(
        __value: &mik_sdk::json::JsonValue,
    ) -> Result<Self, mik_sdk::typed::ParseError> {
        Ok(Self {
            message: __value
                .get("message")
                .str()
                .ok_or_else(|| mik_sdk::typed::ParseError::missing("message"))?,
        })
    }
}
impl mik_sdk::typed::Validate for EchoBody {
    fn validate(&self) -> Result<(), mik_sdk::typed::ValidationError> {
        Ok(())
    }
}
impl mik_sdk::typed::OpenApiSchema for EchoBody {
    fn openapi_schema() -> &'static str {
        "{\"type\":\"object\",\"properties\":{\"message\":{\"type\":\"string\"}},\"required\":[\"message\"]}  "
    }
    fn schema_name() -> &'static str {
        "EchoBody"
    }
}
const _: () = {
    fn __mik_check_bindings_setup() {
        fn __check<T: handler::Guest>() {}
    }
};
/// Handler for /__schema endpoint - returns OpenAPI JSON.
pub fn __schema(_req: &mik_sdk::Request) -> handler::Response {
    let schema_json = {
        let __mik_paths: Vec<String> = <[_]>::into_vec(
            ::alloc::boxed::box_new([
                {
                    let __mik_methods: Vec<String> = <[_]>::into_vec(
                        ::alloc::boxed::box_new([
                            "\"post\":{\"requestBody\":{\"required\":true,\"content\":{\"application/json\":{\"schema\":{\"$ref\":\"#/components/schemas/EchoBody\"}}}},\"responses\":{\"200\":{\"description\":\"Success\"}}}"
                                .to_string(),
                        ]),
                    );
                    ::alloc::__export::must_use({
                        ::alloc::fmt::format(
                            format_args!(
                                "\"{0}\":{{{1}}}", "/echo", __mik_methods.join(","),
                            ),
                        )
                    })
                },
                {
                    let __mik_methods: Vec<String> = <[_]>::into_vec(
                        ::alloc::boxed::box_new([
                            "\"get\":{\"parameters\":[{\"name\":\"name\",\"in\":\"path\",\"required\":true,\"schema\":{\"type\":\"string\"}}],\"responses\":{\"200\":{\"description\":\"Success\"}}}"
                                .to_string(),
                        ]),
                    );
                    ::alloc::__export::must_use({
                        ::alloc::fmt::format(
                            format_args!(
                                "\"{0}\":{{{1}}}", "/hello/{name}", __mik_methods.join(","),
                            ),
                        )
                    })
                },
                {
                    let __mik_methods: Vec<String> = <[_]>::into_vec(
                        ::alloc::boxed::box_new([
                            "\"get\":{\"responses\":{\"200\":{\"description\":\"Success\"}}}"
                                .to_string(),
                        ]),
                    );
                    ::alloc::__export::must_use({
                        ::alloc::fmt::format(
                            format_args!("\"{0}\":{{{1}}}", "/", __mik_methods.join(",")),
                        )
                    })
                },
                {
                    let __mik_methods: Vec<String> = <[_]>::into_vec(
                        ::alloc::boxed::box_new([
                            {
                                let __mik_query_params = <SearchQuery as mik_sdk::typed::OpenApiSchema>::openapi_query_params();
                                let __mik_query_inner = __mik_query_params
                                    .strip_prefix('[')
                                    .and_then(|s| s.strip_suffix(']'))
                                    .unwrap_or("");
                                let __mik_all_params = if "".is_empty()
                                    && __mik_query_inner.is_empty()
                                {
                                    String::new()
                                } else if "".is_empty() {
                                    ::alloc::__export::must_use({
                                        ::alloc::fmt::format(
                                            format_args!("\"parameters\":[{0}]", __mik_query_inner),
                                        )
                                    })
                                } else if __mik_query_inner.is_empty() {
                                    ::alloc::__export::must_use({
                                        ::alloc::fmt::format(
                                            format_args!("\"parameters\":[{0}]", ""),
                                        )
                                    })
                                } else {
                                    ::alloc::__export::must_use({
                                        ::alloc::fmt::format(
                                            format_args!(
                                                "\"parameters\":[{0},{1}]", "", __mik_query_inner,
                                            ),
                                        )
                                    })
                                };
                                let mut __mik_parts: Vec<String> = Vec::new();
                                let __mik_request_body = "";
                                if !__mik_request_body.is_empty() {
                                    __mik_parts.push(__mik_request_body.to_string());
                                }
                                if !__mik_all_params.is_empty() {
                                    __mik_parts.push(__mik_all_params);
                                }
                                __mik_parts
                                    .push(
                                        "\"responses\":{\"200\":{\"description\":\"Success\"}}"
                                            .to_string(),
                                    );
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "\"{0}\":{{{1}}}", "get", __mik_parts.join(","),
                                        ),
                                    )
                                })
                            },
                        ]),
                    );
                    ::alloc::__export::must_use({
                        ::alloc::fmt::format(
                            format_args!(
                                "\"{0}\":{{{1}}}", "/search", __mik_methods.join(","),
                            ),
                        )
                    })
                },
            ]),
        );
        let __mik_paths_json = ::alloc::__export::must_use({
            ::alloc::fmt::format(format_args!("{{{0}}}", __mik_paths.join(",")))
        });
        let mut __mik_schemas: Vec<String> = Vec::new();
        __mik_schemas
            .push(
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "\"{0}\":{1}", "HelloPath", < HelloPath as
                            mik_sdk::typed::OpenApiSchema >::openapi_schema(),
                        ),
                    )
                }),
            );
        __mik_schemas
            .push(
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "\"{0}\":{1}", "SearchQuery", < SearchQuery as
                            mik_sdk::typed::OpenApiSchema >::openapi_schema(),
                        ),
                    )
                }),
            );
        __mik_schemas
            .push(
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "\"{0}\":{1}", "EchoBody", < EchoBody as
                            mik_sdk::typed::OpenApiSchema >::openapi_schema(),
                        ),
                    )
                }),
            );
        ::alloc::__export::must_use({
            ::alloc::fmt::format(
                format_args!(
                    "{{\"openapi\":\"3.0.0\",\"info\":{{\"title\":\"API\",\"version\":\"1.0.0\"}},\"paths\":{0},\"components\":{{\"schemas\":{{{1}}}}}}}",
                    __mik_paths_json, __mik_schemas.join(","),
                ),
            )
        })
    };
    handler::Response {
        status: 200,
        headers: <[_]>::into_vec(
            ::alloc::boxed::box_new([
                (
                    mik_sdk::constants::HEADER_CONTENT_TYPE.to_string(),
                    mik_sdk::constants::MIME_JSON.to_string(),
                ),
            ]),
        ),
        body: Some(schema_json.into_bytes()),
    }
}
struct Handler;
impl Guest for Handler {
    fn handle(__mik_raw: handler::RequestData) -> handler::Response {
        let __mik_method = match __mik_raw.method {
            handler::Method::Get => mik_sdk::Method::Get,
            handler::Method::Post => mik_sdk::Method::Post,
            handler::Method::Put => mik_sdk::Method::Put,
            handler::Method::Patch => mik_sdk::Method::Patch,
            handler::Method::Delete => mik_sdk::Method::Delete,
            handler::Method::Head => mik_sdk::Method::Head,
            handler::Method::Options => mik_sdk::Method::Options,
        };
        let __mik_path = __mik_raw.path.split('?').next().unwrap_or(&__mik_raw.path);
        if __mik_path == "/__schema" {
            let __mik_req = mik_sdk::Request::new(
                __mik_method,
                __mik_raw.path.clone(),
                __mik_raw.headers.clone(),
                __mik_raw.body.clone(),
                ::std::collections::HashMap::new(),
            );
            return __schema(&__mik_req);
        }
        if __mik_method == mik_sdk::Method::Get {
            let __mik_try_match = || -> Option<
                ::std::collections::HashMap<String, String>,
            > {
                if let Some(__mik_params) = (|| -> Option<
                    ::std::collections::HashMap<String, String>,
                > {
                    if __mik_path == "/" {
                        Some(::std::collections::HashMap::new())
                    } else {
                        None
                    }
                })() {
                    return Some(__mik_params);
                }
                None
            };
            if let Some(__mik_params) = __mik_try_match() {
                let __mik_req = mik_sdk::Request::new(
                    __mik_method.clone(),
                    __mik_raw.path.clone(),
                    __mik_raw.headers.clone(),
                    __mik_raw.body.clone(),
                    __mik_params.clone(),
                );
                return home(&__mik_req);
            }
        }
        if __mik_method == mik_sdk::Method::Get {
            let __mik_try_match = || -> Option<
                ::std::collections::HashMap<String, String>,
            > {
                if let Some(__mik_params) = (|| -> Option<
                    ::std::collections::HashMap<String, String>,
                > {
                    let __mik_segments: Vec<&str> = __mik_path.split('/').collect();
                    if __mik_segments.len() == 3usize
                        && __mik_segments[1usize] == "hello"
                    {
                        let mut __mik_params = ::std::collections::HashMap::new();
                        let __mik_raw_param = __mik_segments[2usize];
                        let __mik_decoded_param = mik_sdk::url_decode(__mik_raw_param)
                            .unwrap_or_else(|_| __mik_raw_param.to_string());
                        __mik_params.insert("name".to_string(), __mik_decoded_param);
                        Some(__mik_params)
                    } else {
                        None
                    }
                })() {
                    return Some(__mik_params);
                }
                None
            };
            if let Some(__mik_params) = __mik_try_match() {
                let __mik_req = mik_sdk::Request::new(
                    __mik_method.clone(),
                    __mik_raw.path.clone(),
                    __mik_raw.headers.clone(),
                    __mik_raw.body.clone(),
                    __mik_params.clone(),
                );
                let __mik_input_0 = match <HelloPath as mik_sdk::typed::FromPath>::from_params(
                    &__mik_params,
                ) {
                    Ok(v) => v,
                    Err(e) => {
                        return handler::Response {
                            status: 400,
                            headers: <[_]>::into_vec(
                                ::alloc::boxed::box_new([
                                    (
                                        mik_sdk::constants::HEADER_CONTENT_TYPE.to_string(),
                                        mik_sdk::constants::MIME_PROBLEM_JSON.to_string(),
                                    ),
                                ]),
                            ),
                            body: Some(
                                mik_sdk::json::obj()
                                    .set("type", mik_sdk::json::str("about:blank"))
                                    .set(
                                        "title",
                                        mik_sdk::json::str(mik_sdk::constants::status_title(400)),
                                    )
                                    .set("status", mik_sdk::json::int(400))
                                    .set("detail", mik_sdk::json::str(&e.to_string()))
                                    .to_bytes(),
                            ),
                        };
                    }
                };
                return hello(__mik_input_0, &__mik_req);
            }
        }
        if __mik_method == mik_sdk::Method::Get {
            let __mik_try_match = || -> Option<
                ::std::collections::HashMap<String, String>,
            > {
                if let Some(__mik_params) = (|| -> Option<
                    ::std::collections::HashMap<String, String>,
                > {
                    if __mik_path == "/search" {
                        Some(::std::collections::HashMap::new())
                    } else {
                        None
                    }
                })() {
                    return Some(__mik_params);
                }
                None
            };
            if let Some(__mik_params) = __mik_try_match() {
                let __mik_req = mik_sdk::Request::new(
                    __mik_method.clone(),
                    __mik_raw.path.clone(),
                    __mik_raw.headers.clone(),
                    __mik_raw.body.clone(),
                    __mik_params.clone(),
                );
                let __mik_query_params: Vec<(String, String)> = __mik_req
                    .path()
                    .split_once('?')
                    .map(|(_, q)| {
                        q.split('&')
                            .filter_map(|pair| {
                                let mut parts = pair.splitn(2, '=');
                                Some((
                                    parts.next()?.to_string(),
                                    parts.next().unwrap_or("").to_string(),
                                ))
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                let __mik_input_0 = match <SearchQuery as mik_sdk::typed::FromQuery>::from_query(
                    &__mik_query_params,
                ) {
                    Ok(v) => v,
                    Err(e) => {
                        return handler::Response {
                            status: 400,
                            headers: <[_]>::into_vec(
                                ::alloc::boxed::box_new([
                                    (
                                        mik_sdk::constants::HEADER_CONTENT_TYPE.to_string(),
                                        mik_sdk::constants::MIME_PROBLEM_JSON.to_string(),
                                    ),
                                ]),
                            ),
                            body: Some(
                                mik_sdk::json::obj()
                                    .set("type", mik_sdk::json::str("about:blank"))
                                    .set(
                                        "title",
                                        mik_sdk::json::str(mik_sdk::constants::status_title(400)),
                                    )
                                    .set("status", mik_sdk::json::int(400))
                                    .set("detail", mik_sdk::json::str(&e.to_string()))
                                    .to_bytes(),
                            ),
                        };
                    }
                };
                return search(__mik_input_0, &__mik_req);
            }
        }
        if __mik_method == mik_sdk::Method::Post {
            let __mik_try_match = || -> Option<
                ::std::collections::HashMap<String, String>,
            > {
                if let Some(__mik_params) = (|| -> Option<
                    ::std::collections::HashMap<String, String>,
                > {
                    if __mik_path == "/echo" {
                        Some(::std::collections::HashMap::new())
                    } else {
                        None
                    }
                })() {
                    return Some(__mik_params);
                }
                None
            };
            if let Some(__mik_params) = __mik_try_match() {
                let __mik_req = mik_sdk::Request::new(
                    __mik_method.clone(),
                    __mik_raw.path.clone(),
                    __mik_raw.headers.clone(),
                    __mik_raw.body.clone(),
                    __mik_params.clone(),
                );
                let __mik_input_0 = match __mik_req.body() {
                    Some(bytes) => {
                        match mik_sdk::json::try_parse(bytes) {
                            Some(json) => {
                                match <EchoBody as mik_sdk::typed::FromJson>::from_json(
                                    &json,
                                ) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        return handler::Response {
                                            status: 400,
                                            headers: <[_]>::into_vec(
                                                ::alloc::boxed::box_new([
                                                    (
                                                        mik_sdk::constants::HEADER_CONTENT_TYPE.to_string(),
                                                        mik_sdk::constants::MIME_PROBLEM_JSON.to_string(),
                                                    ),
                                                ]),
                                            ),
                                            body: Some(
                                                mik_sdk::json::obj()
                                                    .set("type", mik_sdk::json::str("about:blank"))
                                                    .set(
                                                        "title",
                                                        mik_sdk::json::str(mik_sdk::constants::status_title(400)),
                                                    )
                                                    .set("status", mik_sdk::json::int(400))
                                                    .set("detail", mik_sdk::json::str(&e.to_string()))
                                                    .to_bytes(),
                                            ),
                                        };
                                    }
                                }
                            }
                            None => {
                                return handler::Response {
                                    status: 400,
                                    headers: <[_]>::into_vec(
                                        ::alloc::boxed::box_new([
                                            (
                                                mik_sdk::constants::HEADER_CONTENT_TYPE.to_string(),
                                                mik_sdk::constants::MIME_PROBLEM_JSON.to_string(),
                                            ),
                                        ]),
                                    ),
                                    body: Some(
                                        mik_sdk::json::obj()
                                            .set("type", mik_sdk::json::str("about:blank"))
                                            .set(
                                                "title",
                                                mik_sdk::json::str(mik_sdk::constants::status_title(400)),
                                            )
                                            .set("status", mik_sdk::json::int(400))
                                            .set("detail", mik_sdk::json::str("Invalid JSON body"))
                                            .to_bytes(),
                                    ),
                                };
                            }
                        }
                    }
                    None => {
                        return handler::Response {
                            status: 400,
                            headers: <[_]>::into_vec(
                                ::alloc::boxed::box_new([
                                    (
                                        mik_sdk::constants::HEADER_CONTENT_TYPE.to_string(),
                                        mik_sdk::constants::MIME_PROBLEM_JSON.to_string(),
                                    ),
                                ]),
                            ),
                            body: Some(
                                mik_sdk::json::obj()
                                    .set("type", mik_sdk::json::str("about:blank"))
                                    .set(
                                        "title",
                                        mik_sdk::json::str(mik_sdk::constants::status_title(400)),
                                    )
                                    .set("status", mik_sdk::json::int(400))
                                    .set("detail", mik_sdk::json::str("Request body required"))
                                    .to_bytes(),
                            ),
                        };
                    }
                };
                return echo(__mik_input_0, &__mik_req);
            }
        }
        handler::Response {
            status: 404,
            headers: <[_]>::into_vec(
                ::alloc::boxed::box_new([
                    (
                        mik_sdk::constants::HEADER_CONTENT_TYPE.to_string(),
                        mik_sdk::constants::MIME_PROBLEM_JSON.to_string(),
                    ),
                ]),
            ),
            body: Some(
                mik_sdk::json::obj()
                    .set("type", mik_sdk::json::str("about:blank"))
                    .set(
                        "title",
                        mik_sdk::json::str(mik_sdk::constants::status_title(404)),
                    )
                    .set("status", mik_sdk::json::int(404))
                    .set("detail", mik_sdk::json::str("Route not found"))
                    .to_bytes(),
            ),
        }
    }
}
#[allow(unsafe_code)]
const _: () = {
    (/*ERROR*/);
};
fn home(_req: &Request) -> Response {
    handler::Response {
        status: 200,
        headers: ::alloc::vec::Vec::new(),
        body: None,
    }
}
fn hello(_path: HelloPath, _req: &Request) -> Response {
    handler::Response {
        status: 200,
        headers: ::alloc::vec::Vec::new(),
        body: None,
    }
}
fn search(_query: SearchQuery, _req: &Request) -> Response {
    handler::Response {
        status: 200,
        headers: ::alloc::vec::Vec::new(),
        body: None,
    }
}
fn echo(_body: EchoBody, _req: &Request) -> Response {
    handler::Response {
        status: 200,
        headers: ::alloc::vec::Vec::new(),
        body: None,
    }
}
fn main() {}

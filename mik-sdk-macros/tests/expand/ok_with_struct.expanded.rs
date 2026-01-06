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
pub struct HelloResponse {
    pub greeting: String,
    pub count: i32,
}
impl mik_sdk::typed::FromJson for HelloResponse {
    fn from_json(
        __value: &mik_sdk::json::JsonValue,
    ) -> Result<Self, mik_sdk::typed::ParseError> {
        Ok(Self {
            greeting: __value
                .get("greeting")
                .str()
                .ok_or_else(|| mik_sdk::typed::ParseError::missing("greeting"))?,
            count: __value
                .get("count")
                .int()
                .map(|n| n as _)
                .ok_or_else(|| mik_sdk::typed::ParseError::missing("count"))?,
        })
    }
}
impl mik_sdk::json::ToJson for HelloResponse {
    fn to_json(&self) -> mik_sdk::json::JsonValue {
        mik_sdk::json::obj()
            .set("greeting", mik_sdk::json::ToJson::to_json(&self.greeting))
            .set("count", mik_sdk::json::ToJson::to_json(&self.count))
    }
}
impl mik_sdk::typed::Validate for HelloResponse {
    fn validate(&self) -> Result<(), mik_sdk::typed::ValidationError> {
        Ok(())
    }
}
impl mik_sdk::typed::OpenApiSchema for HelloResponse {
    fn openapi_schema() -> &'static str {
        "{\"type\":\"object\",\"properties\":{\"greeting\":{\"type\":\"string\"},\"count\":{\"type\":\"integer\"}},\"required\":[\"greeting\",\"count\"]}"
    }
    fn schema_name() -> &'static str {
        "HelloResponse"
    }
}
fn with_struct_literal() -> handler::Response {
    handler::Response {
        status: 200,
        headers: vec![
            (
                ::mik_sdk::constants::HEADER_CONTENT_TYPE.to_string(),
                ::mik_sdk::constants::MIME_JSON.to_string(),
            ),
        ],
        body: Some(
            json::ToJson::to_json(
                &(HelloResponse {
                    greeting: "Hello".to_string(),
                    count: 42,
                }),
            )
            .to_bytes(),
        ),
    }
}
fn with_variable() -> handler::Response {
    let response = HelloResponse {
        greeting: "World".to_string(),
        count: 100,
    };
    handler::Response {
        status: 200,
        headers: vec![
            (
                ::mik_sdk::constants::HEADER_CONTENT_TYPE.to_string(),
                ::mik_sdk::constants::MIME_JSON.to_string(),
            ),
        ],
        body: Some(json::ToJson::to_json(&(response)).to_bytes()),
    }
}
fn main() {
    let _ = with_struct_literal();
    let _ = with_variable();
}

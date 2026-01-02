use mik_sdk::prelude::*;
pub struct User {
    pub name: String,
    pub age: u32,
    pub email: Option<String>,
}
impl mik_sdk::typed::FromJson for User {
    fn from_json(
        __value: &mik_sdk::json::JsonValue,
    ) -> Result<Self, mik_sdk::typed::ParseError> {
        Ok(Self {
            name: __value
                .get("name")
                .str()
                .ok_or_else(|| mik_sdk::typed::ParseError::missing("name"))?,
            age: __value
                .get("age")
                .int()
                .map(|n| n as _)
                .ok_or_else(|| mik_sdk::typed::ParseError::missing("age"))?,
            email: {
                let v = __value.get("email");
                if v.is_null() {
                    None
                } else {
                    Some(
                        v
                            .str()
                            .ok_or_else(|| mik_sdk::typed::ParseError::type_mismatch(
                                "email",
                                "string",
                            ))?,
                    )
                }
            },
        })
    }
}
impl mik_sdk::typed::Validate for User {
    fn validate(&self) -> Result<(), mik_sdk::typed::ValidationError> {
        Ok(())
    }
}
impl mik_sdk::typed::OpenApiSchema for User {
    fn openapi_schema() -> &'static str {
        "{\"type\":\"object\",\"properties\":{\"name\":{\"type\":\"string\"},\"age\":{\"type\":\"integer\"},\"email\":{\"nullable\":true,\"type\":\"string\"}},\"required\":[\"name\",\"age\"]}  "
    }
    fn schema_name() -> &'static str {
        "User"
    }
}
fn main() {}

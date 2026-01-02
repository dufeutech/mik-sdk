use mik_sdk::prelude::*;
pub enum Status {
    Active,
    Inactive,
    Pending,
}
impl ::mik_sdk::typed::FromJson for Status {
    fn from_json(
        __value: &::mik_sdk::json::JsonValue,
    ) -> Result<Self, ::mik_sdk::typed::ParseError> {
        let __s = __value
            .str()
            .ok_or_else(|| {
                ::mik_sdk::typed::ParseError::type_mismatch("value", "string")
            })?;
        match __s.as_str() {
            "active" => Ok(Self::Active),
            "inactive" => Ok(Self::Inactive),
            "pending" => Ok(Self::Pending),
            __other => {
                Err(
                    ::mik_sdk::typed::ParseError::custom(
                        "value",
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!(
                                    "unknown enum variant \"{0}\". Valid values: {1}", __other,
                                    "\"active\", \"inactive\", \"pending\"",
                                ),
                            )
                        }),
                    ),
                )
            }
        }
    }
}
impl ::mik_sdk::json::ToJson for Status {
    fn to_json(&self) -> ::mik_sdk::json::JsonValue {
        match self {
            Self::Active => ::mik_sdk::json::str("active"),
            Self::Inactive => ::mik_sdk::json::str("inactive"),
            Self::Pending => ::mik_sdk::json::str("pending"),
        }
    }
}
impl ::mik_sdk::typed::Validate for Status {
    fn validate(&self) -> Result<(), ::mik_sdk::typed::ValidationError> {
        Ok(())
    }
}
impl ::mik_sdk::typed::OpenApiSchema for Status {
    fn openapi_schema() -> &'static str {
        "{\"type\":\"string\",\"enum\":[\"active\",\"inactive\",\"pending\"]}"
    }
    fn schema_name() -> &'static str {
        "Status"
    }
}
fn main() {}

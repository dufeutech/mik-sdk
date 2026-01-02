use mik_sdk::prelude::*;
pub struct ListParams {
    #[field(default = 1)]
    pub page: u32,
    #[field(default = 20)]
    pub limit: u32,
    pub search: Option<String>,
}
impl mik_sdk::typed::FromQuery for ListParams {
    fn from_query(
        __params: &[(String, String)],
    ) -> Result<Self, mik_sdk::typed::ParseError> {
        let mut page: u32 = 1;
        let mut limit: u32 = 20;
        let mut search: Option<String> = None;
        for (__k, __v) in __params {
            match __k.as_str() {
                "page" => {
                    page = __v
                        .parse()
                        .map_err(|_| mik_sdk::typed::ParseError::type_mismatch(
                            "page",
                            "integer",
                        ))?;
                }
                "limit" => {
                    limit = __v
                        .parse()
                        .map_err(|_| mik_sdk::typed::ParseError::type_mismatch(
                            "limit",
                            "integer",
                        ))?;
                }
                "search" => {
                    search = Some(
                        __v
                            .parse()
                            .map_err(|_| mik_sdk::typed::ParseError::type_mismatch(
                                "search",
                                "string",
                            ))?,
                    );
                }
                _ => {}
            }
        }
        Ok(Self { page, limit, search })
    }
}
impl mik_sdk::typed::OpenApiSchema for ListParams {
    fn openapi_schema() -> &'static str {
        "{\"type\":\"object\",\"properties\":{\"page\":{\"type\":\"integer\"},\"limit\":{\"type\":\"integer\"},\"search\":{\"type\":\"string\"}}}"
    }
    fn schema_name() -> &'static str {
        "ListParams"
    }
    fn openapi_query_params() -> &'static str {
        "[{\"name\":\"page\",\"in\":\"query\",\"required\":false,\"schema\":{\"type\":\"integer\"}},{\"name\":\"limit\",\"in\":\"query\",\"required\":false,\"schema\":{\"type\":\"integer\"}},{\"name\":\"search\",\"in\":\"query\",\"required\":false,\"schema\":{\"type\":\"string\"}}]"
    }
}
fn main() {}

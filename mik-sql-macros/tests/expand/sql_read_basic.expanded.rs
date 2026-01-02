use mik_sql::sql_read;
fn main() {
    let (sql, params) = {
        let __sql_result = ::mik_sql::postgres("users")
            .fields(&["id", "name", "email"])
            .filter_expr(
                ::mik_sql::simple(
                    "active",
                    ::mik_sql::Operator::Eq,
                    ::mik_sql::Value::Bool(true),
                ),
            )
            .sort("name", ::mik_sql::SortDir::Asc)
            .limit_offset(10 as u32, 0)
            .build();
        (__sql_result.sql, __sql_result.params)
    };
    let _ = (sql, params);
}

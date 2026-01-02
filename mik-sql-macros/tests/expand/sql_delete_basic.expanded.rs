use mik_sql::sql_delete;
fn main() {
    let user_id = 123_i64;
    let (sql, params) = {
        let __result = ::mik_sql::delete("users")
            .filter_expr(
                ::mik_sql::simple(
                    "id",
                    ::mik_sql::Operator::Eq,
                    ::mik_sql::Value::Int(user_id as i64),
                ),
            )
            .build();
        (__result.sql, __result.params)
    };
    let _ = (sql, params);
}

use mik_sql::sql_create;
fn main() {
    let name = "Alice";
    let email = "alice@example.com";
    let (sql, params) = {
        let __result = ::mik_sql::insert("users")
            .columns(&["name", "email"])
            .values(
                <[_]>::into_vec(
                    ::alloc::boxed::box_new([
                        ::mik_sql::Value::String((name).to_string()),
                        ::mik_sql::Value::String((email).to_string()),
                    ]),
                ),
            )
            .returning(&["id"])
            .build();
        (__result.sql, __result.params)
    };
    let _ = (sql, params);
}

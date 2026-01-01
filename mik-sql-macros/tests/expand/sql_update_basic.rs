// Test sql_update! macro expansion
use mik_sql::sql_update;

fn main() {
    let user_id = 123_i64;
    let new_name = "Bob";

    let (sql, params) = sql_update!(users {
        set: { name: str(new_name) },
        filter: { id: int(user_id) },
    });
    let _ = (sql, params);
}

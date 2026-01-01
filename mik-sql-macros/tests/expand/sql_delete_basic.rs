// Test sql_delete! macro expansion
use mik_sql::sql_delete;

fn main() {
    let user_id = 123_i64;

    let (sql, params) = sql_delete!(users {
        filter: { id: int(user_id) },
    });
    let _ = (sql, params);
}

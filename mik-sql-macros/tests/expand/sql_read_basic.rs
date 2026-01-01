// Test sql_read! macro expansion
use mik_sql::sql_read;

fn main() {
    let (sql, params) = sql_read!(users {
        select: [id, name, email],
        filter: { active: true },
        order: name,
        limit: 10,
    });
    let _ = (sql, params);
}

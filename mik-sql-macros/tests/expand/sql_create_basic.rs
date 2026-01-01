// Test sql_create! macro expansion
use mik_sql::sql_create;

fn main() {
    let name = "Alice";
    let email = "alice@example.com";

    let (sql, params) = sql_create!(users {
        name: str(name),
        email: str(email),
        returning: [id],
    });
    let _ = (sql, params);
}

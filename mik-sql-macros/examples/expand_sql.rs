//! Example to test SQL macro expansion.

#![allow(dead_code, missing_docs)]

use mik_sql::{sql_create, sql_delete, sql_read, sql_update};

fn read_example() {
    let (sql, params) = sql_read!(users {
        select: [id, name, email],
        filter: { active: true, age: { $gte: 18 } },
        order: name,
        limit: 10,
    });
    println!("SQL: {sql}");
    println!("Params: {params:?}");
}

fn create_example() {
    let name = "Alice";
    let email = "alice@example.com";

    let (sql, params) = sql_create!(users {
        name: str(name),
        email: str(email),
        returning: [id],
    });
    println!("SQL: {sql}");
    println!("Params: {params:?}");
}

fn update_example() {
    let user_id = 123_i64;
    let new_name = "Bob";

    let (sql, params) = sql_update!(users {
        set: { name: str(new_name) },
        filter: { id: int(user_id) },
    });
    println!("SQL: {sql}");
    println!("Params: {params:?}");
}

fn delete_example() {
    let user_id = 123_i64;

    let (sql, params) = sql_delete!(users {
        filter: { id: int(user_id) },
    });
    println!("SQL: {sql}");
    println!("Params: {params:?}");
}

fn main() {
    read_example();
    create_example();
    update_example();
    delete_example();
    println!("\nRun with: cargo expand --example expand_sql -p mik-sql-macros");
}

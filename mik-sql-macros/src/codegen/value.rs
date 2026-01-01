//! Value and aggregate code generation for SQL CRUD macros.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::types::{
    SqlAggregate, SqlAggregateFunc, SqlComputeBinOp, SqlComputeExpr, SqlComputeFunc, SqlValue,
};

/// Convert a SQL value to tokens.
pub fn sql_value_to_tokens(value: &SqlValue) -> TokenStream2 {
    match value {
        SqlValue::Null => quote! { ::mik_sql::Value::Null },
        SqlValue::Bool(b) => quote! { ::mik_sql::Value::Bool(#b) },
        SqlValue::Int(i) => quote! { ::mik_sql::Value::Int(#i as i64) },
        SqlValue::Float(f) => quote! { ::mik_sql::Value::Float(#f as f64) },
        SqlValue::String(s) => quote! { ::mik_sql::Value::String(#s.to_string()) },
        SqlValue::Array(arr) => {
            let elements: Vec<_> = arr.iter().map(sql_value_to_tokens).collect();
            quote! { ::mik_sql::Value::Array(vec![#(#elements),*]) }
        },
        SqlValue::IntHint(e) => quote! { ::mik_sql::Value::Int(#e as i64) },
        SqlValue::StrHint(e) | SqlValue::Expr(e) => {
            quote! { ::mik_sql::Value::String((#e).to_string()) }
        },
        SqlValue::FloatHint(e) => quote! { ::mik_sql::Value::Float(#e as f64) },
        SqlValue::BoolHint(e) => quote! { ::mik_sql::Value::Bool(#e) },
    }
}

/// Convert an aggregate to tokens.
pub fn sql_aggregate_to_tokens(agg: &SqlAggregate) -> TokenStream2 {
    let field_str = agg.field.as_ref().map(std::string::ToString::to_string);
    let alias_str = agg.alias.as_ref().map(std::string::ToString::to_string);

    let base = match (&agg.func, &field_str) {
        (SqlAggregateFunc::Count, Some(f)) => quote! { ::mik_sql::Aggregate::count_field(#f) },
        (SqlAggregateFunc::CountDistinct, Some(f)) => {
            quote! { ::mik_sql::Aggregate::count_distinct(#f) }
        },
        (SqlAggregateFunc::Sum, Some(f)) => quote! { ::mik_sql::Aggregate::sum(#f) },
        (SqlAggregateFunc::Avg, Some(f)) => quote! { ::mik_sql::Aggregate::avg(#f) },
        (SqlAggregateFunc::Min, Some(f)) => quote! { ::mik_sql::Aggregate::min(#f) },
        (SqlAggregateFunc::Max, Some(f)) => quote! { ::mik_sql::Aggregate::max(#f) },
        // Count without field, or missing required field - default to count()
        _ => quote! { ::mik_sql::Aggregate::count() },
    };

    if let Some(alias) = alias_str {
        quote! { #base.as_alias(#alias) }
    } else {
        base
    }
}

/// Convert a compute expression to SQL string.
pub fn compute_expr_to_sql(expr: &SqlComputeExpr) -> String {
    match expr {
        SqlComputeExpr::Column(ident) => ident.to_string(),
        SqlComputeExpr::LitStr(lit) => {
            let s = lit.value();
            format!("'{}'", s.replace('\'', "''"))
        },
        SqlComputeExpr::LitInt(lit) => lit.to_string(),
        SqlComputeExpr::LitFloat(lit) => lit.to_string(),
        SqlComputeExpr::BinOp { left, op, right } => {
            let left_sql = compute_expr_to_sql(left);
            let right_sql = compute_expr_to_sql(right);
            let op_str = match op {
                SqlComputeBinOp::Add => "+",
                SqlComputeBinOp::Sub => "-",
                SqlComputeBinOp::Mul => "*",
                SqlComputeBinOp::Div => "/",
            };
            format!("{left_sql} {op_str} {right_sql}")
        },
        SqlComputeExpr::Func { name, args } => {
            let args_sql: Vec<String> = args.iter().map(compute_expr_to_sql).collect();
            match name {
                SqlComputeFunc::Concat => args_sql.join(" || "),
                SqlComputeFunc::Coalesce => format!("COALESCE({})", args_sql.join(", ")),
                SqlComputeFunc::Upper => format!("UPPER({})", args_sql.join(", ")),
                SqlComputeFunc::Lower => format!("LOWER({})", args_sql.join(", ")),
                SqlComputeFunc::Round => format!("ROUND({})", args_sql.join(", ")),
                SqlComputeFunc::Abs => format!("ABS({})", args_sql.join(", ")),
                SqlComputeFunc::Length => format!("LENGTH({})", args_sql.join(", ")),
            }
        },
        SqlComputeExpr::Paren(inner) => format!("({})", compute_expr_to_sql(inner)),
    }
}

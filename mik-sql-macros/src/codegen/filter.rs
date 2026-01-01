//! Filter code generation for SQL CRUD macros.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use super::value::sql_value_to_tokens;
use crate::types::{SqlFilterExpr, SqlLogicalOp, SqlOperator};

/// Convert a SQL operator to tokens.
pub fn sql_operator_to_tokens(op: &SqlOperator) -> TokenStream2 {
    match op {
        SqlOperator::Eq => quote! { ::mik_sql::Operator::Eq },
        SqlOperator::Ne => quote! { ::mik_sql::Operator::Ne },
        SqlOperator::Gt => quote! { ::mik_sql::Operator::Gt },
        SqlOperator::Gte => quote! { ::mik_sql::Operator::Gte },
        SqlOperator::Lt => quote! { ::mik_sql::Operator::Lt },
        SqlOperator::Lte => quote! { ::mik_sql::Operator::Lte },
        SqlOperator::In => quote! { ::mik_sql::Operator::In },
        SqlOperator::NotIn => quote! { ::mik_sql::Operator::NotIn },
        SqlOperator::Like => quote! { ::mik_sql::Operator::Like },
        SqlOperator::ILike => quote! { ::mik_sql::Operator::ILike },
        SqlOperator::Regex => quote! { ::mik_sql::Operator::Regex },
        SqlOperator::StartsWith => quote! { ::mik_sql::Operator::StartsWith },
        SqlOperator::EndsWith => quote! { ::mik_sql::Operator::EndsWith },
        SqlOperator::Contains => quote! { ::mik_sql::Operator::Contains },
        SqlOperator::Between => quote! { ::mik_sql::Operator::Between },
    }
}

/// Convert a filter expression to tokens.
pub fn sql_filter_expr_to_tokens(expr: &SqlFilterExpr) -> TokenStream2 {
    match expr {
        SqlFilterExpr::Simple(filter) => {
            let field_str = filter.field.to_string();
            let op = sql_operator_to_tokens(&filter.op);
            let value = sql_value_to_tokens(&filter.value);
            quote! { ::mik_sql::simple(#field_str, #op, #value) }
        },
        SqlFilterExpr::Compound { op, filters } => {
            let filter_tokens: Vec<TokenStream2> =
                filters.iter().map(sql_filter_expr_to_tokens).collect();

            match op {
                SqlLogicalOp::And => quote! { ::mik_sql::and(vec![#(#filter_tokens),*]) },
                SqlLogicalOp::Or => quote! { ::mik_sql::or(vec![#(#filter_tokens),*]) },
                SqlLogicalOp::Not => {
                    let inner = filter_tokens.into_iter().next().unwrap_or_default();
                    quote! { ::mik_sql::not(#inner) }
                },
            }
        },
    }
}

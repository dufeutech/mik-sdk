//! Code generation for `sql_read!` macro.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use super::input::SqlInput;
use crate::codegen::{compute_expr_to_sql, sql_aggregate_to_tokens, sql_filter_expr_to_tokens};
use crate::types::SqlSort;

/// Generate the token stream for a `sql_read!` invocation.
#[allow(clippy::too_many_lines)]
#[allow(clippy::cognitive_complexity)]
pub fn generate_sql_tokens(input: SqlInput) -> TokenStream2 {
    let SqlInput {
        dialect,
        table,
        select_fields,
        computed,
        aggregates,
        filter_expr,
        group_by,
        having,
        sorts,
        dynamic_sort,
        allow_sort,
        merge_filters,
        allow_fields,
        deny_ops,
        max_depth,
        page,
        limit,
        offset,
        after,
        before,
    } = input;

    let (sorts, dynamic_sort) = if let Some(ref expr) = dynamic_sort {
        if allow_sort.is_empty() {
            if let syn::Expr::Path(syn::ExprPath { path, .. }) = expr {
                if path.segments.len() == 1 && path.segments[0].arguments.is_empty() {
                    let field_name = path.segments[0].ident.clone();
                    let mut new_sorts = sorts;
                    new_sorts.push(SqlSort {
                        field: field_name,
                        desc: false,
                    });
                    (new_sorts, None)
                } else {
                    (sorts, dynamic_sort)
                }
            } else {
                (sorts, dynamic_sort)
            }
        } else {
            (sorts, dynamic_sort)
        }
    } else {
        (sorts, dynamic_sort)
    };

    let table_str = table.to_string();

    let fields_chain = if select_fields.is_empty() {
        quote! {}
    } else {
        let field_strs: Vec<String> = select_fields
            .iter()
            .map(std::string::ToString::to_string)
            .collect();
        quote! { .fields(&[#(#field_strs),*]) }
    };

    let computed_chain: Vec<TokenStream2> = computed
        .iter()
        .map(|c| {
            let alias = c.alias.to_string();
            let expr_sql = compute_expr_to_sql(&c.expr);
            quote! { .computed(#alias, #expr_sql) }
        })
        .collect();

    let aggregate_chain: Vec<TokenStream2> = aggregates
        .iter()
        .map(|agg| {
            let agg_tokens = sql_aggregate_to_tokens(agg);
            quote! { .aggregate(#agg_tokens) }
        })
        .collect();

    let filter_chain = filter_expr.map_or_else(
        || quote! {},
        |expr| {
            let expr_tokens = sql_filter_expr_to_tokens(&expr);
            quote! { .filter_expr(#expr_tokens) }
        },
    );

    let group_by_chain = if group_by.is_empty() {
        quote! {}
    } else {
        let field_strs: Vec<String> = group_by
            .iter()
            .map(std::string::ToString::to_string)
            .collect();
        quote! { .group_by(&[#(#field_strs),*]) }
    };

    let having_chain = having.map_or_else(
        || quote! {},
        |expr| {
            let expr_tokens = sql_filter_expr_to_tokens(&expr);
            quote! { .having(#expr_tokens) }
        },
    );

    let sort_chain: Vec<TokenStream2> = sorts
        .iter()
        .map(|s| {
            let field_str = s.field.to_string();
            let dir = if s.desc {
                quote! { ::mik_sql::SortDir::Desc }
            } else {
                quote! { ::mik_sql::SortDir::Asc }
            };
            quote! { .sort(#field_str, #dir) }
        })
        .collect();

    let dynamic_sort_setup = dynamic_sort.as_ref().map_or_else(
        || quote! {},
        |sort_expr| {
            let allow_strs: Vec<String> = allow_sort
                .iter()
                .map(std::string::ToString::to_string)
                .collect();
            if allow_strs.is_empty() {
                quote! {
                    let __dynamic_sorts = ::mik_sql::SortField::parse_sort_string(
                        &#sort_expr,
                        &[]
                    ).map_err(|e| e)?;
                }
            } else {
                quote! {
                    let __dynamic_sorts = ::mik_sql::SortField::parse_sort_string(
                        &#sort_expr,
                        &[#(#allow_strs),*]
                    ).map_err(|e| e)?;
                }
            }
        },
    );

    let dynamic_sort_chain = dynamic_sort
        .as_ref()
        .map_or_else(|| quote! {}, |_| quote! { .sorts(&__dynamic_sorts) });

    let (merge_setup, merge_chain) = merge_filters.as_ref().map_or_else(
        || (quote! {}, quote! {}),
        |merge_expr| {
            let allow_strs: Vec<String> = allow_fields
                .iter()
                .map(std::string::ToString::to_string)
                .collect();
            let deny_op_tokens: Vec<TokenStream2> = deny_ops
                .iter()
                .map(|op| {
                    let op_str = op.to_string();
                    match op_str.as_str() {
                        "ne" => quote! { ::mik_sql::Operator::Ne },
                        "gt" => quote! { ::mik_sql::Operator::Gt },
                        "gte" => quote! { ::mik_sql::Operator::Gte },
                        "lt" => quote! { ::mik_sql::Operator::Lt },
                        "lte" => quote! { ::mik_sql::Operator::Lte },
                        "in" => quote! { ::mik_sql::Operator::In },
                        "nin" | "notIn" => quote! { ::mik_sql::Operator::NotIn },
                        "like" => quote! { ::mik_sql::Operator::Like },
                        "ilike" => quote! { ::mik_sql::Operator::ILike },
                        "regex" => quote! { ::mik_sql::Operator::Regex },
                        "startsWith" | "starts_with" => quote! { ::mik_sql::Operator::StartsWith },
                        "endsWith" | "ends_with" => quote! { ::mik_sql::Operator::EndsWith },
                        "contains" => quote! { ::mik_sql::Operator::Contains },
                        "between" => quote! { ::mik_sql::Operator::Between },
                        // "eq" and unknown operators default to Eq
                        _ => quote! { ::mik_sql::Operator::Eq },
                    }
                })
                .collect();

            let max_depth_val = max_depth.map_or_else(|| quote! { 5 }, |d| quote! { #d as usize });

            let setup = quote! {
                let __validator = ::mik_sql::FilterValidator::new()
                    .allow_fields(&[#(#allow_strs),*])
                    .deny_operators(&[#(#deny_op_tokens),*])
                    .max_depth(#max_depth_val);

                for __user_filter in &#merge_expr {
                    __validator.validate(&__user_filter).map_err(|e| e.to_string())?;
                }
            };

            let chain = quote! {
                for __f in &#merge_expr {
                    __builder = __builder.filter(__f.field.clone(), __f.op, __f.value.clone());
                }
            };

            (setup, chain)
        },
    );

    let needs_result = dynamic_sort.is_some() || merge_filters.is_some();

    let pagination_chain = match (page, limit, offset) {
        (Some(p), Some(l), None) => quote! { .page(#p as u32, #l as u32) },
        (None, Some(l), Some(o)) => quote! { .limit_offset(#l as u32, #o as u32) },
        (None, Some(l), None) => quote! { .limit_offset(#l as u32, 0) },
        _ => quote! {},
    };

    let after_chain = after
        .as_ref()
        .map_or_else(|| quote! {}, |expr| quote! { .after_cursor(#expr) });

    let before_chain = before
        .as_ref()
        .map_or_else(|| quote! {}, |expr| quote! { .before_cursor(#expr) });

    let builder_constructor = dialect.builder_tokens(&table_str);

    if needs_result {
        quote! {
            (|| -> ::std::result::Result<(String, Vec<::mik_sql::Value>), String> {
                #dynamic_sort_setup
                #merge_setup

                let mut __builder = #builder_constructor
                    #fields_chain
                    #(#computed_chain)*
                    #(#aggregate_chain)*
                    #filter_chain;

                #merge_chain

                let __sql_result = __builder
                    #group_by_chain
                    #having_chain
                    #(#sort_chain)*
                    #dynamic_sort_chain
                    #after_chain
                    #before_chain
                    #pagination_chain
                    .build();

                Ok((__sql_result.sql, __sql_result.params))
            })()
        }
    } else {
        quote! {
            {
                let __sql_result = #builder_constructor
                    #fields_chain
                    #(#computed_chain)*
                    #(#aggregate_chain)*
                    #filter_chain
                    #group_by_chain
                    #having_chain
                    #(#sort_chain)*
                    #after_chain
                    #before_chain
                    #pagination_chain
                    .build();
                (__sql_result.sql, __sql_result.params)
            }
        }
    }
}

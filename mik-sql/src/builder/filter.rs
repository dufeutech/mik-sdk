//! Shared filter building functions.

use super::types::{CompoundFilter, Filter, FilterExpr, LogicalOp, Operator, Value};
use crate::dialect::Dialect;

/// Build a filter expression (simple or compound).
pub(super) fn build_filter_expr_impl<D: Dialect>(
    dialect: &D,
    expr: &FilterExpr,
    start_idx: usize,
) -> (String, Vec<Value>, usize) {
    match expr {
        FilterExpr::Simple(filter) => build_condition_impl(dialect, filter, start_idx),
        FilterExpr::Compound(compound) => build_compound_filter_impl(dialect, compound, start_idx),
    }
}

/// Build a compound filter (AND, OR, NOT).
pub(super) fn build_compound_filter_impl<D: Dialect>(
    dialect: &D,
    compound: &CompoundFilter,
    start_idx: usize,
) -> (String, Vec<Value>, usize) {
    let mut idx = start_idx;
    let mut all_params = Vec::new();
    let mut conditions = Vec::new();

    for filter_expr in &compound.filters {
        let (condition, params, new_idx) = build_filter_expr_impl(dialect, filter_expr, idx);
        conditions.push(condition);
        all_params.extend(params);
        idx = new_idx;
    }

    let sql = match compound.op {
        LogicalOp::And => {
            if conditions.len() == 1 {
                conditions.into_iter().next().unwrap()
            } else {
                format!("({})", conditions.join(" AND "))
            }
        },
        LogicalOp::Or => {
            if conditions.len() == 1 {
                conditions.into_iter().next().unwrap()
            } else {
                format!("({})", conditions.join(" OR "))
            }
        },
        LogicalOp::Not => {
            let inner = conditions.into_iter().next().unwrap_or_default();
            format!("NOT ({inner})")
        },
    };

    (sql, all_params, idx)
}

/// Build a single filter condition.
pub(super) fn build_condition_impl<D: Dialect>(
    dialect: &D,
    filter: &Filter,
    start_idx: usize,
) -> (String, Vec<Value>, usize) {
    let field = &filter.field;
    let idx = start_idx;

    match (&filter.op, &filter.value) {
        // NULL handling
        (Operator::Eq, Value::Null) => (format!("{field} IS NULL"), vec![], idx),
        (Operator::Ne, Value::Null) => (format!("{field} IS NOT NULL"), vec![], idx),

        // IN/NOT IN with arrays
        (Operator::In, Value::Array(values)) => {
            let (sql, params) = dialect.in_clause(field, values, idx);
            let new_idx = idx + params.len();
            (sql, params, new_idx)
        },
        (Operator::NotIn, Value::Array(values)) => {
            let (sql, params) = dialect.not_in_clause(field, values, idx);
            let new_idx = idx + params.len();
            (sql, params, new_idx)
        },

        // Boolean values - parameterized
        (Operator::Eq, Value::Bool(_)) => {
            let sql = format!("{} = {}", field, dialect.param(idx));
            (sql, vec![filter.value.clone()], idx + 1)
        },
        (Operator::Ne, Value::Bool(_)) => {
            let sql = format!("{} != {}", field, dialect.param(idx));
            (sql, vec![filter.value.clone()], idx + 1)
        },

        // Regex
        (Operator::Regex, value) => {
            let op = dialect.regex_op();
            let sql = format!("{} {} {}", field, op, dialect.param(idx));
            (sql, vec![value.clone()], idx + 1)
        },

        // ILIKE (falls back to LIKE on SQLite)
        (Operator::ILike, value) => {
            let sql = if dialect.supports_ilike() {
                format!("{} ILIKE {}", field, dialect.param(idx))
            } else {
                format!("{} LIKE {}", field, dialect.param(idx))
            };
            (sql, vec![value.clone()], idx + 1)
        },

        // String pattern operators
        (Operator::StartsWith, value) => {
            let sql = dialect.starts_with_clause(field, idx);
            (sql, vec![value.clone()], idx + 1)
        },
        (Operator::EndsWith, value) => {
            let sql = dialect.ends_with_clause(field, idx);
            (sql, vec![value.clone()], idx + 1)
        },
        (Operator::Contains, value) => {
            let sql = dialect.contains_clause(field, idx);
            (sql, vec![value.clone()], idx + 1)
        },

        // BETWEEN operator - takes an array with exactly 2 values
        (Operator::Between, Value::Array(values)) => {
            if values.len() != 2 {
                // Return a safe fallback that produces no results (consistent in debug and release)
                return (
                    format!("1=0 /* BETWEEN requires 2 values, got {} */", values.len()),
                    vec![],
                    idx,
                );
            }
            let sql = format!(
                "{} BETWEEN {} AND {}",
                field,
                dialect.param(idx),
                dialect.param(idx + 1)
            );
            (sql, values.clone(), idx + 2)
        },

        // Standard comparisons
        (op, value) => {
            let op_str = match op {
                Operator::Eq => "=",
                Operator::Ne => "!=",
                Operator::Gt => ">",
                Operator::Gte => ">=",
                Operator::Lt => "<",
                Operator::Lte => "<=",
                Operator::Like => "LIKE",
                _ => "=", // fallback for unhandled cases
            };
            let sql = format!("{} {} {}", field, op_str, dialect.param(idx));
            (sql, vec![value.clone()], idx + 1)
        },
    }
}

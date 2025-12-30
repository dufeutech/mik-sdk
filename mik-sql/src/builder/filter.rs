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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialect::{Postgres, Sqlite};

    #[test]
    fn test_eq_null_postgres() {
        let pg = Postgres;
        let filter = Filter {
            field: "deleted_at".to_string(),
            op: Operator::Eq,
            value: Value::Null,
        };
        let (sql, params, idx) = build_condition_impl(&pg, &filter, 1);
        assert_eq!(sql, "deleted_at IS NULL");
        assert!(params.is_empty());
        assert_eq!(idx, 1);
    }

    #[test]
    fn test_ne_null_postgres() {
        let pg = Postgres;
        let filter = Filter {
            field: "deleted_at".to_string(),
            op: Operator::Ne,
            value: Value::Null,
        };
        let (sql, params, idx) = build_condition_impl(&pg, &filter, 1);
        assert_eq!(sql, "deleted_at IS NOT NULL");
        assert!(params.is_empty());
        assert_eq!(idx, 1);
    }

    #[test]
    fn test_in_array_postgres() {
        let pg = Postgres;
        let filter = Filter {
            field: "status".to_string(),
            op: Operator::In,
            value: Value::Array(vec![
                Value::String("active".to_string()),
                Value::String("pending".to_string()),
            ]),
        };
        let (sql, params, idx) = build_condition_impl(&pg, &filter, 1);
        assert_eq!(sql, "status = ANY($1)");
        assert_eq!(params.len(), 1);
        assert_eq!(idx, 2);
    }

    #[test]
    fn test_in_array_sqlite() {
        let sqlite = Sqlite;
        let filter = Filter {
            field: "status".to_string(),
            op: Operator::In,
            value: Value::Array(vec![
                Value::String("active".to_string()),
                Value::String("pending".to_string()),
            ]),
        };
        let (sql, params, idx) = build_condition_impl(&sqlite, &filter, 1);
        assert_eq!(sql, "status IN (?1, ?2)");
        assert_eq!(params.len(), 2);
        assert_eq!(idx, 3);
    }

    #[test]
    fn test_not_in_array_postgres() {
        let pg = Postgres;
        let filter = Filter {
            field: "status".to_string(),
            op: Operator::NotIn,
            value: Value::Array(vec![Value::String("deleted".to_string())]),
        };
        let (sql, params, idx) = build_condition_impl(&pg, &filter, 1);
        assert_eq!(sql, "status != ALL($1)");
        assert_eq!(params.len(), 1);
        assert_eq!(idx, 2);
    }

    #[test]
    fn test_not_in_array_sqlite() {
        let sqlite = Sqlite;
        let filter = Filter {
            field: "status".to_string(),
            op: Operator::NotIn,
            value: Value::Array(vec![
                Value::String("deleted".to_string()),
                Value::String("archived".to_string()),
            ]),
        };
        let (sql, params, idx) = build_condition_impl(&sqlite, &filter, 1);
        assert_eq!(sql, "status NOT IN (?1, ?2)");
        assert_eq!(params.len(), 2);
        assert_eq!(idx, 3);
    }

    #[test]
    fn test_eq_bool_true_postgres() {
        let pg = Postgres;
        let filter = Filter {
            field: "active".to_string(),
            op: Operator::Eq,
            value: Value::Bool(true),
        };
        let (sql, params, idx) = build_condition_impl(&pg, &filter, 1);
        assert_eq!(sql, "active = $1");
        assert_eq!(params, vec![Value::Bool(true)]);
        assert_eq!(idx, 2);
    }

    #[test]
    fn test_ne_bool_sqlite() {
        let sqlite = Sqlite;
        let filter = Filter {
            field: "verified".to_string(),
            op: Operator::Ne,
            value: Value::Bool(false),
        };
        let (sql, params, idx) = build_condition_impl(&sqlite, &filter, 1);
        assert_eq!(sql, "verified != ?1");
        assert_eq!(params, vec![Value::Bool(false)]);
        assert_eq!(idx, 2);
    }

    #[test]
    fn test_regex_postgres() {
        let pg = Postgres;
        let filter = Filter {
            field: "email".to_string(),
            op: Operator::Regex,
            value: Value::String("^[a-z]+@".to_string()),
        };
        let (sql, params, idx) = build_condition_impl(&pg, &filter, 1);
        assert_eq!(sql, "email ~ $1");
        assert_eq!(params, vec![Value::String("^[a-z]+@".to_string())]);
        assert_eq!(idx, 2);
    }

    #[test]
    fn test_regex_sqlite() {
        let sqlite = Sqlite;
        let filter = Filter {
            field: "email".to_string(),
            op: Operator::Regex,
            value: Value::String("%@example%".to_string()),
        };
        let (sql, params, idx) = build_condition_impl(&sqlite, &filter, 1);
        assert_eq!(sql, "email LIKE ?1");
        assert_eq!(params, vec![Value::String("%@example%".to_string())]);
        assert_eq!(idx, 2);
    }

    #[test]
    fn test_ilike_postgres() {
        let pg = Postgres;
        let filter = Filter {
            field: "name".to_string(),
            op: Operator::ILike,
            value: Value::String("%john%".to_string()),
        };
        let (sql, params, idx) = build_condition_impl(&pg, &filter, 1);
        assert_eq!(sql, "name ILIKE $1");
        assert_eq!(params, vec![Value::String("%john%".to_string())]);
        assert_eq!(idx, 2);
    }

    #[test]
    fn test_ilike_sqlite() {
        let sqlite = Sqlite;
        let filter = Filter {
            field: "name".to_string(),
            op: Operator::ILike,
            value: Value::String("%john%".to_string()),
        };
        let (sql, params, idx) = build_condition_impl(&sqlite, &filter, 1);
        assert_eq!(sql, "name LIKE ?1");
        assert_eq!(params, vec![Value::String("%john%".to_string())]);
        assert_eq!(idx, 2);
    }

    #[test]
    fn test_starts_with_postgres() {
        let pg = Postgres;
        let filter = Filter {
            field: "name".to_string(),
            op: Operator::StartsWith,
            value: Value::String("John".to_string()),
        };
        let (sql, params, idx) = build_condition_impl(&pg, &filter, 1);
        assert!(sql.contains("LIKE"));
        assert_eq!(params, vec![Value::String("John".to_string())]);
        assert_eq!(idx, 2);
    }

    #[test]
    fn test_ends_with_postgres() {
        let pg = Postgres;
        let filter = Filter {
            field: "email".to_string(),
            op: Operator::EndsWith,
            value: Value::String("@example.com".to_string()),
        };
        let (sql, params, idx) = build_condition_impl(&pg, &filter, 1);
        assert!(sql.contains("LIKE"));
        assert_eq!(params, vec![Value::String("@example.com".to_string())]);
        assert_eq!(idx, 2);
    }

    #[test]
    fn test_contains_postgres() {
        let pg = Postgres;
        let filter = Filter {
            field: "description".to_string(),
            op: Operator::Contains,
            value: Value::String("rust".to_string()),
        };
        let (sql, params, idx) = build_condition_impl(&pg, &filter, 1);
        assert!(sql.contains("LIKE"));
        assert_eq!(params, vec![Value::String("rust".to_string())]);
        assert_eq!(idx, 2);
    }

    #[test]
    fn test_between_postgres() {
        let pg = Postgres;
        let filter = Filter {
            field: "age".to_string(),
            op: Operator::Between,
            value: Value::Array(vec![Value::Int(18), Value::Int(65)]),
        };
        let (sql, params, idx) = build_condition_impl(&pg, &filter, 1);
        assert_eq!(sql, "age BETWEEN $1 AND $2");
        assert_eq!(params, vec![Value::Int(18), Value::Int(65)]);
        assert_eq!(idx, 3);
    }

    #[test]
    fn test_between_invalid_single() {
        let pg = Postgres;
        let filter = Filter {
            field: "age".to_string(),
            op: Operator::Between,
            value: Value::Array(vec![Value::Int(18)]),
        };
        let (sql, params, idx) = build_condition_impl(&pg, &filter, 1);
        assert!(sql.contains("1=0"));
        assert!(params.is_empty());
        assert_eq!(idx, 1);
    }

    #[test]
    fn test_between_invalid_empty() {
        let pg = Postgres;
        let filter = Filter {
            field: "count".to_string(),
            op: Operator::Between,
            value: Value::Array(vec![]),
        };
        let (sql, params, idx) = build_condition_impl(&pg, &filter, 1);
        assert!(sql.contains("1=0"));
        assert!(params.is_empty());
        assert_eq!(idx, 1);
    }

    #[test]
    fn test_eq_int() {
        let pg = Postgres;
        let filter = Filter {
            field: "id".to_string(),
            op: Operator::Eq,
            value: Value::Int(42),
        };
        let (sql, params, idx) = build_condition_impl(&pg, &filter, 1);
        assert_eq!(sql, "id = $1");
        assert_eq!(params, vec![Value::Int(42)]);
        assert_eq!(idx, 2);
    }

    #[test]
    fn test_ne_string() {
        let sqlite = Sqlite;
        let filter = Filter {
            field: "status".to_string(),
            op: Operator::Ne,
            value: Value::String("deleted".to_string()),
        };
        let (sql, params, idx) = build_condition_impl(&sqlite, &filter, 1);
        assert_eq!(sql, "status != ?1");
        assert_eq!(params, vec![Value::String("deleted".to_string())]);
        assert_eq!(idx, 2);
    }

    #[test]
    fn test_gt_int() {
        let pg = Postgres;
        let filter = Filter {
            field: "age".to_string(),
            op: Operator::Gt,
            value: Value::Int(18),
        };
        let (sql, params, idx) = build_condition_impl(&pg, &filter, 1);
        assert_eq!(sql, "age > $1");
        assert_eq!(params, vec![Value::Int(18)]);
        assert_eq!(idx, 2);
    }

    #[test]
    fn test_gte_float() {
        let sqlite = Sqlite;
        let filter = Filter {
            field: "price".to_string(),
            op: Operator::Gte,
            value: Value::Float(9.99),
        };
        let (sql, params, idx) = build_condition_impl(&sqlite, &filter, 1);
        assert_eq!(sql, "price >= ?1");
        assert_eq!(params, vec![Value::Float(9.99)]);
        assert_eq!(idx, 2);
    }

    #[test]
    fn test_lt_int() {
        let pg = Postgres;
        let filter = Filter {
            field: "quantity".to_string(),
            op: Operator::Lt,
            value: Value::Int(100),
        };
        let (sql, params, idx) = build_condition_impl(&pg, &filter, 1);
        assert_eq!(sql, "quantity < $1");
        assert_eq!(params, vec![Value::Int(100)]);
        assert_eq!(idx, 2);
    }

    #[test]
    fn test_lte_float() {
        let sqlite = Sqlite;
        let filter = Filter {
            field: "discount".to_string(),
            op: Operator::Lte,
            value: Value::Float(0.5),
        };
        let (sql, params, idx) = build_condition_impl(&sqlite, &filter, 1);
        assert_eq!(sql, "discount <= ?1");
        assert_eq!(params, vec![Value::Float(0.5)]);
        assert_eq!(idx, 2);
    }

    #[test]
    fn test_like() {
        let pg = Postgres;
        let filter = Filter {
            field: "name".to_string(),
            op: Operator::Like,
            value: Value::String("%smith%".to_string()),
        };
        let (sql, params, idx) = build_condition_impl(&pg, &filter, 1);
        assert_eq!(sql, "name LIKE $1");
        assert_eq!(params, vec![Value::String("%smith%".to_string())]);
        assert_eq!(idx, 2);
    }

    #[test]
    fn test_compound_and_single() {
        let pg = Postgres;
        let compound = CompoundFilter {
            op: LogicalOp::And,
            filters: vec![FilterExpr::Simple(Filter {
                field: "active".to_string(),
                op: Operator::Eq,
                value: Value::Bool(true),
            })],
        };
        let (sql, params, idx) = build_compound_filter_impl(&pg, &compound, 1);
        assert_eq!(sql, "active = $1");
        assert_eq!(params, vec![Value::Bool(true)]);
        assert_eq!(idx, 2);
    }

    #[test]
    fn test_compound_and_multiple() {
        let pg = Postgres;
        let compound = CompoundFilter {
            op: LogicalOp::And,
            filters: vec![
                FilterExpr::Simple(Filter {
                    field: "active".to_string(),
                    op: Operator::Eq,
                    value: Value::Bool(true),
                }),
                FilterExpr::Simple(Filter {
                    field: "age".to_string(),
                    op: Operator::Gte,
                    value: Value::Int(18),
                }),
            ],
        };
        let (sql, params, idx) = build_compound_filter_impl(&pg, &compound, 1);
        assert_eq!(sql, "(active = $1 AND age >= $2)");
        assert_eq!(params, vec![Value::Bool(true), Value::Int(18)]);
        assert_eq!(idx, 3);
    }

    #[test]
    fn test_compound_or_single() {
        let sqlite = Sqlite;
        let compound = CompoundFilter {
            op: LogicalOp::Or,
            filters: vec![FilterExpr::Simple(Filter {
                field: "status".to_string(),
                op: Operator::Eq,
                value: Value::String("active".to_string()),
            })],
        };
        let (sql, params, idx) = build_compound_filter_impl(&sqlite, &compound, 1);
        assert_eq!(sql, "status = ?1");
        assert_eq!(params, vec![Value::String("active".to_string())]);
        assert_eq!(idx, 2);
    }

    #[test]
    fn test_compound_or_multiple() {
        let sqlite = Sqlite;
        let compound = CompoundFilter {
            op: LogicalOp::Or,
            filters: vec![
                FilterExpr::Simple(Filter {
                    field: "status".to_string(),
                    op: Operator::Eq,
                    value: Value::String("active".to_string()),
                }),
                FilterExpr::Simple(Filter {
                    field: "status".to_string(),
                    op: Operator::Eq,
                    value: Value::String("pending".to_string()),
                }),
            ],
        };
        let (sql, _params, idx) = build_compound_filter_impl(&sqlite, &compound, 1);
        assert_eq!(sql, "(status = ?1 OR status = ?2)");
        assert_eq!(idx, 3);
    }

    #[test]
    fn test_compound_not() {
        let pg = Postgres;
        let compound = CompoundFilter {
            op: LogicalOp::Not,
            filters: vec![FilterExpr::Simple(Filter {
                field: "deleted".to_string(),
                op: Operator::Eq,
                value: Value::Bool(true),
            })],
        };
        let (sql, params, idx) = build_compound_filter_impl(&pg, &compound, 1);
        assert_eq!(sql, "NOT (deleted = $1)");
        assert_eq!(params, vec![Value::Bool(true)]);
        assert_eq!(idx, 2);
    }

    #[test]
    fn test_compound_not_empty() {
        let pg = Postgres;
        let compound = CompoundFilter {
            op: LogicalOp::Not,
            filters: vec![],
        };
        let (sql, params, idx) = build_compound_filter_impl(&pg, &compound, 1);
        assert_eq!(sql, "NOT ()");
        assert!(params.is_empty());
        assert_eq!(idx, 1);
    }

    #[test]
    fn test_filter_expr_simple() {
        let pg = Postgres;
        let expr = FilterExpr::Simple(Filter {
            field: "name".to_string(),
            op: Operator::Eq,
            value: Value::String("test".to_string()),
        });
        let (sql, params, idx) = build_filter_expr_impl(&pg, &expr, 1);
        assert_eq!(sql, "name = $1");
        assert_eq!(params, vec![Value::String("test".to_string())]);
        assert_eq!(idx, 2);
    }

    #[test]
    fn test_filter_expr_compound() {
        let sqlite = Sqlite;
        let expr = FilterExpr::Compound(CompoundFilter {
            op: LogicalOp::Or,
            filters: vec![
                FilterExpr::Simple(Filter {
                    field: "a".to_string(),
                    op: Operator::Eq,
                    value: Value::Int(1),
                }),
                FilterExpr::Simple(Filter {
                    field: "b".to_string(),
                    op: Operator::Eq,
                    value: Value::Int(2),
                }),
            ],
        });
        let (sql, params, idx) = build_filter_expr_impl(&sqlite, &expr, 1);
        assert_eq!(sql, "(a = ?1 OR b = ?2)");
        assert_eq!(params, vec![Value::Int(1), Value::Int(2)]);
        assert_eq!(idx, 3);
    }

    #[test]
    fn test_compound_and_empty() {
        let pg = Postgres;
        let compound = CompoundFilter {
            op: LogicalOp::And,
            filters: vec![],
        };
        let (sql, params, idx) = build_compound_filter_impl(&pg, &compound, 1);
        assert_eq!(sql, "()");
        assert!(params.is_empty());
        assert_eq!(idx, 1);
    }

    #[test]
    fn test_compound_or_empty() {
        let sqlite = Sqlite;
        let compound = CompoundFilter {
            op: LogicalOp::Or,
            filters: vec![],
        };
        let (sql, params, idx) = build_compound_filter_impl(&sqlite, &compound, 1);
        assert_eq!(sql, "()");
        assert!(params.is_empty());
        assert_eq!(idx, 1);
    }

    #[test]
    fn test_fallback_operator() {
        let pg = Postgres;
        let filter = Filter {
            field: "test".to_string(),
            op: Operator::In,
            value: Value::Int(42),
        };
        let (sql, params, idx) = build_condition_impl(&pg, &filter, 1);
        assert_eq!(sql, "test = $1");
        assert_eq!(params, vec![Value::Int(42)]);
        assert_eq!(idx, 2);
    }

    #[test]
    fn test_between_non_array_fallback() {
        let pg = Postgres;
        let filter = Filter {
            field: "value".to_string(),
            op: Operator::Between,
            value: Value::Int(50),
        };
        let (sql, params, idx) = build_condition_impl(&pg, &filter, 1);
        assert_eq!(sql, "value = $1");
        assert_eq!(params, vec![Value::Int(50)]);
        assert_eq!(idx, 2);
    }
}

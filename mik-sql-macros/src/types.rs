//! Type definitions for SQL CRUD macros.
//!
//! This module contains all shared types used across the SQL macro implementations:
//! - `SqlDialect`: Postgres/SQLite dialect selection
//! - Filter types: `SqlFilterExpr`, `SqlFilter`, `SqlOperator`, `SqlValue`
//! - Aggregate types: `SqlAggregate`, `SqlAggregateFunc`
//! - Sort types: `SqlSort`
//! - Compute types: `SqlCompute`, `SqlComputeExpr`, `SqlComputeFunc`

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Expr, LitFloat, LitInt, LitStr, Result, Token};

// ============================================================================
// SQL Dialect
// ============================================================================

/// SQL dialect for query generation.
#[derive(Clone, Copy, Default)]
pub enum SqlDialect {
    #[default]
    Postgres,
    Sqlite,
}

impl SqlDialect {
    /// Parse dialect from identifier, returns None if not a dialect keyword.
    pub fn from_ident(ident: &syn::Ident) -> Option<Self> {
        match ident.to_string().as_str() {
            "postgres" | "pg" => Some(Self::Postgres),
            "sqlite" => Some(Self::Sqlite),
            _ => None,
        }
    }

    /// Generate the query builder constructor call.
    pub fn builder_tokens(self, table: &str) -> TokenStream2 {
        match self {
            Self::Postgres => quote! { ::mik_sql::postgres(#table) },
            Self::Sqlite => quote! { ::mik_sql::sqlite(#table) },
        }
    }

    /// Generate the insert builder constructor call.
    pub fn insert_tokens(self, table: &str) -> TokenStream2 {
        match self {
            Self::Postgres => quote! { ::mik_sql::insert(#table) },
            Self::Sqlite => quote! { ::mik_sql::insert_sqlite(#table) },
        }
    }

    /// Generate the update builder constructor call.
    pub fn update_tokens(self, table: &str) -> TokenStream2 {
        match self {
            Self::Postgres => quote! { ::mik_sql::update(#table) },
            Self::Sqlite => quote! { ::mik_sql::update_sqlite(#table) },
        }
    }

    /// Generate the delete builder constructor call.
    pub fn delete_tokens(self, table: &str) -> TokenStream2 {
        match self {
            Self::Postgres => quote! { ::mik_sql::delete(#table) },
            Self::Sqlite => quote! { ::mik_sql::delete_sqlite(#table) },
        }
    }
}

// ============================================================================
// Filter Types
// ============================================================================

/// A filter expression - can be simple or compound.
pub enum SqlFilterExpr {
    Simple(SqlFilter),
    Compound {
        op: SqlLogicalOp,
        filters: Vec<Self>,
    },
}

/// Logical operators for compound filters.
#[derive(Clone, Copy)]
pub enum SqlLogicalOp {
    And,
    Or,
    Not,
}

/// A filter condition in the sql! macro.
pub struct SqlFilter {
    pub field: syn::Ident,
    pub op: SqlOperator,
    pub value: SqlValue,
}

/// SQL operators.
#[derive(Clone)]
pub enum SqlOperator {
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
    In,
    NotIn,
    Like,
    ILike,
    Regex,
    StartsWith,
    EndsWith,
    Contains,
    Between,
}

/// Value in a filter.
pub enum SqlValue {
    Null,
    Bool(bool),
    Int(LitInt),
    Float(LitFloat),
    String(LitStr),
    Array(Vec<Self>),
    IntHint(Expr),
    StrHint(Expr),
    FloatHint(Expr),
    BoolHint(Expr),
    Expr(Expr),
}

// ============================================================================
// Aggregate Types
// ============================================================================

/// An aggregation in the sql! macro.
pub struct SqlAggregate {
    pub func: SqlAggregateFunc,
    pub field: Option<syn::Ident>,
    pub alias: Option<syn::Ident>,
}

/// Aggregation functions.
#[derive(Clone, Copy)]
pub enum SqlAggregateFunc {
    Count,
    CountDistinct,
    Sum,
    Avg,
    Min,
    Max,
}

// ============================================================================
// Sort Types
// ============================================================================

/// Sort field with direction.
pub struct SqlSort {
    pub field: syn::Ident,
    pub desc: bool,
}

impl syn::parse::Parse for SqlSort {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let desc = if input.peek(Token![-]) {
            input.parse::<Token![-]>()?;
            true
        } else {
            false
        };
        let field: syn::Ident = input.parse()?;
        Ok(Self { field, desc })
    }
}

// ============================================================================
// Compute Types
// ============================================================================

/// A computed field in the sql! macro.
pub struct SqlCompute {
    pub alias: syn::Ident,
    pub expr: SqlComputeExpr,
}

/// A compute expression (arithmetic, function call, or literal).
pub enum SqlComputeExpr {
    Column(syn::Ident),
    LitStr(LitStr),
    LitInt(LitInt),
    LitFloat(LitFloat),
    BinOp {
        left: Box<Self>,
        op: SqlComputeBinOp,
        right: Box<Self>,
    },
    Func {
        name: SqlComputeFunc,
        args: Vec<Self>,
    },
    Paren(Box<Self>),
}

/// Binary operators for compute expressions.
#[derive(Clone, Copy)]
pub enum SqlComputeBinOp {
    Add,
    Sub,
    Mul,
    Div,
}

/// Whitelisted compute functions.
#[derive(Clone, Copy)]
pub enum SqlComputeFunc {
    Concat,
    Coalesce,
    Upper,
    Lower,
    Round,
    Abs,
    Length,
}

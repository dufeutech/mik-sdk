//! SQL expression validation for computed fields.

/// Validate a SQL expression for computed fields.
///
/// Computed field expressions are dangerous because they're inserted directly
/// into SQL. This function performs defense-in-depth validation to catch
/// injection attempts, but **cannot provide complete protection**.
///
/// # Security Model
///
/// This validation is a safety net, not a security boundary. It catches:
/// - Obvious injection patterns (comments, semicolons, SQL keywords)
/// - Common attack vectors
///
/// It **cannot** catch:
/// - All possible SQL injection variants
/// - Database-specific syntax
/// - Encoded or obfuscated attacks
///
/// **CRITICAL**: Only use computed fields with trusted expressions from code.
/// Never pass user input to computed field expressions, even with validation.
///
/// # Valid expressions
///
/// - Simple field references: `first_name`, `price`
/// - Arithmetic: `quantity * price`
/// - String concatenation: `first_name || ' ' || last_name`
/// - Functions: `COALESCE(nickname, name)`, `UPPER(name)`
///
/// # Invalid expressions (rejected)
///
/// - Comments: `--`, `/*`, `*/`
/// - Statement terminators: `;`
/// - SQL keywords: SELECT, INSERT, UPDATE, DELETE, DROP, etc.
/// - System functions: `pg_`, `sqlite_`
///
/// # Examples
///
/// ```
/// use mik_sql::is_valid_sql_expression;
///
/// assert!(is_valid_sql_expression("first_name || ' ' || last_name"));
/// assert!(is_valid_sql_expression("quantity * price"));
/// assert!(is_valid_sql_expression("COALESCE(nickname, name)"));
///
/// // Dangerous patterns are rejected
/// assert!(!is_valid_sql_expression("1; DROP TABLE users"));
/// assert!(!is_valid_sql_expression("name -- comment"));
/// assert!(!is_valid_sql_expression("/* comment */ name"));
/// ```
#[inline]
#[must_use]
pub fn is_valid_sql_expression(s: &str) -> bool {
    // Empty or oversized expressions are invalid
    if s.is_empty() || s.len() > 1000 {
        return false;
    }

    // No SQL comments
    if s.contains("--") || s.contains("/*") || s.contains("*/") {
        return false;
    }

    // No statement terminators
    if s.contains(';') {
        return false;
    }

    // No backticks (MySQL identifier quotes that could be used for injection)
    if s.contains('`') {
        return false;
    }

    // Check for dangerous SQL keywords using word boundary detection
    let lower = s.to_ascii_lowercase();

    // Dangerous DML/DDL keywords and functions
    const DANGEROUS_KEYWORDS: &[&str] = &[
        // DML/DDL statements
        "select",
        "insert",
        "update",
        "delete",
        "drop",
        "truncate",
        "alter",
        "create",
        "grant",
        "revoke",
        "exec",
        "execute",
        "union",
        "into",
        "from",
        "where",
        "having",
        "group",
        "order",
        "limit",
        "offset",
        "fetch",
        "returning",
        // Dangerous functions (timing attacks, DoS)
        "sleep",
        "benchmark",
        "waitfor",
        "pg_sleep",
        "dbms_lock",
        // File/network operations
        "load_file",
        "into_outfile",
        "into_dumpfile",
        // Encoding/conversion functions that could bypass keyword detection
        "chr",
        "char",
        "ascii",
        "unicode",
        "hex",
        "unhex",
        "convert",
        "cast",
        "encode",
        "decode",
    ];

    for keyword in DANGEROUS_KEYWORDS {
        if contains_sql_keyword(&lower, keyword) {
            return false;
        }
    }

    // Block system catalog access patterns
    if lower.contains("pg_")
        || lower.contains("sqlite_")
        || lower.contains("information_schema")
        || lower.contains("sys.")
    {
        return false;
    }

    // Block hex escapes that could bypass other checks
    if lower.contains("0x") || lower.contains("\\x") {
        return false;
    }

    true
}

/// Check if a string contains a SQL keyword as a whole word.
///
/// This prevents false positives like "update" in "`last_updated`".
#[inline]
fn contains_sql_keyword(haystack: &str, keyword: &str) -> bool {
    let bytes = haystack.as_bytes();
    let kw_bytes = keyword.as_bytes();
    let kw_len = kw_bytes.len();

    if kw_len == 0 || bytes.len() < kw_len {
        return false;
    }

    for i in 0..=(bytes.len() - kw_len) {
        // Check if keyword matches at this position
        if &bytes[i..i + kw_len] == kw_bytes {
            // Check word boundaries (parentheses fix operator precedence: && binds tighter than ||)
            let before_ok =
                i == 0 || (!bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_');
            let after_ok = i + kw_len == bytes.len()
                || (!bytes[i + kw_len].is_ascii_alphanumeric() && bytes[i + kw_len] != b'_');

            if before_ok && after_ok {
                return true;
            }
        }
    }

    false
}

/// Assert that a SQL expression is valid for computed fields.
///
/// # Panics
///
/// Panics if the expression contains dangerous patterns.
#[inline]
pub fn assert_valid_sql_expression(s: &str, context: &str) {
    assert!(
        is_valid_sql_expression(s),
        "Invalid SQL expression for {context}: '{s}' contains dangerous patterns \
             (comments, semicolons, or SQL keywords)"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_sql_expressions() {
        // Valid expressions
        assert!(is_valid_sql_expression("first_name || ' ' || last_name"));
        assert!(is_valid_sql_expression("quantity * price"));
        assert!(is_valid_sql_expression("COALESCE(nickname, name)"));
        assert!(is_valid_sql_expression("age + 1"));
        assert!(is_valid_sql_expression("CASE WHEN x > 0 THEN y ELSE z END"));
        assert!(is_valid_sql_expression("price * 1.1"));
        assert!(is_valid_sql_expression("UPPER(name)"));
        assert!(is_valid_sql_expression("LENGTH(description)"));

        // Word boundary detection - these contain keywords as substrings but should be allowed
        assert!(is_valid_sql_expression("last_updated")); // contains "update"
        assert!(is_valid_sql_expression("created_at")); // contains "create"
        assert!(is_valid_sql_expression("selected_items")); // contains "select"
        assert!(is_valid_sql_expression("deleted_at")); // contains "delete"
        assert!(is_valid_sql_expression("order_total")); // contains "order"
        assert!(is_valid_sql_expression("group_name")); // contains "group"
        assert!(is_valid_sql_expression("from_date")); // contains "from"
        assert!(is_valid_sql_expression("where_clause")); // contains "where"
    }

    #[test]
    fn test_invalid_sql_expressions() {
        // Empty
        assert!(!is_valid_sql_expression(""));

        // SQL comments
        assert!(!is_valid_sql_expression("name -- comment"));
        assert!(!is_valid_sql_expression("/* comment */ name"));
        assert!(!is_valid_sql_expression("name */ attack"));

        // Statement terminators
        assert!(!is_valid_sql_expression("1; DROP TABLE users"));
        assert!(!is_valid_sql_expression("name;"));

        // Backticks
        assert!(!is_valid_sql_expression("`table`"));

        // SQL keywords as standalone words
        assert!(!is_valid_sql_expression("(SELECT password)"));
        assert!(!is_valid_sql_expression("INSERT INTO x"));
        assert!(!is_valid_sql_expression("DELETE FROM x"));
        assert!(!is_valid_sql_expression("DROP TABLE x"));
        assert!(!is_valid_sql_expression("UPDATE SET y=1"));
        assert!(!is_valid_sql_expression("UNION ALL"));
        assert!(!is_valid_sql_expression("x FROM y"));
        assert!(!is_valid_sql_expression("x WHERE y"));

        // System catalog access
        assert!(!is_valid_sql_expression("pg_catalog.pg_tables"));
        assert!(!is_valid_sql_expression("sqlite_master"));
        assert!(!is_valid_sql_expression("information_schema.tables"));

        // Hex escapes
        assert!(!is_valid_sql_expression("0x48454C4C4F"));
        assert!(!is_valid_sql_expression("\\x48454C4C4F"));

        // Dangerous functions (timing attacks, DoS)
        assert!(!is_valid_sql_expression("SLEEP(10)"));
        assert!(!is_valid_sql_expression("pg_sleep(5)"));
        assert!(!is_valid_sql_expression("BENCHMARK(1000000, SHA1('test'))"));
        assert!(!is_valid_sql_expression("WAITFOR DELAY '0:0:5'"));

        // File operations
        assert!(!is_valid_sql_expression("LOAD_FILE('/etc/passwd')"));
    }

    #[test]
    #[should_panic(expected = "Invalid SQL expression")]
    fn test_assert_valid_expression_panics() {
        assert_valid_sql_expression("1; DROP TABLE users", "computed field");
    }

    // =========================================================================
    // SQL INJECTION FUZZING TESTS
    // =========================================================================

    #[test]
    fn test_sqli_classic_or_true() {
        // Classic OR-based injection - these are blocked by comment/semicolon detection
        assert!(!is_valid_sql_expression("' OR 1=1--")); // Blocked by --
        assert!(!is_valid_sql_expression("1; OR 1=1")); // Blocked by ;
    }

    #[test]
    fn test_sqli_drop_table() {
        // DROP TABLE attacks
        assert!(!is_valid_sql_expression("'; DROP TABLE users--"));
        assert!(!is_valid_sql_expression("'; DROP TABLE users;--"));
        assert!(!is_valid_sql_expression("1; DROP TABLE users"));
        assert!(!is_valid_sql_expression("DROP TABLE users"));
        assert!(!is_valid_sql_expression("drop table users"));
        assert!(!is_valid_sql_expression("DrOp TaBlE users"));
    }

    #[test]
    fn test_sqli_union_attacks() {
        // UNION-based injection
        assert!(!is_valid_sql_expression("' UNION SELECT * FROM users--"));
        assert!(!is_valid_sql_expression(
            "' UNION ALL SELECT password FROM users--"
        ));
        assert!(!is_valid_sql_expression("1 UNION SELECT 1,2,3"));
        assert!(!is_valid_sql_expression(
            "UNION SELECT username,password FROM admin"
        ));
        assert!(!is_valid_sql_expression("' union select null,null,null--"));
    }

    #[test]
    fn test_sqli_comment_injection() {
        // Comment-based attacks - blocked by comment detection
        assert!(!is_valid_sql_expression("admin'--")); // SQL comment
        assert!(!is_valid_sql_expression("admin'/*")); // Block comment start
        assert!(!is_valid_sql_expression("*/; DROP TABLE users--")); // Block comment end + semicolon
        assert!(!is_valid_sql_expression("1/**/OR/**/1=1")); // Block comments
    }

    #[test]
    fn test_sqli_stacked_queries() {
        // Stacked query attacks (semicolon-based)
        assert!(!is_valid_sql_expression(
            "; INSERT INTO users VALUES('hacker')"
        ));
        assert!(!is_valid_sql_expression("; UPDATE users SET role='admin'"));
        assert!(!is_valid_sql_expression("; DELETE FROM users"));
        assert!(!is_valid_sql_expression("1; SELECT * FROM passwords"));
        assert!(!is_valid_sql_expression("'; TRUNCATE TABLE logs;--"));
    }

    #[test]
    fn test_sqli_time_based_blind() {
        // Time-based blind injection
        assert!(!is_valid_sql_expression("SLEEP(5)"));
        assert!(!is_valid_sql_expression("1 AND SLEEP(5)"));
        assert!(!is_valid_sql_expression("pg_sleep(5)"));
        assert!(!is_valid_sql_expression("1; SELECT pg_sleep(10)"));
        assert!(!is_valid_sql_expression("BENCHMARK(10000000,SHA1('test'))"));
        assert!(!is_valid_sql_expression("WAITFOR DELAY '0:0:5'"));
        assert!(!is_valid_sql_expression("dbms_lock.sleep(5)"));
    }

    #[test]
    fn test_sqli_file_operations() {
        // File read/write attacks
        assert!(!is_valid_sql_expression("LOAD_FILE('/etc/passwd')"));
        assert!(!is_valid_sql_expression("load_file('/etc/shadow')"));
        assert!(!is_valid_sql_expression(
            "INTO OUTFILE '/var/www/shell.php'"
        ));
        assert!(!is_valid_sql_expression("INTO DUMPFILE '/tmp/data'"));
        assert!(!is_valid_sql_expression("into_outfile('/tmp/x')"));
        assert!(!is_valid_sql_expression("into_dumpfile('/tmp/x')"));
    }

    #[test]
    fn test_sqli_system_catalog_access() {
        // System catalog enumeration
        assert!(!is_valid_sql_expression("pg_tables"));
        assert!(!is_valid_sql_expression("pg_catalog.pg_tables"));
        assert!(!is_valid_sql_expression("sqlite_master"));
        assert!(!is_valid_sql_expression("information_schema.tables"));
        assert!(!is_valid_sql_expression("sys.tables"));
        assert!(!is_valid_sql_expression("SELECT FROM information_schema"));
    }

    #[test]
    fn test_sqli_hex_encoding() {
        // Hex-encoded attacks
        assert!(!is_valid_sql_expression("0x27")); // Single quote
        assert!(!is_valid_sql_expression("0x4F5220313D31")); // OR 1=1
        assert!(!is_valid_sql_expression("\\x27"));
        assert!(!is_valid_sql_expression("CHAR(0x27)"));
    }

    #[test]
    fn test_sqli_keyword_boundary_detection() {
        // These SHOULD be allowed - keywords as substrings of identifiers
        assert!(is_valid_sql_expression("order_id")); // order
        assert!(is_valid_sql_expression("reorder_count")); // order
        assert!(is_valid_sql_expression("group_name")); // group
        assert!(is_valid_sql_expression("ungroup")); // group
        assert!(is_valid_sql_expression("from_date")); // from
        assert!(is_valid_sql_expression("wherefrom")); // where, from
        assert!(is_valid_sql_expression("selected_items")); // select
        assert!(is_valid_sql_expression("preselect")); // select
        assert!(is_valid_sql_expression("delete_flag")); // delete
        assert!(is_valid_sql_expression("undelete")); // delete
        assert!(is_valid_sql_expression("update_time")); // update
        assert!(is_valid_sql_expression("last_updated")); // update

        // These SHOULD be blocked - standalone keywords
        assert!(!is_valid_sql_expression("ORDER BY name"));
        assert!(!is_valid_sql_expression("GROUP BY id"));
        assert!(!is_valid_sql_expression("FROM users"));
        assert!(!is_valid_sql_expression("WHERE id=1"));
        assert!(!is_valid_sql_expression("SELECT *"));
        assert!(!is_valid_sql_expression("DELETE FROM"));
        assert!(!is_valid_sql_expression("UPDATE SET"));
    }

    #[test]
    fn test_sqli_case_variations() {
        // Case variations of dangerous keywords
        assert!(!is_valid_sql_expression("SELECT"));
        assert!(!is_valid_sql_expression("select"));
        assert!(!is_valid_sql_expression("SeLeCt"));
        assert!(!is_valid_sql_expression("sElEcT"));

        assert!(!is_valid_sql_expression("UNION"));
        assert!(!is_valid_sql_expression("union"));
        assert!(!is_valid_sql_expression("UnIoN"));

        assert!(!is_valid_sql_expression("DROP"));
        assert!(!is_valid_sql_expression("drop"));
        assert!(!is_valid_sql_expression("DrOp"));
    }

    #[test]
    fn test_sqli_whitespace_variations() {
        // Whitespace-based evasion - these should still be caught
        assert!(!is_valid_sql_expression("SELECT\t*"));
        assert!(!is_valid_sql_expression("SELECT\n*"));
        assert!(!is_valid_sql_expression("  SELECT  "));
        assert!(!is_valid_sql_expression("DROP\t\tTABLE"));
    }

    #[test]
    fn test_sqli_expression_length_limit() {
        // Very long expressions should be rejected
        let long_expr = "a".repeat(1001);
        assert!(!is_valid_sql_expression(&long_expr));

        // At limit should be OK
        let at_limit = "a".repeat(1000);
        assert!(is_valid_sql_expression(&at_limit));
    }

    #[test]
    fn test_valid_safe_expressions() {
        // Legitimate expressions that should be allowed
        assert!(is_valid_sql_expression("first_name || ' ' || last_name"));
        assert!(is_valid_sql_expression("price * quantity"));
        assert!(is_valid_sql_expression("price * 1.15")); // With tax
        assert!(is_valid_sql_expression(
            "COALESCE(nickname, first_name, 'Anonymous')"
        ));
        assert!(is_valid_sql_expression("UPPER(TRIM(name))"));
        assert!(is_valid_sql_expression("LENGTH(description)"));
        assert!(is_valid_sql_expression("ABS(balance)"));
        assert!(is_valid_sql_expression("ROUND(price, 2)"));
        assert!(is_valid_sql_expression("LOWER(email)"));
        assert!(is_valid_sql_expression("created_at + INTERVAL '1 day'"));
        assert!(is_valid_sql_expression("age >= 18"));
        assert!(is_valid_sql_expression("status = 'active'"));
        assert!(is_valid_sql_expression("NOT is_deleted"));
        assert!(is_valid_sql_expression("(price > 0) AND (quantity > 0)"));
    }
}

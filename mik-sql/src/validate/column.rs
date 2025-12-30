//! Column/identifier validation logic for SQL injection prevention.

/// Maximum length for SQL identifiers (`PostgreSQL` limit is 63).
const MAX_IDENTIFIER_LENGTH: usize = 63;

/// Validate that a string is a safe SQL identifier.
///
/// A valid SQL identifier:
/// - Starts with a letter (a-z, A-Z) or underscore
/// - Contains only letters, digits (0-9), and underscores
/// - Is not empty and not longer than 63 characters
///
/// This prevents SQL injection attacks by rejecting:
/// - Special characters (quotes, semicolons, etc.)
/// - SQL keywords as standalone identifiers
/// - Unicode characters that could cause confusion
///
/// # Examples
///
/// ```
/// use mik_sql::is_valid_sql_identifier;
///
/// assert!(is_valid_sql_identifier("users"));
/// assert!(is_valid_sql_identifier("user_id"));
/// assert!(is_valid_sql_identifier("_private"));
/// assert!(is_valid_sql_identifier("Table123"));
///
/// // Invalid identifiers
/// assert!(!is_valid_sql_identifier(""));           // empty
/// assert!(!is_valid_sql_identifier("123abc"));     // starts with digit
/// assert!(!is_valid_sql_identifier("user-name"));  // contains hyphen
/// assert!(!is_valid_sql_identifier("user.id"));    // contains dot
/// assert!(!is_valid_sql_identifier("user; DROP")); // contains special chars
/// ```
#[inline]
#[must_use]
pub fn is_valid_sql_identifier(s: &str) -> bool {
    if s.is_empty() || s.len() > MAX_IDENTIFIER_LENGTH {
        return false;
    }

    let mut chars = s.chars();

    // First character must be letter or underscore
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {},
        _ => return false,
    }

    // Rest must be letters, digits, or underscores
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Assert that a string is a valid SQL identifier.
///
/// # Panics
///
/// Panics with a descriptive error if the identifier is invalid.
/// This is intended for programmer errors (invalid table/column names in code),
/// not for user input validation.
///
/// # Examples
///
/// ```
/// use mik_sql::assert_valid_sql_identifier;
///
/// assert_valid_sql_identifier("users", "table");    // OK
/// assert_valid_sql_identifier("user_id", "column"); // OK
/// ```
///
/// ```should_panic
/// use mik_sql::assert_valid_sql_identifier;
///
/// assert_valid_sql_identifier("user; DROP TABLE", "table"); // Panics!
/// ```
#[inline]
pub fn assert_valid_sql_identifier(s: &str, context: &str) {
    assert!(
        is_valid_sql_identifier(s),
        "Invalid SQL {context} name '{s}': must start with letter/underscore, \
             contain only ASCII alphanumeric/underscore, and be 1-63 chars"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_sql_identifiers() {
        // Valid identifiers
        assert!(is_valid_sql_identifier("users"));
        assert!(is_valid_sql_identifier("user_id"));
        assert!(is_valid_sql_identifier("_private"));
        assert!(is_valid_sql_identifier("Table123"));
        assert!(is_valid_sql_identifier("a"));
        assert!(is_valid_sql_identifier("_"));
        assert!(is_valid_sql_identifier("UPPERCASE"));
        assert!(is_valid_sql_identifier("mixedCase"));
        assert!(is_valid_sql_identifier("with_123_numbers"));
    }

    #[test]
    fn test_invalid_sql_identifiers() {
        // Empty
        assert!(!is_valid_sql_identifier(""));

        // Starts with digit
        assert!(!is_valid_sql_identifier("123abc"));
        assert!(!is_valid_sql_identifier("1"));

        // Contains special characters
        assert!(!is_valid_sql_identifier("user-name"));
        assert!(!is_valid_sql_identifier("user.id"));
        assert!(!is_valid_sql_identifier("user name"));
        assert!(!is_valid_sql_identifier("user;drop"));
        assert!(!is_valid_sql_identifier("table'"));
        assert!(!is_valid_sql_identifier("table\""));
        assert!(!is_valid_sql_identifier("table`"));
        assert!(!is_valid_sql_identifier("table("));
        assert!(!is_valid_sql_identifier("table)"));

        // SQL injection attempts
        assert!(!is_valid_sql_identifier("users; DROP TABLE"));
        assert!(!is_valid_sql_identifier("users--"));
        assert!(!is_valid_sql_identifier("users/*"));
    }

    #[test]
    fn test_sql_identifier_length_limit() {
        // 63 chars = OK (PostgreSQL limit)
        let valid_63 = "a".repeat(63);
        assert!(is_valid_sql_identifier(&valid_63));

        // 64 chars = too long
        let invalid_64 = "a".repeat(64);
        assert!(!is_valid_sql_identifier(&invalid_64));
    }

    #[test]
    fn test_identifier_injection_attempts() {
        // SQL injection via identifier names
        assert!(!is_valid_sql_identifier("users; DROP TABLE x"));
        assert!(!is_valid_sql_identifier("users--"));
        assert!(!is_valid_sql_identifier("users/*comment*/"));
        assert!(!is_valid_sql_identifier("users'"));
        assert!(!is_valid_sql_identifier("users\""));
        assert!(!is_valid_sql_identifier("users`"));
        assert!(!is_valid_sql_identifier("users;"));
        assert!(!is_valid_sql_identifier("(SELECT 1)"));
        assert!(!is_valid_sql_identifier("1 OR 1=1"));

        // Unicode injection attempts
        assert!(!is_valid_sql_identifier("users\u{0000}")); // Null byte
        assert!(!is_valid_sql_identifier("users\u{200B}")); // Zero-width space
        assert!(!is_valid_sql_identifier("usërs")); // Non-ASCII letter
        assert!(!is_valid_sql_identifier("用户")); // Chinese characters

        // Fullwidth characters (potential bypass)
        assert!(!is_valid_sql_identifier("ｕｓｅｒｓ")); // Fullwidth letters
    }

    #[test]
    #[should_panic(expected = "Invalid SQL table name")]
    fn test_assert_valid_identifier_panics() {
        assert_valid_sql_identifier("users; DROP TABLE", "table");
    }
}

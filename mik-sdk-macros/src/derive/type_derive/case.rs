//! PascalCase to snake_case conversion utility.

/// Convert PascalCase to snake_case.
///
/// Examples:
/// - `Active` → `active`
/// - `SuperAdmin` → `super_admin`
/// - `HTTPRequest` → `http_request`
pub fn pascal_to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    let mut prev_was_upper = false;
    let mut prev_was_underscore = true; // Start as true to avoid leading underscore

    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            // Add underscore before uppercase if:
            // - Not at start
            // - Previous char wasn't uppercase (handles "HTTPRequest" → "http_request")
            // - OR next char is lowercase (handles "XMLParser" → "xml_parser")
            let next_is_lower = s.chars().nth(i + 1).is_some_and(char::is_lowercase);
            if !prev_was_underscore && (!prev_was_upper || next_is_lower) {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
            prev_was_upper = true;
            prev_was_underscore = false;
        } else if c == '_' {
            result.push(c);
            prev_was_upper = false;
            prev_was_underscore = true;
        } else {
            result.push(c);
            prev_was_upper = false;
            prev_was_underscore = false;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::pascal_to_snake_case;

    #[test]
    fn test_pascal_to_snake_case_simple() {
        assert_eq!(pascal_to_snake_case("Active"), "active");
        assert_eq!(pascal_to_snake_case("Inactive"), "inactive");
        assert_eq!(pascal_to_snake_case("Pending"), "pending");
    }

    #[test]
    fn test_pascal_to_snake_case_multi_word() {
        assert_eq!(pascal_to_snake_case("SuperAdmin"), "super_admin");
        assert_eq!(pascal_to_snake_case("RegularUser"), "regular_user");
        assert_eq!(pascal_to_snake_case("GuestUser"), "guest_user");
    }

    #[test]
    fn test_pascal_to_snake_case_acronyms() {
        assert_eq!(pascal_to_snake_case("HTTPRequest"), "http_request");
        assert_eq!(pascal_to_snake_case("XMLParser"), "xml_parser");
        assert_eq!(pascal_to_snake_case("APIResponse"), "api_response");
    }

    #[test]
    fn test_pascal_to_snake_case_single_letter() {
        assert_eq!(pascal_to_snake_case("A"), "a");
        assert_eq!(pascal_to_snake_case("AB"), "ab");
    }

    #[test]
    fn test_pascal_to_snake_case_already_lower() {
        assert_eq!(pascal_to_snake_case("active"), "active");
        assert_eq!(pascal_to_snake_case("already_snake"), "already_snake");
    }

    #[test]
    fn test_pascal_to_snake_case_numbers() {
        assert_eq!(pascal_to_snake_case("Status2"), "status2");
        assert_eq!(pascal_to_snake_case("OAuth2Token"), "o_auth2_token");
    }
}

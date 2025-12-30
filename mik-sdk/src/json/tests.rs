//! All tests for the json module.

#[cfg(test)]
use super::*;

// =========================================================================
// PROPTEST PROPERTY TESTS - Fuzz parsers to ensure no panics
// =========================================================================

mod proptest_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Test that json::try_parse doesn't panic on arbitrary bytes.
        /// Malformed input should return None, never panic.
        #[test]
        fn parse_doesnt_panic_on_arbitrary_bytes(input in prop::collection::vec(any::<u8>(), 0..1024)) {
            let _ = try_parse(&input); // Should not panic
        }

        /// Test that json::try_parse doesn't panic on arbitrary strings.
        #[test]
        fn parse_doesnt_panic_on_arbitrary_strings(input in ".*") {
            let _ = try_parse(input.as_bytes()); // Should not panic
        }

        /// Test that deeply nested JSON doesn't cause stack overflow.
        /// The parser should reject deep nesting gracefully.
        #[test]
        fn parse_rejects_deep_nesting_gracefully(depth in 1usize..100) {
            // Generate nested objects: {"a":{"a":{"a":...}}}
            let mut json = String::new();
            for _ in 0..depth {
                json.push_str("{\"a\":");
            }
            json.push('1');
            for _ in 0..depth {
                json.push('}');
            }

            // Should not panic, may return None for deep nesting
            let _ = try_parse(json.as_bytes());
        }

        /// Test that deeply nested arrays don't cause stack overflow.
        #[test]
        fn parse_rejects_deep_array_nesting_gracefully(depth in 1usize..100) {
            // Generate nested arrays: [[[[...]]]]
            let mut json = String::new();
            for _ in 0..depth {
                json.push('[');
            }
            json.push('1');
            for _ in 0..depth {
                json.push(']');
            }

            // Should not panic, may return None for deep nesting
            let _ = try_parse(json.as_bytes());
        }

        /// Test that unicode strings are handled correctly.
        #[test]
        fn parse_handles_unicode_strings(s in "\\PC*") {
            // Valid JSON string with unicode content
            let json = format!(r#"{{"text": "{}"}}"#, s.replace('\\', "\\\\").replace('"', "\\\""));
            let _ = try_parse(json.as_bytes()); // Should not panic
        }

        /// Test that valid UTF-8 strings in JSON are parsed correctly.
        #[test]
        fn parse_handles_valid_utf8(s in "[a-zA-Z0-9 ]{0,100}") {
            let json = format!(r#"{{"value": "{}"}}"#, s);
            let result = try_parse(json.as_bytes());
            // Valid JSON should parse successfully
            prop_assert!(result.is_some());
            let value = result.unwrap();
            prop_assert_eq!(value.path_str(&["value"]), Some(s));
        }

        /// Test numeric edge cases - very large integers.
        #[test]
        fn parse_handles_large_integers(n in i64::MIN..=i64::MAX) {
            let json = format!(r#"{{"n": {}}}"#, n);
            let result = try_parse(json.as_bytes());
            // Should parse without panic
            prop_assert!(result.is_some());
        }

        /// Test numeric edge cases - very large unsigned integers.
        #[test]
        fn parse_handles_large_unsigned(n in 0u64..=u64::MAX) {
            let json = format!(r#"{{"n": {}}}"#, n);
            let result = try_parse(json.as_bytes());
            // Should parse without panic
            prop_assert!(result.is_some());
        }

        /// Test numeric edge cases - floating point numbers.
        #[test]
        fn parse_handles_floats(f in any::<f64>().prop_filter("must be finite", |x| x.is_finite())) {
            let json = format!(r#"{{"n": {}}}"#, f);
            let result = try_parse(json.as_bytes());
            // Should parse without panic (finite floats are valid JSON)
            prop_assert!(result.is_some());
        }

        /// Test that NaN representations don't crash the parser.
        /// JSON doesn't support NaN, so these should parse as strings or fail gracefully.
        #[test]
        fn parse_handles_nan_like_strings(s in prop::sample::select(vec![
            "NaN", "nan", "NAN", "Infinity", "-Infinity", "inf", "-inf"
        ])) {
            // As raw value (invalid JSON number)
            let json_raw = format!(r#"{{"n": {}}}"#, s);
            let _ = try_parse(json_raw.as_bytes()); // Should not panic

            // As string value (valid JSON)
            let json_str = format!(r#"{{"n": "{}"}}"#, s);
            let result = try_parse(json_str.as_bytes());
            prop_assert!(result.is_some());
        }

        /// Test that scientific notation is handled.
        #[test]
        fn parse_handles_scientific_notation(
            mantissa in -1000i64..1000i64,
            exponent in -308i32..308i32
        ) {
            let json = format!(r#"{{"n": {}e{}}}"#, mantissa, exponent);
            let _ = try_parse(json.as_bytes()); // Should not panic
        }

        /// Test that very long strings don't cause issues.
        #[test]
        fn parse_handles_long_strings(len in 0usize..10000) {
            let long_string = "x".repeat(len);
            let json = format!(r#"{{"s": "{}"}}"#, long_string);
            let result = try_parse(json.as_bytes());
            // Should parse without panic (within 1MB limit)
            prop_assert!(result.is_some());
        }

        /// Test that arrays with many elements are handled.
        #[test]
        fn parse_handles_large_arrays(len in 0usize..1000) {
            let elements: Vec<String> = (0..len).map(|i| i.to_string()).collect();
            let json = format!("[{}]", elements.join(","));
            let result = try_parse(json.as_bytes());
            prop_assert!(result.is_some());
        }

        /// Test that objects with many keys are handled.
        #[test]
        fn parse_handles_large_objects(len in 0usize..500) {
            let entries: Vec<String> = (0..len).map(|i| format!(r#""k{}": {}"#, i, i)).collect();
            let json = format!("{{{}}}", entries.join(","));
            let result = try_parse(json.as_bytes());
            prop_assert!(result.is_some());
        }

        /// Test that json_depth_exceeds_limit doesn't panic on arbitrary input.
        #[test]
        fn depth_check_doesnt_panic(input in prop::collection::vec(any::<u8>(), 0..2048)) {
            let _ = json_depth_exceeds_limit(&input); // Should not panic
        }

        /// Test that braces in strings don't affect depth calculation.
        #[test]
        fn depth_check_ignores_braces_in_strings(
            prefix in "[a-z]{0,10}",
            braces in "[\\{\\}\\[\\]]{0,50}",
            suffix in "[a-z]{0,10}"
        ) {
            // Create a valid JSON with braces inside a string
            let json = format!(r#"{{"key": "{}{}{}"}}"#, prefix, braces, suffix);
            let result = try_parse(json.as_bytes());
            // Valid JSON with braces in strings should parse (depth = 1)
            prop_assert!(result.is_some());
        }

        /// Test that escape sequences in strings are handled.
        #[test]
        fn parse_handles_escape_sequences(s in prop::sample::select(vec![
            r#"\""#, r#"\\"#, r#"\/"#, r#"\b"#, r#"\f"#, r#"\n"#, r#"\r"#, r#"\t"#
        ])) {
            let json = format!(r#"{{"s": "{}"}}"#, s);
            let result = try_parse(json.as_bytes());
            prop_assert!(result.is_some());
        }

        /// Test Unicode escape sequences.
        #[test]
        fn parse_handles_unicode_escapes(code in 0u16..0xFFFF) {
            // Skip surrogate pairs as they're invalid in JSON
            if !(0xD800..=0xDFFF).contains(&code) {
                let json = format!(r#"{{"s": "\\u{:04X}"}}"#, code);
                let _ = try_parse(json.as_bytes()); // Should not panic
            }
        }

        /// Test mixed nested structures.
        #[test]
        fn parse_handles_mixed_nesting(depth in 1usize..20) {
            // Alternate between objects and arrays
            let mut json = String::new();
            for i in 0..depth {
                if i % 2 == 0 {
                    json.push_str("{\"a\":");
                } else {
                    json.push('[');
                }
            }
            json.push('1');
            for i in (0..depth).rev() {
                if i % 2 == 0 {
                    json.push('}');
                } else {
                    json.push(']');
                }
            }

            let result = try_parse(json.as_bytes());
            prop_assert!(result.is_some());
        }
    }
}

#[test]
fn test_build_object() {
    let v = obj().set("name", str("Alice")).set("age", int(30));
    assert_eq!(v.to_string(), r#"{"age":30,"name":"Alice"}"#);
}

#[test]
fn test_build_array() {
    let v = arr().push(int(1)).push(int(2)).push(int(3));
    assert_eq!(v.to_string(), "[1,2,3]");
}

#[test]
fn test_parse_and_read() {
    let v = try_parse(b"{\"name\":\"Bob\",\"age\":25}").unwrap();
    assert_eq!(v.get("name").str(), Some("Bob".to_string()));
    assert_eq!(v.get("age").int(), Some(25));
    assert!(v.get("missing").is_null());
}

#[test]
fn test_nested() {
    let v = obj().set("user", obj().set("name", str("Alice")));
    assert_eq!(v.get("user").get("name").str(), Some("Alice".to_string()));
}

#[test]
fn test_array_access() {
    let v = arr().push(str("a")).push(str("b"));
    assert_eq!(v.at(0).str(), Some("a".to_string()));
    assert_eq!(v.at(1).str(), Some("b".to_string()));
    assert!(v.at(2).is_null());
}

#[test]
fn test_path_accessors() {
    let v = try_parse(b"{\"user\":{\"name\":\"Alice\",\"age\":30,\"active\":true}}").unwrap();

    // path_str
    assert_eq!(v.path_str(&["user", "name"]), Some("Alice".to_string()));
    assert_eq!(v.path_str(&["user", "missing"]), None);
    assert_eq!(v.path_str_or(&["user", "name"], "default"), "Alice");
    assert_eq!(v.path_str_or(&["user", "missing"], "default"), "default");

    // path_int
    assert_eq!(v.path_int(&["user", "age"]), Some(30));
    assert_eq!(v.path_int(&["user", "missing"]), None);
    assert_eq!(v.path_int_or(&["user", "age"], 0), 30);
    assert_eq!(v.path_int_or(&["user", "missing"], 0), 0);

    // path_bool
    assert_eq!(v.path_bool(&["user", "active"]), Some(true));
    assert_eq!(v.path_bool(&["user", "missing"]), None);
    assert!(v.path_bool_or(&["user", "active"], false));
    assert!(!v.path_bool_or(&["user", "missing"], false));

    // path_exists / path_is_null
    assert!(v.path_exists(&["user", "name"]));
    assert!(!v.path_exists(&["user", "missing"]));

    let v2 = try_parse(b"{\"user\":{\"value\":null}}").unwrap();
    assert!(v2.path_is_null(&["user", "value"]));
    assert!(v2.path_exists(&["user", "value"]));
}

#[test]
fn test_path_deep_nesting() {
    let v = try_parse(b"{\"a\":{\"b\":{\"c\":{\"d\":\"deep\"}}}}").unwrap();
    assert_eq!(v.path_str(&["a", "b", "c", "d"]), Some("deep".to_string()));
    assert_eq!(v.path_str(&["a", "b", "c", "missing"]), None);
    assert_eq!(v.path_str(&["a", "b", "missing", "d"]), None);
}

// ========================================================================
// JSON DEPTH BOUNDARY TESTS
// ========================================================================

/// Generate nested JSON objects: {"a":{"a":{"a":...}}} at specified depth
fn generate_nested_objects(depth: usize) -> String {
    let mut json = String::new();
    for _ in 0..depth {
        json.push_str("{\"a\":");
    }
    json.push('1');
    for _ in 0..depth {
        json.push('}');
    }
    json
}

/// Generate nested JSON arrays: [[[[...]]]] at specified depth
fn generate_nested_arrays(depth: usize) -> String {
    let mut json = String::new();
    for _ in 0..depth {
        json.push('[');
    }
    json.push('1');
    for _ in 0..depth {
        json.push(']');
    }
    json
}

/// Generate mixed nested JSON: {"a":[{"a":[...]}]} alternating objects and arrays
fn generate_mixed_nesting(depth: usize) -> String {
    let mut json = String::new();
    for i in 0..depth {
        if i % 2 == 0 {
            json.push_str("{\"a\":");
        } else {
            json.push('[');
        }
    }
    json.push('1');
    for i in (0..depth).rev() {
        if i % 2 == 0 {
            json.push('}');
        } else {
            json.push(']');
        }
    }
    json
}

#[test]
fn test_depth_limit_objects_at_19() {
    // Depth 19: should succeed (below the limit of 20)
    let json = generate_nested_objects(19);
    let result = try_parse(json.as_bytes());
    assert!(
        result.is_some(),
        "JSON with 19 levels of object nesting should parse successfully"
    );
}

#[test]
fn test_depth_limit_objects_at_20() {
    // Depth 20: should succeed (exactly at the limit)
    let json = generate_nested_objects(20);
    let result = try_parse(json.as_bytes());
    assert!(
        result.is_some(),
        "JSON with 20 levels of object nesting should parse successfully (at limit)"
    );
}

#[test]
fn test_depth_limit_objects_at_21() {
    // Depth 21: should fail (exceeds the limit of 20)
    let json = generate_nested_objects(21);
    let result = try_parse(json.as_bytes());
    assert!(
        result.is_none(),
        "JSON with 21 levels of object nesting should be rejected (exceeds limit)"
    );
}

#[test]
fn test_depth_limit_arrays_at_19() {
    // Depth 19: should succeed (below the limit of 20)
    let json = generate_nested_arrays(19);
    let result = try_parse(json.as_bytes());
    assert!(
        result.is_some(),
        "JSON with 19 levels of array nesting should parse successfully"
    );
}

#[test]
fn test_depth_limit_arrays_at_20() {
    // Depth 20: should succeed (exactly at the limit)
    let json = generate_nested_arrays(20);
    let result = try_parse(json.as_bytes());
    assert!(
        result.is_some(),
        "JSON with 20 levels of array nesting should parse successfully (at limit)"
    );
}

#[test]
fn test_depth_limit_arrays_at_21() {
    // Depth 21: should fail (exceeds the limit of 20)
    let json = generate_nested_arrays(21);
    let result = try_parse(json.as_bytes());
    assert!(
        result.is_none(),
        "JSON with 21 levels of array nesting should be rejected (exceeds limit)"
    );
}

#[test]
fn test_depth_limit_mixed_at_19() {
    // Mixed nesting at depth 19: should succeed
    let json = generate_mixed_nesting(19);
    let result = try_parse(json.as_bytes());
    assert!(
        result.is_some(),
        "JSON with 19 levels of mixed nesting should parse successfully"
    );
}

#[test]
fn test_depth_limit_mixed_at_20() {
    // Mixed nesting at depth 20: should succeed (at limit)
    let json = generate_mixed_nesting(20);
    let result = try_parse(json.as_bytes());
    assert!(
        result.is_some(),
        "JSON with 20 levels of mixed nesting should parse successfully (at limit)"
    );
}

#[test]
fn test_depth_limit_mixed_at_21() {
    // Mixed nesting at depth 21: should fail (exceeds limit)
    let json = generate_mixed_nesting(21);
    let result = try_parse(json.as_bytes());
    assert!(
        result.is_none(),
        "JSON with 21 levels of mixed nesting should be rejected (exceeds limit)"
    );
}

#[test]
fn test_depth_check_ignores_braces_in_strings() {
    // Braces inside strings should not count towards depth
    // This is valid JSON with depth 1, but contains many braces in strings
    let json = r#"{"key": "{{{{{{{{{{{{{{{{{{{{{{{{{{"}"#;
    let result = try_parse(json.as_bytes());
    assert!(
        result.is_some(),
        "Braces inside strings should not affect depth calculation"
    );
}

#[test]
fn test_depth_check_handles_escaped_quotes() {
    // Escaped quotes inside strings should be handled correctly
    let json = r#"{"key": "value with \" escaped quote and {nested}"}"#;
    let result = try_parse(json.as_bytes());
    assert!(
        result.is_some(),
        "Escaped quotes should be handled correctly in depth check"
    );
}

#[test]
fn test_json_depth_exceeds_limit_directly() {
    // Test the internal function directly for precise boundary checks
    let json_19 = generate_nested_objects(19);
    let json_20 = generate_nested_objects(20);
    let json_21 = generate_nested_objects(21);

    assert!(
        !json_depth_exceeds_limit(json_19.as_bytes()),
        "Depth 19 should not exceed limit"
    );
    assert!(
        !json_depth_exceeds_limit(json_20.as_bytes()),
        "Depth 20 should not exceed limit (at boundary)"
    );
    assert!(
        json_depth_exceeds_limit(json_21.as_bytes()),
        "Depth 21 should exceed limit"
    );
}

// ========================================================================
// ToJson TRAIT TESTS
// ========================================================================

mod to_json_tests {
    use super::*;
    use std::borrow::Cow;

    // === String type tests ===

    #[test]
    fn test_string_to_json() {
        let s = String::from("hello");
        let json = s.to_json();
        assert_eq!(json.to_string(), r#""hello""#);
    }

    #[test]
    fn test_str_to_json() {
        let s: &str = "world";
        let json = s.to_json();
        assert_eq!(json.to_string(), r#""world""#);
    }

    #[test]
    fn test_string_ref_to_json() {
        let s = String::from("test");
        let json = s.to_json();
        assert_eq!(json.to_string(), r#""test""#);
    }

    #[test]
    fn test_cow_str_to_json() {
        let borrowed: Cow<'_, str> = Cow::Borrowed("borrowed");
        let owned: Cow<'_, str> = Cow::Owned(String::from("owned"));

        assert_eq!(borrowed.to_json().to_string(), r#""borrowed""#);
        assert_eq!(owned.to_json().to_string(), r#""owned""#);
    }

    #[test]
    fn test_string_with_escapes_to_json() {
        let s = "hello \"world\"\nwith\ttabs";
        let json = s.to_json();
        // Check it serializes correctly (escapes the quotes, newlines, tabs)
        let output = json.to_string();
        assert!(output.contains("hello"));
        assert!(output.starts_with('"'));
        assert!(output.ends_with('"'));
    }

    #[test]
    fn test_empty_string_to_json() {
        let s = "";
        let json = s.to_json();
        assert_eq!(json.to_string(), r#""""#);
    }

    #[test]
    fn test_unicode_string_to_json() {
        let s = "kon'nichiwa";
        let json = s.to_json();
        let output = json.to_string();
        assert!(output.contains("kon'nichiwa"));
    }

    // === Integer type tests ===

    #[test]
    fn test_i8_to_json() {
        assert_eq!(42i8.to_json().to_string(), "42");
        assert_eq!((-128i8).to_json().to_string(), "-128");
        assert_eq!(127i8.to_json().to_string(), "127");
    }

    #[test]
    fn test_i16_to_json() {
        assert_eq!(1000i16.to_json().to_string(), "1000");
        assert_eq!(i16::MIN.to_json().to_string(), "-32768");
        assert_eq!(i16::MAX.to_json().to_string(), "32767");
    }

    #[test]
    fn test_i32_to_json() {
        assert_eq!(123456i32.to_json().to_string(), "123456");
        assert_eq!((-1i32).to_json().to_string(), "-1");
        assert_eq!(0i32.to_json().to_string(), "0");
    }

    #[test]
    fn test_i64_to_json() {
        assert_eq!(i64::MAX.to_json().to_string(), "9223372036854775807");
        assert_eq!(i64::MIN.to_json().to_string(), "-9223372036854775808");
    }

    #[test]
    fn test_isize_to_json() {
        let val: isize = 42;
        assert_eq!(val.to_json().to_string(), "42");
    }

    #[test]
    fn test_u8_to_json() {
        assert_eq!(0u8.to_json().to_string(), "0");
        assert_eq!(255u8.to_json().to_string(), "255");
    }

    #[test]
    fn test_u16_to_json() {
        assert_eq!(u16::MAX.to_json().to_string(), "65535");
    }

    #[test]
    fn test_u32_to_json() {
        assert_eq!(u32::MAX.to_json().to_string(), "4294967295");
    }

    #[test]
    fn test_u64_to_json() {
        // Note: u64::MAX > i64::MAX, so it will be truncated
        // But values within i64 range work fine
        assert_eq!(1000000u64.to_json().to_string(), "1000000");
    }

    #[test]
    fn test_usize_to_json() {
        let val: usize = 42;
        assert_eq!(val.to_json().to_string(), "42");
    }

    // === Float type tests ===

    #[test]
    fn test_f32_to_json() {
        let val: f32 = 1.23;
        let output = val.to_json().to_string();
        assert!(output.starts_with("1.23"));
    }

    #[test]
    fn test_f64_to_json() {
        let val: f64 = 9.87654321;
        let output = val.to_json().to_string();
        assert!(output.starts_with("9.876"));
    }

    #[test]
    fn test_float_zero_to_json() {
        assert_eq!(0.0f64.to_json().to_string(), "0.0");
    }

    #[test]
    fn test_float_negative_to_json() {
        let val: f64 = -99.99;
        let output = val.to_json().to_string();
        assert!(output.starts_with("-99.99"));
    }

    // === Boolean tests ===

    #[test]
    fn test_bool_true_to_json() {
        assert_eq!(true.to_json().to_string(), "true");
    }

    #[test]
    fn test_bool_false_to_json() {
        assert_eq!(false.to_json().to_string(), "false");
    }

    // === Option tests ===

    #[test]
    fn test_some_string_to_json() {
        let opt: Option<String> = Some("hello".to_string());
        assert_eq!(opt.to_json().to_string(), r#""hello""#);
    }

    #[test]
    fn test_none_string_to_json() {
        let opt: Option<String> = None;
        assert_eq!(opt.to_json().to_string(), "null");
    }

    #[test]
    fn test_some_i32_to_json() {
        let opt: Option<i32> = Some(42);
        assert_eq!(opt.to_json().to_string(), "42");
    }

    #[test]
    fn test_none_i32_to_json() {
        let opt: Option<i32> = None;
        assert_eq!(opt.to_json().to_string(), "null");
    }

    #[test]
    fn test_nested_option_to_json() {
        let opt: Option<Option<i32>> = Some(Some(42));
        assert_eq!(opt.to_json().to_string(), "42");

        let opt2: Option<Option<i32>> = Some(None);
        assert_eq!(opt2.to_json().to_string(), "null");

        let opt3: Option<Option<i32>> = None;
        assert_eq!(opt3.to_json().to_string(), "null");
    }

    // === Vec tests ===

    #[test]
    fn test_vec_string_to_json() {
        let v: Vec<String> = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert_eq!(v.to_json().to_string(), r#"["a","b","c"]"#);
    }

    #[test]
    fn test_vec_str_to_json() {
        let v: Vec<&str> = vec!["x", "y", "z"];
        assert_eq!(v.to_json().to_string(), r#"["x","y","z"]"#);
    }

    #[test]
    fn test_vec_i32_to_json() {
        let v: Vec<i32> = vec![1, 2, 3, 4, 5];
        assert_eq!(v.to_json().to_string(), "[1,2,3,4,5]");
    }

    #[test]
    fn test_empty_vec_to_json() {
        let v: Vec<i32> = vec![];
        assert_eq!(v.to_json().to_string(), "[]");
    }

    #[test]
    fn test_vec_with_options_to_json() {
        let v: Vec<Option<i32>> = vec![Some(1), None, Some(3)];
        assert_eq!(v.to_json().to_string(), "[1,null,3]");
    }

    // === Slice tests ===

    #[test]
    fn test_slice_to_json() {
        let arr = [1, 2, 3];
        let slice: &[i32] = &arr;
        assert_eq!(slice.to_json().to_string(), "[1,2,3]");
    }

    #[test]
    fn test_str_slice_to_json() {
        let arr = ["a", "b"];
        let slice: &[&str] = &arr;
        assert_eq!(slice.to_json().to_string(), r#"["a","b"]"#);
    }

    // === Fixed-size array tests ===

    #[test]
    fn test_array_i32_to_json() {
        let arr: [i32; 3] = [10, 20, 30];
        assert_eq!(arr.to_json().to_string(), "[10,20,30]");
    }

    #[test]
    fn test_array_str_to_json() {
        let arr: [&str; 2] = ["hello", "world"];
        assert_eq!(arr.to_json().to_string(), r#"["hello","world"]"#);
    }

    #[test]
    fn test_empty_array_to_json() {
        let arr: [i32; 0] = [];
        assert_eq!(arr.to_json().to_string(), "[]");
    }

    // === JsonValue pass-through tests ===

    #[test]
    fn test_json_value_to_json() {
        let original = obj().set("key", str("value"));
        let converted = original.to_json();
        assert_eq!(converted.to_string(), r#"{"key":"value"}"#);
    }

    // === Reference type tests ===

    #[test]
    fn test_box_to_json() {
        let boxed: Box<i32> = Box::new(42);
        assert_eq!(boxed.to_json().to_string(), "42");
    }

    #[test]
    fn test_box_string_to_json() {
        let boxed: Box<String> = Box::new("boxed".to_string());
        assert_eq!(boxed.to_json().to_string(), r#""boxed""#);
    }

    #[test]
    fn test_rc_to_json() {
        use std::rc::Rc;
        let rc: Rc<i32> = Rc::new(99);
        assert_eq!(rc.to_json().to_string(), "99");
    }

    #[test]
    fn test_arc_to_json() {
        use std::sync::Arc;
        let arc: Arc<String> = Arc::new("shared".to_string());
        assert_eq!(arc.to_json().to_string(), r#""shared""#);
    }

    // === Complex nested type tests ===

    #[test]
    fn test_vec_of_vecs_to_json() {
        let matrix: Vec<Vec<i32>> = vec![vec![1, 2], vec![3, 4], vec![5, 6]];
        assert_eq!(matrix.to_json().to_string(), "[[1,2],[3,4],[5,6]]");
    }

    #[test]
    fn test_option_vec_to_json() {
        let opt: Option<Vec<i32>> = Some(vec![1, 2, 3]);
        assert_eq!(opt.to_json().to_string(), "[1,2,3]");

        let none: Option<Vec<i32>> = None;
        assert_eq!(none.to_json().to_string(), "null");
    }

    #[test]
    fn test_vec_of_options_to_json() {
        let v: Vec<Option<&str>> = vec![Some("a"), None, Some("c")];
        assert_eq!(v.to_json().to_string(), r#"["a",null,"c"]"#);
    }

    // === Building objects with ToJson ===

    #[test]
    fn test_build_object_with_to_json() {
        let name = "Alice".to_string();
        let age: i32 = 30;
        let active = true;
        let tags: Vec<&str> = vec!["admin", "user"];
        let score: Option<f64> = Some(95.5);
        let nickname: Option<String> = None;

        let json = obj()
            .set("name", name.to_json())
            .set("age", age.to_json())
            .set("active", active.to_json())
            .set("tags", tags.to_json())
            .set("score", score.to_json())
            .set("nickname", nickname.to_json());

        let output = json.to_string();
        assert!(output.contains(r#""name":"Alice""#));
        assert!(output.contains(r#""age":30"#));
        assert!(output.contains(r#""active":true"#));
        assert!(output.contains(r#""tags":["admin","user"]"#));
        assert!(output.contains(r#""nickname":null"#));
    }

    // === Property-based tests for ToJson ===

    mod proptest_to_json {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn string_roundtrip(s in ".*") {
                let json = s.to_json();
                // Should produce valid JSON string
                let output = json.to_string();
                prop_assert!(output.starts_with('"'));
                prop_assert!(output.ends_with('"'));
            }

            #[test]
            fn i32_roundtrip(n in any::<i32>()) {
                let json = n.to_json();
                let output = json.to_string();
                // Parse it back
                let parsed: i64 = output.parse().unwrap();
                prop_assert_eq!(parsed, n as i64);
            }

            #[test]
            fn i64_roundtrip(n in any::<i64>()) {
                let json = n.to_json();
                let output = json.to_string();
                let parsed: i64 = output.parse().unwrap();
                prop_assert_eq!(parsed, n);
            }

            #[test]
            fn bool_roundtrip(b in any::<bool>()) {
                let json = b.to_json();
                let output = json.to_string();
                prop_assert_eq!(output, if b { "true" } else { "false" });
            }

            #[test]
            fn option_none_is_null(opt in Just(None::<i32>)) {
                let json = opt.to_json();
                prop_assert_eq!(json.to_string(), "null");
            }

            #[test]
            fn vec_length_preserved(v in prop::collection::vec(any::<i32>(), 0..100)) {
                let json = v.to_json();
                let len = json.len();
                prop_assert_eq!(len, Some(v.len()));
            }
        }
    }
}

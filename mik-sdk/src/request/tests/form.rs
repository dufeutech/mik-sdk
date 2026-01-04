//! Form data parsing tests

use super::super::*;
use std::collections::HashMap;

#[test]
fn test_form_basic() {
    let req = Request::new(
        Method::Post,
        "/submit".to_string(),
        vec![(
            "content-type".to_string(),
            "application/x-www-form-urlencoded".to_string(),
        )],
        Some(b"name=Alice&email=alice%40example.com".to_vec()),
        HashMap::new(),
    );

    assert!(req.is_form());
    assert_eq!(req.form_or("name", ""), "Alice");
    assert_eq!(req.form_or("email", ""), "alice@example.com");
    assert!(req.form_or("missing", "").is_empty());
}

#[test]
fn test_form_all_values() {
    let req = Request::new(
        Method::Post,
        "/submit".to_string(),
        vec![(
            "content-type".to_string(),
            "application/x-www-form-urlencoded".to_string(),
        )],
        Some(b"tags=rust&tags=wasm&tags=http".to_vec()),
        HashMap::new(),
    );

    let tags = req.form_all("tags");
    assert_eq!(tags.len(), 3);
    assert_eq!(tags[0], "rust");
    assert_eq!(tags[1], "wasm");
    assert_eq!(tags[2], "http");

    assert_eq!(req.form_or("tags", ""), "rust"); // First value
}

#[test]
fn test_form_url_decoding() {
    let req = Request::new(
        Method::Post,
        "/submit".to_string(),
        vec![],
        Some(b"message=hello+world&special=%26%3D%3F".to_vec()),
        HashMap::new(),
    );

    assert_eq!(req.form_or("message", ""), "hello world"); // + becomes space
    assert_eq!(req.form_or("special", ""), "&=?"); // URL decoded
}

#[test]
fn test_form_empty_body() {
    let req = Request::new(
        Method::Post,
        "/submit".to_string(),
        vec![],
        None,
        HashMap::new(),
    );

    assert!(req.form_or("anything", "").is_empty());
    assert_eq!(req.form_all("anything").len(), 0);
}

#[test]
fn test_form_empty_values() {
    let req = Request::new(
        Method::Post,
        "/submit".to_string(),
        vec![],
        Some(b"name=&flag&empty=".to_vec()),
        HashMap::new(),
    );

    assert_eq!(req.form_or("name", "MISSING"), ""); // Empty value
    assert_eq!(req.form_or("flag", "MISSING"), ""); // Key without value
    assert_eq!(req.form_or("empty", "MISSING"), ""); // Explicit empty
}

#[test]
fn test_is_form_with_charset() {
    let req = Request::new(
        Method::Post,
        "/".to_string(),
        vec![(
            "content-type".to_string(),
            "application/x-www-form-urlencoded; charset=utf-8".to_string(),
        )],
        None,
        HashMap::new(),
    );

    assert!(req.is_form());
}

#[test]
fn test_unicode_form_data() {
    // Unicode in form submission
    let req = Request::new(
        Method::Post,
        "/submit".to_string(),
        vec![(
            "content-type".to_string(),
            "application/x-www-form-urlencoded".to_string(),
        )],
        Some(b"name=%C3%89milie&city=%E6%9D%B1%E4%BA%AC".to_vec()), // Émilie, 東京
        HashMap::new(),
    );

    assert_eq!(req.form_or("name", ""), "Émilie");
    assert_eq!(req.form_or("city", ""), "東京");
}

#[test]
fn test_malformed_form_body() {
    // Malformed form data - documents actual parsing behavior
    // Using "MISSING" as default to distinguish between "not present" and "present but empty"
    let test_cases = [
        (b"".to_vec(), "MISSING"),        // Empty
        (b"=".to_vec(), "MISSING"),       // Just equals - empty key, not "key"
        (b"===".to_vec(), "MISSING"),     // Multiple equals - empty key
        (b"&&&".to_vec(), "MISSING"),     // Just ampersands
        (b"key".to_vec(), ""),            // Key without value or equals
        (b"%ZZ=bad".to_vec(), "MISSING"), // Invalid percent encoding in key - "%ZZ" != "key"
        (b"key=%ZZ".to_vec(), "%ZZ"),     // Invalid percent encoding in value - preserved
    ];

    for (body, expected_key) in test_cases {
        let req = Request::new(
            Method::Post,
            "/".to_string(),
            vec![(
                "content-type".to_string(),
                "application/x-www-form-urlencoded".to_string(),
            )],
            Some(body.clone()),
            HashMap::new(),
        );
        let result = req.form_or("key", "MISSING");
        assert_eq!(
            result,
            expected_key,
            "Failed for body: {:?}",
            String::from_utf8_lossy(&body)
        );
    }
}

#[test]
fn test_form_injection_attempts() {
    // Form data injection attempts
    let req = Request::new(
        Method::Post,
        "/submit".to_string(),
        vec![(
            "content-type".to_string(),
            "application/x-www-form-urlencoded".to_string(),
        )],
        Some(b"file=../../../etc/passwd&cmd=rm%20-rf%20/".to_vec()),
        HashMap::new(),
    );

    // Values are decoded but application must validate
    assert_eq!(req.form_or("file", ""), "../../../etc/passwd");
    assert_eq!(req.form_or("cmd", ""), "rm -rf /");
}

#[test]
fn test_form_with_file_upload_boundary() {
    // Form data that looks like multipart but isn't
    let body =
        b"------WebKitFormBoundary\r\nContent-Disposition: form-data; name=\"file\"\r\n\r\ndata";

    let req = Request::new(
        Method::Post,
        "/upload".to_string(),
        vec![(
            "content-type".to_string(),
            "application/x-www-form-urlencoded".to_string(),
        )],
        Some(body.to_vec()),
        HashMap::new(),
    );

    // Should parse as regular form, not crash
    let _form = req.form_or("file", "");
}

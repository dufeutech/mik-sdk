//! Path parameter tests

use super::super::*;

#[test]
fn test_param_not_found() {
    let req = Request::new(
        Method::Get,
        "/users/123".to_string(),
        vec![],
        None,
        [("id".to_string(), "123".to_string())]
            .into_iter()
            .collect(),
    );

    assert_eq!(req.param_or("id", ""), "123");
    assert!(req.param_or("missing", "").is_empty());
    assert!(req.param_or("", "").is_empty());
}

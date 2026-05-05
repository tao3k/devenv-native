use super::matches_scope_filter;

#[test]
fn test_matches_scope_filter_no_scope() {
    assert!(matches_scope_filter("src/api/handler.rs", None));
    assert!(matches_scope_filter("any/path/file.rs", None));
}

#[test]
fn test_matches_scope_filter_with_scope() {
    assert!(matches_scope_filter(
        "src/api/handler.rs",
        Some("src/api/**")
    ));
    assert!(!matches_scope_filter(
        "src/db/handler.rs",
        Some("src/api/**")
    ));
}

#[test]
fn test_matches_scope_filter_double_star() {
    assert!(matches_scope_filter(
        "deep/nested/path/file.rs",
        Some("**/*.rs")
    ));
    assert!(matches_scope_filter("lib.rs", Some("**/*.rs")));
    assert!(!matches_scope_filter("lib.py", Some("**/*.rs")));
}

#[test]
fn test_matches_scope_filter_package_specific() {
    assert!(matches_scope_filter(
        "packages/core/src/lib.rs",
        Some("packages/core/**")
    ));
    assert!(!matches_scope_filter(
        "packages/api/src/lib.rs",
        Some("packages/core/**")
    ));
}

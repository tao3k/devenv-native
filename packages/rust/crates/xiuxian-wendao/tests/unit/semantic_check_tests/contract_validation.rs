use super::*;

#[test]
fn test_validate_contract_must_contain() {
    let content = "This document describes Rust and Lock mechanisms.";
    assert!(validate_contract("must_contain(\"Rust\", \"Lock\")", content).is_none());
    assert!(validate_contract("must_contain(\"Python\")", content).is_some());
}

#[test]
fn test_validate_contract_must_not_contain() {
    let content = "This is a stable API.";
    assert!(validate_contract("must_not_contain(\"deprecated\")", content).is_none());
    assert!(validate_contract("must_not_contain(\"stable\")", content).is_some());
}

#[test]
fn test_validate_contract_min_length() {
    let content = "Short";
    assert!(validate_contract("min_length(3)", content).is_none());
    assert!(validate_contract("min_length(100)", content).is_some());
}

#[test]
fn test_extract_function_args() {
    assert_eq!(
        extract_function_args("must_contain(\"Rust\", \"Lock\")", "must_contain"),
        Some("\"Rust\", \"Lock\"")
    );
    assert_eq!(
        extract_function_args("min_length(100)", "min_length"),
        Some("100")
    );
    assert_eq!(extract_function_args("unknown()", "must_contain"), None);
}

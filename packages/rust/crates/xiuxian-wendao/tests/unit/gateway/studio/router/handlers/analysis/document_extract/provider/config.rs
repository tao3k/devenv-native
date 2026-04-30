use super::*;

#[test]
fn document_extract_conversion_limit_defaults_to_bounded_parallelism() {
    let limit = document_extract_conversion_concurrency_limit_with_lookup(&|_| None, Some(12));

    assert_eq!(limit, DEFAULT_DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS);
}

#[test]
fn document_extract_conversion_limit_accepts_positive_override() {
    let limit = document_extract_conversion_concurrency_limit_with_lookup(
        &|key| (key == DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS_ENV).then(|| "7".to_string()),
        Some(12),
    );

    assert_eq!(limit, 7);
}

#[test]
fn document_extract_conversion_limit_ignores_invalid_override() {
    let limit = document_extract_conversion_concurrency_limit_with_lookup(
        &|key| (key == DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS_ENV).then(|| "0".to_string()),
        Some(2),
    );

    assert_eq!(limit, 2);
}

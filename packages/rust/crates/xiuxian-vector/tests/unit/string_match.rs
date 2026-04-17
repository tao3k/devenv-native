//! Integration coverage for Arrow-backed substring mask helpers.

use xiuxian_vector::{LanceArray, LanceStringArray, string_contains_mask};

#[test]
fn string_contains_mask_marks_matching_rows() {
    let values = LanceStringArray::from(vec![
        Some("module BaseModelica"),
        Some("@reexport using ModelingToolkit"),
        Some("end"),
        None,
    ]);

    let mask = string_contains_mask(&values, "reexport");

    assert_eq!(mask.len(), 4);
    assert!(!mask.is_null(0));
    assert!(!mask.value(0));
    assert!(!mask.is_null(1));
    assert!(mask.value(1));
    assert!(!mask.is_null(2));
    assert!(!mask.value(2));
    assert!(mask.is_null(3));
}

#[test]
fn string_contains_mask_supports_code_punctuation_needles() {
    let values = LanceStringArray::from(vec![
        "include(\"src/BaseModelica.jl\")",
        "using ModelingToolkit",
    ]);

    let mask = string_contains_mask(&values, "src/BaseModelica.jl");

    assert_eq!(mask.len(), 2);
    assert!(mask.value(0));
    assert!(!mask.value(1));
}

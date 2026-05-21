use crate::parsers::markdown::sections::{extract_property_drawers, parse_property_drawer};
use xiuxian_wendao_parsers::sections::PropertyDrawerLine;

#[test]
fn test_parse_property_drawer_valid() {
    let line = ":ID: arch-v1";
    let result = parse_property_drawer(line);
    assert_eq!(result, Some(PropertyDrawerLine::new("ID", "arch-v1")));
}

#[test]
fn test_parse_property_drawer_with_spaces() {
    let line = "  :TAGS: core, design  ";
    let result = parse_property_drawer(line);
    assert_eq!(
        result,
        Some(PropertyDrawerLine::new("TAGS", "core, design"))
    );
}

#[test]
fn test_parse_property_drawer_no_leading_colon() {
    let line = "ID: arch-v1";
    let result = parse_property_drawer(line);
    assert!(result.is_none());
}

#[test]
fn test_parse_property_drawer_empty_value() {
    let line = ":ID:   ";
    let result = parse_property_drawer(line);
    assert!(result.is_none());
}

#[test]
fn test_extract_property_drawers_multiple() {
    let lines = vec![
        ":ID: test-123".to_string(),
        ":TAGS: one, two".to_string(),
        String::new(),
        "Content starts here".to_string(),
    ];
    let attrs = extract_property_drawers(&lines);
    assert_eq!(attrs.get("ID"), Some(&"test-123".to_string()));
    assert_eq!(attrs.get("TAGS"), Some(&"one, two".to_string()));
}

#[test]
fn test_extract_property_drawers_stops_at_content() {
    let lines = vec![
        ":ID: test-456".to_string(),
        "Not a property".to_string(),
        ":TAGS: ignored".to_string(),
    ];
    let attrs = extract_property_drawers(&lines);
    assert_eq!(attrs.get("ID"), Some(&"test-456".to_string()));
    assert!(!attrs.contains_key("TAGS"));
}

#[test]
fn test_extract_property_drawers_org_block_format() {
    let lines = vec![
        ":PROPERTIES:".to_string(),
        ":ID:       uuid-v4-or-slug".to_string(),
        ":STATUS:   STABLE".to_string(),
        ":CONTRACT: must_contain(\"Rust\", \"Lock\")".to_string(),
        ":HASH:     blake3_fingerprint".to_string(),
        ":END:".to_string(),
        String::new(),
        "Content starts here".to_string(),
    ];
    let attrs = extract_property_drawers(&lines);
    assert_eq!(attrs.get("ID"), Some(&"uuid-v4-or-slug".to_string()));
    assert_eq!(attrs.get("STATUS"), Some(&"STABLE".to_string()));
    assert_eq!(
        attrs.get("CONTRACT"),
        Some(&"must_contain(\"Rust\", \"Lock\")".to_string())
    );
    assert_eq!(attrs.get("HASH"), Some(&"blake3_fingerprint".to_string()));
}

#[test]
fn test_extract_property_drawers_mixed_format() {
    let lines = vec![
        ":PROPERTIES:".to_string(),
        ":ID: block-id".to_string(),
        ":STATUS: DRAFT".to_string(),
        ":END:".to_string(),
        ":TAGS: ignored-after-end".to_string(),
    ];
    let attrs = extract_property_drawers(&lines);
    assert_eq!(attrs.get("ID"), Some(&"block-id".to_string()));
    assert_eq!(attrs.get("STATUS"), Some(&"DRAFT".to_string()));
    assert!(!attrs.contains_key("TAGS"));
}

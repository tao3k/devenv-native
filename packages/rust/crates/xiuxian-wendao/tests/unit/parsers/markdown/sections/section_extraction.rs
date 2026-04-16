use super::super::*;
use super::support::extract_sections_from;
use std::path::Path;

#[test]
fn test_extract_sections_with_property_drawer() {
    let body = r"# Main Title
:ID: main-section
:TAGS: important

Content here.

## Subsection
:ID: sub-001

More content.
";
    let sections = extract_sections_from(body);

    let first = sections.iter().find(|s| s.heading_title == "Main Title");
    assert!(first.is_some());
    let Some(first) = first else {
        panic!("expected Main Title section");
    };
    assert_eq!(
        first.attributes.get("ID"),
        Some(&"main-section".to_string())
    );
    assert_eq!(first.attributes.get("TAGS"), Some(&"important".to_string()));

    let sub = sections.iter().find(|s| s.heading_title == "Subsection");
    assert!(sub.is_some());
    let Some(sub) = sub else {
        panic!("expected Subsection section");
    };
    assert_eq!(sub.attributes.get("ID"), Some(&"sub-001".to_string()));
}

#[test]
fn test_extract_sections_with_org_block_properties() {
    let body = r#"# Architecture Node
:PROPERTIES:
:ID:       arch-v1
:STATUS:   STABLE
:CONTRACT: must_contain("Rust", "Lock")
:HASH:     abc123def
:END:

This is the architecture section.

## Implementation
:PROPERTIES:
:ID:       impl-v1
:STATUS:   DRAFT
:END:

Implementation details here.
"#;
    let sections = extract_sections_from(body);

    let arch = sections
        .iter()
        .find(|s| s.heading_title == "Architecture Node");
    assert!(arch.is_some());
    let Some(arch) = arch else {
        panic!("expected Architecture Node section");
    };
    assert_eq!(arch.attributes.get("ID"), Some(&"arch-v1".to_string()));
    assert_eq!(arch.attributes.get("STATUS"), Some(&"STABLE".to_string()));
    assert_eq!(
        arch.attributes.get("CONTRACT"),
        Some(&"must_contain(\"Rust\", \"Lock\")".to_string())
    );
    assert_eq!(arch.attributes.get("HASH"), Some(&"abc123def".to_string()));

    let impl_section = sections
        .iter()
        .find(|s| s.heading_title == "Implementation");
    assert!(impl_section.is_some());
    let Some(impl_section) = impl_section else {
        panic!("expected Implementation section");
    };
    assert_eq!(
        impl_section.attributes.get("ID"),
        Some(&"impl-v1".to_string())
    );
    assert_eq!(
        impl_section.attributes.get("STATUS"),
        Some(&"DRAFT".to_string())
    );
}

#[test]
fn test_extract_sections_filters_entities_by_section_byte_range() {
    let body = r"# Alpha
Section alpha links to [One](one.md).

# Beta
Section beta links to [Two](two.md).
";
    let sections = extract_sections(
        body,
        Path::new("/workspace/docs/note.md"),
        Path::new("/workspace"),
    );

    let alpha = sections
        .iter()
        .find(|section| section.heading_title == "Alpha");
    assert!(alpha.is_some());
    let Some(alpha) = alpha else {
        panic!("expected Alpha section");
    };
    assert_eq!(alpha.entities, vec!["docs/one".to_string()]);

    let beta = sections
        .iter()
        .find(|section| section.heading_title == "Beta");
    assert!(beta.is_some());
    let Some(beta) = beta else {
        panic!("expected Beta section");
    };
    assert_eq!(beta.entities, vec!["docs/two".to_string()]);
}

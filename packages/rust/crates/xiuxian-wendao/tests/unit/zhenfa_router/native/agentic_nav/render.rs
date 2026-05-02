use super::support::{SkeletonValidatedHit, render_agentic_nav_result, validated_hit};

#[test]
fn test_render_agentic_nav_result_basic() {
    let validated = vec![validated_hit(
        "doc.md#intro",
        0.9,
        true,
        "intro",
        Some(vec!["Introduction"]),
        0.95,
    )];

    let xml = render_agentic_nav_result("test query", &validated, 10);

    assert!(xml.contains("<query>test query</query>"));
    assert!(xml.contains("<anchor_id>doc.md#intro</anchor_id>"));
    assert!(xml.contains("<is_valid>true</is_valid>"));
    assert!(xml.contains("<score>0.9500</score>"));
    assert!(xml.contains("<total_found>1</total_found>"));
}

#[test]
fn test_render_agentic_nav_result_with_navigation_hint() {
    let validated = vec![validated_hit(
        "doc.md#intro",
        0.9,
        true,
        "intro",
        Some(vec!["Introduction"]),
        0.95,
    )];

    let xml = render_agentic_nav_result("test query", &validated, 10);

    assert!(
        xml.contains("<navigation_hint>"),
        "XML should contain navigation_hint element"
    );
    assert!(
        xml.contains("</navigation_hint>"),
        "XML should contain closing navigation_hint tag"
    );
    assert!(
        xml.contains("entry point"),
        "XML should contain entry point hint for depth 1, got: {xml}"
    );
}

#[test]
fn test_render_agentic_nav_result_with_invalid_anchor_hint() {
    let validated = vec![validated_hit(
        "doc.md#missing",
        0.8,
        false,
        "missing",
        None,
        0.3,
    )];

    let xml = render_agentic_nav_result("test query", &validated, 10);

    assert!(
        xml.contains("<navigation_hint>"),
        "XML should contain navigation_hint element"
    );
    assert!(
        xml.contains("Orphaned anchor"),
        "XML should contain orphaned anchor hint, got: {xml}"
    );
}

#[test]
fn test_render_agentic_nav_result_with_structural_path() {
    let validated = vec![validated_hit(
        "doc.md#storage",
        0.85,
        true,
        "storage",
        Some(vec!["Architecture", "Storage"]),
        0.90,
    )];

    let xml = render_agentic_nav_result("storage systems", &validated, 10);

    assert!(
        xml.contains("<structural_path>"),
        "XML should contain structural_path element"
    );
    assert!(
        xml.contains("<segment>Architecture</segment>"),
        "XML should contain Architecture segment, got: {xml}"
    );
    assert!(
        xml.contains("<segment>Storage</segment>"),
        "XML should contain Storage segment, got: {xml}"
    );
}

#[test]
fn test_render_agentic_nav_result_limit() {
    let validated: Vec<SkeletonValidatedHit> = (0..5)
        .map(|i| {
            validated_hit(
                &format!("doc.md#section{i}"),
                0.9 - (f64::from(i) * 0.1),
                true,
                &format!("section{i}"),
                Some(vec![&format!("Section {i}")]),
                0.95 - (f64::from(i) * 0.1),
            )
        })
        .collect();

    let xml = render_agentic_nav_result("test query", &validated, 2);

    assert!(
        xml.contains("<total_found>5</total_found>"),
        "XML should report 5 total found"
    );
    let candidate_count = xml.matches("<candidate>").count();
    assert_eq!(
        candidate_count, 2,
        "Should have exactly 2 candidates, got {candidate_count}"
    );
}

#[test]
fn test_render_agentic_nav_result_xml_escapes_query() {
    let validated = vec![validated_hit(
        "doc.md#test",
        0.9,
        true,
        "test",
        Some(vec!["Test"]),
        0.95,
    )];

    let xml = render_agentic_nav_result("test <query> & \"data\"", &validated, 10);

    assert!(
        xml.contains("&lt;query&gt;"),
        "XML should escape angle brackets"
    );
    assert!(xml.contains("&amp;"), "XML should escape ampersand");
    assert!(xml.contains("&quot;"), "XML should escape quotes");
}

use super::support::*;

#[test]
fn test_generate_navigation_hint_invalid_anchor() {
    let hit = validated_hit("doc.md#missing", 0.8, false, "missing", None, 0.3);

    let navigation_hint = generate_navigation_hint(&hit);
    assert!(
        navigation_hint.contains("Orphaned anchor"),
        "Expected orphaned hint for invalid anchor, got: {navigation_hint}"
    );
    assert!(
        navigation_hint.contains("content may have changed"),
        "Hint should mention content change, got: {navigation_hint}"
    );
}

#[test]
fn test_generate_navigation_hint_root_level() {
    let hit = validated_hit("doc.md#root", 0.9, true, "root", Some(vec![]), 0.95);

    let navigation_hint = generate_navigation_hint(&hit);
    assert!(
        navigation_hint.contains("Root-level"),
        "Expected root-level hint, got: {navigation_hint}"
    );
    assert!(
        navigation_hint.contains("high-level overview"),
        "Hint should mention overview, got: {navigation_hint}"
    );
}

#[test]
fn test_generate_navigation_hint_top_level() {
    let hit = validated_hit(
        "doc.md#intro",
        0.9,
        true,
        "intro",
        Some(vec!["Introduction"]),
        0.95,
    );

    let navigation_hint = generate_navigation_hint(&hit);
    assert!(
        navigation_hint.contains("Top-level section"),
        "Expected top-level hint, got: {navigation_hint}"
    );
    assert!(
        navigation_hint.contains("good entry point"),
        "Hint should mention entry point, got: {navigation_hint}"
    );
}

#[test]
fn test_generate_navigation_hint_nested_moderate() {
    let hit = validated_hit(
        "doc.md#storage",
        0.85,
        true,
        "storage",
        Some(vec!["Architecture", "Storage"]),
        0.90,
    );

    let navigation_hint = generate_navigation_hint(&hit);
    assert!(
        navigation_hint.contains("Nested section"),
        "Expected nested hint, got: {navigation_hint}"
    );
    assert!(
        navigation_hint.contains("depth 2"),
        "Hint should mention depth, got: {navigation_hint}"
    );
    assert!(
        navigation_hint.contains("implementation details"),
        "Hint should mention details, got: {navigation_hint}"
    );
}

#[test]
fn test_generate_navigation_hint_nested_deep() {
    let hit = validated_hit(
        "doc.md#engine",
        0.8,
        true,
        "engine",
        Some(vec!["Architecture", "Engine", "Core"]),
        0.85,
    );

    let navigation_hint = generate_navigation_hint(&hit);
    assert!(
        navigation_hint.contains("depth 3"),
        "Hint should mention depth 3, got: {navigation_hint}"
    );
}

#[test]
fn test_generate_navigation_hint_deeply_nested() {
    let hit = validated_hit(
        "doc.md#deep",
        0.75,
        true,
        "deep",
        Some(vec!["Level1", "Level2", "Level3", "Level4", "Level5"]),
        0.80,
    );

    let navigation_hint = generate_navigation_hint(&hit);
    assert!(
        navigation_hint.contains("Deeply nested"),
        "Expected deeply nested hint, got: {navigation_hint}"
    );
    assert!(
        navigation_hint.contains("highly specific"),
        "Hint should mention specificity, got: {navigation_hint}"
    );
    assert!(
        navigation_hint.contains("parent context"),
        "Hint should mention context, got: {navigation_hint}"
    );
}

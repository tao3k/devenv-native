use xiuxian_wendao::{KnowledgeCategory, KnowledgeEntry};

#[test]
fn test_knowledge_entry_with_options() {
    let entry = KnowledgeEntry::new(
        "test-002".to_string(),
        "Async Error Handling".to_string(),
        "Handling errors in async Rust code...".to_string(),
        KnowledgeCategory::Technique,
    )
    .with_tags(vec![
        "async".to_string(),
        "error".to_string(),
        "rust".to_string(),
    ])
    .with_source(Some("docs/async-errors.md".to_string()));

    assert_eq!(entry.tags.len(), 3);
    assert_eq!(entry.source, Some("docs/async-errors.md".to_string()));
}

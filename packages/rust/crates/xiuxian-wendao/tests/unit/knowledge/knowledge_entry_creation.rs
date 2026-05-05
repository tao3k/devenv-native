use xiuxian_wendao::{KnowledgeCategory, KnowledgeEntry};

#[test]
fn test_knowledge_entry_creation() {
    let entry = KnowledgeEntry::new(
        "test-001".to_string(),
        "Error Handling Pattern".to_string(),
        "Best practices for error handling in Rust...".to_string(),
        KnowledgeCategory::Pattern,
    );

    assert_eq!(entry.id, "test-001");
    assert_eq!(entry.title, "Error Handling Pattern");
    assert_eq!(entry.category, KnowledgeCategory::Pattern);
    assert!(entry.tags.is_empty());
    assert!(entry.source.is_none());
    assert_eq!(entry.version, 1);
}

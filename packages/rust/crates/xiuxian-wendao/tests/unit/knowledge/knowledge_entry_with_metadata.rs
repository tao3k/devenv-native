use xiuxian_wendao::{KnowledgeCategory, KnowledgeEntry};

#[test]
fn test_knowledge_entry_with_metadata() {
    use serde_json::json;

    let mut entry = KnowledgeEntry::new(
        "metadata-test".to_string(),
        "With Metadata".to_string(),
        "Entry with extra metadata...".to_string(),
        KnowledgeCategory::Reference,
    );

    // Add metadata
    entry
        .metadata
        .insert("author".to_string(), json!("test-author"));
    entry.metadata.insert("reviewed".to_string(), json!(true));

    assert_eq!(entry.metadata.len(), 2);
    assert_eq!(entry.metadata.get("author"), Some(&json!("test-author")));
}

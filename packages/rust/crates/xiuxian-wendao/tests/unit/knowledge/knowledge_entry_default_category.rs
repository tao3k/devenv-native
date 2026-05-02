use xiuxian_wendao::{KnowledgeCategory, KnowledgeEntry};

#[test]
fn test_knowledge_entry_default_category() {
    let entry = KnowledgeEntry::new(
        "test-004".to_string(),
        "Simple Note".to_string(),
        "Just a note...".to_string(),
        KnowledgeCategory::default(),
    );

    assert_eq!(entry.category, KnowledgeCategory::Note);
}

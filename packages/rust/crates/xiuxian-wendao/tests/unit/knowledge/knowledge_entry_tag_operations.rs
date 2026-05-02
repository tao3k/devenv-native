use xiuxian_wendao::{KnowledgeCategory, KnowledgeEntry};

#[test]
fn test_knowledge_entry_tag_operations() {
    let mut entry = KnowledgeEntry::new(
        "test-003".to_string(),
        "Tagged Entry".to_string(),
        "Content with tags...".to_string(),
        KnowledgeCategory::Note,
    );

    // Add unique tag
    entry.add_tag("unique-tag".to_string());
    assert_eq!(entry.tags.len(), 1);

    // Add duplicate tag (should not increase count)
    entry.add_tag("unique-tag".to_string());
    assert_eq!(entry.tags.len(), 1);
}

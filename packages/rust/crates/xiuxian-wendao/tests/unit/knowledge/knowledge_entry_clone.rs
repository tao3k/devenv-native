use xiuxian_wendao::{KnowledgeCategory, KnowledgeEntry};

#[test]
fn test_knowledge_entry_clone() {
    let entry = KnowledgeEntry::new(
        "clone-test".to_string(),
        "Clone This".to_string(),
        "Content to clone...".to_string(),
        KnowledgeCategory::Solution,
    )
    .with_tags(vec!["clone".to_string()])
    .with_source(Some("clone.md".to_string()));

    let cloned = entry.clone();

    assert_eq!(entry.id, cloned.id);
    assert_eq!(entry.title, cloned.title);
    assert_eq!(entry.content, cloned.content);
    assert_eq!(entry.category, cloned.category);
    assert_eq!(entry.tags, cloned.tags);
    assert_eq!(entry.source, cloned.source);
}

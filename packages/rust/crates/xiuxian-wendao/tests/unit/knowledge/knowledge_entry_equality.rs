use xiuxian_wendao::{KnowledgeCategory, KnowledgeEntry};

#[test]
fn test_knowledge_entry_equality() -> Result<(), Box<dyn std::error::Error>> {
    let timestamp =
        chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")?.with_timezone(&chrono::Utc);

    let entry1 = KnowledgeEntry {
        id: "same-id".to_string(),
        title: "Title".to_string(),
        content: "Content".to_string(),
        category: KnowledgeCategory::Note,
        tags: vec!["tag1".to_string()],
        source: None,
        created_at: timestamp,
        updated_at: timestamp,
        version: 1,
        metadata: std::collections::HashMap::new(),
    };

    let timestamp =
        chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")?.with_timezone(&chrono::Utc);
    let entry2 = KnowledgeEntry {
        id: "same-id".to_string(),
        title: "Title".to_string(),
        content: "Content".to_string(),
        category: KnowledgeCategory::Note,
        tags: vec!["tag1".to_string()],
        source: None,
        created_at: timestamp,
        updated_at: timestamp,
        version: 1,
        metadata: std::collections::HashMap::new(),
    };

    assert_eq!(entry1, entry2);
    Ok(())
}

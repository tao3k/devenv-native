use xiuxian_wendao::{KnowledgeCategory, KnowledgeSearchQuery};

#[test]
fn test_search_query_builder() {
    let query = KnowledgeSearchQuery::new("database error".to_string())
        .with_category(KnowledgeCategory::Error)
        .with_tags(vec!["sql".to_string(), "postgres".to_string()])
        .with_limit(10);

    assert_eq!(query.query, "database error");
    assert_eq!(query.category, Some(KnowledgeCategory::Error));
    assert_eq!(query.tags.len(), 2);
    assert_eq!(query.limit, 10);
}

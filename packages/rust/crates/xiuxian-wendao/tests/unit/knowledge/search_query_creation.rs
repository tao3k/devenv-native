use xiuxian_wendao::KnowledgeSearchQuery;

#[test]
fn test_search_query_creation() {
    let query = KnowledgeSearchQuery::new("error handling".to_string());

    assert_eq!(query.query, "error handling");
    assert!(query.category.is_none());
    assert!(query.tags.is_empty());
    assert_eq!(query.limit, 5);
}

use xiuxian_wendao::KnowledgeSearchQuery;

#[test]
fn test_search_query_default() {
    let query = KnowledgeSearchQuery::default();

    assert!(query.query.is_empty());
    assert!(query.category.is_none());
    assert!(query.tags.is_empty());
    assert_eq!(query.limit, 0);
}

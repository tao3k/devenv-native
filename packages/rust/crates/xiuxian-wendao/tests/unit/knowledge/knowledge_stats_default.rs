use xiuxian_wendao::KnowledgeStats;

#[test]
fn test_knowledge_stats_default() {
    let stats = KnowledgeStats::default();

    assert_eq!(stats.total_entries, 0);
    assert!(stats.entries_by_category.is_empty());
    assert_eq!(stats.total_tags, 0);
    assert!(stats.last_updated.is_none());
}

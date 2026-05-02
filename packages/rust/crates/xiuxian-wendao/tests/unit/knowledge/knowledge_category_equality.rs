use xiuxian_wendao::KnowledgeCategory;

#[test]
fn test_knowledge_category_equality() {
    assert_eq!(KnowledgeCategory::Pattern, KnowledgeCategory::Pattern);
    assert_ne!(KnowledgeCategory::Pattern, KnowledgeCategory::Solution);
}

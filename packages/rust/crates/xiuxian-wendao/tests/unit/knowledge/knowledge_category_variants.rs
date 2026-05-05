use xiuxian_wendao::KnowledgeCategory;

#[test]
fn test_knowledge_category_variants() {
    // Test all category variants exist
    let _ = KnowledgeCategory::Pattern;
    let _ = KnowledgeCategory::Solution;
    let _ = KnowledgeCategory::Error;
    let _ = KnowledgeCategory::Technique;
    let _ = KnowledgeCategory::Note;
    let _ = KnowledgeCategory::Reference;
    let _ = KnowledgeCategory::Architecture;
    let _ = KnowledgeCategory::Workflow;
}

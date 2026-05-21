use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::compute_repository_saliency;
use crate::analyzers::{ModuleRecord, RelationKind, RelationRecord, SymbolRecord};
use std::collections::BTreeMap;

#[test]
fn test_compute_repository_saliency_basic() {
    let mut analysis = RepositoryAnalysisOutput::default();

    // Setup: A hub module with two symbols
    analysis.modules.push(ModuleRecord {
        repo_id: "test".to_string().into(),
        module_id: "mod1".to_string().into(),
        qualified_name: "Mod1".to_string(),
        path: "src/Mod1.jl".to_string().into(),
    });

    analysis.symbols.push(SymbolRecord {
        repo_id: "test".to_string().into(),
        symbol_id: "sym1".to_string().into(),
        module_id: Some("mod1".to_string().into()),
        name: "sym1".to_string(),
        qualified_name: "Mod1.sym1".to_string(),
        kind: crate::analyzers::RepoSymbolKind::Function,
        path: "src/Mod1.jl".to_string().into(),
        line_start: None,
        line_end: None,
        signature: None,
        audit_status: None,
        verification_state: None,
        attributes: BTreeMap::new(),
    });

    // Relation: mod1 contains sym1
    analysis.relations.push(RelationRecord {
        repo_id: "test".to_string().into(),
        source_id: "mod1".to_string(),
        target_id: "sym1".to_string(),
        kind: RelationKind::Contains,
    });

    let scores = compute_repository_saliency(&analysis);

    assert!(scores.contains_key("mod1"));
    assert!(scores.contains_key("sym1"));

    // mod1 has out-degree 1, sym1 has in-degree 1
    // raw_score(mod1) = 0*2 + 1*0.5 = 0.5
    // raw_score(sym1) = 1*2 + 0*0.5 = 2.0
    // Normalized: sym1 should be 1.0, mod1 should be 0.25
    let sym1_score = *scores
        .get("sym1")
        .unwrap_or_else(|| panic!("sym1 score should be present"));
    let mod1_score = *scores
        .get("mod1")
        .unwrap_or_else(|| panic!("mod1 score should be present"));
    assert!((sym1_score - 1.0).abs() < f64::EPSILON);
    assert!((mod1_score - 0.25).abs() < f64::EPSILON);
}

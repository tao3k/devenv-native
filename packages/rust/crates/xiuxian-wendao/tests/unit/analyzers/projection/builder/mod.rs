use crate::analyzers::ProjectionPageKind;
use crate::analyzers::projection::builder::anchors::TargetAnchors;
use crate::analyzers::projection::builder::kinds::doc_projection_kind;

#[test]
fn doc_projection_kind_honors_reference_format_without_symbol_targets() {
    let doc = crate::analyzers::DocRecord {
        repo_id: "repo".to_string().into(),
        doc_id: "repo:doc:solve".to_string().into(),
        title: "Solve Linear Systems".to_string(),
        path: "docs/solve.md".to_string().into(),
        format: Some("reference".to_string()),
        doc_target: None,
    };

    assert_eq!(
        doc_projection_kind(&doc, &TargetAnchors::default()),
        ProjectionPageKind::Reference
    );
}

#[test]
fn doc_projection_kind_upgrades_explanation_docs_when_symbol_anchored() {
    let doc = crate::analyzers::DocRecord {
        repo_id: "repo".to_string().into(),
        doc_id: "repo:doc:solver".to_string().into(),
        title: "Solver Notes".to_string(),
        path: "docs/solver.md".to_string().into(),
        format: None,
        doc_target: None,
    };

    let targets = TargetAnchors {
        module_ids: Vec::new(),
        symbol_ids: vec!["repo:symbol:solve".to_string()],
    };

    assert_eq!(
        doc_projection_kind(&doc, &targets),
        ProjectionPageKind::Reference
    );
}

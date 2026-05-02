use std::fs;

use tempfile::TempDir;
use xiuxian_zhenfa::ZhenfaContext;

use crate::link_graph::LinkGraphIndex;
use crate::parsers::docs_governance::derive_opaque_doc_id;
use crate::zhenfa_router::native::semantic_check::docs_governance::tests::support::PanicExt;
use crate::zhenfa_router::native::semantic_check::docs_governance::{
    MISSING_PACKAGE_DOCS_SECTION_LANDING_ISSUE_TYPE, collect_workspace_doc_governance_issues,
};
use crate::zhenfa_router::native::semantic_check::{
    CheckType, WendaoSemanticCheckArgs, run_audit_core,
};

#[test]
fn detects_missing_standard_section_landings_for_existing_docs_tree() {
    let temp = TempDir::new().or_panic("tempdir");
    let crate_dir = temp.path().join("packages/rust/crates/demo");
    fs::create_dir_all(crate_dir.join("docs")).or_panic("create docs dir");
    fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .or_panic("write cargo");
    let index_path = crate_dir.join("docs/index.md");
    let index_path_str = index_path.to_string_lossy().to_string();
    fs::write(
        &index_path,
        format!(
            "# Demo\n\n:PROPERTIES:\n:ID: {}\n:END:\n",
            derive_opaque_doc_id(&index_path_str)
        ),
    )
    .or_panic("write index");

    let issues = collect_workspace_doc_governance_issues(temp.path(), None);
    let section_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.issue_type == MISSING_PACKAGE_DOCS_SECTION_LANDING_ISSUE_TYPE)
        .collect();

    assert_eq!(section_issues.len(), 4);
    assert!(
        section_issues
            .iter()
            .any(|issue| issue.doc.ends_with("01_core/101_demo_core_boundary.md"))
    );
    assert!(section_issues.iter().any(|issue| {
        issue
            .doc
            .ends_with("03_features/201_demo_feature_ledger.md")
    }));
    assert!(section_issues.iter().any(|issue| {
        issue
            .doc
            .ends_with("05_research/301_demo_research_agenda.md")
    }));
    assert!(
        section_issues
            .iter()
            .any(|issue| issue.doc.ends_with("06_roadmap/401_demo_roadmap.md"))
    );
}

#[test]
fn run_audit_core_reports_missing_standard_section_landings() {
    let temp = TempDir::new().or_panic("tempdir");
    let crate_dir = temp.path().join("packages/rust/crates/demo");
    fs::create_dir_all(crate_dir.join("docs")).or_panic("create docs dir");
    fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .or_panic("write cargo");
    let index_path = crate_dir.join("docs/index.md");
    let index_path_str = index_path.to_string_lossy().to_string();
    fs::write(
        &index_path,
        format!(
            "# Demo\n\n:PROPERTIES:\n:ID: {}\n:END:\n",
            derive_opaque_doc_id(&index_path_str)
        ),
    )
    .or_panic("write index");

    let index = LinkGraphIndex::build(temp.path()).or_panic("build index");
    let mut ctx = ZhenfaContext::default();
    ctx.insert_extension(index);

    let args = WendaoSemanticCheckArgs {
        doc: Some(".".to_string()),
        checks: Some(vec![CheckType::DocGovernance]),
        include_warnings: Some(true),
        source_paths: None,
        fuzzy_confidence_threshold: None,
        episteme_load: None,
    };
    let (issues, _file_contents) = run_audit_core(&ctx, &args).or_panic("audit");

    assert!(issues.iter().any(|issue| {
        issue.issue_type == MISSING_PACKAGE_DOCS_SECTION_LANDING_ISSUE_TYPE
            && issue
                .doc
                .ends_with("03_features/201_demo_feature_ledger.md")
    }));
}

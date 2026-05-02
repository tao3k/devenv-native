use std::fs;

use tempfile::TempDir;
use xiuxian_zhenfa::ZhenfaContext;

use crate::link_graph::LinkGraphIndex;
use crate::parsers::docs_governance::derive_opaque_doc_id;
use crate::zhenfa_router::native::semantic_check::docs_governance::tests::support::PanicExt;
use crate::zhenfa_router::native::semantic_check::docs_governance::{
    DOC_IDENTITY_PROTOCOL_ISSUE_TYPE, MISSING_PACKAGE_DOCS_INDEX_ISSUE_TYPE,
    MISSING_PACKAGE_DOCS_TREE_ISSUE_TYPE, collect_workspace_doc_governance_issues,
};
use crate::zhenfa_router::native::semantic_check::{
    CheckType, WendaoSemanticCheckArgs, run_audit_core,
};

#[test]
fn detects_doc_identity_for_workspace_package_docs_tree_files() {
    let temp = TempDir::new().or_panic("tempdir");
    let crate_dir = temp.path().join("packages/rust/crates/demo");
    let core_dir = crate_dir.join("docs/01_core");
    fs::create_dir_all(&core_dir).or_panic("create core docs dir");
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
            "# Demo\n\n:PROPERTIES:\n:ID: {}\n:TYPE: INDEX\n:END:\n",
            derive_opaque_doc_id(&index_path_str)
        ),
    )
    .or_panic("write index");

    let intro_path = core_dir.join("101_intro.md");
    let intro_path_str = intro_path.to_string_lossy().to_string();
    fs::write(
        &intro_path,
        "# Intro\n\n:PROPERTIES:\n:ID: readable-intro\n:TYPE: CORE\n:END:\n",
    )
    .or_panic("write intro");

    let issues = collect_workspace_doc_governance_issues(temp.path(), None);
    let issue = issues
        .iter()
        .find(|issue| {
            issue.issue_type == DOC_IDENTITY_PROTOCOL_ISSUE_TYPE && issue.doc == intro_path_str
        })
        .or_panic("workspace doc identity issue");

    assert_eq!(issue.severity, "error");
    assert_eq!(
        issue.suggestion.as_deref(),
        Some(derive_opaque_doc_id(&intro_path_str).as_str())
    );
}

#[test]
fn run_audit_core_reports_missing_package_docs_index() {
    let temp = TempDir::new().or_panic("tempdir");
    let crate_dir = temp.path().join("packages/rust/crates/demo");
    fs::create_dir_all(&crate_dir).or_panic("create crate dir");
    fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .or_panic("write cargo");
    let docs_dir = temp.path().join("packages/rust/crates/demo/docs/01_core");
    fs::create_dir_all(&docs_dir).or_panic("create docs dir");
    let doc_path = docs_dir.join("101_intro.md");
    let doc_path_str = doc_path.to_string_lossy().to_string();
    let content = format!(
        "# Intro\n\n:PROPERTIES:\n:ID: {}\n:END:\n\nIntro.\n",
        derive_opaque_doc_id(&doc_path_str)
    );
    fs::write(&doc_path, content).or_panic("write doc");

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

    assert!(
        issues
            .iter()
            .any(|issue| issue.issue_type == MISSING_PACKAGE_DOCS_INDEX_ISSUE_TYPE)
    );
}

#[test]
fn run_audit_core_reports_missing_package_docs_tree() {
    let temp = TempDir::new().or_panic("tempdir");
    let crate_dir = temp.path().join("packages/rust/crates/demo");
    fs::create_dir_all(&crate_dir).or_panic("create crate dir");
    fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .or_panic("write cargo");

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

    assert!(
        issues
            .iter()
            .any(|issue| issue.issue_type == MISSING_PACKAGE_DOCS_TREE_ISSUE_TYPE)
    );
}

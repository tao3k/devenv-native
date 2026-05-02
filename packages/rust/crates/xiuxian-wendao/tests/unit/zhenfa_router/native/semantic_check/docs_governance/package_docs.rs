use std::fs;

use tempfile::TempDir;

use crate::parsers::docs_governance::derive_opaque_doc_id;
use crate::zhenfa_router::native::semantic_check::docs_governance::tests::support::PanicExt;
use crate::zhenfa_router::native::semantic_check::docs_governance::{
    MISSING_PACKAGE_DOCS_INDEX_ISSUE_TYPE, MISSING_PACKAGE_DOCS_TREE_ISSUE_TYPE,
    collect_workspace_doc_governance_issues,
};

#[test]
fn detects_missing_package_docs_index_for_workspace_crate_docs() {
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

    let issues = collect_workspace_doc_governance_issues(temp.path(), None);
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].issue_type, MISSING_PACKAGE_DOCS_INDEX_ISSUE_TYPE);
    assert!(
        issues[0]
            .doc
            .ends_with("packages/rust/crates/demo/docs/index.md")
    );
    let suggestion = issues[0].suggestion.as_ref().or_panic("suggestion");
    assert!(suggestion.contains("# demo: Map of Content"));
    assert!(suggestion.contains("[[01_core/101_intro]]"));
}

#[test]
fn detects_missing_package_docs_tree_for_workspace_crate() {
    let temp = TempDir::new().or_panic("tempdir");
    let crate_dir = temp.path().join("packages/rust/crates/demo");
    fs::create_dir_all(&crate_dir).or_panic("create crate dir");
    fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .or_panic("write cargo");

    let issues = collect_workspace_doc_governance_issues(temp.path(), None);
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].issue_type, MISSING_PACKAGE_DOCS_TREE_ISSUE_TYPE);
    assert_eq!(issues[0].severity, "warning");
    assert!(
        issues[0]
            .doc
            .ends_with("packages/rust/crates/demo/docs/index.md")
    );
    let suggestion = issues[0].suggestion.as_ref().or_panic("suggestion");
    assert!(suggestion.contains("# demo: Map of Content"));
    assert!(suggestion.contains("Standardized documentation index"));
}

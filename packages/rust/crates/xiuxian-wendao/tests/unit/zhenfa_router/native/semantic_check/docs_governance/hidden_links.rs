use std::fs;

use tempfile::TempDir;

use crate::parsers::docs_governance::derive_opaque_doc_id;
use crate::zhenfa_router::native::semantic_check::docs_governance::tests::support::PanicExt;
use crate::zhenfa_router::native::semantic_check::docs_governance::{
    CANONICAL_DOC_HIDDEN_PATH_LINK_ISSUE_TYPE, collect_doc_governance_issues,
    collect_workspace_doc_governance_issues,
};

#[test]
fn detects_hidden_wikilink_in_package_doc() {
    let temp = TempDir::new().or_panic("tempdir");
    let doc_path = temp
        .path()
        .join("packages/rust/crates/demo/docs/01_core/101_intro.md");
    fs::create_dir_all(doc_path.parent().or_panic("doc parent")).or_panic("create doc parent");
    let doc_path_str = doc_path.to_string_lossy().to_string();
    let content = format!(
        "# Intro\n\n:PROPERTIES:\n:ID: {}\n:END:\n\nSee [[.cache/codex/execplans/demo.md]].\n",
        derive_opaque_doc_id(&doc_path_str)
    );

    let issues = collect_doc_governance_issues(&doc_path_str, &content);
    let issue = issues
        .iter()
        .find(|issue| issue.issue_type == CANONICAL_DOC_HIDDEN_PATH_LINK_ISSUE_TYPE)
        .or_panic("hidden path issue");

    assert_eq!(issue.severity, "warning");
    assert!(issue.message.contains(".cache/codex/execplans/demo.md"));
    assert_eq!(issue.location.as_ref().or_panic("location").line, 7);
}

#[test]
fn detects_hidden_markdown_link_in_docs_tree() {
    let content = "# RFC\n\nThe prior draft lived in [tracking](.data/blueprints/demo_plan.md).\n";
    let issues = collect_doc_governance_issues("docs/rfcs/demo.md", content);

    let issue = issues
        .iter()
        .find(|issue| issue.issue_type == CANONICAL_DOC_HIDDEN_PATH_LINK_ISSUE_TYPE)
        .or_panic("hidden markdown link issue");
    assert!(issue.message.contains(".data/blueprints/demo_plan.md"));
    assert!(
        issue
            .suggestion
            .as_deref()
            .or_panic("suggestion")
            .contains("Remove")
    );
}

#[test]
fn ignores_hidden_links_in_tracking_docs() {
    let content = "# Daily\n\nTracked in [[.cache/codex/execplans/demo.md]].\n";
    let issues = collect_doc_governance_issues(".cache/agent/GTD/DAILY_2026_04_04.md", content);

    assert!(
        !issues
            .iter()
            .any(|issue| issue.issue_type == CANONICAL_DOC_HIDDEN_PATH_LINK_ISSUE_TYPE)
    );
}

#[test]
fn workspace_scan_reports_root_claude_hidden_path_link() {
    let temp = TempDir::new().or_panic("tempdir");
    fs::write(
        temp.path().join("CLAUDE.md"),
        "# Claude\n\nActive reference: [[.data/blueprints/demo.md]]\n",
    )
    .or_panic("write claude");

    let issues = collect_workspace_doc_governance_issues(temp.path(), None);
    let issue = issues
        .iter()
        .find(|issue| issue.issue_type == CANONICAL_DOC_HIDDEN_PATH_LINK_ISSUE_TYPE)
        .or_panic("workspace hidden path issue");

    assert!(issue.doc.ends_with("CLAUDE.md"));
    assert_eq!(issue.severity, "warning");
}

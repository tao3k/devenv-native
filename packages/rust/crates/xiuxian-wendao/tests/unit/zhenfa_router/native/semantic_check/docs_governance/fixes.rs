use std::collections::HashMap;
use std::fs;

use tempfile::TempDir;
use xiuxian_zhenfa::ZhenfaContext;

use crate::link_graph::LinkGraphIndex;
use crate::parsers::docs_governance::derive_opaque_doc_id;
use crate::zhenfa_router::native::audit::fix::AtomicFixBatch;
use crate::zhenfa_router::native::audit::generate_surgical_fixes;
use crate::zhenfa_router::native::semantic_check::docs_governance::collection::collect_stale_index_footer_standards;
use crate::zhenfa_router::native::semantic_check::docs_governance::tests::support::PanicExt;
use crate::zhenfa_router::native::semantic_check::docs_governance::{
    DOC_IDENTITY_PROTOCOL_ISSUE_TYPE, STALE_PACKAGE_DOCS_INDEX_FOOTER_STANDARDS_ISSUE_TYPE,
    collect_doc_governance_issues,
};
use crate::zhenfa_router::native::semantic_check::{
    CheckType, WendaoSemanticCheckArgs, run_audit_core,
};

#[test]
fn surgical_fixes_repair_non_opaque_doc_identity() {
    let doc_key = "packages/rust/crates/demo/docs/01_core/101_external_modelica_plugin_boundary.md";
    let original = "# Demo\n\n:PROPERTIES:\n:ID: readable-id\n:TYPE: CORE\n:END:\n\nBody.\n";
    let issues = collect_doc_governance_issues(doc_key, original);
    assert_eq!(issues.len(), 1);

    let file_contents = HashMap::from([(doc_key.to_string(), original.to_string())]);
    let fixes = generate_surgical_fixes(&issues, &file_contents);
    assert_eq!(fixes.len(), 1);

    let mut content = original.to_string();
    let result = fixes[0].apply_surgical(&mut content);
    assert!(matches!(
        result,
        crate::zhenfa_router::native::audit::FixResult::Success
    ));
    assert!(content.contains(&format!(":ID: {}", derive_opaque_doc_id(doc_key))));
}

#[test]
fn run_audit_core_loads_explicit_workspace_doc_file_for_fix_generation() {
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
            "# Demo\n\n:PROPERTIES:\n:ID: {}\n:END:\n\n## 01_core\n\n- [[01_core/101_demo_core_boundary]]\n\n:RELATIONS:\n:LINKS: [[01_core/101_demo_core_boundary]]\n:END:\n\n---\n\n:FOOTER:\n:STANDARDS: v1.0\n:LAST_SYNC: 2026-03-20\n:END:\n",
            derive_opaque_doc_id(&index_path_str)
        ),
    )
    .or_panic("write index");

    let index = LinkGraphIndex::build(temp.path()).or_panic("build index");
    let mut ctx = ZhenfaContext::default();
    ctx.insert_extension(index);

    let args = WendaoSemanticCheckArgs {
        doc: Some(index_path_str.clone()),
        checks: Some(vec![CheckType::DocGovernance]),
        include_warnings: Some(true),
        source_paths: None,
        fuzzy_confidence_threshold: None,
        episteme_load: None,
    };
    let (_issues, file_contents) = run_audit_core(&ctx, &args).or_panic("audit");

    assert!(file_contents.contains_key(&index_path_str));

    let content = file_contents.get(&index_path_str).or_panic("missing value");
    let issues = collect_stale_index_footer_standards(&index_path_str, content);
    assert_eq!(issues.len(), 1);

    let fixes = generate_surgical_fixes(&issues, &file_contents);
    assert_eq!(fixes.len(), 1);
    assert_eq!(
        fixes[0].issue_type,
        STALE_PACKAGE_DOCS_INDEX_FOOTER_STANDARDS_ISSUE_TYPE
    );
}

#[test]
fn run_audit_core_reports_doc_identity_for_explicit_workspace_doc_file() {
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
        "# Demo\n\n:PROPERTIES:\n:ID: readable-demo-index\n:TYPE: INDEX\n:END:\n",
    )
    .or_panic("write index");

    let index = LinkGraphIndex::build(temp.path()).or_panic("build index");
    let mut ctx = ZhenfaContext::default();
    ctx.insert_extension(index);

    let args = WendaoSemanticCheckArgs {
        doc: Some(index_path_str.clone()),
        checks: Some(vec![CheckType::DocGovernance]),
        include_warnings: Some(true),
        source_paths: None,
        fuzzy_confidence_threshold: None,
        episteme_load: None,
    };
    let (issues, _file_contents) = run_audit_core(&ctx, &args).or_panic("audit");

    let issue = issues
        .iter()
        .find(|issue| issue.issue_type == DOC_IDENTITY_PROTOCOL_ISSUE_TYPE)
        .or_panic("doc identity issue");
    let canonical_index_path = index_path
        .canonicalize()
        .or_panic("canonical index path")
        .to_string_lossy()
        .to_string();
    let expected_id = derive_opaque_doc_id(&canonical_index_path);
    assert_eq!(issue.doc, canonical_index_path);
    assert_eq!(issue.severity, "error");
    assert_eq!(issue.suggestion.as_deref(), Some(expected_id.as_str()));
}

#[test]
fn run_audit_core_seeds_workspace_doc_identity_issue_files_for_fix_generation() {
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
    fs::write(
        &intro_path,
        "# Intro\n\n:PROPERTIES:\n:ID: readable-intro\n:TYPE: CORE\n:END:\n",
    )
    .or_panic("write intro");

    let index = LinkGraphIndex::build(temp.path()).or_panic("build index");
    let mut ctx = ZhenfaContext::default();
    ctx.insert_extension(index);

    let docs_scope = crate_dir.join("docs").to_string_lossy().to_string();
    let args = WendaoSemanticCheckArgs {
        doc: Some(docs_scope),
        checks: Some(vec![CheckType::DocGovernance]),
        include_warnings: Some(true),
        source_paths: None,
        fuzzy_confidence_threshold: None,
        episteme_load: None,
    };
    let (issues, file_contents) = run_audit_core(&ctx, &args).or_panic("audit");
    let canonical_intro_path = intro_path
        .canonicalize()
        .or_panic("canonical intro path")
        .to_string_lossy()
        .to_string();

    let identity_issue = issues
        .iter()
        .find(|issue| {
            issue.issue_type == DOC_IDENTITY_PROTOCOL_ISSUE_TYPE
                && issue.doc == canonical_intro_path
        })
        .or_panic("workspace doc identity issue");

    assert!(file_contents.contains_key(&canonical_intro_path));

    let fixes = generate_surgical_fixes(std::slice::from_ref(identity_issue), &file_contents);
    assert_eq!(fixes.len(), 1);
    assert_eq!(fixes[0].issue_type, DOC_IDENTITY_PROTOCOL_ISSUE_TYPE);
}

#[test]
fn package_docs_directory_scope_fix_rewrites_doc_identity_issues_end_to_end() {
    let temp = TempDir::new().or_panic("tempdir");
    let crate_dir = temp.path().join("packages/rust/crates/demo");
    let core_dir = crate_dir.join("docs/01_core");
    let feature_dir = crate_dir.join("docs/03_features");
    fs::create_dir_all(&core_dir).or_panic("create core docs dir");
    fs::create_dir_all(&feature_dir).or_panic("create feature docs dir");
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

    let core_doc = core_dir.join("101_intro.md");
    let core_doc_str = core_doc.to_string_lossy().to_string();
    fs::write(
        &core_doc,
        "# Intro\n\n:PROPERTIES:\n:ID: readable-intro\n:TYPE: CORE\n:END:\n",
    )
    .or_panic("write core doc");

    let feature_doc = feature_dir.join("201_feature_ledger.md");
    let feature_doc_str = feature_doc.to_string_lossy().to_string();
    fs::write(
        &feature_doc,
        "# Feature Ledger\n\n:PROPERTIES:\n:ID: readable-feature-ledger\n:TYPE: FEATURE\n:END:\n",
    )
    .or_panic("write feature doc");

    let index = LinkGraphIndex::build(temp.path()).or_panic("build index");
    let mut ctx = ZhenfaContext::default();
    ctx.insert_extension(index);

    let args = WendaoSemanticCheckArgs {
        doc: Some(crate_dir.join("docs").to_string_lossy().to_string()),
        checks: Some(vec![CheckType::DocGovernance]),
        include_warnings: Some(true),
        source_paths: None,
        fuzzy_confidence_threshold: None,
        episteme_load: None,
    };
    let (issues, file_contents) = run_audit_core(&ctx, &args).or_panic("audit");

    let doc_identity_issues = issues
        .iter()
        .filter(|issue| issue.issue_type == DOC_IDENTITY_PROTOCOL_ISSUE_TYPE)
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(doc_identity_issues.len(), 2);

    let fixes = generate_surgical_fixes(&doc_identity_issues, &file_contents);
    assert_eq!(fixes.len(), 2);

    let report = AtomicFixBatch::new(fixes).apply_all();
    assert!(report.is_success(), "{}", report.summary());

    let core_doc_content = fs::read_to_string(&core_doc).or_panic("read core doc");
    assert!(core_doc_content.contains(&format!(":ID: {}", derive_opaque_doc_id(&core_doc_str))));

    let feature_doc_content = fs::read_to_string(&feature_doc).or_panic("read feature doc");
    assert!(
        feature_doc_content.contains(&format!(":ID: {}", derive_opaque_doc_id(&feature_doc_str)))
    );
}

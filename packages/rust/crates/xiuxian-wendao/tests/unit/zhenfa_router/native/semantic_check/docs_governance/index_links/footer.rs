use crate::zhenfa_router::native::semantic_check::docs_governance::tests::support::*;

#[test]
fn detects_missing_index_footer_block_for_existing_relations_block() {
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
    let index_content = format!(
        "# Demo\n\n:PROPERTIES:\n:ID: {}\n:END:\n\n## 01_core\n\n- [[01_core/101_demo_core_boundary]]\n\n:RELATIONS:\n:LINKS: [[01_core/101_demo_core_boundary]]\n:END:\n",
        derive_opaque_doc_id(&index_path_str)
    );
    fs::write(&index_path, &index_content).or_panic("write index");

    let issues = collect_workspace_doc_governance_issues(temp.path(), None);
    let footer_issue = issues
        .iter()
        .find(|issue| issue.issue_type == MISSING_PACKAGE_DOCS_INDEX_FOOTER_BLOCK_ISSUE_TYPE)
        .or_panic("missing footer-block issue");

    assert_eq!(footer_issue.doc, index_path_str);
    assert_eq!(footer_issue.severity, "warning");
    assert!(footer_issue.message.contains(":FOOTER:"));
    assert_eq!(
        footer_issue.suggestion.as_deref(),
        Some("\n---\n\n:FOOTER:\n:STANDARDS: v2.0\n:LAST_SYNC: pending\n:END:\n")
    );
    assert_eq!(
        footer_issue
            .location
            .as_ref()
            .and_then(|location| location.byte_range),
        Some((index_content.len(), index_content.len()))
    );
}

#[test]
fn detects_incomplete_index_footer_block_for_existing_footer() {
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
    let footer_block = ":FOOTER:\n:STANDARDS: v2.0\n:END:\n";
    let index_content = format!(
        "# Demo\n\n:PROPERTIES:\n:ID: {}\n:END:\n\n## 01_core\n\n- [[01_core/101_demo_core_boundary]]\n\n:RELATIONS:\n:LINKS: [[01_core/101_demo_core_boundary]]\n:END:\n\n---\n\n{footer_block}",
        derive_opaque_doc_id(&index_path_str)
    );
    fs::write(&index_path, &index_content).or_panic("write index");

    let issues = collect_workspace_doc_governance_issues(temp.path(), None);
    let footer_issue = issues
        .iter()
        .find(|issue| issue.issue_type == INCOMPLETE_PACKAGE_DOCS_INDEX_FOOTER_BLOCK_ISSUE_TYPE)
        .or_panic("missing incomplete footer-block issue");

    let footer_start = index_content.find(":FOOTER:").or_panic("find footer start");
    let footer_end = footer_start + footer_block.len();

    assert_eq!(footer_issue.doc, index_path_str);
    assert_eq!(footer_issue.severity, "warning");
    assert!(footer_issue.message.contains(":LAST_SYNC:"));
    assert_eq!(
        footer_issue.suggestion.as_deref(),
        Some(":FOOTER:\n:STANDARDS: v2.0\n:LAST_SYNC: pending\n:END:\n")
    );
    assert_eq!(
        footer_issue
            .location
            .as_ref()
            .and_then(|location| location.byte_range),
        Some((footer_start, footer_end))
    );
}

#[test]
fn detects_stale_index_footer_standards_for_existing_footer() {
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
    let footer_block = ":FOOTER:\n:STANDARDS: v1.0\n:LAST_SYNC: 2026-03-20\n:END:\n";
    let index_content = format!(
        "# Demo\n\n:PROPERTIES:\n:ID: {}\n:END:\n\n## 01_core\n\n- [[01_core/101_demo_core_boundary]]\n\n:RELATIONS:\n:LINKS: [[01_core/101_demo_core_boundary]]\n:END:\n\n---\n\n{footer_block}",
        derive_opaque_doc_id(&index_path_str)
    );
    fs::write(&index_path, &index_content).or_panic("write index");

    let issues = collect_workspace_doc_governance_issues(temp.path(), None);
    let footer_issue = issues
        .iter()
        .find(|issue| issue.issue_type == STALE_PACKAGE_DOCS_INDEX_FOOTER_STANDARDS_ISSUE_TYPE)
        .or_panic("missing stale footer-standards issue");

    let footer_start = index_content.find(":FOOTER:").or_panic("find footer start");
    let footer_end = footer_start + footer_block.len();

    assert_eq!(footer_issue.doc, index_path_str);
    assert_eq!(footer_issue.severity, "warning");
    assert!(footer_issue.message.contains("v1.0"));
    assert_eq!(
        footer_issue.suggestion.as_deref(),
        Some(":FOOTER:\n:STANDARDS: v2.0\n:LAST_SYNC: 2026-03-20\n:END:\n")
    );
    assert_eq!(
        footer_issue
            .location
            .as_ref()
            .and_then(|location| location.byte_range),
        Some((footer_start, footer_end))
    );
}

#[test]
fn run_audit_core_reports_missing_index_footer_block() {
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
            "# Demo\n\n:PROPERTIES:\n:ID: {}\n:END:\n\n## 01_core\n\n- [[01_core/101_demo_core_boundary]]\n\n:RELATIONS:\n:LINKS: [[01_core/101_demo_core_boundary]]\n:END:\n",
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
        issue.issue_type == MISSING_PACKAGE_DOCS_INDEX_FOOTER_BLOCK_ISSUE_TYPE
            && issue
                .doc
                .ends_with("packages/rust/crates/demo/docs/index.md")
    }));
}

#[test]
fn run_audit_core_reports_incomplete_index_footer_block() {
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
            "# Demo\n\n:PROPERTIES:\n:ID: {}\n:END:\n\n## 01_core\n\n- [[01_core/101_demo_core_boundary]]\n\n:RELATIONS:\n:LINKS: [[01_core/101_demo_core_boundary]]\n:END:\n\n---\n\n:FOOTER:\n:STANDARDS: v2.0\n:END:\n",
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
        issue.issue_type == INCOMPLETE_PACKAGE_DOCS_INDEX_FOOTER_BLOCK_ISSUE_TYPE
            && issue
                .doc
                .ends_with("packages/rust/crates/demo/docs/index.md")
    }));
}

#[test]
fn run_audit_core_reports_stale_index_footer_standards() {
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
        doc: Some(".".to_string()),
        checks: Some(vec![CheckType::DocGovernance]),
        include_warnings: Some(true),
        source_paths: None,
        fuzzy_confidence_threshold: None,
        episteme_load: None,
    };
    let (issues, _file_contents) = run_audit_core(&ctx, &args).or_panic("audit");

    assert!(issues.iter().any(|issue| {
        issue.issue_type == STALE_PACKAGE_DOCS_INDEX_FOOTER_STANDARDS_ISSUE_TYPE
            && issue
                .doc
                .ends_with("packages/rust/crates/demo/docs/index.md")
    }));
}

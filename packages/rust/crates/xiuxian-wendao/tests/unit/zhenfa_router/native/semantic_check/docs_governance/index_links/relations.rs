use crate::zhenfa_router::native::semantic_check::docs_governance::tests::support::*;

#[test]
fn relation_line_wikilinks_use_canonical_parser_contract() {
    let links = extract_wikilinks(
        ":LINKS: [[01_core/101_demo_core_boundary|Boundary]], [[01_core/102_demo_contracts#Appendix]], ![[ignored/embed]]",
    );

    assert_eq!(
        links,
        vec![
            "01_core/101_demo_core_boundary|Boundary".to_string(),
            "01_core/102_demo_contracts#Appendix".to_string(),
        ]
    );
}

#[test]
fn detects_missing_index_relation_links_for_existing_body_links() {
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
        "# Demo\n\n:PROPERTIES:\n:ID: {}\n:END:\n\n## 01_core\n\n- [[01_core/101_demo_core_boundary]]\n- [[01_core/102_demo_contracts]]\n\n:RELATIONS:\n:LINKS: [[01_core/101_demo_core_boundary]]\n:END:\n",
        derive_opaque_doc_id(&index_path_str)
    );
    fs::write(&index_path, &index_content).or_panic("write index");

    let issues = collect_workspace_doc_governance_issues(temp.path(), None);
    let relation_issue = issues
        .iter()
        .find(|issue| issue.issue_type == MISSING_PACKAGE_DOCS_INDEX_RELATION_LINK_ISSUE_TYPE)
        .or_panic("missing relation-link issue");

    assert_eq!(relation_issue.doc, index_path_str);
    assert_eq!(relation_issue.severity, "warning");
    assert!(
        relation_issue
            .message
            .contains("[[01_core/102_demo_contracts]]")
    );
    assert_eq!(
        relation_issue.suggestion.as_deref(),
        Some("[[01_core/101_demo_core_boundary]], [[01_core/102_demo_contracts]]")
    );
    let links_value = "[[01_core/101_demo_core_boundary]]";
    let links_line_start = index_content.find(":LINKS: ").or_panic("find links line");
    let value_start = links_line_start
        + ":LINKS: ".len()
        + index_content[links_line_start + ":LINKS: ".len()..]
            .find(links_value)
            .or_panic("find relation links value");
    let value_end = value_start + links_value.len();
    assert_eq!(
        relation_issue
            .location
            .as_ref()
            .and_then(|location| location.byte_range),
        Some((value_start, value_end))
    );
}

#[test]
fn detects_missing_index_relations_block_for_existing_body_links() {
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
        "# Demo\n\n:PROPERTIES:\n:ID: {}\n:END:\n\n## 01_core\n\n- [[01_core/101_demo_core_boundary]]\n\n---\n\n:FOOTER:\n:STANDARDS: v2.0\n:END:\n",
        derive_opaque_doc_id(&index_path_str)
    );
    fs::write(&index_path, &index_content).or_panic("write index");

    let issues = collect_workspace_doc_governance_issues(temp.path(), None);
    let block_issue = issues
        .iter()
        .find(|issue| issue.issue_type == MISSING_PACKAGE_DOCS_INDEX_RELATIONS_BLOCK_ISSUE_TYPE)
        .or_panic("missing relations-block issue");

    assert_eq!(block_issue.doc, index_path_str);
    assert_eq!(block_issue.severity, "warning");
    assert!(
        block_issue
            .message
            .contains("[[01_core/101_demo_core_boundary]]")
    );
    assert_eq!(
        block_issue.suggestion.as_deref(),
        Some(":RELATIONS:\n:LINKS: [[01_core/101_demo_core_boundary]]\n:END:\n\n")
    );
    let insert_offset = index_content.find("---").or_panic("find footer separator");
    assert_eq!(
        block_issue
            .location
            .as_ref()
            .and_then(|location| location.byte_range),
        Some((insert_offset, insert_offset))
    );
}

#[test]
fn run_audit_core_reports_missing_index_relation_links() {
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
            "# Demo\n\n:PROPERTIES:\n:ID: {}\n:END:\n\n## 01_core\n\n- [[01_core/101_demo_core_boundary]]\n\n:RELATIONS:\n:LINKS: \n:END:\n",
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
        issue.issue_type == MISSING_PACKAGE_DOCS_INDEX_RELATION_LINK_ISSUE_TYPE
            && issue
                .doc
                .ends_with("packages/rust/crates/demo/docs/index.md")
    }));
}

#[test]
fn detects_stale_index_relation_links_without_missing_links() {
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
        "# Demo\n\n:PROPERTIES:\n:ID: {}\n:END:\n\n## 01_core\n\n- [[01_core/101_demo_core_boundary]]\n\n:RELATIONS:\n:LINKS: [[01_core/101_demo_core_boundary]], [[01_core/999_stale]]\n:END:\n",
        derive_opaque_doc_id(&index_path_str)
    );
    fs::write(&index_path, &index_content).or_panic("write index");

    let issues = collect_workspace_doc_governance_issues(temp.path(), None);
    let stale_issue = issues
        .iter()
        .find(|issue| issue.issue_type == STALE_PACKAGE_DOCS_INDEX_RELATION_LINK_ISSUE_TYPE)
        .or_panic("missing stale relation-link issue");

    assert_eq!(stale_issue.doc, index_path_str);
    assert_eq!(stale_issue.severity, "warning");
    assert!(stale_issue.message.contains("[[01_core/999_stale]]"));
    assert_eq!(
        stale_issue.suggestion.as_deref(),
        Some("[[01_core/101_demo_core_boundary]]")
    );
    let relation_value = "[[01_core/101_demo_core_boundary]], [[01_core/999_stale]]";
    let links_line_start = index_content.find(":LINKS: ").or_panic("find links line");
    let value_start = links_line_start
        + ":LINKS: ".len()
        + index_content[links_line_start + ":LINKS: ".len()..]
            .find(relation_value)
            .or_panic("find stale relation links value");
    let value_end = value_start + relation_value.len();
    assert_eq!(
        stale_issue
            .location
            .as_ref()
            .and_then(|location| location.byte_range),
        Some((value_start, value_end))
    );
}

#[test]
fn run_audit_core_reports_stale_index_relation_links() {
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
            "# Demo\n\n:PROPERTIES:\n:ID: {}\n:END:\n\n## 01_core\n\n- [[01_core/101_demo_core_boundary]]\n\n:RELATIONS:\n:LINKS: [[01_core/101_demo_core_boundary]], [[01_core/999_stale]]\n:END:\n",
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
        issue.issue_type == STALE_PACKAGE_DOCS_INDEX_RELATION_LINK_ISSUE_TYPE
            && issue
                .doc
                .ends_with("packages/rust/crates/demo/docs/index.md")
    }));
}

#[test]
fn run_audit_core_reports_missing_index_relations_block() {
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
            "# Demo\n\n:PROPERTIES:\n:ID: {}\n:END:\n\n## 01_core\n\n- [[01_core/101_demo_core_boundary]]\n",
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
        issue.issue_type == MISSING_PACKAGE_DOCS_INDEX_RELATIONS_BLOCK_ISSUE_TYPE
            && issue
                .doc
                .ends_with("packages/rust/crates/demo/docs/index.md")
    }));
}

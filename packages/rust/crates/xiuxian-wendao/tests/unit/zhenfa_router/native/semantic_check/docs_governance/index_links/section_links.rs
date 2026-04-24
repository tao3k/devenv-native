use crate::zhenfa_router::native::semantic_check::docs_governance::tests::support::*;

#[test]
fn detects_missing_standard_index_section_links_for_existing_landing_pages() {
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
            "# Demo\n\n:PROPERTIES:\n:ID: {}\n:END:\n\n## 01_core: Architecture and Foundation\n\n",
            derive_opaque_doc_id(&index_path_str)
        ),
    )
    .or_panic("write index");

    let landing_path = core_dir.join("101_demo_core_boundary.md");
    let landing_path_str = landing_path.to_string_lossy().to_string();
    fs::write(
        &landing_path,
        format!(
            "# Core Boundary\n\n:PROPERTIES:\n:ID: {}\n:END:\n",
            derive_opaque_doc_id(&landing_path_str)
        ),
    )
    .or_panic("write landing");

    let issues = collect_workspace_doc_governance_issues(temp.path(), None);
    let link_issue = issues
        .iter()
        .find(|issue| issue.issue_type == MISSING_PACKAGE_DOCS_INDEX_SECTION_LINK_ISSUE_TYPE)
        .or_panic("missing index section-link issue");
    let expected_insert_offset = format!(
        "# Demo\n\n:PROPERTIES:\n:ID: {}\n:END:\n\n## 01_core: Architecture and Foundation\n\n",
        derive_opaque_doc_id(&index_path_str)
    )
    .len();

    assert_eq!(link_issue.doc, index_path_str);
    assert_eq!(link_issue.severity, "warning");
    assert_eq!(
        link_issue.suggestion.as_deref(),
        Some("- [[01_core/101_demo_core_boundary]]\n")
    );
    assert_eq!(
        link_issue
            .location
            .as_ref()
            .and_then(|location| location.byte_range),
        Some((expected_insert_offset, expected_insert_offset))
    );
}

#[test]
fn detects_missing_standard_index_section_links_before_relations_or_footer_when_heading_missing() {
    let temp = TempDir::new().or_panic("tempdir");
    let crate_dir = temp.path().join("packages/rust/crates/demo");
    let feature_dir = crate_dir.join("docs/03_features");
    fs::create_dir_all(&feature_dir).or_panic("create feature docs dir");
    fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .or_panic("write cargo");

    let index_path = crate_dir.join("docs/index.md");
    let index_path_str = index_path.to_string_lossy().to_string();
    let index_content = format!(
        "# Demo\n\n:PROPERTIES:\n:ID: {}\n:END:\n\n## 01_core: Architecture and Foundation\n\n- [[01_core/101_demo_core_boundary]]\n\n:RELATIONS:\n:LINKS: [[01_core/101_demo_core_boundary]]\n:END:\n\n---\n\n:FOOTER:\n:STANDARDS: v2.0\n:LAST_SYNC: 2026-03-20\n:END:\n",
        derive_opaque_doc_id(&index_path_str)
    );
    fs::write(&index_path, &index_content).or_panic("write index");

    let landing_path = feature_dir.join("201_demo_feature_ledger.md");
    let landing_path_str = landing_path.to_string_lossy().to_string();
    fs::write(
        &landing_path,
        format!(
            "# Feature Ledger\n\n:PROPERTIES:\n:ID: {}\n:END:\n",
            derive_opaque_doc_id(&landing_path_str)
        ),
    )
    .or_panic("write landing");

    let issues = collect_workspace_doc_governance_issues(temp.path(), None);
    let link_issue = issues
        .iter()
        .find(|issue| {
            issue.issue_type == MISSING_PACKAGE_DOCS_INDEX_SECTION_LINK_ISSUE_TYPE
                && issue.message.contains("03_features")
        })
        .or_panic("missing feature section-link issue");

    let relations_offset = index_content
        .find(":RELATIONS:")
        .or_panic("find relations block");

    assert_eq!(link_issue.doc, index_path_str);
    assert_eq!(
        link_issue.suggestion.as_deref(),
        Some("## 03_features\n\n- [[03_features/201_demo_feature_ledger]]\n\n")
    );
    assert_eq!(
        link_issue
            .location
            .as_ref()
            .and_then(|location| location.byte_range),
        Some((relations_offset, relations_offset))
    );
}

#[test]
fn run_audit_core_reports_missing_standard_index_section_links() {
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
            "# Demo\n\n:PROPERTIES:\n:ID: {}\n:END:\n\n",
            derive_opaque_doc_id(&index_path_str)
        ),
    )
    .or_panic("write index");

    let landing_path = core_dir.join("101_demo_core_boundary.md");
    let landing_path_str = landing_path.to_string_lossy().to_string();
    fs::write(
        &landing_path,
        format!(
            "# Core Boundary\n\n:PROPERTIES:\n:ID: {}\n:END:\n",
            derive_opaque_doc_id(&landing_path_str)
        ),
    )
    .or_panic("write landing");

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
        issue.issue_type == MISSING_PACKAGE_DOCS_INDEX_SECTION_LINK_ISSUE_TYPE
            && issue
                .doc
                .ends_with("packages/rust/crates/demo/docs/index.md")
    }));
}

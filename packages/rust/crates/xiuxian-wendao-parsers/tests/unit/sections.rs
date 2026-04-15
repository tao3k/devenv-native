use xiuxian_wendao_parsers::sections::{
    SectionCore, SectionMetadata, extract_logbook_entries, extract_property_drawers,
    extract_sections, parse_logbook_entry, parse_property_drawer,
};

#[test]
fn parse_property_drawer_valid() {
    let line = ":ID: arch-v1";
    let result = parse_property_drawer(line);
    assert_eq!(result, Some(("ID".to_string(), "arch-v1".to_string())));
}

#[test]
fn extract_property_drawers_support_org_block_format() {
    let lines = vec![
        ":PROPERTIES:".to_string(),
        ":ID:       uuid-v4-or-slug".to_string(),
        ":STATUS:   STABLE".to_string(),
        ":END:".to_string(),
        "Content starts here".to_string(),
    ];
    let attrs = extract_property_drawers(&lines);
    assert_eq!(attrs.get("ID"), Some(&"uuid-v4-or-slug".to_string()));
    assert_eq!(attrs.get("STATUS"), Some(&"STABLE".to_string()));
}

#[test]
fn parse_logbook_entry_valid() {
    let line = "- [2025-03-14] Agent Started: Initiating structural audit.";
    let entry = parse_logbook_entry(line, 1).unwrap_or_else(|| panic!("expected valid logbook"));
    assert_eq!(entry.timestamp, "2025-03-14");
    assert_eq!(entry.message, "Agent Started: Initiating structural audit.");
    assert_eq!(entry.line_number, 1);
}

#[test]
fn extract_logbook_entries_basic() {
    let lines = vec![
        ":LOGBOOK:".to_string(),
        "- [2025-03-14] Agent Started: Initiating structural audit.".to_string(),
        "- [2025-03-14] Step [audit] completed with status OK.".to_string(),
        ":END:".to_string(),
    ];
    let entries = extract_logbook_entries(&lines, 1);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].timestamp, "2025-03-14");
    assert_eq!(entries[1].message, "Step [audit] completed with status OK.");
}

#[test]
fn extract_sections_keeps_structure_properties_and_logbook() {
    let body = r"# Task: Refactor Authentication
:PROPERTIES:
:ID:       task-auth-001
:STATUS:   RUNNING
:END:

:LOGBOOK:
- [2025-03-14] Agent Started: Initiating structural audit.
- [2025-03-14] Step [audit] completed with status OK.
:END:

Some task content here.
";
    let sections = extract_sections(body);

    assert_eq!(sections.len(), 1);
    let section: &SectionCore = &sections[0];
    let metadata: &SectionMetadata = &section.metadata;
    assert_eq!(section.scope.heading_title, "Task: Refactor Authentication");
    assert_eq!(
        metadata.attributes.get("ID"),
        Some(&"task-auth-001".to_string())
    );
    assert_eq!(
        metadata.attributes.get("STATUS"),
        Some(&"RUNNING".to_string())
    );
    assert_eq!(metadata.logbook.len(), 2);
    assert_eq!(metadata.logbook[0].timestamp, "2025-03-14");
}

#[test]
fn extract_sections_builds_nested_heading_paths_from_comrak_headings() {
    let body = concat!(
        "# Root\n\n",
        "Root body.\n\n",
        "## Child\n\n",
        "Child body.\n\n",
        "### Leaf\n\n",
        "Leaf body.\n",
    );

    let sections = extract_sections(body);

    assert_eq!(sections.len(), 3);
    assert_eq!(sections[0].heading_path(), "Root");
    assert_eq!(sections[1].heading_path(), "Root / Child");
    assert_eq!(sections[2].heading_path(), "Root / Child / Leaf");
}

#[test]
fn extract_sections_ignores_heading_like_lines_inside_code_fences() {
    let body = concat!(
        "# Root\n\n",
        "```md\n",
        "## Not a section\n",
        "```\n\n",
        "Body after fence.\n",
    );

    let sections = extract_sections(body);

    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].heading_title(), "Root");
    assert!(sections[0].section_text.contains("## Not a section"));
}

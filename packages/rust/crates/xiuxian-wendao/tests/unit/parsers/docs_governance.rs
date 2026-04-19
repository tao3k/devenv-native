use crate::parsers::docs_governance::{
    collect_lines, derive_opaque_doc_id, extract_hidden_path_links, is_opaque_doc_id,
    parse_top_properties_drawer,
};

#[test]
fn collect_lines_preserves_offsets_for_terminal_line_without_newline() {
    let lines = collect_lines("# Title\nbody");
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].line_number, 1);
    assert_eq!(lines[0].start_offset, 0);
    assert_eq!(lines[0].end_offset, 8);
    assert_eq!(lines[1].line_number, 2);
    assert_eq!(lines[1].start_offset, 8);
    assert_eq!(lines[1].end_offset, 12);
    assert_eq!(lines[1].without_newline, "body");
    assert_eq!(lines[1].newline, "");
}

#[test]
fn parse_top_properties_drawer_extracts_existing_id_offsets() {
    let content = "# Title\n:PROPERTIES:\n:ID: opaque-doc-id\n:END:\n";
    let drawer = parse_top_properties_drawer(content)
        .unwrap_or_else(|| panic!("expected top properties drawer"));
    let id_line = drawer
        .id_line
        .unwrap_or_else(|| panic!("expected id line inside drawer"));
    assert_eq!(drawer.properties_line, 2);
    assert_eq!(id_line.line, 3);
    assert_eq!(id_line.value, "opaque-doc-id");
    assert_eq!(
        &content[id_line.value_start..id_line.value_end],
        "opaque-doc-id"
    );
}

#[test]
fn extract_hidden_path_links_detects_hidden_targets_in_wikilinks_and_markdown_links() {
    let content = concat!(
        "# Title\n",
        "- [[.cache/agent/execplans/example.md]]\n",
        "- [plan](<.data/private.md>)\n",
    );
    let hidden = extract_hidden_path_links(content);
    assert_eq!(hidden.len(), 2);
    assert_eq!(hidden[0].target, ".cache/agent/execplans/example.md");
    assert_eq!(hidden[1].target, ".data/private.md");
}

#[test]
fn derive_opaque_doc_id_preserves_hash_shaped_protocol() {
    let opaque_id = derive_opaque_doc_id("packages/rust/crates/demo/docs/index.md");
    assert_eq!(opaque_id.len(), 40);
    assert!(is_opaque_doc_id(&opaque_id));
}

#[test]
fn derive_opaque_doc_id_normalizes_repo_prefix_and_path_separators() {
    let relative = derive_opaque_doc_id("packages/rust/crates/demo/docs/index.md");
    let absolute = derive_opaque_doc_id("/tmp/workspace/packages/rust/crates/demo/docs/index.md");
    let windowsish =
        derive_opaque_doc_id(r"C:\tmp\workspace\packages\rust\crates\demo\docs\index.md");

    assert_eq!(relative, absolute);
    assert_eq!(relative, windowsish);
}

use super::{NodeStatus, generate_suggested_id, issue_type_to_code, xml_escape};

#[test]
fn test_xml_escape() {
    assert_eq!(xml_escape("a<b>c&d"), "a&lt;b&gt;c&amp;d");
    assert_eq!(xml_escape("\"quoted\""), "&quot;quoted&quot;");
}

#[test]
fn test_node_status_parsing() {
    assert_eq!(NodeStatus::parse_lossy("STABLE"), NodeStatus::Stable);
    assert_eq!(NodeStatus::parse_lossy("stable"), NodeStatus::Stable);
    assert_eq!(NodeStatus::parse_lossy("DRAFT"), NodeStatus::Draft);
    assert_eq!(
        NodeStatus::parse_lossy("DEPRECATED"),
        NodeStatus::Deprecated
    );
    assert_eq!(NodeStatus::parse_lossy("UNKNOWN"), NodeStatus::Stable);
}

#[test]
fn test_generate_suggested_id() {
    assert_eq!(
        generate_suggested_id("Architecture Overview"),
        "architecture-overview"
    );
    assert_eq!(generate_suggested_id("API Reference!"), "api-reference");
    assert_eq!(generate_suggested_id("  Test  "), "test");
}

#[test]
fn test_issue_type_to_code() {
    assert_eq!(issue_type_to_code("dead_link"), "ERR_DEAD_LINK");
    assert_eq!(issue_type_to_code("deprecated_ref"), "WARN_DEPRECATED_REF");
    assert_eq!(
        issue_type_to_code("contract_violation"),
        "ERR_CONTRACT_VIOLATION"
    );
    assert_eq!(issue_type_to_code("id_collision"), "ERR_DUPLICATE_ID");
    assert_eq!(
        issue_type_to_code("missing_identity"),
        "ERR_MISSING_IDENTITY"
    );
    assert_eq!(issue_type_to_code("legacy_syntax"), "WARN_LEGACY_SYNTAX");
    assert_eq!(
        issue_type_to_code("invalid_observation_pattern"),
        "ERR_INVALID_OBSERVER_PATTERN"
    );
    assert_eq!(
        issue_type_to_code("doc_identity_protocol"),
        "ERR_DOC_IDENTITY_PROTOCOL"
    );
    assert_eq!(
        issue_type_to_code("canonical_doc_hidden_path_link"),
        "WARN_CANONICAL_DOC_HIDDEN_PATH_LINK"
    );
    assert_eq!(
        issue_type_to_code("missing_package_docs_tree"),
        "WARN_MISSING_PACKAGE_DOCS_TREE"
    );
    assert_eq!(
        issue_type_to_code("missing_package_docs_index"),
        "ERR_MISSING_PACKAGE_DOCS_INDEX"
    );
    assert_eq!(
        issue_type_to_code("missing_package_docs_section_landing"),
        "WARN_MISSING_PACKAGE_DOCS_SECTION"
    );
    assert_eq!(
        issue_type_to_code("missing_package_docs_index_section_link"),
        "WARN_MISSING_PACKAGE_DOCS_INDEX_LINK"
    );
    assert_eq!(
        issue_type_to_code("missing_package_docs_index_relations_block"),
        "WARN_MISSING_PACKAGE_DOCS_RELATIONS_BLOCK"
    );
    assert_eq!(
        issue_type_to_code("missing_package_docs_index_footer_block"),
        "WARN_MISSING_PACKAGE_DOCS_FOOTER_BLOCK"
    );
    assert_eq!(
        issue_type_to_code("incomplete_package_docs_index_footer_block"),
        "WARN_INCOMPLETE_PACKAGE_DOCS_FOOTER_BLOCK"
    );
    assert_eq!(
        issue_type_to_code("stale_package_docs_index_footer_standards"),
        "WARN_STALE_PACKAGE_DOCS_FOOTER_STANDARDS"
    );
    assert_eq!(
        issue_type_to_code("missing_package_docs_index_relation_link"),
        "WARN_MISSING_PACKAGE_DOCS_RELATION_LINK"
    );
    assert_eq!(
        issue_type_to_code("stale_package_docs_index_relation_link"),
        "WARN_STALE_PACKAGE_DOCS_RELATION_LINK"
    );
    assert_eq!(issue_type_to_code("unknown"), "UNKNOWN");
}

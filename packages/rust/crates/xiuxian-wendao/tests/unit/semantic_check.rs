//! Tests for semantic_check module (Blueprint v2.2).
//!
//! Tests the semantic sentinel functionality:
//! - Dead link detection
//! - Deprecated reference warnings
//! - Contract validation
//! - ID collision detection
//! - Hash alignment checks
//! - Missing identity warnings
//! - Legacy syntax detection
//! - Health score calculation

use xiuxian_wendao::zhenfa_router::native::semantic_check::test_api::{
    NodeStatus, SemanticIssue, build_file_reports, extract_function_args, extract_hash_references,
    extract_id_references, generate_suggested_id, issue_type_to_code, validate_contract,
    xml_escape,
};

// =============================================================================
// ID Reference Extraction Tests
// =============================================================================

#[test]
fn test_extract_id_references() {
    let text = "See [[#intro]] and [[#architecture]] for details.";
    let refs = extract_id_references(text);
    assert_eq!(refs, vec!["#intro", "#architecture"]);
}

#[test]
fn test_extract_id_references_no_match() {
    let text = "No wiki links here, just [[regular-link]] text.";
    let refs = extract_id_references(text);
    assert!(refs.is_empty());
}

// =============================================================================
// Hash Reference Extraction Tests
// =============================================================================

#[test]
fn test_extract_hash_references_with_hash() {
    let text = "See [[#arch-v1@abc123]] for the architecture.";
    let refs = extract_hash_references(text);
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].target_id, "arch-v1");
    assert_eq!(refs[0].expect_hash, Some("abc123".to_string()));
}

#[test]
fn test_extract_hash_references_without_hash() {
    let text = "See [[#intro]] for the introduction.";
    let refs = extract_hash_references(text);
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].target_id, "intro");
    assert_eq!(refs[0].expect_hash, None);
}

#[test]
fn test_extract_hash_references_mixed() {
    let text = "See [[#arch-v1@abc123]] and [[#intro]] and [[#config@def456]].";
    let refs = extract_hash_references(text);
    assert_eq!(refs.len(), 3);
    assert_eq!(refs[0].target_id, "arch-v1");
    assert_eq!(refs[0].expect_hash, Some("abc123".to_string()));
    assert_eq!(refs[1].target_id, "intro");
    assert_eq!(refs[1].expect_hash, None);
    assert_eq!(refs[2].target_id, "config");
    assert_eq!(refs[2].expect_hash, Some("def456".to_string()));
}

#[test]
fn test_extract_hash_references_empty() {
    let text = "No hash-annotated references here.";
    let refs = extract_hash_references(text);
    assert!(refs.is_empty());
}

// =============================================================================
// Contract Validation Tests
// =============================================================================

#[test]
fn test_validate_contract_must_contain() {
    let content = "This document describes Rust and Lock mechanisms.";
    assert!(validate_contract("must_contain(\"Rust\", \"Lock\")", content).is_none());
    assert!(validate_contract("must_contain(\"Python\")", content).is_some());
}

#[test]
fn test_validate_contract_must_not_contain() {
    let content = "This is a stable API.";
    assert!(validate_contract("must_not_contain(\"deprecated\")", content).is_none());
    assert!(validate_contract("must_not_contain(\"stable\")", content).is_some());
}

#[test]
fn test_validate_contract_min_length() {
    let content = "Short";
    assert!(validate_contract("min_length(3)", content).is_none());
    assert!(validate_contract("min_length(100)", content).is_some());
}

#[test]
fn test_extract_function_args() {
    assert_eq!(
        extract_function_args("must_contain(\"Rust\", \"Lock\")", "must_contain"),
        Some("\"Rust\", \"Lock\"")
    );
    assert_eq!(
        extract_function_args("min_length(100)", "min_length"),
        Some("100")
    );
    assert_eq!(extract_function_args("unknown()", "must_contain"), None);
}

// =============================================================================
// Helper Function Tests
// =============================================================================

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
        issue_type_to_code("doc_identity_protocol"),
        "ERR_DOC_IDENTITY_PROTOCOL"
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

// =============================================================================
// Health Score Tests
// =============================================================================

#[test]
fn test_build_file_reports() {
    let issues = vec![
        SemanticIssue {
            severity: "error".to_string(),
            issue_type: "dead_link".to_string(),
            doc: "doc1.md".to_string(),
            node_id: "node1".to_string(),
            message: "Test error".to_string(),
            location: None,
            suggestion: None,
        },
        SemanticIssue {
            severity: "warning".to_string(),
            issue_type: "legacy_syntax".to_string(),
            doc: "doc1.md".to_string(),
            node_id: "node2".to_string(),
            message: "Test warning".to_string(),
            location: None,
            suggestion: None,
        },
        SemanticIssue {
            severity: "error".to_string(),
            issue_type: "dead_link".to_string(),
            doc: "doc2.md".to_string(),
            node_id: "node3".to_string(),
            message: "Another error".to_string(),
            location: None,
            suggestion: None,
        },
    ];

    let docs = vec!["doc1.md".to_string(), "doc2.md".to_string()];
    let reports = build_file_reports(&issues, &docs);

    assert_eq!(reports.len(), 2);

    // doc1.md: 1 error, 1 warning -> 100 - 20 - 5 = 75
    assert_eq!(reports[0].path, "doc1.md");
    assert_eq!(reports[0].error_count, 1);
    assert_eq!(reports[0].warning_count, 1);
    assert_eq!(reports[0].health_score, 75);

    // doc2.md: 1 error, 0 warnings -> 100 - 20 = 80
    assert_eq!(reports[1].path, "doc2.md");
    assert_eq!(reports[1].error_count, 1);
    assert_eq!(reports[1].warning_count, 0);
    assert_eq!(reports[1].health_score, 80);
}

#[test]
fn test_health_score_bounds() {
    // Test that health score doesn't go below 0
    let issues: Vec<SemanticIssue> = (0..10)
        .map(|_| SemanticIssue {
            severity: "error".to_string(),
            issue_type: "dead_link".to_string(),
            doc: "doc.md".to_string(),
            node_id: "node".to_string(),
            message: "Error".to_string(),
            location: None,
            suggestion: None,
        })
        .collect();

    let docs = vec!["doc.md".to_string()];
    let reports = build_file_reports(&issues, &docs);

    // 10 errors * 20 = 200 penalty, but score should be 0 (not negative)
    assert_eq!(reports[0].health_score, 0);
}

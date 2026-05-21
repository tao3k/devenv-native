//! `zhenfa_router::native::semantic_check::docs_governance::collection::package_docs` owns Wendao docs governance collection package docs behavior.

use super::footer::collect_stale_index_footer_standards;
use super::hidden_links::collect_hidden_path_link_issues;
use super::relations::collect_stale_index_relation_links;
use crate::parsers::docs_governance::{
    derive_opaque_doc_id, is_opaque_doc_id, is_package_local_crate_doc, parse_top_properties_drawer,
};
use crate::zhenfa_router::native::semantic_check::docs_governance::types::DOC_IDENTITY_PROTOCOL_ISSUE_TYPE;
use crate::zhenfa_router::native::semantic_check::{IssueLocation, SemanticIssue};

/// Collects doc governance issues for a single document.
#[must_use]
pub fn collect_doc_governance_issues(doc_path: &str, content: &str) -> Vec<SemanticIssue> {
    let mut issues = collect_hidden_path_link_issues(doc_path, content);

    if !is_package_local_crate_doc(doc_path) {
        return issues;
    }

    let expected_id = derive_opaque_doc_id(doc_path);
    let Some(top_drawer) = parse_top_properties_drawer(content) else {
        return issues;
    };

    if let Some(existing_id) = top_drawer.id_line {
        if !is_opaque_doc_id(existing_id.value) {
            issues.push(SemanticIssue {
                severity: "error".to_string(),
                issue_type: DOC_IDENTITY_PROTOCOL_ISSUE_TYPE.to_string(),
                doc: doc_path.to_string(),
                node_id: doc_path.to_string(),
                message: format!(
                    "Top-level :ID: in package-local crate docs must be an opaque hash-shaped identifier, found '{}'",
                    existing_id.value
                ),
                location: Some(IssueLocation {
                    line: existing_id.line,
                    heading_path: "Document Identity".to_string(),
                    byte_range: Some((existing_id.value_start, existing_id.value_end)),
                }),
                suggestion: Some(expected_id),
                fuzzy_suggestion: None,
            });
        }
    } else {
        issues.push(SemanticIssue {
            severity: "error".to_string(),
            issue_type: DOC_IDENTITY_PROTOCOL_ISSUE_TYPE.to_string(),
            doc: doc_path.to_string(),
            node_id: doc_path.to_string(),
            message: "Top-level :ID: is missing from the package-local crate docs property drawer"
                .to_string(),
            location: Some(IssueLocation {
                line: top_drawer.properties_line + 1,
                heading_path: "Document Identity".to_string(),
                byte_range: Some((top_drawer.insert_offset, top_drawer.insert_offset)),
            }),
            suggestion: Some(format!(":ID: {expected_id}{}", top_drawer.newline)),
            fuzzy_suggestion: None,
        });
    }

    issues.extend(collect_stale_index_footer_standards(doc_path, content));
    issues.extend(collect_stale_index_relation_links(doc_path, content));

    issues
}

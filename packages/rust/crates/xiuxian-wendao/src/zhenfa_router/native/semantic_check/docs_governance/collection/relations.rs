//! `zhenfa_router::native::semantic_check::docs_governance::collection::relations` owns Wendao docs governance collection relations behavior.

use crate::parsers::docs_governance::{
    collect_index_body_links, collect_lines, extract_wikilinks, parse_relations_links_line,
};
use crate::zhenfa_router::native::semantic_check::docs_governance::types::STALE_PACKAGE_DOCS_INDEX_RELATION_LINK_ISSUE_TYPE;
use crate::zhenfa_router::native::semantic_check::{IssueLocation, SemanticIssue};

/// Collect relation-link issues where links listed in the index `:RELATIONS:` block
/// no longer match the links present in the document body.
#[must_use]
pub fn collect_stale_index_relation_links(doc_path: &str, content: &str) -> Vec<SemanticIssue> {
    let lines = collect_lines(content);
    let mut issues = Vec::new();

    if let Some(links_line) = parse_relations_links_line(&lines) {
        let links_in_relations = extract_wikilinks(links_line.value);
        let links_in_body = collect_index_body_links(&lines);

        let stale_links = links_in_relations
            .iter()
            .filter(|l| !links_in_body.contains(l))
            .cloned()
            .collect::<Vec<_>>();

        if !stale_links.is_empty() {
            issues.push(SemanticIssue {
                severity: "warning".to_string(),
                issue_type: STALE_PACKAGE_DOCS_INDEX_RELATION_LINK_ISSUE_TYPE.to_string(),
                doc: doc_path.to_string(),
                node_id: doc_path.to_string(),
                message: format!(
                    "Documentation links in :RELATIONS: block are no longer present in body: {}",
                    stale_links
                        .iter()
                        .map(|l| format!("[[{l}]]"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                location: Some(IssueLocation {
                    line: links_line.line,
                    heading_path: "Index Relations".to_string(),
                    byte_range: Some((links_line.value_start, links_line.value_end)),
                }),
                suggestion: Some(
                    links_in_body
                        .iter()
                        .map(|l| format!("[[{l}]]"))
                        .collect::<Vec<_>>()
                        .join(", "),
                ),
                fuzzy_suggestion: None,
            });
        }
    }

    issues
}

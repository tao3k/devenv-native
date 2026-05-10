use std::path::Path;

use serde_json::Value;

pub(super) fn projected_page_id(repo_id: &str, doc_id: &str, source_path: &str) -> String {
    if doc_id.contains(":projection:") {
        return doc_id.to_owned();
    }
    let projection_kind = projection_kind_token_for_source_path(source_path);
    let effective_doc_id = if doc_id.trim().is_empty() {
        format!("repo:{repo_id}:doc:{source_path}")
    } else if doc_id.starts_with("repo:") {
        doc_id.to_owned()
    } else if !source_path.trim().is_empty() {
        format!("repo:{repo_id}:doc:{source_path}")
    } else {
        format!("repo:{repo_id}:doc:{doc_id}")
    };
    format!("repo:{repo_id}:projection:{projection_kind}:doc:{effective_doc_id}")
}

pub(super) fn graph_node_display_id_candidates(repo_id: &str, source_path: &str) -> Vec<String> {
    let mut candidates = vec![graph_node_display_id(repo_id, source_path)];
    for surrogate in markdown_surrogate_source_paths(source_path) {
        push_unique(
            &mut candidates,
            graph_node_display_id(repo_id, surrogate.as_str()),
        );
    }
    candidates
}

fn graph_node_display_id(repo_id: &str, source_path: &str) -> String {
    let normalized = source_path.trim().trim_matches('/');
    if normalized.starts_with(format!("{repo_id}/").as_str()) {
        normalized.to_owned()
    } else if normalized.is_empty() {
        repo_id.to_owned()
    } else {
        format!("{repo_id}/{normalized}")
    }
}

fn markdown_surrogate_source_paths(source_path: &str) -> Vec<String> {
    let normalized = source_path.trim().trim_matches('/');
    if normalized.is_empty() || has_markdown_extension(normalized) {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    push_unique(&mut candidates, format!("{normalized}.md"));
    if let Some((stem, _extension)) = normalized.rsplit_once('.') {
        push_unique(&mut candidates, format!("{stem}.md"));
    }
    candidates
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn projection_kind_token_for_source_path(source_path: &str) -> &'static str {
    if has_markdown_extension(source_path) {
        "explanation"
    } else {
        "reference"
    }
}

fn has_markdown_extension(path: &str) -> bool {
    Path::new(path.trim()).extension().is_some_and(|extension| {
        extension.eq_ignore_ascii_case("md") || extension.eq_ignore_ascii_case("markdown")
    })
}

pub(super) fn repo_relative_source_path(repo_id: &str, source_path: &str) -> String {
    let normalized = source_path.trim().trim_matches('/');
    let repo_prefix = format!("{}/", repo_id.trim().trim_matches('/'));
    normalized
        .strip_prefix(repo_prefix.as_str())
        .unwrap_or(normalized)
        .to_owned()
}

pub(super) fn normalized_repo_search_doc_id(
    repo_id: &str,
    repo_relative_path: &str,
    doc_id: Option<&str>,
) -> String {
    let repo_id = repo_id.trim().trim_matches('/');
    let repo_relative_path = repo_relative_path.trim().trim_matches('/');
    let canonical_doc_id = || format!("repo:{repo_id}:doc:{repo_relative_path}");
    let Some(doc_id) = doc_id.map(str::trim).filter(|value| !value.is_empty()) else {
        return canonical_doc_id();
    };
    let repo_display_doc_prefix = format!("repo:{repo_id}:doc:{repo_id}/");
    if let Some(relative_path) = doc_id.strip_prefix(repo_display_doc_prefix.as_str()) {
        return format!("repo:{repo_id}:doc:{relative_path}");
    }
    if doc_id.starts_with("repo:") {
        doc_id.to_owned()
    } else {
        canonical_doc_id()
    }
}

pub(super) fn find_node_id_by_anchor_or_title(value: &Value, anchor: &str) -> Option<String> {
    match value {
        Value::Array(nodes) => nodes
            .iter()
            .find_map(|node| find_node_id_by_anchor_or_title(node, anchor)),
        Value::Object(node) => {
            let expected_slug = normalize_anchor(anchor);
            if ["anchor", "headingAnchor", "slug"]
                .iter()
                .filter_map(|key| node.get(*key).and_then(Value::as_str))
                .any(|value| normalize_anchor(value) == expected_slug)
                || node
                    .get("title")
                    .and_then(Value::as_str)
                    .is_some_and(|title| normalize_anchor(title) == expected_slug)
                || node
                    .get("node_id")
                    .and_then(Value::as_str)
                    .is_some_and(|node_id| node_id.ends_with(anchor))
            {
                return node
                    .get("node_id")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned);
            }
            node.get("children")
                .and_then(|children| find_node_id_by_anchor_or_title(children, anchor))
        }
        _ => None,
    }
}

pub(super) fn first_node_id(value: &Value) -> Option<String> {
    match value {
        Value::Array(nodes) => nodes.iter().find_map(first_node_id),
        Value::Object(node) => node
            .get("node_id")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .or_else(|| node.get("children").and_then(first_node_id)),
        _ => None,
    }
}

fn normalize_anchor(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
#[path = "../../../tests/unit/integration_support/search_strategy_flow_flight/ids.rs"]
mod tests;

//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use std::collections::BTreeMap;
use std::path::Path;

use xiuxian_db_store::VectorStoreError;

use crate::analyzers::{ImportKind, RepoBacklinkItem, RepoSymbolKind};
use crate::search::contracts::{SearchBacklinkItem, SearchHit, StudioNavigationTarget};

pub(crate) fn serialize_hit_json(hit: &SearchHit) -> Result<String, VectorStoreError> {
    serde_json::to_string(hit)
        .map_err(|error| VectorStoreError::General(format!("serialize repo entity hit: {error}")))
}

pub(crate) fn serialize_symbol_attributes_json(
    attributes: &BTreeMap<String, String>,
) -> Result<Option<String>, VectorStoreError> {
    if attributes.is_empty() {
        return Ok(None);
    }
    serde_json::to_string(attributes)
        .map(Some)
        .map_err(|error| {
            VectorStoreError::General(format!("serialize repo entity symbol attributes: {error}"))
        })
}

pub(crate) fn serialize_backlink_items_json(
    items: Option<&Vec<SearchBacklinkItem>>,
) -> Result<Option<String>, VectorStoreError> {
    let Some(items) = items else {
        return Ok(None);
    };
    if items.is_empty() {
        return Ok(None);
    }
    serde_json::to_string(items).map(Some).map_err(|error| {
        VectorStoreError::General(format!("serialize repo entity backlink items: {error}"))
    })
}

pub(crate) fn repo_entity_tags(
    repo_id: &str,
    entity_kind: &str,
    language: Option<String>,
    normalized_kind: Option<&str>,
    audit_status: Option<&str>,
) -> Vec<String> {
    let mut tags = vec![
        repo_id.to_string(),
        "code".to_string(),
        entity_kind.to_string(),
    ];
    if let Some(kind) = normalized_kind {
        tags.push(format!("kind:{kind}"));
    }
    if let Some(language) = language {
        tags.push(language.clone());
        tags.push(format!("lang:{language}"));
    }
    if let Some(audit_status) = audit_status {
        tags.push(audit_status.to_string());
    }
    tags
}

pub(crate) fn map_backlink_items(
    items: Option<Vec<RepoBacklinkItem>>,
) -> Option<Vec<SearchBacklinkItem>> {
    items.map(|items| {
        items
            .into_iter()
            .map(|item| SearchBacklinkItem {
                id: item.id,
                title: item.title,
                path: item.path,
                kind: item.kind,
            })
            .collect()
    })
}

pub(crate) fn repo_navigation_target(
    repo_id: &str,
    path: &str,
    line: Option<usize>,
    line_end: Option<usize>,
) -> StudioNavigationTarget {
    let normalized_path = path.replace('\\', "/");
    let path = if normalized_path.starts_with(&format!("{repo_id}/")) {
        normalized_path
    } else {
        format!("{repo_id}/{normalized_path}")
    };
    StudioNavigationTarget {
        path,
        category: "repo_code".to_string(),
        project_name: Some(repo_id.to_string()),
        root_label: Some(repo_id.to_string()),
        line,
        line_end,
        column: None,
    }
}

pub(crate) fn infer_code_language(path: &str) -> Option<String> {
    if path_has_extension(path, "jl") {
        return Some("julia".to_string());
    }
    if path_has_extension(path, "mo") {
        return Some("modelica".to_string());
    }
    if path_has_extension(path, "rs") {
        return Some("rust".to_string());
    }
    if path_has_extension(path, "py") {
        return Some("python".to_string());
    }
    if path_has_extension(path, "ts") || path_has_extension(path, "tsx") {
        return Some("typescript".to_string());
    }
    None
}

pub(crate) fn path_has_extension(path: &str, expected: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(expected))
}

pub(crate) fn symbol_kind_tag(kind: RepoSymbolKind) -> &'static str {
    match kind {
        RepoSymbolKind::Function => "function",
        RepoSymbolKind::Type => "type",
        RepoSymbolKind::Constant => "constant",
        RepoSymbolKind::ModuleExport => "module_export",
        RepoSymbolKind::Other => "other",
    }
}

pub(crate) fn import_kind_tag(kind: ImportKind) -> &'static str {
    match kind {
        ImportKind::Symbol => "symbol",
        ImportKind::Module => "module",
        ImportKind::Reexport => "reexport",
    }
}

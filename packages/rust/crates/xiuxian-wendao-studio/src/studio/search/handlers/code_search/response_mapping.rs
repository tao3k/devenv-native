#[cfg(test)]
use crate::studio::types::{SearchBacklinkItem, StudioNavigationTarget};
#[cfg(test)]
use xiuxian_wendao::analyzers::{RepoBacklinkItem, RepoSymbolKind, SymbolSearchHit};

#[cfg(test)]
pub(crate) fn symbol_search_hit_to_search_hit(
    repo_id: &str,
    hit: SymbolSearchHit,
) -> crate::studio::types::SearchHit {
    let language = infer_code_language(hit.symbol.path.as_str());
    let kind = symbol_kind_tag(hit.symbol.kind);
    let mut tags = vec![
        repo_id.to_string(),
        "code".to_string(),
        "symbol".to_string(),
        format!("kind:{kind}"),
    ];
    if let Some(language) = language.as_deref() {
        tags.push(language.to_string());
        tags.push(format!("lang:{language}"));
    }
    if let Some(status) = hit.audit_status.clone() {
        tags.push(status);
    }

    crate::studio::types::SearchHit {
        stem: hit.symbol.name.clone(),
        title: Some(hit.symbol.qualified_name.clone()),
        path: hit.symbol.path.clone(),
        doc_type: Some("symbol".to_string()),
        tags,
        score: hit.saliency_score.or(hit.score).unwrap_or(0.0),
        best_section: hit
            .symbol
            .signature
            .clone()
            .or_else(|| Some(hit.symbol.qualified_name.clone())),
        match_reason: Some("repo_symbol_search".to_string()),
        hierarchical_uri: hit.hierarchical_uri,
        hierarchy: hit.hierarchy,
        saliency_score: hit.saliency_score,
        audit_status: hit.audit_status,
        verification_state: hit.verification_state,
        implicit_backlinks: hit.implicit_backlinks,
        implicit_backlink_items: map_backlink_items(hit.implicit_backlink_items),
        navigation_target: Some(repo_navigation_target(
            repo_id,
            hit.symbol.path.as_str(),
            Some("repo_code".to_string()),
            hit.symbol.line_start,
            hit.symbol.line_end,
        )),
    }
}

#[cfg(test)]
fn map_backlink_items(items: Option<Vec<RepoBacklinkItem>>) -> Option<Vec<SearchBacklinkItem>> {
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

#[cfg(test)]
pub(crate) fn repo_navigation_target(
    repo_id: &str,
    path: &str,
    category: Option<String>,
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
        category: category.unwrap_or_else(|| "repo_code".to_string()),
        project_name: Some(repo_id.to_string()),
        root_label: Some(repo_id.to_string()),
        line,
        line_end,
        column: None,
    }
}

#[cfg(test)]
fn infer_code_language(path: &str) -> Option<String> {
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

#[cfg(test)]
fn path_has_extension(path: &str, expected: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case(expected))
}

#[cfg(test)]
fn symbol_kind_tag(kind: RepoSymbolKind) -> &'static str {
    match kind {
        RepoSymbolKind::Function => "function",
        RepoSymbolKind::Type => "type",
        RepoSymbolKind::Constant => "constant",
        RepoSymbolKind::ModuleExport => "module_export",
        RepoSymbolKind::Other => "other",
    }
}

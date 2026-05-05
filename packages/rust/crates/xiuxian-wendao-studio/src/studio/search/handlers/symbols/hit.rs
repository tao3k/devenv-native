use std::path::Path;

use crate::studio::search::project_scope::project_metadata_for_path;
use crate::studio::search::support::source_language_label;
use crate::studio::types::{
    AstSearchHit, StudioNavigationTarget, SymbolSearchHit, UiProjectConfig,
};

pub(super) fn symbol_search_hit(
    project_root: &Path,
    config_root: &Path,
    projects: &[UiProjectConfig],
    hit: &AstSearchHit,
) -> Option<SymbolSearchHit> {
    let path = hit.path.clone();
    let language = source_language_label(Path::new(path.as_str()))?.to_string();
    let kind = symbol_kind(hit)?;
    let line = hit.line_start;
    let metadata = project_metadata_for_path(project_root, config_root, projects, path.as_str());

    Some(SymbolSearchHit {
        name: hit.name.clone(),
        kind,
        path: path.clone(),
        line,
        location: format!("{path}:{line}"),
        language,
        source: "project".to_string(),
        crate_name: hit.crate_name.clone(),
        project_name: metadata.project_name.clone(),
        root_label: metadata.root_label.clone(),
        navigation_target: StudioNavigationTarget {
            path,
            category: "doc".to_string(),
            project_name: metadata.project_name,
            root_label: metadata.root_label,
            line: Some(line),
            line_end: Some(line),
            column: None,
        },
        score: 0.95,
    })
}

fn symbol_kind(hit: &AstSearchHit) -> Option<String> {
    match hit.node_kind.as_deref() {
        Some("section" | "task" | "property" | "observation") => return None,
        Some(kind) if !kind.trim().is_empty() => return Some(kind.to_string()),
        _ => {}
    }

    let signature = hit.signature.trim_start();
    let tokens = signature.split_whitespace().collect::<Vec<_>>();
    if tokens.is_empty() {
        return None;
    }

    if tokens.windows(2).any(|window| window == ["pub", "struct"]) || tokens[0] == "struct" {
        return Some("struct".to_string());
    }
    if tokens.windows(2).any(|window| window == ["pub", "enum"]) || tokens[0] == "enum" {
        return Some("enum".to_string());
    }
    if tokens.windows(2).any(|window| window == ["pub", "trait"]) || tokens[0] == "trait" {
        return Some("trait".to_string());
    }
    if tokens.windows(2).any(|window| window == ["pub", "fn"])
        || tokens[0] == "fn"
        || tokens[0] == "def"
        || (tokens.len() >= 2 && tokens[0] == "async" && tokens[1] == "def")
    {
        return Some("function".to_string());
    }
    if tokens[0] == "class" {
        return Some("struct".to_string());
    }
    if tokens.windows(2).any(|window| window == ["pub", "mod"]) || tokens[0] == "mod" {
        return Some("module".to_string());
    }
    if tokens.windows(2).any(|window| window == ["pub", "const"]) || tokens[0] == "const" {
        return Some("const".to_string());
    }
    if tokens.windows(2).any(|window| window == ["pub", "static"]) || tokens[0] == "static" {
        return Some("static".to_string());
    }
    if tokens.windows(2).any(|window| window == ["pub", "type"]) || tokens[0] == "type" {
        return Some("type_alias".to_string());
    }
    if tokens[0] == "impl" {
        return Some("impl".to_string());
    }

    Some("unknown".to_string())
}

use std::collections::HashSet;
use std::path::Path;
use walkdir::WalkDir;

use crate::studio::types::AstSearchHit;
use crate::studio::types::UiProjectConfig;

use super::filters::{is_markdown_path, should_skip_entry};
use super::markdown::{build_markdown_ast_hits, markdown_scope_name};
use crate::studio::search::project_scope::{configured_project_scan_roots, index_path_for_entry};

pub(crate) fn build_ast_index(
    project_root: &Path,
    config_root: &Path,
    projects: &[UiProjectConfig],
) -> Vec<AstSearchHit> {
    let hits = configured_project_scan_roots(config_root, projects)
        .into_iter()
        .flat_map(|root| ast_hits_for_scan_root(project_root, root.as_path()))
        .collect::<Vec<_>>();
    dedupe_ast_hits(hits)
}

pub(crate) fn build_ast_hits_for_file(
    project_root: &Path,
    scan_root: &Path,
    source_path: &Path,
) -> Vec<AstSearchHit> {
    let normalized_path = index_path_for_entry(project_root, source_path);
    let normalized_path_ref = Path::new(normalized_path.as_str());
    let Ok(content) = std::fs::read_to_string(source_path) else {
        return Vec::new();
    };
    if is_markdown_path(normalized_path_ref) {
        return build_markdown_ast_hits_for_content(
            scan_root,
            source_path,
            normalized_path.as_str(),
            &content,
        );
    }
    build_code_ast_hits_from_content(normalized_path.as_str(), &content)
}

fn build_markdown_ast_hits_for_content(
    scan_root: &Path,
    source_path: &Path,
    normalized_path: &str,
    content: &str,
) -> Vec<AstSearchHit> {
    let normalized_path_ref = Path::new(normalized_path);
    if !is_markdown_path(normalized_path_ref) {
        return Vec::new();
    }
    let crate_name = markdown_scope_name(normalized_path_ref);
    let mut seen = HashSet::new();
    build_markdown_ast_hits(
        scan_root,
        source_path,
        normalized_path,
        content,
        crate_name.as_str(),
    )
    .into_iter()
    .filter(|hit| {
        seen.insert(format!(
            "{}:{}:{}:{}",
            hit.path, hit.line_start, hit.line_end, hit.name
        ))
    })
    .collect()
}

fn build_code_ast_hits_from_content(_normalized_path: &str, _content: &str) -> Vec<AstSearchHit> {
    Vec::new()
}

fn ast_hits_for_scan_root(project_root: &Path, scan_root: &Path) -> Vec<AstSearchHit> {
    WalkDir::new(scan_root)
        .into_iter()
        .filter_entry(|entry| !should_skip_entry(entry))
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .flat_map(|entry| build_ast_hits_for_file(project_root, scan_root, entry.path()))
        .collect()
}

fn dedupe_ast_hits(hits: Vec<AstSearchHit>) -> Vec<AstSearchHit> {
    let mut seen = HashSet::new();
    hits.into_iter()
        .filter(|hit| seen.insert(ast_hit_dedupe_key(hit)))
        .collect()
}

fn ast_hit_dedupe_key(hit: &AstSearchHit) -> String {
    format!(
        "{}:{}:{}:{}",
        hit.path, hit.line_start, hit.line_end, hit.name
    )
}

//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use std::path::{Component, Path, PathBuf};

use walkdir::DirEntry;
use xiuxian_ast::Lang;
use xiuxian_code_intelligence::{
    extract_code_structure_symbols, supported_code_language_from_path,
};
use xiuxian_wendao_parsers::sections::MarkdownSection;

use crate::parsers::markdown::extract_observations;

use super::{
    AnalysisNode, AnalysisNodeKind, AstSearchHit, SearchProjectConfig, StudioNavigationTarget,
};

#[derive(Debug, Clone, Default)]
pub(crate) struct SearchProjectMetadata {
    pub(crate) project_name: Option<String>,
    pub(crate) root_label: Option<String>,
}

pub(crate) fn resolve_project_root_path(
    config_root: &Path,
    configured_root: &str,
) -> Option<PathBuf> {
    resolve_path_like(config_root, configured_root)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConfiguredProjectScope {
    pub(crate) scope_path: PathBuf,
    pub(crate) normalized_scope: String,
    pub(crate) project_name: String,
    pub(crate) root_label: Option<String>,
}

impl ConfiguredProjectScope {
    #[must_use]
    pub(crate) fn partition_id(&self) -> String {
        blake3::hash(self.normalized_scope.as_bytes())
            .to_hex()
            .to_string()
    }
}

pub(crate) fn configured_project_scopes(
    config_root: &Path,
    projects: &[SearchProjectConfig],
) -> Vec<ConfiguredProjectScope> {
    let mut roots = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for project in projects {
        for configured_path in &project.dirs {
            let Some(normalized_configured_path) = normalize_project_dir_root(configured_path)
            else {
                continue;
            };
            let Some(scope_path) = resolve_project_scope_path(
                config_root,
                project.root.as_str(),
                normalized_configured_path.as_str(),
            ) else {
                continue;
            };
            if !scope_path.exists() {
                continue;
            }
            let normalized_scope = normalize_path(scope_path.as_path());
            if seen.insert(normalized_scope.clone()) {
                roots.push(ConfiguredProjectScope {
                    scope_path,
                    normalized_scope,
                    project_name: project.name.clone(),
                    root_label: configured_root_label(
                        normalized_configured_path.as_str(),
                        project.name.as_str(),
                    ),
                });
            }
        }
    }

    roots
}

pub(crate) fn index_path_for_entry(project_root: &Path, path: &Path) -> String {
    path.strip_prefix(project_root)
        .map_or_else(|_| normalize_path(path), normalize_path)
}

pub(crate) fn should_skip_entry(entry: &DirEntry) -> bool {
    if !entry.file_type().is_dir() {
        return false;
    }

    matches!(
        entry.file_name().to_string_lossy().as_ref(),
        ".git"
            | ".cache"
            | ".devenv"
            | ".direnv"
            | ".run"
            | "target"
            | "node_modules"
            | "dist"
            | "coverage"
            | "__pycache__"
    )
}

pub(crate) fn project_metadata_for_path(
    project_root: &Path,
    config_root: &Path,
    projects: &[SearchProjectConfig],
    hit_path: &str,
) -> SearchProjectMetadata {
    let absolute_hit = if Path::new(hit_path).is_absolute() {
        PathBuf::from(hit_path)
    } else {
        project_root.join(hit_path)
    };
    let mut best_path_match: Option<(usize, SearchProjectMetadata)> = None;
    let mut best_root_match: Option<(usize, SearchProjectMetadata)> = None;

    for project in projects {
        let Some(project_root_path) = resolve_project_root_path(config_root, project.root.as_str())
        else {
            continue;
        };
        if !path_within_scope(absolute_hit.as_path(), project_root_path.as_path()) {
            continue;
        }
        update_best_match(
            &mut best_root_match,
            path_specificity(normalize_path(project_root_path.as_path()).as_str()),
            SearchProjectMetadata {
                project_name: Some(project.name.clone()),
                root_label: None,
            },
        );

        for configured_path in &project.dirs {
            let Some(normalized_path) = normalize_project_dir_root(configured_path.as_str()) else {
                continue;
            };
            let Some(candidate_scope) = resolve_project_scope_path(
                config_root,
                project.root.as_str(),
                normalized_path.as_str(),
            ) else {
                continue;
            };
            if !path_within_scope(absolute_hit.as_path(), candidate_scope.as_path()) {
                continue;
            }
            update_best_match(
                &mut best_path_match,
                path_specificity(normalize_path(candidate_scope.as_path()).as_str()),
                SearchProjectMetadata {
                    project_name: Some(project.name.clone()),
                    root_label: configured_root_label(
                        normalized_path.as_str(),
                        project.name.as_str(),
                    ),
                },
            );
        }
    }

    best_path_match
        .map(|(_, metadata)| metadata)
        .or_else(|| best_root_match.map(|(_, metadata)| metadata))
        .unwrap_or_default()
}

pub(crate) fn build_code_ast_hits_from_content(
    normalized_path: &str,
    content: &str,
) -> Vec<AstSearchHit> {
    let normalized_path_ref = Path::new(normalized_path);
    let Some(lang) = ast_search_lang(normalized_path_ref) else {
        return Vec::new();
    };
    let crate_name = infer_crate_name(normalized_path_ref);
    let mut hits = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for symbol in extract_code_structure_symbols(content, lang) {
        if symbol.signature.is_empty() {
            continue;
        }
        let dedupe_key = format!(
            "{normalized_path}:{}:{}:{}",
            symbol.line_start,
            symbol.line_end,
            symbol.name.as_str()
        );
        if !seen.insert(dedupe_key) {
            continue;
        }

        hits.push(AstSearchHit {
            name: symbol.name,
            signature: symbol.signature,
            path: normalized_path.to_string(),
            language: lang.as_str().to_string(),
            crate_name: crate_name.clone(),
            project_name: None,
            root_label: None,
            node_kind: None,
            owner_title: None,
            navigation_target: ast_navigation_target(
                normalized_path,
                crate_name.as_str(),
                None,
                None,
                symbol.line_start,
                symbol.line_end,
            ),
            line_start: symbol.line_start,
            line_end: symbol.line_end,
            score: 0.0,
        });
    }
    hits
}

pub(crate) fn ast_search_lang(path: &Path) -> Option<Lang> {
    supported_code_language_from_path(path)
}

pub(crate) fn markdown_scope_name(path: &Path) -> String {
    path.components()
        .find_map(|component| match component {
            Component::Normal(segment) => segment.to_str().map(ToString::to_string),
            _ => None,
        })
        .filter(|segment| !segment.is_empty())
        .unwrap_or_else(|| "docs".to_string())
}

pub(crate) fn compile_markdown_nodes(_path: &str, content: &str) -> Vec<AnalysisNode> {
    let parsed = xiuxian_wendao_parsers::parse_markdown_note_artifacts(content, "page");
    parsed
        .note
        .core
        .sections
        .iter()
        .map(|section| AnalysisNode {
            id: format!("section-{}", section.line_start()),
            label: section.heading_title().to_string(),
            kind: AnalysisNodeKind::Section,
            depth: section.heading_level(),
            line_start: section.line_start(),
            line_end: section.line_end(),
            parent_id: None,
        })
        .collect()
}

pub(crate) fn build_markdown_ast_hits_from_sections(
    path: &str,
    crate_name: &str,
    nodes: &[AnalysisNode],
    sections: &[MarkdownSection],
) -> Vec<AstSearchHit> {
    let mut hits = build_markdown_node_hits(path, crate_name, nodes);
    for section in sections {
        hits.extend(build_markdown_property_hits_from_toc_section(
            path, crate_name, section,
        ));
        hits.extend(build_markdown_observation_hits_from_toc_section(
            path, crate_name, section,
        ));
    }
    hits
}

pub(crate) fn is_markdown_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            ext.eq_ignore_ascii_case("md")
                || ext.eq_ignore_ascii_case("markdown")
                || ext.eq_ignore_ascii_case("org")
        })
}

pub(crate) fn infer_crate_name(relative_path: &Path) -> String {
    let components = relative_path
        .components()
        .filter_map(|component| match component {
            Component::Normal(segment) => segment.to_str().map(ToString::to_string),
            _ => None,
        })
        .collect::<Vec<_>>();

    match components.as_slice() {
        [packages, rust, crates, crate_name, ..]
            if packages == "packages" && rust == "rust" && crates == "crates" =>
        {
            crate_name.clone()
        }
        [packages, python, package_name, ..] if packages == "packages" && python == "python" => {
            package_name.clone()
        }
        [data, workspace_name, ..] if data == ".data" => workspace_name.clone(),
        [skills, skill_name, ..] if skills == "skills" => skill_name.clone(),
        [first, ..] => first.clone(),
        [] => "workspace".to_string(),
    }
}

pub(crate) fn score_reference_hit(line_text: &str, query: &str) -> f64 {
    let line = line_text.to_ascii_lowercase();
    let query = query.trim().to_ascii_lowercase();
    if query.is_empty() {
        return 0.0;
    }
    if line == query {
        1.0
    } else if line.contains(query.as_str()) {
        0.85
    } else {
        0.0
    }
}

fn build_markdown_node_hits(
    path: &str,
    crate_name: &str,
    nodes: &[AnalysisNode],
) -> Vec<AstSearchHit> {
    nodes
        .iter()
        .filter_map(|node| {
            let signature = markdown_signature(node.kind, node.depth, node.label.as_str())?;
            Some(AstSearchHit {
                name: node.label.clone(),
                signature,
                path: path.to_string(),
                language: "markdown".to_string(),
                crate_name: crate_name.to_string(),
                project_name: None,
                root_label: None,
                node_kind: markdown_node_kind(node.kind).map(ToOwned::to_owned),
                owner_title: None,
                navigation_target: ast_navigation_target(
                    path,
                    crate_name,
                    None,
                    None,
                    node.line_start,
                    node.line_end,
                ),
                line_start: node.line_start,
                line_end: node.line_end,
                score: 0.0,
            })
        })
        .collect()
}

fn markdown_signature(kind: AnalysisNodeKind, depth: usize, label: &str) -> Option<String> {
    match kind {
        AnalysisNodeKind::Section => Some(format!("{} {label}", "#".repeat(depth.clamp(1, 6)))),
        AnalysisNodeKind::Task => Some(format!("- [ ] {label}")),
        _ => None,
    }
}

fn markdown_node_kind(kind: AnalysisNodeKind) -> Option<&'static str> {
    match kind {
        AnalysisNodeKind::Section => Some("section"),
        AnalysisNodeKind::Task => Some("task"),
        _ => None,
    }
}

fn build_markdown_property_hits_from_toc_section(
    path: &str,
    crate_name: &str,
    section: &MarkdownSection,
) -> Vec<AstSearchHit> {
    let owner_title = markdown_owner_title_from_toc_section(section);
    section
        .attributes()
        .iter()
        .filter(|(key, _)| !is_observation_attribute(key.as_str()))
        .map(|(key, value)| AstSearchHit {
            name: key.clone(),
            signature: format!(":{key}: {value}"),
            path: path.to_string(),
            language: "markdown".to_string(),
            crate_name: crate_name.to_string(),
            project_name: None,
            root_label: None,
            node_kind: Some("property".to_string()),
            owner_title: owner_title.clone(),
            navigation_target: ast_navigation_target(
                path,
                crate_name,
                None,
                None,
                section.line_start(),
                section.line_end(),
            ),
            line_start: section.line_start(),
            line_end: section.line_end(),
            score: 0.0,
        })
        .collect()
}

fn build_markdown_observation_hits_from_toc_section(
    path: &str,
    crate_name: &str,
    section: &MarkdownSection,
) -> Vec<AstSearchHit> {
    let owner_title = markdown_owner_title_from_toc_section(section);
    extract_observations(section.attributes())
        .into_iter()
        .map(|observation| AstSearchHit {
            name: "OBSERVE".to_string(),
            signature: format!(":OBSERVE: {}", observation.raw_value),
            path: path.to_string(),
            language: "markdown".to_string(),
            crate_name: crate_name.to_string(),
            project_name: None,
            root_label: None,
            node_kind: Some("observation".to_string()),
            owner_title: owner_title.clone(),
            navigation_target: ast_navigation_target(
                path,
                crate_name,
                None,
                None,
                section.line_start(),
                section.line_end(),
            ),
            line_start: section.line_start(),
            line_end: section.line_end(),
            score: 0.0,
        })
        .collect()
}

fn markdown_owner_title_from_toc_section(section: &MarkdownSection) -> Option<String> {
    if !section.heading_path().trim().is_empty() {
        Some(section.heading_path().to_string())
    } else if !section.heading_title().trim().is_empty() {
        Some(section.heading_title().to_string())
    } else {
        None
    }
}

fn is_observation_attribute(key: &str) -> bool {
    key == "OBSERVE" || key.starts_with("OBSERVE_")
}

fn ast_navigation_target(
    path: &str,
    crate_name: &str,
    project_name: Option<&str>,
    root_label: Option<&str>,
    line_start: usize,
    line_end: usize,
) -> StudioNavigationTarget {
    StudioNavigationTarget {
        path: path.to_string(),
        category: "doc".to_string(),
        project_name: project_name
            .map(ToString::to_string)
            .or_else(|| Some(crate_name.to_string())),
        root_label: root_label.map(ToString::to_string),
        line: Some(line_start),
        line_end: Some(line_end),
        column: None,
    }
}

fn resolve_project_scope_path(
    config_root: &Path,
    configured_root: &str,
    configured_path: &str,
) -> Option<PathBuf> {
    let project_base = resolve_project_root_path(config_root, configured_root)?;
    resolve_path_like(project_base.as_path(), configured_path)
}

fn resolve_path_like(base: &Path, input: &str) -> Option<PathBuf> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    let path = Path::new(trimmed);
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    };
    Some(normalize_pathbuf(joined.as_path()))
}

fn normalize_project_dir_root(dir: &str) -> Option<String> {
    let normalized = dir.trim().replace('\\', "/");
    if normalized.is_empty() {
        None
    } else {
        Some(normalized.trim_end_matches('/').to_string())
    }
}

fn configured_root_label(configured_path: &str, project_name: &str) -> Option<String> {
    if configured_path == "." {
        return Some(project_name.to_string());
    }

    Path::new(configured_path)
        .file_name()
        .map(|segment| segment.to_string_lossy().into_owned())
        .or_else(|| Some(project_name.to_string()))
}

fn path_within_scope(path: &Path, scope: &Path) -> bool {
    let normalized_path = normalize_pathbuf(path);
    let normalized_scope = normalize_pathbuf(scope);
    normalized_path == normalized_scope || normalized_path.strip_prefix(&normalized_scope).is_ok()
}

fn normalize_path(path: &Path) -> String {
    normalize_pathbuf(path).to_string_lossy().replace('\\', "/")
}

fn normalize_pathbuf(path: &Path) -> PathBuf {
    path.components()
        .fold(PathBuf::new(), |mut acc, component| {
            match component {
                std::path::Component::CurDir => {}
                other => acc.push(other.as_os_str()),
            }
            acc
        })
}

fn path_specificity(path: &str) -> usize {
    if path == "." {
        0
    } else {
        path.split('/').count()
    }
}

fn update_best_match(
    slot: &mut Option<(usize, SearchProjectMetadata)>,
    specificity: usize,
    metadata: SearchProjectMetadata,
) {
    match slot {
        Some((current_specificity, _)) if *current_specificity >= specificity => {}
        _ => *slot = Some((specificity, metadata)),
    }
}

//! Zhixing mount artifact definitions for runtime package wiring.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use include_dir::Dir;
use xiuxian_wendao_parsers::parse_frontmatter;

use super::paths::{embedded_resource_dir, normalize_embedded_resource_path};

static EMBEDDED_MOUNTS_BY_SEMANTIC: OnceLock<HashMap<String, Vec<PathBuf>>> = OnceLock::new();

/// Returns the parser-derived `references/` mount roots keyed by semantic name.
#[must_use]
pub fn embedded_semantic_reference_mounts() -> &'static HashMap<String, Vec<PathBuf>> {
    embedded_skill_mount_index()
}

/// Returns the cached embedded semantic mount index.
#[must_use]
pub fn embedded_skill_mount_index() -> &'static HashMap<String, Vec<PathBuf>> {
    EMBEDDED_MOUNTS_BY_SEMANTIC.get_or_init(resolve_embedded_skill_mount_index)
}

fn resolve_embedded_skill_mount_index() -> HashMap<String, Vec<PathBuf>> {
    let mut markdown_files = Vec::new();
    collect_embedded_markdown_files(embedded_resource_dir(), &mut markdown_files);
    markdown_files.sort_by(|left, right| left.path().cmp(right.path()));

    let mut mounts_by_semantic: HashMap<String, Vec<PathBuf>> = HashMap::new();
    for file in markdown_files {
        let path = normalize_embedded_resource_path(file.path().to_string_lossy().as_ref());
        if !is_skill_descriptor(path.as_str()) {
            continue;
        }
        let Some(content) = file.contents_utf8() else {
            continue;
        };
        let semantic_name = parse_frontmatter(content)
            .name
            .map(|value| value.trim().to_ascii_lowercase())
            .filter(|value| !value.is_empty());
        let Some(semantic_name) = semantic_name else {
            continue;
        };

        let references_dir = Path::new(path.as_str()).parent().map_or_else(
            || PathBuf::from("references"),
            |parent| parent.join("references"),
        );
        mounts_by_semantic
            .entry(semantic_name)
            .or_default()
            .push(references_dir);
    }

    for references_dirs in mounts_by_semantic.values_mut() {
        references_dirs.sort();
        references_dirs.dedup();
    }

    mounts_by_semantic
}

fn collect_embedded_markdown_files<'a>(dir: &'a Dir<'a>, out: &mut Vec<&'a include_dir::File<'a>>) {
    for file in dir.files() {
        let path = file.path().to_string_lossy().replace('\\', "/");
        if is_markdown_file(path.as_str()) {
            out.push(file);
        }
    }
    for child in dir.dirs() {
        collect_embedded_markdown_files(child, out);
    }
}

fn is_markdown_file(path: &str) -> bool {
    matches!(
        path.rsplit('.').next().map(str::to_ascii_lowercase),
        Some(ext) if ext == "md" || ext == "markdown"
    )
}

fn is_skill_descriptor(path: &str) -> bool {
    Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("SKILL.md") || name == "skill.md")
}

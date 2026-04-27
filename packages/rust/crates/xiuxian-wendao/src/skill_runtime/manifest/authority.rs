use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use walkdir::WalkDir;
use xiuxian_wendao_parsers::{
    discover_skill_documents, extract_references, parse_skill_frontmatter,
};

use super::load::load_skill_manifest_from_path;
use super::types::{
    SKILL_RUNTIME_URI_PREFIX, SkillAuthorityOutcome, SkillAuthorityReport, SkillManifestError,
};

/// Resolve skill manifest authority by intersecting intent links and physical manifests.
///
/// # Errors
/// Returns [`SkillManifestError`] when manifest parsing fails.
pub fn resolve_skill_authority(
    runtime_root: &Path,
) -> Result<SkillAuthorityOutcome, SkillManifestError> {
    let mut intent_uris = HashSet::new();
    let mut ghost_links = HashSet::new();
    let mut manifest_paths = HashMap::new();
    let skills = discover_skill_docs(runtime_root);
    for skill_doc in &skills {
        let Some(skill_root) = skill_doc.parent() else {
            continue;
        };
        let semantic_name = resolve_skill_semantic_name(skill_doc)?;
        let (intent, ghosts) = collect_intent_manifest_uris(skill_root, skill_doc, &semantic_name);
        intent_uris.extend(intent);
        ghost_links.extend(ghosts);
        let physical = collect_physical_manifest_uris(skill_root, &semantic_name);
        for (uri, path) in physical {
            manifest_paths.insert(uri, path);
        }
    }

    let physical_uris: HashSet<String> = manifest_paths.keys().cloned().collect();
    let authorized_uris: HashSet<String> =
        intent_uris.intersection(&physical_uris).cloned().collect();
    let unauthorized_uris: HashSet<String> =
        physical_uris.difference(&intent_uris).cloned().collect();
    let mut authorized_manifests = Vec::new();
    for uri in &authorized_uris {
        if let Some(path) = manifest_paths.get(uri) {
            authorized_manifests.push(load_skill_manifest_from_path(path)?);
        }
    }
    authorized_manifests.sort_by(|left, right| left.tool_name.cmp(&right.tool_name));
    let mut report = SkillAuthorityReport {
        authorized_manifests: authorized_uris.into_iter().collect(),
        ghost_links: ghost_links.into_iter().collect(),
        unauthorized_manifests: unauthorized_uris.into_iter().collect(),
    };
    report.authorized_manifests.sort();
    report.ghost_links.sort();
    report.unauthorized_manifests.sort();
    Ok(SkillAuthorityOutcome {
        report,
        authorized: authorized_manifests,
    })
}

fn discover_skill_docs(root: &Path) -> Vec<PathBuf> {
    discover_skill_documents(root)
        .into_iter()
        .filter(|path| {
            path.parent()
                .and_then(Path::parent)
                .is_some_and(|parent| parent == root)
        })
        .collect()
}

fn resolve_skill_semantic_name(skill_doc: &Path) -> Result<String, SkillManifestError> {
    let content = std::fs::read_to_string(skill_doc).map_err(|source| SkillManifestError::Io {
        path: skill_doc.to_string_lossy().to_string(),
        source,
    })?;
    let frontmatter = parse_skill_frontmatter(content.as_str()).map_err(|error| {
        SkillManifestError::SkillFrontmatter {
            path: skill_doc.to_string_lossy().to_string(),
            reason: error.to_string(),
        }
    })?;
    Ok(frontmatter
        .name
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase())
}

fn collect_physical_manifest_uris(
    skill_root: &Path,
    semantic_name: &str,
) -> Vec<(String, PathBuf)> {
    let mut manifests = Vec::new();
    let references_root = skill_root.join("references");
    if !references_root.is_dir() {
        return manifests;
    }
    for entry in WalkDir::new(&references_root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if !is_qianji_manifest(path) {
            continue;
        }
        let Ok(relative) = path.strip_prefix(&references_root) else {
            continue;
        };
        let relative = relative.to_string_lossy().replace('\\', "/");
        let uri = canonical_manifest_uri(semantic_name, &relative);
        manifests.push((uri, path.to_path_buf()));
    }
    manifests
}

fn collect_intent_manifest_uris(
    skill_root: &Path,
    skill_doc: &Path,
    semantic_name: &str,
) -> (HashSet<String>, HashSet<String>) {
    let markdown = std::fs::read_to_string(skill_doc).unwrap_or_default();
    let targets = extract_markdown_links(&markdown);
    let mut intents = HashSet::new();
    let mut ghosts = HashSet::new();
    let references_root = skill_root.join("references");
    for target in targets {
        if let Some(uri) = normalize_manifest_uri(&target, semantic_name) {
            intents.insert(uri);
            continue;
        }

        let Some(normalized) = normalize_local_target(&target, skill_doc, skill_root) else {
            continue;
        };
        if !normalized.starts_with(&references_root) {
            continue;
        }
        if !is_qianji_manifest(&normalized) {
            continue;
        }
        let Ok(relative) = normalized.strip_prefix(&references_root) else {
            continue;
        };
        let relative = relative.to_string_lossy().replace('\\', "/");
        let uri = canonical_manifest_uri(semantic_name, &relative);
        if normalized.exists() {
            intents.insert(uri);
        } else {
            ghosts.insert(uri);
        }
    }
    (intents, ghosts)
}

fn extract_markdown_links(markdown: &str) -> Vec<String> {
    extract_references(markdown)
        .into_iter()
        .filter_map(|reference| {
            reference
                .literal_addressed_target
                .addressed_target
                .target
                .clone()
        })
        .collect()
}

fn normalize_manifest_uri(raw: &str, fallback_semantic: &str) -> Option<String> {
    let trimmed = raw.trim();
    if !trimmed.starts_with(SKILL_RUNTIME_URI_PREFIX) {
        return None;
    }
    let mut payload = trimmed.trim_start_matches(SKILL_RUNTIME_URI_PREFIX);
    payload = payload.trim_start_matches('/');
    let mut segments = payload.split('/').collect::<Vec<_>>();
    if segments.len() < 3 {
        return None;
    }
    let semantic = segments.first().copied().unwrap_or(fallback_semantic);
    if segments.get(1).copied()? != "references" {
        return None;
    }
    let entity = segments.split_off(2).join("/");
    if entity.is_empty() {
        return None;
    }
    Some(canonical_manifest_uri(semantic, entity.trim_matches('/')))
}

fn canonical_manifest_uri(semantic_name: &str, relative: &str) -> String {
    format!(
        "{}/{}/references/{}",
        SKILL_RUNTIME_URI_PREFIX,
        semantic_name.trim().to_ascii_lowercase(),
        relative.trim_matches('/')
    )
}

fn normalize_local_target(raw: &str, skill_doc: &Path, skill_root: &Path) -> Option<PathBuf> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if is_external_link(trimmed) || trimmed.starts_with('#') {
        return None;
    }
    let trimmed = strip_fragment_and_query(trimmed);
    if trimmed.is_empty() {
        return None;
    }
    let base_dir = skill_doc.parent().unwrap_or(skill_root);
    let candidate = Path::new(trimmed);
    let joined = if candidate.is_absolute() {
        skill_root.join(candidate.strip_prefix("/").ok()?)
    } else {
        base_dir.join(candidate)
    };
    let normalized = normalize_path_no_parent(&joined)?;
    if normalized.starts_with(skill_root) {
        Some(normalized)
    } else {
        None
    }
}

fn strip_fragment_and_query(raw: &str) -> &str {
    let mut end = raw.len();
    if let Some(idx) = raw.find('#') {
        end = end.min(idx);
    }
    if let Some(idx) = raw.find('?') {
        end = end.min(idx);
    }
    raw[..end].trim_matches('/')
}

fn is_external_link(raw: &str) -> bool {
    let lower = raw.to_ascii_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:")
        || lower.starts_with("tel:")
        || lower.starts_with("data:")
        || lower.starts_with("javascript:")
}

fn normalize_path_no_parent(path: &Path) -> Option<PathBuf> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::Normal(value) => normalized.push(value),
            std::path::Component::CurDir
            | std::path::Component::RootDir
            | std::path::Component::Prefix(_) => {}
            std::path::Component::ParentDir => return None,
        }
    }
    Some(normalized)
}

fn is_qianji_manifest(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("qianji.toml"))
}

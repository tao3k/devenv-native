use super::mounts::runtime_wendao_mounts;
use crate::runtime_config::{resolve_process_env_path, resolve_process_project_root};
use std::fs;
use std::path::{Path, PathBuf};
use xiuxian_wendao::SkillRuntimeResolver;
use xiuxian_wendao_core::WendaoResourceUri;
use xiuxian_wendao_runtime::artifacts::zhixing::embedded_resource_text_from_wendao_uri;
use xiuxian_zhenfa::ZhenfaTransmuter;

/// Resolve one `wendao://` URI and delegate validation/refinement to Zhenfa.
pub(crate) fn resolve_wendao_uri_with_zhenfa(uri: &str) -> Result<String, String> {
    ZhenfaTransmuter::resolve_and_wash(uri, resolve_wendao_uri_text)
        .map_err(|error| error.to_string())
}

fn normalize_relative_path(path: &str) -> String {
    path.trim()
        .replace('\\', "/")
        .split('/')
        .filter(|segment| !segment.is_empty() && *segment != ".")
        .collect::<Vec<_>>()
        .join("/")
}

fn resolve_wendao_uri_from_runtime_mounts(uri: &str) -> Option<String> {
    let parsed = WendaoResourceUri::parse(uri).ok()?;
    let semantic_name = parsed.semantic_name();
    let entity_relative_path =
        normalize_relative_path(parsed.entity_relative_path().to_string_lossy().as_ref());
    let mounts = runtime_wendao_mounts().read().ok()?;
    for mount in mounts.iter() {
        if !semantic_name.eq_ignore_ascii_case(mount.semantic_name) {
            continue;
        }
        let references_dir = normalize_relative_path(mount.references_dir);
        if references_dir.is_empty() {
            continue;
        }
        let candidate = format!("{references_dir}/{entity_relative_path}");
        let Some(content) = mount
            .dir
            .get_file(candidate.as_str())
            .and_then(include_dir::File::contents_utf8)
        else {
            continue;
        };
        return Some(content.to_string());
    }
    None
}

/// Resolve semantic resources through the shared skill runtime loader.
fn resolve_wendao_uri_from_skill_loader(uri: &str) -> Option<String> {
    WendaoResourceUri::parse(uri).ok()?;
    let roots = resolve_skill_runtime_roots();
    if roots.is_empty() {
        return None;
    }
    let resolver = SkillRuntimeResolver::from_roots(roots.as_slice()).ok()?;
    resolver.read_utf8(uri).ok()
}

/// Direct-path fallback for internal callers that pass an explicit file path.
fn resolve_wendao_uri_from_explicit_path(uri_or_path: &str) -> Option<String> {
    let trimmed = uri_or_path.trim();
    if trimmed.is_empty() || trimmed.contains("://") {
        return None;
    }

    let candidate = PathBuf::from(trimmed);
    if candidate.is_file() {
        return fs::read_to_string(candidate).ok();
    }

    let rooted = resolve_process_project_root()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(candidate);
    if rooted.is_file() {
        return fs::read_to_string(rooted).ok();
    }

    None
}

fn resolve_skill_runtime_roots() -> Vec<PathBuf> {
    let project_root = resolve_process_project_root().unwrap_or_else(|| PathBuf::from("."));
    let mut roots = discover_crate_skill_roots(
        project_root
            .join("packages")
            .join("rust")
            .join("crates")
            .as_path(),
    );
    roots.push(project_root.join("assets").join("skills"));

    let config_home = resolve_process_env_path("PRJ_CONFIG_HOME", project_root.as_path())
        .unwrap_or_else(|| project_root.join(".config"));
    roots.push(config_home.join("xiuxian-artisan-workshop").join("skills"));

    if let Some(resource_root) =
        resolve_process_env_path("XIUXIAN_RESOURCE_ROOT", project_root.as_path())
    {
        roots.push(resource_root.join("skills"));
    }

    if let Ok(executable_path) = std::env::current_exe()
        && let Some(executable_dir) = executable_path.parent()
    {
        roots.push(executable_dir.join("resources").join("skills"));
        roots.push(executable_dir.join("..").join("resources").join("skills"));
    }

    roots.retain(|path| path.exists() && path.is_dir());
    dedup_paths(&mut roots);
    roots
}

fn discover_crate_skill_roots(crates_root: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(crates_root) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .map(|crate_dir| crate_dir.join("resources").join("skills"))
        .collect()
}

fn dedup_paths(paths: &mut Vec<PathBuf>) {
    let mut unique = Vec::new();
    for path in std::mem::take(paths) {
        if !unique.contains(&path) {
            unique.push(path);
        }
    }
    *paths = unique;
}

pub(super) fn resolve_wendao_uri_text(uri: &str) -> Option<String> {
    resolve_wendao_uri_from_runtime_mounts(uri)
        .or_else(|| embedded_resource_text_from_wendao_uri(uri).map(str::to_string))
        .or_else(|| resolve_wendao_uri_from_skill_loader(uri))
        .or_else(|| resolve_wendao_uri_from_explicit_path(uri))
}

#[cfg(test)]
#[path = "../../../tests/unit/scheduler/preflight/wendao_uri.rs"]
mod tests;

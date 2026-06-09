#[cfg(feature = "wendao-integration")]
use super::mounts::runtime_wendao_mounts_snapshot;
#[cfg(feature = "wendao-integration")]
use crate::runtime_config::{resolve_process_env_path, resolve_process_project_root};
#[cfg(feature = "wendao-integration")]
use serde_yaml::{Mapping, Value};
#[cfg(feature = "wendao-integration")]
use std::collections::HashMap;
#[cfg(feature = "wendao-integration")]
use std::fs;
#[cfg(feature = "wendao-integration")]
use std::path::{Path, PathBuf};
#[cfg(feature = "wendao-integration")]
use walkdir::WalkDir;
#[cfg(feature = "wendao-integration")]
use xiuxian_wendao_core::WendaoResourceUri;
#[cfg(feature = "wendao-integration")]
use xiuxian_wendao_runtime::artifacts::zhixing::embedded_resource_text_from_wendao_uri;
#[cfg(feature = "wendao-integration")]
use xiuxian_zhenfa::ZhenfaTransmuter;

/// Resolve one `wendao://` URI and delegate validation/refinement to Zhenfa.
#[cfg(feature = "wendao-integration")]
pub(crate) fn resolve_wendao_uri_with_zhenfa(uri: &str) -> Result<String, String> {
    ZhenfaTransmuter::resolve_and_wash(uri, resolve_wendao_uri_text)
        .map_err(|error| error.to_string())
}

/// Resolve one `wendao://` URI and delegate validation/refinement to Zhenfa.
#[cfg(not(feature = "wendao-integration"))]
pub(crate) fn resolve_wendao_uri_with_zhenfa(uri: &str) -> Result<String, String> {
    Err(format!(
        "`wendao://` semantic resource `{uri}` requires the `wendao-integration` feature"
    ))
}

#[cfg(feature = "wendao-integration")]
fn normalize_relative_path(path: &str) -> String {
    path.trim()
        .replace('\\', "/")
        .split('/')
        .filter(|segment| !segment.is_empty() && *segment != ".")
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(feature = "wendao-integration")]
fn resolve_wendao_uri_from_runtime_mounts(uri: &str) -> Option<String> {
    let parsed = WendaoResourceUri::parse(uri).ok()?;
    let semantic_name = parsed.semantic_name();
    let entity_relative_path =
        normalize_relative_path(parsed.entity_relative_path().to_string_lossy().as_ref());
    for mount in runtime_wendao_mounts_snapshot() {
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
#[cfg(feature = "wendao-integration")]
fn resolve_wendao_uri_from_skill_loader(uri: &str) -> Option<String> {
    WendaoResourceUri::parse(uri).ok()?;
    let roots = resolve_skill_runtime_roots();
    if roots.is_empty() {
        return None;
    }
    let resolver = LocalSkillRuntimeResolver::from_roots(roots.as_slice());
    resolver.read_utf8(uri).ok()
}

#[cfg(feature = "wendao-integration")]
struct LocalSkillRuntimeResolver {
    references_by_semantic: HashMap<String, Vec<PathBuf>>,
}

#[cfg(feature = "wendao-integration")]
impl LocalSkillRuntimeResolver {
    fn from_roots(roots: &[PathBuf]) -> Self {
        let mut references_by_semantic: HashMap<String, Vec<PathBuf>> = HashMap::new();
        for root in roots {
            for skill_path in discover_skill_descriptor_paths(root) {
                let Some(semantic_name) = read_skill_semantic_name(skill_path.as_path()) else {
                    continue;
                };
                let Some(skill_dir) = skill_path.parent() else {
                    continue;
                };
                references_by_semantic
                    .entry(semantic_name)
                    .or_default()
                    .push(skill_dir.join("references"));
            }
        }
        for references_dirs in references_by_semantic.values_mut() {
            references_dirs.sort();
            references_dirs.dedup();
        }
        Self {
            references_by_semantic,
        }
    }

    fn read_utf8(&self, uri: &str) -> Result<String, String> {
        let parsed = WendaoResourceUri::parse(uri).map_err(|error| error.to_string())?;
        let semantic_name = parsed.semantic_name();
        let Some(references_dirs) = self.references_by_semantic.get(semantic_name).or_else(|| {
            self.references_by_semantic
                .get(semantic_name.to_ascii_lowercase().as_str())
        }) else {
            return Err(format!("unknown Wendao semantic skill `{semantic_name}`"));
        };
        let relative_entity = parsed.entity_relative_path();
        for references_dir in references_dirs {
            let candidate = references_dir.join(relative_entity);
            if !candidate.is_file() {
                continue;
            }
            return fs::read_to_string(candidate.as_path())
                .map_err(|error| format!("failed to read `{}`: {error}", candidate.display()));
        }
        Err(format!(
            "Wendao semantic resource `{}` was not found",
            parsed.canonical_uri()
        ))
    }
}

#[cfg(feature = "wendao-integration")]
fn discover_skill_descriptor_paths(root: &Path) -> Vec<PathBuf> {
    if !root.is_dir() {
        return Vec::new();
    }
    let mut paths = WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .filter(|path| is_skill_descriptor_path(path.as_path()))
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

#[cfg(feature = "wendao-integration")]
fn is_skill_descriptor_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("SKILL.md"))
}

#[cfg(feature = "wendao-integration")]
fn read_skill_semantic_name(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let frontmatter = leading_yaml_frontmatter(content.as_str())?;
    let value = serde_yaml::from_str::<Value>(frontmatter).ok()?;
    let mapping = value.as_mapping()?;
    mapping_string(mapping, "name").map(|name| name.to_ascii_lowercase())
}

#[cfg(feature = "wendao-integration")]
fn leading_yaml_frontmatter(content: &str) -> Option<&str> {
    let content = content.strip_prefix("---")?;
    let content = content
        .strip_prefix('\n')
        .or_else(|| content.strip_prefix("\r\n"))?;
    let end = content.find("\n---")?;
    Some(&content[..end])
}

#[cfg(feature = "wendao-integration")]
fn mapping_string(mapping: &Mapping, key: &str) -> Option<String> {
    mapping
        .get(Value::String(key.to_string()))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

/// Direct-path fallback for internal callers that pass an explicit file path.
#[cfg(feature = "wendao-integration")]
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

#[cfg(feature = "wendao-integration")]
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

#[cfg(feature = "wendao-integration")]
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

#[cfg(feature = "wendao-integration")]
fn dedup_paths(paths: &mut Vec<PathBuf>) {
    *paths = std::mem::take(paths)
        .into_iter()
        .fold(Vec::new(), |mut unique, path| {
            if !unique.contains(&path) {
                unique.push(path);
            }
            unique
        });
}

#[cfg(feature = "wendao-integration")]
pub(super) fn resolve_wendao_uri_text(uri: &str) -> Option<String> {
    resolve_wendao_uri_from_runtime_mounts(uri)
        .or_else(|| embedded_resource_text_from_wendao_uri(uri).map(str::to_string))
        .or_else(|| resolve_wendao_uri_from_skill_loader(uri))
        .or_else(|| resolve_wendao_uri_from_explicit_path(uri))
}

#[cfg(feature = "wendao-integration")]
#[cfg(test)]
#[path = "../../../tests/unit/scheduler/preflight/wendao_uri.rs"]
mod tests;

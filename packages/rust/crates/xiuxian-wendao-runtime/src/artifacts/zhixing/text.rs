//! Zhixing artifact text rendering for runtime package wiring.

use xiuxian_wendao_core::WendaoResourceUri;

use super::mounts::embedded_skill_mount_index;
use super::paths::{ZHIXING_SKILL_DOC_PATH, normalize_embedded_resource_path};

/// Returns the embedded markdown source of `zhixing/skills/agenda-management/SKILL.md`.
#[must_use]
pub fn embedded_skill_markdown() -> Option<&'static str> {
    embedded_resource_text(ZHIXING_SKILL_DOC_PATH)
}

/// Returns UTF-8 text content for one embedded resource path.
///
/// Paths are normalized to slash separators and accept optional `./` prefix.
#[must_use]
pub fn embedded_resource_text(path: &str) -> Option<&'static str> {
    let normalized = normalize_embedded_resource_path(path);
    super::paths::embedded_resource_dir()
        .get_file(normalized.as_str())
        .and_then(include_dir::File::contents_utf8)
}

/// Resolves one semantic `wendao://` URI from embedded zhixing resources.
///
/// This API is intentionally strict: only semantic URIs are supported.
#[must_use]
pub fn embedded_resource_text_from_wendao_uri(uri: &str) -> Option<&'static str> {
    let parsed = WendaoResourceUri::parse(uri).ok()?;
    embedded_resource_text_from_parsed_wendao_uri(&parsed)
}

fn embedded_resource_text_from_parsed_wendao_uri(uri: &WendaoResourceUri) -> Option<&'static str> {
    let mounts = embedded_skill_mount_index().get(uri.semantic_name())?;
    let candidate = uri.entity_relative_path();
    for mount in mounts {
        let target =
            normalize_embedded_resource_path(mount.join(candidate).to_string_lossy().as_ref());
        let Some(content) = embedded_resource_text(target.as_str()) else {
            continue;
        };
        return Some(content);
    }
    None
}

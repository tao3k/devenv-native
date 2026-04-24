use std::path::{Component, Path};

use crate::skill_runtime::SkillRuntimeError;

pub(super) fn normalize_package_id(raw: &str) -> Option<String> {
    let normalized = raw.trim().to_ascii_lowercase().replace('_', "-");
    if normalized.is_empty() {
        return None;
    }
    if normalized
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
    {
        Some(normalized)
    } else {
        None
    }
}

pub(super) fn normalize_relative_asset_path(raw: &str) -> Result<String, SkillRuntimeError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(SkillRuntimeError::InvalidRelativeAssetPath {
            path: raw.to_string(),
        });
    }
    let path = Path::new(trimmed);
    if path.is_absolute() {
        return Err(SkillRuntimeError::InvalidRelativeAssetPath {
            path: raw.to_string(),
        });
    }
    let mut normalized = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => {
                let rendered = value.to_string_lossy().trim().to_string();
                if rendered.is_empty() {
                    return Err(SkillRuntimeError::InvalidRelativeAssetPath {
                        path: raw.to_string(),
                    });
                }
                normalized.push(rendered);
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(SkillRuntimeError::InvalidRelativeAssetPath {
                    path: raw.to_string(),
                });
            }
        }
    }
    if normalized.is_empty() {
        return Err(SkillRuntimeError::InvalidRelativeAssetPath {
            path: raw.to_string(),
        });
    }
    Ok(normalized.join("/"))
}

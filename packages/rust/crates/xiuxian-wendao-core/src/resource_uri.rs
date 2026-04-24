use std::path::{Component, Path, PathBuf};

use thiserror::Error;

/// Canonical URI scheme for semantic skill resource addressing.
pub const WENDAO_URI_SCHEME: &str = "wendao";
const SKILLS_SEGMENT: &str = "skills";
const REFERENCES_SEGMENT: &str = "references";

/// Parse-only errors for the stable `wendao://` resource URI contract.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WendaoResourceUriError {
    /// URI scheme is not `wendao`.
    #[error("unsupported URI scheme `{scheme}` in `{uri}`; expected `wendao://`")]
    UnsupportedScheme {
        /// Full URI string.
        uri: String,
        /// Parsed unsupported scheme.
        scheme: String,
    },
    /// URI payload does not follow a supported Wendao shape.
    #[error("invalid wendao resource URI `{0}`")]
    InvalidUri(String),
    /// One required segment is absent or empty.
    #[error("missing URI segment `{segment}` in `{uri}`")]
    MissingUriSegment {
        /// Full URI string.
        uri: String,
        /// Required segment identifier.
        segment: &'static str,
    },
    /// Entity path includes forbidden traversal or empty components.
    #[error("invalid entity path `{entity}` in URI `{uri}`")]
    InvalidEntityPath {
        /// Full URI string.
        uri: String,
        /// Entity path payload.
        entity: String,
    },
    /// Entity path does not include a file extension.
    #[error("entity path `{entity}` in URI `{uri}` must include a file extension")]
    MissingEntityExtension {
        /// Full URI string.
        uri: String,
        /// Entity path payload.
        entity: String,
    },
}

/// Parsed semantic skill resource URI.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WendaoResourceUri {
    semantic_name: String,
    entity_name: String,
}

impl WendaoResourceUri {
    /// Parse one URI string in the shape:
    /// `wendao://skills/<semantic_name>/references/<entity_name>.<ext>`.
    ///
    /// # Errors
    ///
    /// Returns [`WendaoResourceUriError`] when URI syntax or path safety checks
    /// fail.
    pub fn parse(uri: &str) -> Result<Self, WendaoResourceUriError> {
        let trimmed = uri.trim();
        let (scheme, payload) = split_scheme(trimmed)?;
        if !scheme.eq_ignore_ascii_case(WENDAO_URI_SCHEME) {
            return Err(WendaoResourceUriError::UnsupportedScheme {
                uri: trimmed.to_string(),
                scheme: scheme.to_string(),
            });
        }

        let payload = strip_uri_suffix(payload);
        let segments: Vec<&str> = payload.split('/').collect();
        if segments.len() < 4 {
            return Err(WendaoResourceUriError::InvalidUri(trimmed.to_string()));
        }

        match segments.first().copied() {
            Some(SKILLS_SEGMENT) => {}
            _ => return Err(WendaoResourceUriError::InvalidUri(trimmed.to_string())),
        }

        if segments.get(2).copied() != Some(REFERENCES_SEGMENT) {
            return Err(WendaoResourceUriError::InvalidUri(trimmed.to_string()));
        }

        let semantic_name = normalize_segment(segments.get(1).copied()).ok_or_else(|| {
            WendaoResourceUriError::MissingUriSegment {
                uri: trimmed.to_string(),
                segment: "semantic_name",
            }
        })?;
        let raw_entity = segments[3..].join("/");
        let entity_name = normalize_entity_path(&raw_entity, trimmed).map_err(|entity| {
            WendaoResourceUriError::InvalidEntityPath {
                uri: trimmed.to_string(),
                entity,
            }
        })?;
        if Path::new(entity_name.as_str()).extension().is_none() {
            return Err(WendaoResourceUriError::MissingEntityExtension {
                uri: trimmed.to_string(),
                entity: entity_name,
            });
        }

        Ok(Self {
            semantic_name,
            entity_name,
        })
    }

    /// Semantic namespace (`skills/<semantic_name>`).
    #[must_use]
    pub fn semantic_name(&self) -> &str {
        &self.semantic_name
    }

    /// Entity id under the `references` namespace.
    #[must_use]
    pub fn entity_name(&self) -> &str {
        &self.entity_name
    }

    /// Canonical semantic URI string with normalized segments.
    #[must_use]
    pub fn canonical_uri(&self) -> String {
        format!(
            "{WENDAO_URI_SCHEME}://{SKILLS_SEGMENT}/{}/{REFERENCES_SEGMENT}/{}",
            self.semantic_name, self.entity_name
        )
    }

    /// Zero-allocation relative entity path under the `references` namespace.
    #[must_use]
    pub fn entity_relative_path(&self) -> &Path {
        Path::new(self.entity_name())
    }

    /// Candidate relative paths under a skill's `references/` directory.
    #[must_use]
    pub fn candidate_paths(&self) -> Vec<PathBuf> {
        vec![PathBuf::from(self.entity_relative_path())]
    }
}

fn split_scheme(uri: &str) -> Result<(&str, &str), WendaoResourceUriError> {
    uri.split_once("://")
        .ok_or_else(|| WendaoResourceUriError::InvalidUri(uri.to_string()))
}

fn strip_uri_suffix(payload: &str) -> &str {
    let mut end = payload.len();
    if let Some(index) = payload.find('#') {
        end = end.min(index);
    }
    if let Some(index) = payload.find('?') {
        end = end.min(index);
    }
    payload[..end].trim_matches('/')
}

fn normalize_segment(segment: Option<&str>) -> Option<String> {
    segment
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase)
}

fn normalize_entity_path(raw_entity: &str, uri: &str) -> Result<String, String> {
    let trimmed = raw_entity.trim();
    if trimmed.is_empty() {
        return Err(trimmed.to_string());
    }
    if trimmed.split('/').any(|segment| segment.trim().is_empty()) {
        return Err(trimmed.to_string());
    }

    let path = Path::new(trimmed);
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => normalized.push(value),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(format!("{trimmed} (uri={uri})"));
            }
        }
    }

    let rendered = normalized.to_string_lossy().replace('\\', "/");
    if rendered.is_empty() {
        Err(trimmed.to_string())
    } else {
        Ok(rendered)
    }
}

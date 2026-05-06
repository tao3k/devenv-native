//! Parsers for repo-native semantic `SSOT` Markdown artifacts.

use crate::frontmatter::split_frontmatter_raw;
use crate::semantic_ssot::types::{SemanticChangeIntent, SemanticObject, SemanticProjection};
use std::fmt;
use std::path::Path;

/// Error returned when parsing one semantic artifact fails.
#[derive(Debug)]
pub enum SemanticArtifactParseError {
    /// The Markdown document does not start with YAML frontmatter.
    MissingFrontmatter,
    /// The YAML frontmatter cannot be deserialized into the expected schema.
    InvalidYaml(serde_yaml::Error),
}

impl fmt::Display for SemanticArtifactParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingFrontmatter => {
                write!(
                    formatter,
                    "document must start with a YAML frontmatter block"
                )
            }
            Self::InvalidYaml(error) => write!(formatter, "invalid semantic frontmatter: {error}"),
        }
    }
}

impl std::error::Error for SemanticArtifactParseError {}

/// Parse one semantic object artifact from Markdown content.
///
/// # Errors
///
/// Returns [`SemanticArtifactParseError`] when the document has no leading
/// YAML frontmatter or the frontmatter does not match the semantic object
/// schema.
pub fn parse_semantic_object(
    path: impl AsRef<Path>,
    content: &str,
) -> Result<SemanticObject, SemanticArtifactParseError> {
    let Some(frontmatter) = split_frontmatter_raw(content) else {
        return Err(SemanticArtifactParseError::MissingFrontmatter);
    };
    let mut object = serde_yaml::from_str::<SemanticObject>(frontmatter.yaml)
        .map_err(SemanticArtifactParseError::InvalidYaml)?;
    object.body = frontmatter.body.trim().to_string();
    object.source_path = path.as_ref().to_path_buf();
    Ok(object)
}

/// Parse one semantic projection artifact from Markdown content.
///
/// # Errors
///
/// Returns [`SemanticArtifactParseError`] when the document has no leading
/// YAML frontmatter or the frontmatter does not match the semantic projection
/// schema.
pub fn parse_semantic_projection(
    path: impl AsRef<Path>,
    content: &str,
) -> Result<SemanticProjection, SemanticArtifactParseError> {
    let Some(frontmatter) = split_frontmatter_raw(content) else {
        return Err(SemanticArtifactParseError::MissingFrontmatter);
    };
    let mut projection = serde_yaml::from_str::<SemanticProjection>(frontmatter.yaml)
        .map_err(SemanticArtifactParseError::InvalidYaml)?;
    projection.body = frontmatter.body.trim().to_string();
    projection.source_path = path.as_ref().to_path_buf();
    Ok(projection)
}

/// Parse one semantic change-intent artifact from Markdown content.
///
/// # Errors
///
/// Returns [`SemanticArtifactParseError`] when the document has no leading
/// YAML frontmatter or the frontmatter does not match the semantic
/// change-intent schema.
pub fn parse_semantic_change_intent(
    path: impl AsRef<Path>,
    content: &str,
) -> Result<SemanticChangeIntent, SemanticArtifactParseError> {
    let Some(frontmatter) = split_frontmatter_raw(content) else {
        return Err(SemanticArtifactParseError::MissingFrontmatter);
    };
    let mut intent = serde_yaml::from_str::<SemanticChangeIntent>(frontmatter.yaml)
        .map_err(SemanticArtifactParseError::InvalidYaml)?;
    intent.body = frontmatter.body.trim().to_string();
    intent.source_path = path.as_ref().to_path_buf();
    Ok(intent)
}

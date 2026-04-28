use std::path::Path;

use xiuxian_wendao_parsers::{SkillFrontmatterParseError, parse_skill_frontmatter};

use crate::skill_runtime::SkillRuntimeError;

pub(super) fn parse_semantic_name_from_skill_doc(
    path: &Path,
) -> Result<Option<String>, SkillRuntimeError> {
    let content =
        std::fs::read_to_string(path).map_err(|source| SkillRuntimeError::ReadSkillDescriptor {
            path: path.to_path_buf(),
            source,
        })?;
    let frontmatter = parse_skill_frontmatter(content.as_str()).map_err(|source| match source {
        SkillFrontmatterParseError::InvalidYaml(source) => {
            SkillRuntimeError::ParseSkillFrontmatter {
                path: path.to_path_buf(),
                source,
            }
        }
        SkillFrontmatterParseError::InvalidSchema(issues) => SkillRuntimeError::ScanSkillMetadata {
            path: path.to_path_buf(),
            reason: issues.join("; "),
        },
    })?;
    let name = frontmatter
        .name
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_default();

    if name.is_empty() {
        return Ok(None);
    }

    Ok(Some(name.to_ascii_lowercase()))
}

#[cfg(test)]
#[path = "../../../tests/unit/skill_runtime/index/semantic.rs"]
mod tests;

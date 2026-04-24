use std::path::Path;

use xiuxian_wendao_parsers::parse_skill_frontmatter_lenient;

use crate::skill_runtime::SkillRuntimeError;

pub(super) fn parse_semantic_name_from_skill_doc(
    path: &Path,
) -> Result<Option<String>, SkillRuntimeError> {
    let Some(skill_dir) = path.parent() else {
        return Ok(None);
    };

    let content =
        std::fs::read_to_string(path).map_err(|source| SkillRuntimeError::ReadSkillDescriptor {
            path: path.to_path_buf(),
            source,
        })?;
    let frontmatter = parse_skill_frontmatter_lenient(content.as_str()).map_err(|source| {
        SkillRuntimeError::ParseSkillFrontmatter {
            path: path.to_path_buf(),
            source,
        }
    })?;
    let fallback_name = skill_dir
        .file_name()
        .and_then(|value| value.to_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);
    let name = frontmatter
        .and_then(|metadata| metadata.name.map(|value| value.trim().to_string()))
        .filter(|value| !value.is_empty())
        .or(fallback_name)
        .unwrap_or_default();

    if name.is_empty() {
        return Ok(None);
    }

    Ok(Some(name.to_ascii_lowercase()))
}

#[cfg(test)]
#[path = "../../../tests/unit/skill_runtime/index/semantic.rs"]
mod tests;

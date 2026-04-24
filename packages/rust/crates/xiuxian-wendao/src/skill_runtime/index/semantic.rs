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
mod tests {
    use super::parse_semantic_name_from_skill_doc;
    use crate::skill_runtime::SkillRuntimeError;
    use tempfile::TempDir;

    #[test]
    fn parse_semantic_name_accepts_lenient_frontmatter_without_metadata()
    -> Result<(), Box<dyn std::error::Error>> {
        let temp = TempDir::new()?;
        let skill_dir = temp.path().join("agenda_skill");
        std::fs::create_dir_all(&skill_dir)?;
        let skill_doc = skill_dir.join("SKILL.md");
        std::fs::write(
            &skill_doc,
            "---\nname: agenda-management\ndescription: Agenda skill\n---\n# Skill\n",
        )?;

        let semantic_name = parse_semantic_name_from_skill_doc(skill_doc.as_path())?;
        assert_eq!(semantic_name.as_deref(), Some("agenda-management"));
        Ok(())
    }

    #[test]
    fn parse_semantic_name_falls_back_to_folder_name_without_frontmatter()
    -> Result<(), Box<dyn std::error::Error>> {
        let temp = TempDir::new()?;
        let skill_dir = temp.path().join("agenda_skill");
        std::fs::create_dir_all(&skill_dir)?;
        let skill_doc = skill_dir.join("SKILL.md");
        std::fs::write(&skill_doc, "# Skill\n")?;

        let semantic_name = parse_semantic_name_from_skill_doc(skill_doc.as_path())?;
        assert_eq!(semantic_name.as_deref(), Some("agenda_skill"));
        Ok(())
    }

    #[test]
    fn parse_semantic_name_rejects_invalid_yaml() -> Result<(), Box<dyn std::error::Error>> {
        let temp = TempDir::new()?;
        let skill_dir = temp.path().join("agenda_skill");
        std::fs::create_dir_all(&skill_dir)?;
        let skill_doc = skill_dir.join("SKILL.md");
        std::fs::write(&skill_doc, "---\nname: [agenda-skill\n---\n# Skill\n")?;

        match parse_semantic_name_from_skill_doc(skill_doc.as_path()) {
            Ok(_) => panic!("invalid yaml should fail"),
            Err(error) => assert!(matches!(
                error,
                SkillRuntimeError::ParseSkillFrontmatter { .. }
            )),
        }
        Ok(())
    }
}

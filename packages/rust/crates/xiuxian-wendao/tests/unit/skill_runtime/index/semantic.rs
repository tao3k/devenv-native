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

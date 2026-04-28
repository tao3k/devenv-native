use super::parse_semantic_name_from_skill_doc;
use crate::skill_runtime::SkillRuntimeError;
use tempfile::TempDir;

#[test]
fn parse_semantic_name_accepts_strict_skill_frontmatter() -> Result<(), Box<dyn std::error::Error>>
{
    let temp = TempDir::new()?;
    let skill_dir = temp.path().join("agenda_skill");
    std::fs::create_dir_all(&skill_dir)?;
    let skill_doc = skill_dir.join("SKILL.md");
    std::fs::write(
        &skill_doc,
        concat!(
            "---\n",
            "kind: SKILL.md\n",
            "type: skill\n",
            "title: Agenda Skill\n",
            "category: skills\n",
            "tags:\n",
            "  - agenda\n",
            "name: agenda-management\n",
            "description: Agenda skill\n",
            "author: xiuxian-artisan-workshop\n",
            "date: 2026-04-26T09:30-07:00\n",
            "metadata:\n",
            "  retrieval:\n",
            "    saliency_base: 5.5\n",
            "    decay_rate: 0.05\n",
            "  version: \"1.0.0\"\n",
            "  source: \"https://example.test/skills/agenda\"\n",
            "  routing_keywords:\n",
            "    - agenda\n",
            "---\n",
            "# Skill\n",
        ),
    )?;

    let semantic_name = parse_semantic_name_from_skill_doc(skill_doc.as_path())?;
    assert_eq!(semantic_name.as_deref(), Some("agenda-management"));
    Ok(())
}

#[test]
fn parse_semantic_name_rejects_missing_frontmatter() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let skill_dir = temp.path().join("agenda_skill");
    std::fs::create_dir_all(&skill_dir)?;
    let skill_doc = skill_dir.join("SKILL.md");
    std::fs::write(&skill_doc, "# Skill\n")?;

    match parse_semantic_name_from_skill_doc(skill_doc.as_path()) {
        Ok(_) => panic!("missing frontmatter should fail"),
        Err(error) => assert!(matches!(error, SkillRuntimeError::ScanSkillMetadata { .. })),
    }
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

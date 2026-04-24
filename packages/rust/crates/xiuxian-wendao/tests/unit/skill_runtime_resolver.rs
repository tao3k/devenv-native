//! Resolver contract tests for `wendao://skills/.../references/...`.

use std::path::Path;

use tempfile::TempDir;
use xiuxian_wendao::skill_runtime::{SkillRuntimeError, SkillRuntimeResolver};

const SKILL_FRONTMATTER: &str = r#"---
name: agenda-management
description: "Agenda skill"
---

# Agenda Skill
"#;

#[test]
fn resolves_reference_from_semantic_uri() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let root = temp.path().join("runtime");
    let skill_dir = root.join("agenda_skill");
    std::fs::create_dir_all(skill_dir.join("references"))?;
    std::fs::write(skill_dir.join("SKILL.md"), SKILL_FRONTMATTER)?;
    std::fs::write(
        skill_dir.join("references").join("steward.md"),
        "persona: strict-teacher",
    )?;

    let resolver = SkillRuntimeResolver::from_roots(&[root])?;
    let content = resolver.read_utf8("wendao://skills/agenda-management/references/steward.md")?;
    assert_eq!(content, "persona: strict-teacher");
    Ok(())
}

#[test]
#[ignore = "overlay precedence not yet implemented for semantic URI resolution"]
fn supports_overlay_precedence_by_root_order() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let bundled = temp.path().join("bundled");
    let user = temp.path().join("user");
    write_skill(
        bundled.as_path(),
        "agenda_skill",
        "steward.md",
        "source = bundled",
    )?;
    write_skill(
        user.as_path(),
        "agenda_skill",
        "steward.md",
        "source = user",
    )?;

    let resolver = SkillRuntimeResolver::from_roots(&[user.clone(), bundled.clone()])?;
    let path = resolver.resolve_path("wendao://skills/agenda-management/references/steward.md")?;
    assert!(path.starts_with(user.as_path()));
    Ok(())
}

#[test]
fn returns_not_found_for_missing_entity() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let root = temp.path().join("runtime");
    write_skill(
        root.as_path(),
        "agenda_skill",
        "steward.md",
        "source = runtime",
    )?;

    let resolver = SkillRuntimeResolver::from_roots(&[root])?;
    let error = resolver
        .resolve_path("wendao://skills/agenda-management/references/teacher.md")
        .expect_err("missing entity should fail");
    assert!(matches!(error, SkillRuntimeError::ResourceNotFound { .. }));
    Ok(())
}

fn write_skill(
    root: &Path,
    folder: &str,
    entity_file: &str,
    content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let skill_dir = root.join(folder);
    std::fs::create_dir_all(skill_dir.join("references"))?;
    std::fs::write(skill_dir.join("SKILL.md"), SKILL_FRONTMATTER)?;
    std::fs::write(skill_dir.join("references").join(entity_file), content)?;
    Ok(())
}

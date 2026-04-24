use super::SkillRuntimeResolver;
use std::path::Path;

#[test]
fn resolve_runtime_skill_root_with_resolves_relative_override() {
    let resolved = SkillRuntimeResolver::resolve_runtime_skill_root_with(
        Path::new("/repo/project"),
        Some(" skills/custom "),
    );
    assert_eq!(resolved, Path::new("/repo/project/skills/custom"));
}

#[test]
fn resolve_runtime_skill_root_with_preserves_absolute_override() {
    let resolved = SkillRuntimeResolver::resolve_runtime_skill_root_with(
        Path::new("/repo/project"),
        Some(" /tmp/runtime-skills "),
    );
    assert_eq!(resolved, Path::new("/tmp/runtime-skills"));
}

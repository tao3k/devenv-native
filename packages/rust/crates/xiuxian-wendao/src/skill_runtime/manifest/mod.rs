//! Skill runtime manifest loading and authority resolution helpers.

mod alias;
mod authority;
mod load;
mod types;

#[cfg(test)]
#[path = "../../../tests/unit/skill_runtime/manifest/mod.rs"]
mod tests;

pub use alias::{
    SKILL_BINDING_PREFIX, SkillBindingDescriptor, SkillNativeAliasCompilation,
    SkillNativeAliasCompileError, SkillNativeAliasMountReport, SkillNativeAliasSeed,
    SkillNativeAliasSpec, compile_skill_manifest_aliases, compile_skill_native_alias,
    resolve_skill_binding_target, skill_bindings, try_compile_skill_native_alias,
};
pub use authority::resolve_skill_authority;
pub use load::load_skill_manifest_from_path;
pub use types::{
    SKILL_RUNTIME_URI_PREFIX, SkillAuthorityOutcome, SkillAuthorityReport, SkillManifest,
    SkillManifestError, SkillManifestScan, SkillMetadata, SkillWorkflowType, ToolAnnotations,
    ToolBehaviorAnnotations,
};

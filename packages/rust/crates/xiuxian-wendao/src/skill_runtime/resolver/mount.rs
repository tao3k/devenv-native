//! Embedded resource mounting for the skill runtime resolver.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use include_dir::Dir;

use super::core::{EmbeddedSemanticMount, SkillRuntimeResolver};
use crate::skill_runtime::zhixing::{
    ZHIXING_EMBEDDED_CRATE_ID, embedded_resource_dir, embedded_semantic_reference_mounts,
};

impl SkillRuntimeResolver {
    /// Mount one embedded resource image and semantic reference map.
    ///
    /// `semantic_mounts` maps semantic name to one or more `references/` base
    /// directories that are relative to the mounted [`Dir`].
    #[must_use]
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn mount(
        mut self,
        crate_id: &str,
        dir: &'static Dir<'static>,
        semantic_mounts: &HashMap<String, Vec<PathBuf>>,
    ) -> Self {
        let Some(normalized_crate_id) = normalized_crate_id(crate_id) else {
            return self;
        };

        self.mounts.insert(normalized_crate_id.clone(), dir);
        mount_semantic_references(&mut self, normalized_crate_id.as_str(), semantic_mounts);

        self
    }

    /// Enable embedded `include_dir` resource mount for semantic reads.
    #[must_use]
    pub fn mount_embedded_dir(mut self) -> Self {
        self = self.mount(
            ZHIXING_EMBEDDED_CRATE_ID,
            embedded_resource_dir(),
            embedded_semantic_reference_mounts(),
        );
        self
    }
}

fn normalized_crate_id(crate_id: &str) -> Option<String> {
    let normalized = crate_id.trim().to_ascii_lowercase();
    (!normalized.is_empty()).then_some(normalized)
}

fn mount_semantic_references(
    resolver: &mut SkillRuntimeResolver,
    crate_id: &str,
    semantic_mounts: &HashMap<String, Vec<PathBuf>>,
) {
    for (semantic_name, references_dirs) in semantic_mounts {
        mount_semantic_reference_dirs(resolver, crate_id, semantic_name, references_dirs);
    }
}

fn mount_semantic_reference_dirs(
    resolver: &mut SkillRuntimeResolver,
    crate_id: &str,
    semantic_name: &str,
    references_dirs: &[PathBuf],
) {
    let semantic = semantic_name.trim().to_ascii_lowercase();
    if semantic.is_empty() {
        return;
    }

    let entry = resolver
        .embedded_mounts_by_semantic
        .entry(semantic)
        .or_default();
    for references_dir in references_dirs {
        push_unique_semantic_mount(entry, crate_id, references_dir);
    }
    entry.sort_by(|left, right| left.references_dir.cmp(&right.references_dir));
}

fn push_unique_semantic_mount(
    entry: &mut Vec<EmbeddedSemanticMount>,
    crate_id: &str,
    references_dir: &Path,
) {
    let mount = EmbeddedSemanticMount {
        crate_id: crate_id.to_string(),
        references_dir: references_dir.to_path_buf(),
    };
    if !entry.iter().any(|existing| existing == &mount) {
        entry.push(mount);
    }
}

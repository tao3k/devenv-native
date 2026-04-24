use std::collections::HashMap;
use std::path::PathBuf;

use include_dir::Dir;

use super::super::zhixing::{
    ZHIXING_EMBEDDED_CRATE_ID, embedded_resource_dir, embedded_semantic_reference_mounts,
};
use super::core::{EmbeddedSemanticMount, SkillRuntimeResolver};

impl SkillRuntimeResolver {
    /// Mount one embedded resource image and semantic reference map.
    ///
    /// `semantic_mounts` maps semantic name to one or more `references/` base
    /// directories that are relative to the mounted [`Dir`].
    #[must_use]
    pub fn mount(
        mut self,
        crate_id: &str,
        dir: &'static Dir<'static>,
        semantic_mounts: &HashMap<String, Vec<PathBuf>>,
    ) -> Self {
        let normalized_crate_id = crate_id.trim().to_ascii_lowercase();
        if normalized_crate_id.is_empty() {
            return self;
        }

        self.mounts.insert(normalized_crate_id.clone(), dir);
        for (semantic_name, references_dirs) in semantic_mounts {
            let semantic = semantic_name.trim().to_ascii_lowercase();
            if semantic.is_empty() {
                continue;
            }
            let entry = self
                .embedded_mounts_by_semantic
                .entry(semantic)
                .or_default();
            for references_dir in references_dirs {
                let mount = EmbeddedSemanticMount {
                    crate_id: normalized_crate_id.clone(),
                    references_dir: references_dir.clone(),
                };
                if !entry.iter().any(|existing| existing == &mount) {
                    entry.push(mount);
                }
            }
            entry.sort_by(|left, right| left.references_dir.cmp(&right.references_dir));
        }

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

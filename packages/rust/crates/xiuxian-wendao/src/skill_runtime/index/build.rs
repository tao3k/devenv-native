use std::path::PathBuf;

use xiuxian_wendao_parsers::discover_skill_documents;

use super::semantic::parse_semantic_name_from_skill_doc;
use super::{SkillInventory, SkillInventoryMount};
use crate::skill_runtime::SkillRuntimeError;

impl SkillInventory {
    /// Build namespace index by scanning skill descriptor files under roots.
    ///
    /// # Errors
    ///
    /// Returns [`SkillRuntimeError`] when descriptor I/O or frontmatter parsing fails.
    pub fn build_from_roots(roots: &[PathBuf]) -> Result<Self, SkillRuntimeError> {
        let mut index = Self::default();

        for root in roots {
            if !root.exists() || !root.is_dir() {
                continue;
            }
            for skill_doc in discover_skill_documents(root) {
                let Some(semantic_name) = parse_semantic_name_from_skill_doc(skill_doc.as_path())?
                else {
                    continue;
                };
                let references_dir = skill_doc
                    .parent()
                    .map_or_else(PathBuf::new, |parent| parent.join("references"));
                index
                    .mounts_by_name
                    .entry(semantic_name.clone())
                    .or_default()
                    .push(SkillInventoryMount {
                        semantic_name: semantic_name.clone(),
                        skill_doc,
                        references_dir,
                    });

                index.preload_references_for_semantic(&semantic_name);
            }
        }

        Ok(index)
    }
}

use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{Result, anyhow};
use xiuxian_wendao_core::WendaoResourceUri;

use super::super::SkillVfsResolver;
use crate::link_graph::LinkGraphIndex;

const INTERNAL_SKILL_DOC_NAME: &str = "SKILL.md";
const QIANJI_TOML_FILE: &str = "qianji.toml";

/// Deduplicated internal manifest intents declared by root `SKILL.md` documents.
///
/// This catalog can be built once from preloaded [`LinkGraphIndex`] values and reused across
/// repeated authority audits to avoid rebuilding link graphs during the same startup flow.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InternalSkillIntentCatalog {
    /// Canonical manifest URIs declared by root `SKILL.md` documents.
    pub intended_manifests: Vec<String>,
}

impl InternalSkillIntentCatalog {
    /// Build a reusable internal-skill intent catalog from prebuilt link-graph indexes.
    #[must_use]
    pub fn from_link_graph_indexes<'a, I>(indexes: I) -> Self
    where
        I: IntoIterator<Item = &'a LinkGraphIndex>,
    {
        let mut intended_manifests = BTreeSet::new();
        for index in indexes {
            extend_manifest_intents_from_index(index, &mut intended_manifests);
        }
        Self {
            intended_manifests: intended_manifests.into_iter().collect(),
        }
    }
}

impl SkillVfsResolver {
    /// Collect internal manifest intents from root `SKILL.md` documents mounted by this resolver.
    ///
    /// # Errors
    ///
    /// Returns an error when the internal-skill [`LinkGraphIndex`] cannot be built or when a
    /// mounted `SKILL.md` document cannot be reparsed for raw intent targets.
    pub fn collect_internal_manifest_intents(&self) -> Result<InternalSkillIntentCatalog> {
        let mut intended_manifests = BTreeSet::new();
        for root in self.internal_roots() {
            let path: &Path = root.as_path();
            let index = LinkGraphIndex::build(path).map_err(|error| {
                anyhow!(
                    "build link graph index for internal skill root `{}`: {error}",
                    root.display()
                )
            })?;
            extend_manifest_intents_from_index(&index, &mut intended_manifests);
        }
        Ok(InternalSkillIntentCatalog {
            intended_manifests: intended_manifests.into_iter().collect(),
        })
    }
}

pub(crate) fn extend_manifest_intents_from_index(
    index: &LinkGraphIndex,
    intended_manifests: &mut BTreeSet<String>,
) {
    let total_notes = index.stats().total_notes;
    for doc in index
        .toc(total_notes.max(1))
        .into_iter()
        .filter(|doc| is_internal_skill_doc_path(doc.path.as_str()))
    {
        let (note_links, attachments) = index.intent_targets(doc.id.as_str());
        let note_links_iter: Vec<String> = note_links;
        let attachments_iter: Vec<String> = attachments;
        for raw_target in note_links_iter.into_iter().chain(attachments_iter) {
            let Some(manifest_uri) = normalize_manifest_intent(raw_target.as_str()) else {
                continue;
            };
            intended_manifests.insert(manifest_uri);
        }
    }
}

fn is_internal_skill_doc_path(path: &str) -> bool {
    let normalized = path.trim().replace('\\', "/");
    normalized.ends_with(&format!("/{INTERNAL_SKILL_DOC_NAME}"))
        && normalized
            .split('/')
            .filter(|segment| !segment.is_empty())
            .count()
            == 2
}

fn normalize_manifest_intent(raw_target: &str) -> Option<String> {
    let trimmed = raw_target.trim();
    if trimmed.is_empty() {
        return None;
    }

    let candidate_uri = if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        let normalized = trimmed
            .trim_start_matches('$')
            .trim_start_matches("./")
            .trim_start_matches('/')
            .replace('\\', "/");
        if normalized.is_empty() {
            return None;
        }
        format!("wendao://skills-internal/{normalized}")
    };

    let uri = WendaoResourceUri::parse(candidate_uri.as_str()).ok()?;
    if !uri.is_internal_skill() {
        return None;
    }

    let entity_path = Path::new(uri.entity_name());
    let segments = uri
        .entity_name()
        .split('/')
        .filter(|segment| !segment.trim().is_empty())
        .collect::<Vec<_>>();
    if segments.len() < 2 {
        return None;
    }
    if entity_path.file_name().and_then(|value| value.to_str()) != Some(QIANJI_TOML_FILE) {
        return None;
    }

    Some(uri.canonical_uri())
}

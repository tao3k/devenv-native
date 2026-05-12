//! `skill_runtime::authority::report` owns Wendao skill runtime authority report behavior.

use std::collections::BTreeSet;

use super::catalog::SkillIntentCatalog;

/// Namespace boundary: this public name is scoped by its module owner.
/// Cross-check result between `SKILL.md` intention links and physically mounted manifests.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SkillAuthorityReport {
    /// Manifests both declared by `SKILL.md` intent and present physically.
    pub authorized_manifests: Vec<String>,
    /// Manifest intents declared in `SKILL.md` that do not exist physically.
    pub ghost_links: Vec<String>,
    /// Physical manifests that exist on disk but are not granted by `SKILL.md` intent.
    pub unauthorized_manifests: Vec<String>,
}

#[must_use]
pub(crate) fn build_authority_report(
    physical_manifests: &BTreeSet<String>,
    catalog: &SkillIntentCatalog,
) -> SkillAuthorityReport {
    let intended_manifests = catalog
        .intended_manifests
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let authorized_manifests = intended_manifests
        .intersection(physical_manifests)
        .cloned()
        .collect::<Vec<_>>();
    let ghost_links = intended_manifests
        .difference(physical_manifests)
        .cloned()
        .collect::<Vec<_>>();
    let unauthorized_manifests = physical_manifests
        .difference(&intended_manifests)
        .cloned()
        .collect::<Vec<_>>();

    SkillAuthorityReport {
        authorized_manifests,
        ghost_links,
        unauthorized_manifests,
    }
}

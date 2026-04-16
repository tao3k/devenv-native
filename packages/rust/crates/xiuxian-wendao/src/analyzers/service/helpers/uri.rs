#[cfg(test)]
use crate::analyzers::RelationKind;

/// Returns a human-readable label for a relation kind.
#[must_use]
#[cfg(test)]
pub(crate) fn relation_kind_label(kind: RelationKind) -> &'static str {
    match kind {
        RelationKind::Contains => "contains",
        RelationKind::Calls => "calls",
        RelationKind::Uses => "uses",
        RelationKind::Documents => "documents",
        RelationKind::ExampleOf => "example_of",
        RelationKind::Declares => "declares",
        RelationKind::Implements => "implements",
        RelationKind::Imports => "imports",
    }
}

pub(crate) fn repo_hierarchical_uri(repo_id: &str) -> String {
    format!("repo://{repo_id}")
}

pub(crate) fn record_hierarchical_uri(
    repo_id: &str,
    ecosystem: &str,
    scope: &str,
    module_path: &str,
    record_id: &str,
) -> String {
    let clean_module = module_path.trim_matches('/').replace('/', ":");
    format!("wendao://repo/{ecosystem}/{repo_id}/{scope}/{clean_module}/{record_id}")
}

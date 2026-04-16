use std::collections::BTreeSet;

use crate::analyzers::{RelationKind, RelationRecord};

pub(super) fn dedupe_relations(relations: &mut Vec<RelationRecord>) {
    let mut seen = BTreeSet::new();
    relations.retain(|relation| {
        seen.insert((
            relation.repo_id.clone(),
            relation.source_id.clone(),
            relation.target_id.clone(),
            relation_kind_key(relation.kind),
        ))
    });
}

fn relation_kind_key(kind: RelationKind) -> &'static str {
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

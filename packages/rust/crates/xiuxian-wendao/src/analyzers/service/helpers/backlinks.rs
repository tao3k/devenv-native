use std::collections::BTreeMap;

use crate::analyzers::RepoBacklinkItem;
use crate::analyzers::{DocRecord, RelationKind, RelationRecord};

pub(crate) fn documents_backlink_lookup(
    relations: &[RelationRecord],
    docs: &[DocRecord],
) -> BTreeMap<String, Vec<RepoBacklinkItem>> {
    let doc_lookup = docs
        .iter()
        .map(|doc| (doc.doc_id.as_str(), doc))
        .collect::<BTreeMap<_, _>>();
    let mut lookup: BTreeMap<String, BTreeMap<String, RepoBacklinkItem>> = BTreeMap::new();

    for relation in relations
        .iter()
        .filter(|relation| relation.kind == RelationKind::Documents)
    {
        let source_id = relation.source_id.trim();
        let target_id = relation.target_id.trim();
        if source_id.is_empty() || target_id.is_empty() {
            continue;
        }
        let item = doc_lookup.get(source_id).map_or_else(
            || RepoBacklinkItem {
                id: source_id.to_string(),
                title: None,
                path: None,
                kind: Some("documents".to_string()),
            },
            |doc| RepoBacklinkItem {
                id: doc.doc_id.clone(),
                title: Some(doc.title.clone()).filter(|title| !title.trim().is_empty()),
                path: Some(doc.path.clone()).filter(|path| !path.trim().is_empty()),
                kind: Some("documents".to_string()),
            },
        );
        lookup
            .entry(target_id.to_string())
            .or_default()
            .insert(item.id.clone(), item);
    }

    lookup
        .into_iter()
        .map(|(target_id, sources)| (target_id, sources.into_values().collect::<Vec<_>>()))
        .collect()
}

pub(crate) fn backlinks_for(
    target_id: &str,
    lookup: &BTreeMap<String, Vec<RepoBacklinkItem>>,
) -> (Option<Vec<String>>, Option<Vec<RepoBacklinkItem>>) {
    let Some(backlinks) = lookup.get(target_id) else {
        return (None, None);
    };
    let items = backlinks
        .iter()
        .filter_map(|backlink| {
            let id = backlink.id.trim();
            (!id.is_empty()).then(|| RepoBacklinkItem {
                id: id.to_string(),
                title: backlink
                    .title
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string),
                path: backlink
                    .path
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string),
                kind: backlink
                    .kind
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string),
            })
        })
        .collect::<Vec<_>>();
    if items.is_empty() {
        return (None, None);
    }
    let ids = items.iter().map(|item| item.id.clone()).collect::<Vec<_>>();
    (Some(ids), Some(items))
}

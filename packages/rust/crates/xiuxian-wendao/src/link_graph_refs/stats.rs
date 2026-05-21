//! `link_graph_refs::stats` owns Wendao link graph refs stats behavior.

use super::extract::extract_entity_refs;
use super::model::LinkGraphEntityRef;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::HashSet;

/// Entity reference statistics for a note.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LinkGraphRefStats {
    /// Total references count
    pub total_refs: usize,
    /// Unique entities referenced
    pub unique_entities: usize,
    /// References grouped by structural link shape.
    pub by_type: Vec<(String, usize)>,
}

impl LinkGraphRefStats {
    /// Create stats from entity references.
    #[must_use]
    pub fn from_refs(refs: &[LinkGraphEntityRef]) -> Self {
        let mut type_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        let mut unique_names: HashSet<String> = HashSet::new();

        for ref_item in refs {
            unique_names.insert(ref_item.name.clone());
            let bucket = if ref_item.target_address.is_some() {
                "addressed".to_string()
            } else {
                "note".to_string()
            };
            *type_counts.entry(bucket).or_insert(0) += 1;
        }

        let mut by_type: Vec<(String, usize)> = type_counts.into_iter().collect();
        by_type.sort_by_key(|item| Reverse(item.1));

        Self {
            total_refs: refs.len(),
            unique_entities: unique_names.len(),
            by_type,
        }
    }
}

/// Get statistics for entity references in content.
#[must_use]
pub fn get_ref_stats(content: &str) -> LinkGraphRefStats {
    LinkGraphRefStats::from_refs(&extract_entity_refs(content))
}

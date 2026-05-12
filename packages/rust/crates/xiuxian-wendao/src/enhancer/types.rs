//! `enhancer::types` owns Wendao enhancer types behavior.

use serde::{Deserialize, Serialize};
use xiuxian_wendao_parsers::NoteFrontmatter;

/// Input for a single note to be enhanced.
#[derive(Debug, Clone)]
pub struct NoteInput {
    /// Relative path to the note (e.g. `docs/architecture/foo.md`).
    pub path: String,
    /// Note title (from backend or frontmatter).
    pub title: String,
    /// Full raw content of the note.
    pub content: String,
}

/// A relation inferred from note structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Stringly state boundary: this public record preserves serialized catalog tokens from external or stored Wendao data.
pub struct InferredRelation {
    /// Source entity name.
    pub source: String,
    /// Optional source section or anchor address.
    pub source_address: Option<String>,
    /// Target entity name.
    pub target: String,
    /// Optional target section or anchor address.
    pub target_address: Option<String>,
    /// Optional explicit semantic relation tag. `None` means structural link only.
    pub relation_type: Option<String>,
    /// Optional explicit metadata owner, such as a property drawer key.
    pub metadata_owner: Option<String>,
    /// Human-readable description of the relation.
    pub description: String,
}

/// A note enriched with secondary analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedNote {
    /// Note path.
    pub path: String,
    /// Note title.
    pub title: String,
    /// Parsed YAML frontmatter.
    pub frontmatter: NoteFrontmatter,
    /// Entity references extracted from wikilinks.
    pub entity_refs: Vec<EntityRefData>,
    /// Reference statistics.
    pub ref_stats: RefStatsData,
    /// Relations inferred from note structure.
    pub inferred_relations: Vec<InferredRelation>,
}

/// Serializable entity reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRefData {
    /// Entity name.
    pub name: String,
    /// Optional Obsidian-style heading or block address.
    pub target_address: Option<String>,
    /// Original matched text.
    pub original: String,
}

/// Serializable reference statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RefStatsData {
    /// Total entity references found.
    pub total_refs: usize,
    /// Number of unique entities referenced.
    pub unique_entities: usize,
    /// Reference counts grouped by structural link shape.
    pub by_type: Vec<(String, usize)>,
}

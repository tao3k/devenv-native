use super::super::models::{
    LinkGraphAttachment, LinkGraphDocument, LinkGraphPassage, PageIndexNode,
};
use super::build;
use crate::parsers::markdown::{CodeObservation, LogbookEntry, ParsedSection};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use xiuxian_wendao_parsers::sections::MarkdownSection;

/// A virtual node synthesized from collapsed dense clusters during knowledge distillation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkGraphVirtualNode {
    /// Synthesized identifier (e.g., "virtual:cluster:0:abc123").
    pub id: String,
    /// Original member node IDs that were collapsed.
    pub members: Vec<String>,
    /// Average saliency of collapsed nodes.
    pub avg_saliency: f64,
    /// Synthesized title.
    pub title: String,
    /// Internal edge count (edges between members).
    pub internal_edges: usize,
    /// Edge density within cluster (0.0-1.0).
    pub edge_density: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct IndexedSection {
    pub(crate) heading_title: String,
    pub(crate) heading_path: String,
    pub(crate) heading_path_lower: String,
    pub(crate) heading_level: usize,
    pub(crate) line_start: usize,
    pub(crate) line_end: usize,
    pub(crate) byte_start: usize,
    pub(crate) byte_end: usize,
    pub(crate) section_text: String,
    pub(crate) section_text_lower: String,
    pub(crate) entities: Vec<String>,
    /// Property drawer attributes extracted from heading (e.g., :ID: arch-v1).
    pub(crate) attributes: HashMap<String, String>,
    /// Execution log entries from :LOGBOOK: drawer (Blueprint v2.4).
    pub(crate) logbook: Vec<LogbookEntry>,
    /// Code observations from :OBSERVE: property drawer (Blueprint v2.7).
    pub(crate) observations: Vec<CodeObservation>,
}

impl IndexedSection {
    pub(crate) fn from_parsed(value: &ParsedSection) -> Self {
        Self {
            heading_title: value.heading_title.clone(),
            heading_path: value.heading_path.clone(),
            heading_path_lower: value.heading_path_lower.clone(),
            heading_level: value.heading_level,
            line_start: value.line_start,
            line_end: value.line_end,
            byte_start: value.byte_start,
            byte_end: value.byte_end,
            section_text: value.section_text.clone(),
            section_text_lower: value.section_text_lower.clone(),
            entities: value.entities.clone(),
            attributes: value.attributes.clone(),
            logbook: value.logbook.clone(),
            observations: value.observations.clone(),
        }
    }

    pub(crate) fn from_markdown_section(value: &MarkdownSection) -> Self {
        Self {
            heading_title: value.scope.heading_title.clone(),
            heading_path: value.scope.heading_path.clone(),
            heading_path_lower: value.scope.heading_path_lower.clone(),
            heading_level: value.scope.heading_level,
            line_start: value.scope.line_start,
            line_end: value.scope.line_end,
            byte_start: value.scope.byte_start,
            byte_end: value.scope.byte_end,
            section_text: value.section_text.clone(),
            section_text_lower: value.section_text_lower.clone(),
            entities: Vec::new(),
            attributes: value.metadata.attributes.clone(),
            logbook: value.metadata.logbook.clone(),
            observations: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SectionMatch {
    pub(crate) score: f64,
    pub(crate) heading_path: Option<String>,
    pub(crate) reason: &'static str,
}

#[derive(Debug, Clone)]
pub(crate) struct SectionCandidate {
    pub(crate) heading_path: String,
    pub(crate) score: f64,
    pub(crate) reason: &'static str,
}

/// Cache build metadata emitted by the `LinkGraph` cache bootstrap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkGraphCacheBuildMeta {
    /// Cache backend name.
    pub backend: String,
    /// Cache status (`hit` or `miss`).
    pub status: String,
    /// Miss reason when status is `miss`.
    pub miss_reason: Option<String>,
    /// Cache schema version string.
    pub schema_version: String,
    /// Cache schema fingerprint (derived from schema JSON content).
    pub schema_fingerprint: String,
}

/// Reference to a code observation symbol within a document.
///
/// Used by the Symbol-to-Node Inverted Index for O(1) semantic change propagation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolRef {
    /// Document ID where the symbol is observed.
    pub doc_id: String,
    /// Node ID within the document.
    pub node_id: String,
    /// The observation pattern containing this symbol.
    pub pattern: String,
    /// Language of the observation.
    pub language: String,
    /// Line number in the document.
    pub line_number: Option<usize>,
    /// Optional scope filter for the observation.
    pub scope: Option<String>,
}

/// Statistics about the symbol cache.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolCacheStats {
    /// Number of unique symbols in the cache.
    pub unique_symbols: usize,
    /// Total number of symbol-to-document references.
    pub total_references: usize,
}

/// Parent resolution for a page-index node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageIndexParent<'a> {
    /// The node exists and is a root node.
    Root,
    /// The node exists and is nested under the given parent node id.
    Parent(&'a str),
}

/// Fast in-memory markdown link graph index.
#[derive(Debug, Clone)]
pub struct LinkGraphIndex {
    pub(crate) root: PathBuf,
    pub(crate) include_dirs: Vec<String>,
    pub(crate) excluded_dirs: Vec<String>,
    pub(crate) docs_by_id: HashMap<String, LinkGraphDocument>,
    pub(crate) sections_by_doc: HashMap<String, Vec<IndexedSection>>,
    pub(crate) passages_by_id: HashMap<String, LinkGraphPassage>,
    pub(crate) attachments_by_doc: HashMap<String, Vec<LinkGraphAttachment>>,
    pub(crate) trees_by_doc: HashMap<String, Vec<PageIndexNode>>,
    /// Map page-index node ids to parent node ids (None for roots).
    pub(crate) node_parent_map: HashMap<String, Option<String>>,
    /// Explicit anchor registry (`doc_id#ID`) for fast semantic resolution.
    pub(crate) explicit_id_registry: HashMap<String, PageIndexNode>,
    pub(crate) alias_to_doc_id: HashMap<String, String>,
    pub(crate) outgoing: HashMap<String, HashSet<String>>,
    pub(crate) incoming: HashMap<String, HashSet<String>>,
    pub(crate) rank_by_id: HashMap<String, f64>,
    pub(crate) edge_count: usize,
    /// Virtual nodes created by knowledge distillation (collapsed dense clusters).
    pub(crate) virtual_nodes: HashMap<String, build::VirtualNode>,
    /// Symbol-to-Node Inverted Index for O(1) semantic change propagation.
    /// Maps symbol names extracted from :OBSERVE: patterns to their document locations.
    pub(crate) symbol_to_docs: HashMap<String, Vec<SymbolRef>>,
}

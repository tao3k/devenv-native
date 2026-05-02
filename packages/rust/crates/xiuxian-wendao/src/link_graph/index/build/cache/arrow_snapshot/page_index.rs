//! Residual page-index JSON encoding for the link-graph snapshot cache.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::link_graph::index::{LinkGraphIndex, SymbolRef, pattern_symbols};
use crate::link_graph::models::{MarkdownBlock, MarkdownBlockKind, PageIndexMeta, PageIndexNode};
use crate::parsers::markdown::{CodeObservation, LogbookEntry};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedPageIndexNode {
    node_id: String,
    parent_id: Option<String>,
    title: String,
    level: usize,
    text: String,
    summary: Option<String>,
    children: Vec<CachedPageIndexNode>,
    metadata: CachedPageIndexMeta,
    blocks: Vec<CachedMarkdownBlock>,
}

impl From<&PageIndexNode> for CachedPageIndexNode {
    fn from(value: &PageIndexNode) -> Self {
        Self {
            node_id: value.node_id.clone(),
            parent_id: value.parent_id.clone(),
            title: value.title.clone(),
            level: value.level,
            text: value.text.to_string(),
            summary: value.summary.clone(),
            children: value.children.iter().map(Self::from).collect(),
            metadata: CachedPageIndexMeta::from(&value.metadata),
            blocks: value.blocks.iter().map(CachedMarkdownBlock::from).collect(),
        }
    }
}

impl CachedPageIndexNode {
    fn into_node(self) -> PageIndexNode {
        PageIndexNode {
            node_id: self.node_id,
            parent_id: self.parent_id,
            title: self.title,
            level: self.level,
            text: Arc::<str>::from(self.text),
            summary: self.summary,
            children: self
                .children
                .into_iter()
                .map(CachedPageIndexNode::into_node)
                .collect(),
            metadata: self.metadata.into_meta(),
            blocks: self
                .blocks
                .into_iter()
                .map(CachedMarkdownBlock::into_block)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedPageIndexMeta {
    line_range: (usize, usize),
    byte_range: Option<(usize, usize)>,
    structural_path: Vec<String>,
    content_hash: Option<String>,
    attributes: HashMap<String, String>,
    token_count: usize,
    is_thinned: bool,
    logbook: Vec<LogbookEntry>,
    observations: Vec<CodeObservation>,
}

impl From<&PageIndexMeta> for CachedPageIndexMeta {
    fn from(value: &PageIndexMeta) -> Self {
        Self {
            line_range: value.line_range,
            byte_range: value.byte_range,
            structural_path: value.structural_path.clone(),
            content_hash: value.content_hash.clone(),
            attributes: value.attributes.clone(),
            token_count: value.token_count,
            is_thinned: value.is_thinned,
            logbook: value.logbook.clone(),
            observations: value.observations.clone(),
        }
    }
}

impl CachedPageIndexMeta {
    fn into_meta(self) -> PageIndexMeta {
        PageIndexMeta {
            line_range: self.line_range,
            byte_range: self.byte_range,
            structural_path: self.structural_path,
            content_hash: self.content_hash,
            attributes: self.attributes,
            token_count: self.token_count,
            is_thinned: self.is_thinned,
            logbook: self.logbook,
            observations: self.observations,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedMarkdownBlock {
    block_id: String,
    kind: CachedMarkdownBlockKind,
    byte_range: (usize, usize),
    line_range: (usize, usize),
    content_hash: String,
    content: String,
    id: Option<String>,
    structural_path: Vec<String>,
}

impl From<&MarkdownBlock> for CachedMarkdownBlock {
    fn from(value: &MarkdownBlock) -> Self {
        Self {
            block_id: value.block_id.clone(),
            kind: CachedMarkdownBlockKind::from(&value.kind),
            byte_range: value.byte_range,
            line_range: value.line_range,
            content_hash: value.content_hash.clone(),
            content: value.content.to_string(),
            id: value.id.clone(),
            structural_path: value.structural_path.clone(),
        }
    }
}

impl CachedMarkdownBlock {
    fn into_block(self) -> MarkdownBlock {
        MarkdownBlock {
            block_id: self.block_id,
            kind: self.kind.into_kind(),
            byte_range: self.byte_range,
            line_range: self.line_range,
            content_hash: self.content_hash,
            content: Arc::<str>::from(self.content),
            id: self.id,
            structural_path: self.structural_path,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum CachedMarkdownBlockKind {
    Paragraph,
    CodeFence { language: String },
    List { ordered: bool },
    BlockQuote,
    ThematicBreak,
    Table,
    HtmlBlock,
}

impl From<&MarkdownBlockKind> for CachedMarkdownBlockKind {
    fn from(value: &MarkdownBlockKind) -> Self {
        match value {
            MarkdownBlockKind::Paragraph => Self::Paragraph,
            MarkdownBlockKind::CodeFence { language } => Self::CodeFence {
                language: language.clone(),
            },
            MarkdownBlockKind::List { ordered } => Self::List { ordered: *ordered },
            MarkdownBlockKind::BlockQuote => Self::BlockQuote,
            MarkdownBlockKind::ThematicBreak => Self::ThematicBreak,
            MarkdownBlockKind::Table => Self::Table,
            MarkdownBlockKind::HtmlBlock => Self::HtmlBlock,
        }
    }
}

impl CachedMarkdownBlockKind {
    fn into_kind(self) -> MarkdownBlockKind {
        match self {
            Self::Paragraph => MarkdownBlockKind::Paragraph,
            Self::CodeFence { language } => MarkdownBlockKind::CodeFence { language },
            Self::List { ordered } => MarkdownBlockKind::List { ordered },
            Self::BlockQuote => MarkdownBlockKind::BlockQuote,
            Self::ThematicBreak => MarkdownBlockKind::ThematicBreak,
            Self::Table => MarkdownBlockKind::Table,
            Self::HtmlBlock => MarkdownBlockKind::HtmlBlock,
        }
    }
}

pub(super) fn encode_cached_page_indices(
    trees_by_doc: &HashMap<String, Vec<PageIndexNode>>,
) -> Result<String, String> {
    let cached = trees_by_doc
        .iter()
        .map(|(doc_id, nodes)| {
            (
                doc_id.clone(),
                nodes.iter().map(CachedPageIndexNode::from).collect(),
            )
        })
        .collect::<BTreeMap<_, Vec<_>>>();
    serde_json::to_string(&cached)
        .map_err(|error| format!("serialize link-graph page-index residuals: {error}"))
}

pub(super) fn decode_cached_page_indices(
    raw: &str,
) -> Result<HashMap<String, Vec<PageIndexNode>>, String> {
    let cached = serde_json::from_str::<BTreeMap<String, Vec<CachedPageIndexNode>>>(raw)
        .map_err(|error| format!("decode link-graph page-index residuals: {error}"))?;
    Ok(cached
        .into_iter()
        .map(|(doc_id, nodes)| {
            (
                doc_id,
                nodes
                    .into_iter()
                    .map(CachedPageIndexNode::into_node)
                    .collect(),
            )
        })
        .collect())
}

pub(super) fn rebuild_cached_page_index_maps(index: &mut LinkGraphIndex) {
    let mut node_parent_map = HashMap::new();
    let mut explicit_id_registry = HashMap::new();
    let mut symbol_to_docs = HashMap::new();
    for nodes in index.trees_by_doc.values() {
        index_cached_page_index_nodes(
            &mut node_parent_map,
            &mut explicit_id_registry,
            &mut symbol_to_docs,
            nodes,
            None,
        );
    }
    index.node_parent_map = node_parent_map;
    index.explicit_id_registry = explicit_id_registry;
    index.symbol_to_docs = symbol_to_docs;
}

fn index_cached_page_index_nodes(
    node_parent_map: &mut HashMap<String, Option<String>>,
    explicit_id_registry: &mut HashMap<String, PageIndexNode>,
    symbol_to_docs: &mut HashMap<String, Vec<SymbolRef>>,
    nodes: &[PageIndexNode],
    parent_id: Option<&str>,
) {
    for node in nodes {
        node_parent_map.insert(node.node_id.clone(), parent_id.map(str::to_string));
        if let Some(id) = node.metadata.attributes.get("ID")
            && !id.trim().is_empty()
        {
            explicit_id_registry
                .entry(node.node_id.clone())
                .or_insert_with(|| node.clone());
        }
        for obs in &node.metadata.observations {
            for symbol in pattern_symbols::extract_pattern_symbols(&obs.pattern) {
                let symbol_ref = SymbolRef {
                    doc_id: node.node_id.split('#').next().unwrap_or("").to_string(),
                    node_id: node.node_id.clone(),
                    pattern: obs.pattern.clone(),
                    language: obs.language.clone(),
                    line_number: obs.line_number,
                    scope: obs.scope.clone(),
                };
                symbol_to_docs.entry(symbol).or_default().push(symbol_ref);
            }
        }
        index_cached_page_index_nodes(
            node_parent_map,
            explicit_id_registry,
            symbol_to_docs,
            &node.children,
            Some(node.node_id.as_str()),
        );
    }
}

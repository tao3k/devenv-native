//! Runner for `page_index` scenario tests.

use std::error::Error;
use std::path::Path;

use serde_json::{Value, json};
use xiuxian_wendao::LinkGraphIndex;
use xiuxian_wendao::link_graph::PageIndexNode;

use crate::scenarios::support::{Scenario, ScenarioRunner, find_first_doc_name};

/// Runner for `page_index` category scenarios.
pub struct PageIndexRunner;

impl ScenarioRunner for PageIndexRunner {
    fn category(&self) -> &'static str {
        "page_index"
    }

    fn run(&self, _scenario: &Scenario, temp_dir: &Path) -> Result<Value, Box<dyn Error>> {
        // Build the link graph index
        let index = LinkGraphIndex::build(temp_dir)?;

        // Get the document name from input
        let doc_name = find_first_doc_name(temp_dir)?;

        // Get the page index roots
        let roots = index
            .page_index(&doc_name)
            .ok_or_else(|| format!("missing page index for {doc_name}"))?;

        // Generate snapshot
        Ok(page_index_tree_snapshot(roots))
    }
}

/// Generate a JSON snapshot of page index nodes.
fn page_index_tree_snapshot(nodes: &[PageIndexNode]) -> Value {
    json!({
        "nodes": nodes.iter().map(snapshot_page_index_node).collect::<Vec<_>>(),
    })
}

fn snapshot_page_index_node(node: &PageIndexNode) -> Value {
    json!({
        "node_id": node.node_id,
        "title": node.title,
        "level": node.level,
        "text": node.text.as_ref(),
        "summary": node.summary,
        "line_range": [node.metadata.line_range.0, node.metadata.line_range.1],
        "token_count": node.metadata.token_count,
        "is_thinned": node.metadata.is_thinned,
        "blocks": node.blocks.iter().map(snapshot_block).collect::<Vec<_>>(),
        "children": node.children.iter().map(snapshot_page_index_node).collect::<Vec<_>>(),
    })
}

fn snapshot_block(block: &xiuxian_wendao::link_graph::MarkdownBlock) -> Value {
    use xiuxian_wendao::link_graph::MarkdownBlockKind;
    let kind_str = match &block.kind {
        MarkdownBlockKind::Paragraph => "Paragraph".to_string(),
        MarkdownBlockKind::CodeFence { language } => format!("CodeFence({language})"),
        MarkdownBlockKind::List { ordered: true } => "OrderedList".to_string(),
        MarkdownBlockKind::List { ordered: false } => "UnorderedList".to_string(),
        MarkdownBlockKind::BlockQuote => "BlockQuote".to_string(),
        MarkdownBlockKind::ThematicBreak => "ThematicBreak".to_string(),
        MarkdownBlockKind::Table => "Table".to_string(),
        MarkdownBlockKind::HtmlBlock => "HtmlBlock".to_string(),
    };
    json!({
        "block_id": block.block_id,
        "kind": kind_str,
        "byte_range": [block.byte_range.0, block.byte_range.1],
        "line_range": [block.line_range.0, block.line_range.1],
        "content_hash": block.content_hash,
    })
}

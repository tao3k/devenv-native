use std::collections::HashMap;

use crate::studio::types::{AnalysisEdge, AnalysisNode, AnalysisNodeKind, MarkdownRetrievalAtom};

pub(crate) struct CompiledDocument {
    pub(crate) document_hash: String,
    pub(crate) nodes: Vec<AnalysisNode>,
    pub(crate) edges: Vec<AnalysisEdge>,
    pub(crate) retrieval_atoms: Vec<MarkdownRetrievalAtom>,
    pub(crate) diagnostics: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct EdgeDraft<'a> {
    pub(crate) kind: crate::studio::types::AnalysisEdgeKind,
    pub(crate) source_id: String,
    pub(crate) target_id: String,
    pub(crate) label: String,
    pub(crate) path: &'a str,
    pub(crate) line_start: usize,
    pub(crate) line_end: usize,
    pub(crate) confidence: f64,
}

pub(crate) struct MarkdownCompiler<'a> {
    pub(crate) path: &'a str,
    pub(crate) content: &'a str,
    pub(crate) nodes: Vec<AnalysisNode>,
    pub(crate) edges: Vec<AnalysisEdge>,
    pub(crate) diagnostics: Vec<String>,
    pub(crate) section_stack: Vec<(usize, String)>,
    pub(crate) task_chain: HashMap<String, String>,
    pub(crate) reference_nodes: HashMap<String, String>,
    pub(crate) node_contexts: HashMap<usize, String>,
    pub(crate) edge_seq: usize,
}

impl<'a> MarkdownCompiler<'a> {
    pub(crate) fn new(path: &'a str, content: &'a str) -> Self {
        Self {
            path,
            content,
            nodes: vec![AnalysisNode {
                id: "doc:0".to_string(),
                kind: AnalysisNodeKind::Document,
                label: path.to_string(),
                depth: 0,
                line_start: 1,
                line_end: content.lines().count().max(1),
                parent_id: None,
            }],
            edges: Vec::new(),
            diagnostics: Vec::new(),
            section_stack: Vec::new(),
            task_chain: HashMap::new(),
            reference_nodes: HashMap::new(),
            node_contexts: HashMap::new(),
            edge_seq: 1,
        }
    }
}

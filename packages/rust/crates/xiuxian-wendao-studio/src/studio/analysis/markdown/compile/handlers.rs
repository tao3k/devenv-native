use comrak::nodes::AstNode;

use crate::studio::analysis::markdown::compile::types::{EdgeDraft, MarkdownCompiler};
use crate::studio::analysis::markdown::compile::utils::{
    current_section_context, line_end, line_start, make_edge, node_key, normalize_reference,
    slugify,
};
use crate::studio::analysis::markdown::text::collect_plain_text;
use crate::studio::types::{AnalysisEdgeKind, AnalysisNode, AnalysisNodeKind};

impl MarkdownCompiler<'_> {
    pub(crate) fn handle_heading<'b>(&mut self, node: &'b AstNode<'b>, level: usize) {
        while self
            .section_stack
            .last()
            .is_some_and(|(active_level, _)| *active_level >= level)
        {
            self.section_stack.pop();
        }

        let line_no = line_start(node);
        let node_id = format!("sec:{line_no}");
        let label = collect_plain_text(node);
        let parent_id = current_section_context(&self.section_stack).to_string();

        self.nodes.push(AnalysisNode {
            id: node_id.clone(),
            kind: AnalysisNodeKind::Section,
            label,
            depth: level,
            line_start: line_no,
            line_end: line_no,
            parent_id: Some(parent_id.clone()),
        });
        self.node_contexts.insert(node_key(node), node_id.clone());
        self.push_edge(EdgeDraft {
            kind: AnalysisEdgeKind::Contains,
            source_id: parent_id,
            target_id: node_id.clone(),
            label: "contains".to_string(),
            path: self.path,
            line_start: line_no,
            line_end: line_no,
            confidence: 1.0,
        });
        self.section_stack.push((level, node_id));
    }

    pub(crate) fn handle_task_item<'b>(&mut self, node: &'b AstNode<'b>) {
        let line_no = line_start(node);
        let parent_id = self.parent_context(node);
        let node_id = format!("task:{line_no}");
        let label = collect_plain_text(node);
        let depth = self
            .section_stack
            .last()
            .map_or(1, |(level, _)| level.saturating_add(1));

        self.nodes.push(AnalysisNode {
            id: node_id.clone(),
            kind: AnalysisNodeKind::Task,
            label,
            depth,
            line_start: line_no,
            line_end: line_no,
            parent_id: Some(parent_id.clone()),
        });
        self.node_contexts.insert(node_key(node), node_id.clone());
        self.push_edge(EdgeDraft {
            kind: AnalysisEdgeKind::Contains,
            source_id: parent_id.clone(),
            target_id: node_id.clone(),
            label: "contains".to_string(),
            path: self.path,
            line_start: line_no,
            line_end: line_no,
            confidence: 1.0,
        });

        if let Some(prev_task) = self.task_chain.get(parent_id.as_str()) {
            self.push_edge(EdgeDraft {
                kind: AnalysisEdgeKind::NextStep,
                source_id: prev_task.clone(),
                target_id: node_id.clone(),
                label: "next".to_string(),
                path: self.path,
                line_start: line_no,
                line_end: line_no,
                confidence: 0.9,
            });
        }
        self.task_chain.insert(parent_id, node_id);
    }

    pub(crate) fn handle_code_block<'b>(&mut self, node: &'b AstNode<'b>, info: &str) {
        let line_start = line_start(node);
        let line_end = line_end(node);
        let node_id = format!("code:{line_start}");
        let parent_id = self.parent_context(node);
        let language = info.trim();
        let label = if language.is_empty() {
            "code block".to_string()
        } else {
            format!("code block ({language})")
        };
        let depth = self
            .section_stack
            .last()
            .map_or(1, |(level, _)| level.saturating_add(1));

        self.nodes.push(AnalysisNode {
            id: node_id.clone(),
            kind: AnalysisNodeKind::CodeBlock,
            label,
            depth,
            line_start,
            line_end,
            parent_id: Some(parent_id.clone()),
        });
        self.node_contexts.insert(node_key(node), node_id.clone());
        self.push_edge(EdgeDraft {
            kind: AnalysisEdgeKind::Contains,
            source_id: parent_id,
            target_id: node_id,
            label: "contains".to_string(),
            path: self.path,
            line_start,
            line_end,
            confidence: 1.0,
        });
    }

    pub(crate) fn handle_math_node<'b>(&mut self, node: &'b AstNode<'b>) {
        let line_start = line_start(node);
        let line_end = line_end(node);
        let node_id = format!("math:{line_start}");
        let parent_id = self.parent_context(node);
        let label = {
            let text = collect_plain_text(node);
            if text.is_empty() {
                "math block".to_string()
            } else {
                text
            }
        };
        let depth = self
            .section_stack
            .last()
            .map_or(1, |(level, _)| level.saturating_add(1));

        self.nodes.push(AnalysisNode {
            id: node_id.clone(),
            kind: AnalysisNodeKind::Math,
            label,
            depth,
            line_start,
            line_end,
            parent_id: Some(parent_id.clone()),
        });
        self.node_contexts.insert(node_key(node), node_id.clone());
        self.push_edge(EdgeDraft {
            kind: AnalysisEdgeKind::Contains,
            source_id: parent_id,
            target_id: node_id,
            label: "contains".to_string(),
            path: self.path,
            line_start,
            line_end,
            confidence: 1.0,
        });
    }

    pub(crate) fn handle_observation<'b>(&mut self, node: &'b AstNode<'b>) {
        let line_start = line_start(node);
        let line_end = line_end(node);
        let node_id = format!("obs:{line_start}");
        let parent_id = self.parent_context(node);
        let label = {
            let text = collect_plain_text(node);
            if text.is_empty() {
                "observation".to_string()
            } else {
                text
            }
        };
        let depth = self
            .section_stack
            .last()
            .map_or(1, |(level, _)| level.saturating_add(1));

        self.nodes.push(AnalysisNode {
            id: node_id.clone(),
            kind: AnalysisNodeKind::Observation,
            label,
            depth,
            line_start,
            line_end,
            parent_id: Some(parent_id.clone()),
        });
        self.node_contexts.insert(node_key(node), node_id.clone());
        self.push_edge(EdgeDraft {
            kind: AnalysisEdgeKind::Contains,
            source_id: parent_id,
            target_id: node_id,
            label: "contains".to_string(),
            path: self.path,
            line_start,
            line_end,
            confidence: 1.0,
        });
    }

    pub(crate) fn handle_reference_node<'b>(&mut self, node: &'b AstNode<'b>, target: &str) {
        let normalized = normalize_reference(target);
        if normalized.is_empty() {
            return;
        }

        let line_no = line_start(node);
        let reference_id = self.reference_node_id(normalized.as_str(), line_no);
        let context_id = self.parent_context(node);
        self.push_edge(EdgeDraft {
            kind: AnalysisEdgeKind::References,
            source_id: context_id,
            target_id: reference_id,
            label: normalized,
            path: self.path,
            line_start: line_no,
            line_end: line_no,
            confidence: 0.85,
        });
    }

    pub(crate) fn handle_table<'b>(&mut self, node: &'b AstNode<'b>) {
        let line_start = line_start(node);
        let line_end = line_end(node);
        let node_id = format!("table:{line_start}");
        let parent_id = self.parent_context(node);
        let depth = self
            .section_stack
            .last()
            .map_or(1, |(level, _)| level.saturating_add(1));

        self.nodes.push(AnalysisNode {
            id: node_id.clone(),
            kind: AnalysisNodeKind::Table,
            label: "table".to_string(),
            depth,
            line_start,
            line_end,
            parent_id: Some(parent_id.clone()),
        });
        self.node_contexts.insert(node_key(node), node_id.clone());
        self.push_edge(EdgeDraft {
            kind: AnalysisEdgeKind::Contains,
            source_id: parent_id,
            target_id: node_id,
            label: "contains".to_string(),
            path: self.path,
            line_start,
            line_end,
            confidence: 1.0,
        });
    }

    pub(crate) fn reference_node_id(&mut self, normalized: &str, line_no: usize) -> String {
        if let Some(existing) = self.reference_nodes.get(normalized) {
            return existing.clone();
        }

        let node_id = format!("ref:{}", slugify(normalized));
        self.nodes.push(AnalysisNode {
            id: node_id.clone(),
            kind: AnalysisNodeKind::Reference,
            label: normalized.to_string(),
            depth: 1,
            line_start: line_no,
            line_end: line_no,
            parent_id: None,
        });
        self.reference_nodes
            .insert(normalized.to_string(), node_id.clone());
        node_id
    }

    pub(crate) fn parent_context<'b>(&self, node: &'b AstNode<'b>) -> String {
        for ancestor in node.ancestors() {
            if let Some(id) = self.node_contexts.get(&node_key(ancestor)) {
                return id.clone();
            }
        }
        current_section_context(&self.section_stack).to_string()
    }

    pub(crate) fn push_edge(&mut self, draft: EdgeDraft<'_>) {
        self.edges.push(make_edge(self.edge_seq, draft));
        self.edge_seq += 1;
    }
}

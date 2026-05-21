use crate::studio::analysis::markdown::compile::syntax::{
    build_stable_fingerprint, estimate_token_count, markdown_code_semantic_type,
    slice_content_lines, slugify,
};
use crate::studio::analysis::markdown::compile::types::MarkdownCompiler;
use crate::studio::types::{AnalysisNodeKind, MarkdownRetrievalAtom, RetrievalChunkSurface};

impl MarkdownCompiler<'_> {
    pub(crate) fn finalize_section_ranges(&mut self) {
        let document_end = self.content.lines().count().max(1);
        let section_indexes = self
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(index, node)| {
                matches!(node.kind, AnalysisNodeKind::Section).then_some(index)
            })
            .collect::<Vec<_>>();

        for (position, &index) in section_indexes.iter().enumerate() {
            let next_line_start = section_indexes
                .get(position + 1)
                .and_then(|next_index| self.nodes.get(*next_index))
                .map_or(document_end + 1, |node| node.line_start);
            let current_line_start = self.nodes[index].line_start;
            self.nodes[index].line_end = next_line_start.saturating_sub(1).max(current_line_start);
        }
    }

    pub(crate) fn build_retrieval_atoms(&self) -> Vec<MarkdownRetrievalAtom> {
        self.nodes
            .iter()
            .filter_map(|node| {
                let semantic_type = match node.kind {
                    AnalysisNodeKind::Document => "document".to_string(),
                    AnalysisNodeKind::Section => format!("h{}", node.depth.clamp(1, 6)),
                    AnalysisNodeKind::CodeBlock => markdown_code_semantic_type(node.label.as_str()),
                    AnalysisNodeKind::Table => "table".to_string(),
                    AnalysisNodeKind::Math => "math:block".to_string(),
                    AnalysisNodeKind::Observation => "observation".to_string(),
                    _ => return None,
                };
                let surface = match node.kind {
                    AnalysisNodeKind::Document => RetrievalChunkSurface::Document,
                    AnalysisNodeKind::Section => RetrievalChunkSurface::Section,
                    AnalysisNodeKind::CodeBlock => RetrievalChunkSurface::CodeBlock,
                    AnalysisNodeKind::Table => RetrievalChunkSurface::Table,
                    AnalysisNodeKind::Math => RetrievalChunkSurface::Math,
                    AnalysisNodeKind::Observation => RetrievalChunkSurface::Observation,
                    _ => return None,
                };
                let excerpt = slice_content_lines(self.content, node.line_start, node.line_end);
                Some(MarkdownRetrievalAtom {
                    owner_id: node.id.clone().into(),
                    chunk_id: format!("md:{}:{}", slugify(self.path), node.id.replace(':', "-"))
                        .into(),
                    semantic_type: semantic_type.into(),
                    fingerprint: build_stable_fingerprint(
                        format!(
                            "{}|{}|{}|{}|{}",
                            self.path, node.id, node.line_start, node.line_end, excerpt
                        )
                        .as_str(),
                    ),
                    token_estimate: estimate_token_count(excerpt.as_str()),
                    display_label: (!node.label.trim().is_empty()).then(|| node.label.clone()),
                    excerpt: Some(excerpt),
                    line_start: Some(node.line_start),
                    line_end: Some(node.line_end),
                    surface: Some(surface),
                    attributes: Vec::new(),
                })
            })
            .collect()
    }
}

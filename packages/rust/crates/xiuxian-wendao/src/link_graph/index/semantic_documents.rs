use std::sync::Arc;

use super::LinkGraphIndex;
use crate::link_graph::models::{
    LinkGraphDocument, LinkGraphSemanticDocument, LinkGraphSemanticDocumentKind, PageIndexNode,
};

impl LinkGraphIndex {
    /// Export all semantic documents derived from the current `PageIndex` state.
    #[must_use]
    pub fn semantic_documents(&self) -> Vec<LinkGraphSemanticDocument> {
        let mut doc_ids = self.docs_by_id.keys().cloned().collect::<Vec<_>>();
        doc_ids.sort();

        doc_ids
            .into_iter()
            .flat_map(|doc_id| self.semantic_documents_for_doc_id(doc_id.as_str()))
            .collect()
    }

    /// Export semantic documents for one note resolved by stem or canonical id.
    #[must_use]
    pub fn semantic_documents_for(
        &self,
        stem_or_id: &str,
    ) -> Option<Vec<LinkGraphSemanticDocument>> {
        let doc_id = self.resolve_doc_id(stem_or_id)?;
        Some(self.semantic_documents_for_doc_id(doc_id))
    }

    fn semantic_documents_for_doc_id(&self, doc_id: &str) -> Vec<LinkGraphSemanticDocument> {
        let Some(doc) = self.docs_by_id.get(doc_id) else {
            return Vec::new();
        };
        let roots = self.trees_by_doc.get(doc_id).map_or(&[][..], Vec::as_slice);
        let mut documents = Vec::new();

        if let Some(summary) = build_summary_document(doc, roots) {
            documents.push(summary);
        }
        collect_section_documents(
            doc.id.as_str(),
            doc.path.as_str(),
            roots,
            &mut documents,
            &mut Vec::new(),
        );

        documents
    }
}

fn build_summary_document(
    doc: &LinkGraphDocument,
    roots: &[PageIndexNode],
) -> Option<LinkGraphSemanticDocument> {
    let semantic_path = roots
        .first()
        .map_or_else(|| vec![doc.title.clone()], |root| vec![root.title.clone()]);
    let line_range = roots.first().map(|root| root.metadata.line_range);
    let summary = roots
        .first()
        .and_then(|root| normalize_text(root.summary.as_deref()))
        .or_else(|| normalize_text(Some(doc.lead.as_str())))
        .or_else(|| {
            roots
                .first()
                .and_then(|root| normalize_text(Some(root.text.as_ref())))
        })
        .or_else(|| normalize_text(Some(doc.title.as_str())))?;

    Some(LinkGraphSemanticDocument {
        anchor_id: doc.id.clone().into(),
        doc_id: doc.id.clone().into(),
        path: doc.path.clone().into(),
        kind: LinkGraphSemanticDocumentKind::Summary,
        semantic_path,
        content: summary,
        line_range,
    })
}

fn collect_section_documents(
    doc_id: &str,
    path: &str,
    nodes: &[PageIndexNode],
    documents: &mut Vec<LinkGraphSemanticDocument>,
    semantic_path: &mut Vec<String>,
) {
    for node in nodes {
        semantic_path.push(node.title.clone());
        if let Some(content) = node
            .summary
            .as_deref()
            .and_then(|summary| normalize_text(Some(summary)))
            .or_else(|| normalize_text(Some(node.text.as_ref())))
        {
            documents.push(LinkGraphSemanticDocument {
                anchor_id: node.node_id.clone().into(),
                doc_id: doc_id.to_string().into(),
                path: path.to_string().into(),
                kind: LinkGraphSemanticDocumentKind::Section,
                semantic_path: semantic_path.clone(),
                content,
                line_range: Some(node.metadata.line_range),
            });
        }
        collect_section_documents(doc_id, path, &node.children, documents, semantic_path);
        semantic_path.pop();
    }
}

fn normalize_text(raw: Option<&str>) -> Option<Arc<str>> {
    let trimmed = raw?.trim();
    (!trimmed.is_empty()).then(|| Arc::<str>::from(trimmed.to_string()))
}

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub use xiuxian_wendao_core::repo_intelligence::ProjectionPageKind;

use crate::analyzers::DocRecord;
use crate::analyzers::RepositoryAnalysisOutput;

/// Seed used to generate one projected page record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct ProjectionPageSeed {
    /// Repository identifier.
    pub repo_id: String,
    /// Page identifier.
    pub page_id: String,
    /// Page kind.
    pub kind: ProjectionPageKind,
    /// Page title.
    pub title: String,
    /// Related module identifiers.
    pub module_ids: Vec<String>,
    /// Related symbol identifiers.
    pub symbol_ids: Vec<String>,
    /// Related example identifiers.
    pub example_ids: Vec<String>,
    /// Related documentation identifiers.
    pub doc_ids: Vec<String>,
    /// Related file paths.
    pub paths: Vec<String>,
    /// Format hints for rendering.
    pub format_hints: Vec<String>,
}

/// Deterministic projection-input bundle generated from stage-1 analysis output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct ProjectionInputBundle {
    /// Repository identifier.
    pub repo_id: String,
    /// Projection page seeds.
    pub pages: Vec<ProjectionPageSeed>,
}

/// One projected section rendered into markdown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct ProjectedPageSection {
    /// Section identifier.
    pub section_id: String,
    /// Section title.
    pub title: String,
    /// Heading level.
    pub level: usize,
    /// Section body content.
    pub body: String,
    /// Related file paths.
    pub paths: Vec<String>,
}

/// One deterministic projected page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct ProjectedPageRecord {
    /// Repository identifier.
    pub repo_id: String,
    /// Page identifier.
    pub page_id: String,
    /// Page kind.
    pub kind: ProjectionPageKind,
    /// Page title.
    pub title: String,
    /// Related module identifiers.
    pub module_ids: Vec<String>,
    /// Related symbol identifiers.
    pub symbol_ids: Vec<String>,
    /// Related example identifiers.
    pub example_ids: Vec<String>,
    /// Related documentation identifiers.
    pub doc_ids: Vec<String>,
    /// Related file paths.
    pub paths: Vec<String>,
    /// Format hints for rendering.
    pub format_hints: Vec<String>,
    /// Page sections.
    pub sections: Vec<ProjectedPageSection>,
    /// Documentation identifier.
    #[serde(default)]
    pub doc_id: String,
    /// Page path.
    #[serde(default)]
    pub path: String,
    /// Search keywords.
    #[serde(default)]
    pub keywords: Vec<String>,
}

impl From<&DocRecord> for ProjectedPageRecord {
    fn from(doc: &DocRecord) -> Self {
        let kind = projection_kind_from_doc_format(doc.format.as_deref());
        let path = doc.path.clone();
        let doc_id = doc.doc_id.clone();
        let format_hints = doc.format.clone().into_iter().collect::<Vec<_>>();
        let mut keywords = vec![doc.title.clone(), path.clone(), doc_id.clone()];
        keywords.extend(format_hints.iter().cloned());
        keywords.sort();
        keywords.dedup();

        Self {
            repo_id: doc.repo_id.clone(),
            page_id: format!(
                "repo:{}:projection:{}:doc:{}",
                doc.repo_id,
                projection_kind_token(kind),
                doc.doc_id
            ),
            kind,
            title: doc.title.clone(),
            module_ids: Vec::new(),
            symbol_ids: Vec::new(),
            example_ids: Vec::new(),
            doc_ids: vec![doc.doc_id.clone()],
            paths: vec![path.clone()],
            format_hints,
            sections: Vec::new(),
            doc_id,
            path,
            keywords,
        }
    }
}

/// Rendered projected markdown document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct ProjectedMarkdownDocument {
    /// Repository identifier.
    pub repo_id: String,
    /// Page identifier.
    pub page_id: String,
    /// Page kind.
    pub kind: ProjectionPageKind,
    /// File path.
    pub path: String,
    /// Page title.
    pub title: String,
    /// Rendered markdown content.
    pub markdown: String,
}

/// One page-index section summary in projected markdown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct ProjectedPageIndexSection {
    /// Heading path identifier.
    pub heading_path: String,
    /// Section title.
    pub title: String,
    /// Heading level (1-6).
    pub level: usize,
    /// Start and end line numbers.
    pub line_range: (usize, usize),
    /// Key-value attributes for the section.
    pub attributes: Vec<(String, String)>,
}

/// Parsed projected markdown document prepared for page-index tree generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct ProjectedPageIndexDocument {
    /// Repository identifier.
    pub repo_id: String,
    /// Page identifier.
    pub page_id: String,
    /// File path.
    pub path: String,
    /// Documentation identifier.
    pub doc_id: String,
    /// Page title.
    pub title: String,
    /// Parsed sections from the document.
    pub sections: Vec<ProjectedPageIndexSection>,
}

/// Snapshot of one projected page-index node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct ProjectedPageIndexNode {
    /// Node identifier.
    pub node_id: String,
    /// Node title.
    pub title: String,
    /// Heading level.
    pub level: usize,
    /// Structural path from root to this node.
    pub structural_path: Vec<String>,
    /// Start and end line numbers.
    pub line_range: (usize, usize),
    /// Approximate token count for this node.
    pub token_count: usize,
    /// Whether this node has been thinned during summarization.
    pub is_thinned: bool,
    /// Full text content of the node.
    pub text: String,
    /// Optional summary of the node content.
    pub summary: Option<String>,
    /// Child nodes.
    pub children: Vec<ProjectedPageIndexNode>,
}

/// Snapshot of one projected page-index tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct ProjectedPageIndexTree {
    /// Repository identifier.
    pub repo_id: String,
    /// Page identifier.
    pub page_id: String,
    /// Page kind.
    pub kind: ProjectionPageKind,
    /// File path.
    pub path: String,
    /// Documentation identifier.
    pub doc_id: String,
    /// Page title.
    pub title: String,
    /// Number of root nodes.
    pub root_count: usize,
    /// Root nodes of the tree.
    pub roots: Vec<ProjectedPageIndexNode>,
}

impl ProjectedPageIndexTree {
    /// Creates a page index tree from a documentation record.
    #[must_use]
    pub fn from_doc(doc: &DocRecord, _analysis: &RepositoryAnalysisOutput) -> Self {
        let page = ProjectedPageRecord::from(doc);
        let root = ProjectedPageIndexNode {
            node_id: format!("{}#root", page.page_id),
            title: page.title.clone(),
            level: 1,
            structural_path: vec![page.title.clone()],
            line_range: (1, 1),
            token_count: page.title.split_whitespace().count().max(1),
            is_thinned: false,
            text: page.title.clone(),
            summary: Some(page.title.clone()),
            children: Vec::new(),
        };

        Self {
            repo_id: page.repo_id,
            page_id: page.page_id,
            kind: page.kind,
            path: page.path,
            doc_id: page.doc_id,
            title: page.title,
            root_count: 1,
            roots: vec![root],
        }
    }
}

pub(crate) fn projection_kind_from_doc_format(format: Option<&str>) -> ProjectionPageKind {
    let Some(format) = format else {
        return ProjectionPageKind::Explanation;
    };
    let normalized = format.trim().to_ascii_lowercase();
    if normalized.contains("tutorial") {
        ProjectionPageKind::Tutorial
    } else if normalized.contains("howto")
        || normalized.contains("how-to")
        || normalized.contains("guide")
    {
        ProjectionPageKind::HowTo
    } else if normalized.contains("reference") || normalized.contains("api") {
        ProjectionPageKind::Reference
    } else {
        ProjectionPageKind::Explanation
    }
}

fn projection_kind_token(kind: ProjectionPageKind) -> &'static str {
    match kind {
        ProjectionPageKind::Reference => "reference",
        ProjectionPageKind::HowTo => "howto",
        ProjectionPageKind::Tutorial => "tutorial",
        ProjectionPageKind::Explanation => "explanation",
    }
}

//! `analyzers::projection::retrieval_context` owns Wendao analyzers projection retrieval context behavior.

use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::projection::contracts::{
    ProjectedPageIndexNode, ProjectedPageIndexTree, ProjectedPageRecord,
};
use crate::analyzers::projection::lookup::build_projected_page;
use crate::analyzers::projection::related_pages::find_related_pages;
use crate::analyzers::projection::tree_lookup::build_projected_page_index_tree;
use crate::analyzers::{
    ProjectedPageIndexNodeContext, ProjectedPageIndexNodeHit, ProjectedRetrievalHit,
    ProjectedRetrievalHitKind, RepoProjectedPageIndexTreeQuery, RepoProjectedPageQuery,
    RepoProjectedRetrievalContextQuery, RepoProjectedRetrievalContextResult,
};

/// Build retrieval context around a projected page and optional page-index node.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the projected page or requested node
/// cannot be resolved.
pub fn build_projected_retrieval_context(
    query: &RepoProjectedRetrievalContextQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<RepoProjectedRetrievalContextResult, RepoIntelligenceError> {
    let page_record = build_projected_page(
        &RepoProjectedPageQuery {
            repo_id: query.repo_id.clone(),
            page_id: query.page_id.clone(),
        },
        analysis,
    )?
    .page;
    let center = ProjectedRetrievalHit {
        kind: ProjectedRetrievalHitKind::Page,
        page: page_record.clone(),
        node: None,
    };

    let related_pages = find_related_pages(&page_record, analysis, query.related_limit);

    let mut node_context = None;
    if let Some(node_id) = &query.node_id {
        let tree_result = build_projected_page_index_tree(
            &RepoProjectedPageIndexTreeQuery {
                repo_id: query.repo_id.clone(),
                page_id: query.page_id.clone(),
            },
            analysis,
        )?;

        if let Some(tree) = tree_result.tree {
            node_context = Some(build_node_context(&page_record, &tree, node_id)?);
        }
    }

    Ok(RepoProjectedRetrievalContextResult {
        repo_id: query.repo_id.clone(),
        center,
        related_pages,
        node_context,
    })
}

fn build_node_context(
    page: &ProjectedPageRecord,
    tree: &ProjectedPageIndexTree,
    node_id: &str,
) -> Result<ProjectedPageIndexNodeContext, RepoIntelligenceError> {
    let raw = find_node_context(tree.roots.as_slice(), node_id, &[]).ok_or_else(|| {
        RepoIntelligenceError::UnknownProjectedPageIndexNode {
            repo_id: page.repo_id.clone().into(),
            page_id: page.page_id.clone().into(),
            node_id: node_id.to_string().into(),
        }
    })?;

    Ok(ProjectedPageIndexNodeContext {
        ancestors: raw
            .ancestors
            .into_iter()
            .map(|n| make_hit(page, n))
            .collect(),
        previous_sibling: raw.previous_sibling.map(|n| make_hit(page, n)),
        next_sibling: raw.next_sibling.map(|n| make_hit(page, n)),
        children: raw
            .children
            .into_iter()
            .map(|n| make_hit(page, n))
            .collect(),
    })
}

fn make_hit(
    page: &ProjectedPageRecord,
    node: &ProjectedPageIndexNode,
) -> ProjectedPageIndexNodeHit {
    ProjectedPageIndexNodeHit {
        repo_id: page.repo_id.clone(),
        page_id: page.page_id.clone(),
        page_title: page.title.clone(),
        page_kind: page.kind,
        path: page.path.clone(),
        doc_id: page.doc_id.clone(),
        node_id: node.node_id.clone(),
        node_title: node.title.clone(),
        structural_path: node.structural_path.clone(),
        line_range: node.line_range,
        text: node.text.clone(),
    }
}

struct RawNodeContext<'a> {
    ancestors: Vec<&'a ProjectedPageIndexNode>,
    previous_sibling: Option<&'a ProjectedPageIndexNode>,
    next_sibling: Option<&'a ProjectedPageIndexNode>,
    children: Vec<&'a ProjectedPageIndexNode>,
}

fn find_node_context<'a>(
    nodes: &'a [ProjectedPageIndexNode],
    node_id: &str,
    ancestors: &[&'a ProjectedPageIndexNode],
) -> Option<RawNodeContext<'a>> {
    for (idx, node) in nodes.iter().enumerate() {
        if node.node_id == node_id {
            return Some(RawNodeContext {
                ancestors: ancestors.to_vec(),
                previous_sibling: idx.checked_sub(1).and_then(|left| nodes.get(left)),
                next_sibling: nodes.get(idx + 1),
                children: node.children.iter().collect(),
            });
        }
        let mut child_ancestors = ancestors.to_vec();
        child_ancestors.push(node);
        if let Some(context) =
            find_node_context(node.children.as_slice(), node_id, &child_ancestors)
        {
            return Some(context);
        }
    }
    None
}

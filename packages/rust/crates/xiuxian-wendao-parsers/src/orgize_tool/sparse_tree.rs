//! Org sparse-tree projection adapter.

use std::path::PathBuf;

use orgize::Org;
use orgize::ast::{SparseTreeCard, SparseTreeProjection};
use orgize::ast::{SparseTreeQuery, TodoState};

use super::OrgizeToolError;
use super::io::{collect_org_paths, join_projection_text, read_to_string};

/// Options for sparse-tree projections derived from Org search syntax.
#[derive(Clone, Debug)]
pub struct OrgizeSparseTreeRequest {
    /// Files or directories to inspect.
    pub paths: Vec<PathBuf>,
    /// Optional text search term.
    pub text: Option<String>,
    /// Optional Org agenda match expression.
    pub match_expression: Option<String>,
    /// Sparse-tree visibility controls.
    pub visibility: OrgizeSparseTreeVisibility,
    /// Include COMMENT tasks.
    pub include_comments: bool,
    /// Sparse-tree render controls.
    pub render: OrgizeSparseTreeRenderOptions,
}

/// Visibility controls for sparse-tree projections.
#[derive(Clone, Debug, Default)]
pub struct OrgizeSparseTreeVisibility {
    /// Exclude DONE-state tasks.
    pub exclude_done: bool,
    /// Exclude archived tasks.
    pub exclude_archived: bool,
}

/// Render controls for sparse-tree projections.
#[derive(Clone, Debug, Default)]
pub struct OrgizeSparseTreeRenderOptions {
    /// Render skipped section receipts.
    pub explain_skips: bool,
}

/// Renders sparse-tree cards from Org search semantics.
///
/// # Errors
///
/// Returns an error when a path cannot be read or a match expression cannot be
/// parsed.
pub fn render_sparse_tree(request: &OrgizeSparseTreeRequest) -> Result<String, OrgizeToolError> {
    let files = collect_org_paths(&request.paths)?;
    let mut query = SparseTreeQuery::new()
        .include_done(!request.visibility.exclude_done)
        .include_archived(!request.visibility.exclude_archived)
        .include_comments(request.include_comments)
        .explain_skips(request.render.explain_skips);
    if let Some(text) = request.text.as_deref() {
        query = query.text(text);
    }
    if let Some(expression) = request.match_expression.as_deref() {
        query = query.match_expression(expression).map_err(|error| {
            OrgizeToolError::InvalidMatchExpression {
                expression: expression.to_string(),
                message: error.to_string(),
            }
        })?;
    }

    let mut rendered = Vec::new();
    for path in files {
        let source = read_to_string(&path)?;
        let document = Org::parse(&source).document();
        let source_file = path.display().to_string();
        let projection = document.sparse_tree_projection(&query.clone().source_file(&source_file));
        let excluded_ancestors = collect_excluded_ancestor_cards(
            &document.sparse_tree_projection(
                &SparseTreeQuery::new()
                    .include_done(true)
                    .include_archived(true)
                    .include_comments(request.include_comments)
                    .source_file(&source_file),
            ),
            &request.visibility,
        );
        let projection = filter_excluded_subtrees(projection, &excluded_ancestors);
        rendered.push(projection.to_compact_text(&path.display().to_string()));
    }
    Ok(join_projection_text(rendered, "[ok] orgize sparse tree\n"))
}

fn collect_excluded_ancestor_cards(
    projection: &SparseTreeProjection,
    visibility: &OrgizeSparseTreeVisibility,
) -> Vec<SparseTreeCard> {
    projection
        .cards
        .iter()
        .filter(|card| {
            (visibility.exclude_done && card_is_done(card))
                || (visibility.exclude_archived && card.archive.archived)
        })
        .cloned()
        .collect()
}

fn filter_excluded_subtrees(
    projection: SparseTreeProjection,
    excluded_ancestors: &[SparseTreeCard],
) -> SparseTreeProjection {
    if excluded_ancestors.is_empty() {
        return projection;
    }

    let total_candidates = projection.total_candidates;
    let skipped = projection.skipped;
    let cards = projection
        .cards
        .into_iter()
        .filter(|card| {
            !excluded_ancestors
                .iter()
                .any(|ancestor| is_inside_excluded_ancestor(card, ancestor))
        })
        .collect::<Vec<_>>();

    SparseTreeProjection {
        total_candidates,
        cards,
        skipped,
    }
}

fn card_is_done(card: &SparseTreeCard) -> bool {
    card.todo
        .as_ref()
        .is_some_and(|todo| todo.state == TodoState::Done)
}

fn is_inside_excluded_ancestor(card: &SparseTreeCard, ancestor: &SparseTreeCard) -> bool {
    ancestor.source.range_start < card.source.range_start
        && ancestor.source.range_end >= card.source.range_end
}

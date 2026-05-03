use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::studio::router::handlers::docs::service::projection::gap_report::run_docs_projected_gap_report;
use crate::studio::router::handlers::docs::types::projected_gap::DocsProjectedGapReportApiQuery;
use crate::studio::router::handlers::repo::parse::repo::required_registered_repo_id;
use crate::studio::router::{GatewayState, StudioApiError};
use xiuxian_wendao::analyzers::DocsProjectedGapReportQuery;

/// Docs projected gap report endpoint.
///
/// # Errors
///
/// Returns an error when `repo` is missing, repository lookup or analysis
/// fails, or the background task panics.
pub async fn projected_gap_report(
    Query(query): Query<DocsProjectedGapReportApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<xiuxian_wendao::analyzers::DocsProjectedGapReportResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let result =
        run_docs_projected_gap_report(Arc::clone(&state), DocsProjectedGapReportQuery { repo_id })
            .await?;
    Ok(Json(result))
}

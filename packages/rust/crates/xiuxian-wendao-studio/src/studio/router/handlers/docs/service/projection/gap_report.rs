use std::sync::Arc;

use crate::studio::router::handlers::docs::service::runtime::run_docs_analysis;
use crate::studio::router::{GatewayState, StudioApiError};
use xiuxian_wendao::analyzers::{
    DocsProjectedGapReportQuery, DocsProjectedGapReportResult, RepoIntelligenceError,
    build_docs_projected_gap_report,
};

pub(crate) async fn run_docs_projected_gap_report(
    state: Arc<GatewayState>,
    query: DocsProjectedGapReportQuery,
) -> Result<DocsProjectedGapReportResult, StudioApiError> {
    run_docs_analysis(
        Arc::clone(&state),
        query.repo_id.clone(),
        "DOCS_PROJECTED_GAP_REPORT_PANIC",
        "Docs projected gap report task failed unexpectedly",
        move |analysis| {
            Ok::<_, RepoIntelligenceError>(build_docs_projected_gap_report(&query, &analysis))
        },
    )
    .await
}

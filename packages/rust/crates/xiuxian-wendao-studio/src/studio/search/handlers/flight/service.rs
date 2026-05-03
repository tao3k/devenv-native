use std::sync::Arc;

use async_trait::async_trait;
use xiuxian_wendao_web::transport::{
    RepoSearchFlightRouteProvider, RerankScoreWeights, SqlFlightRouteProvider,
    SqlFlightRouteResponse, WendaoFlightRouteProviders, WendaoFlightService,
};

use super::provider::StudioSearchFlightRouteProvider;
use crate::studio::GatewayState;
use crate::studio::router::handlers::analysis::{
    StudioCodeAstAnalysisFlightRouteProvider, StudioDocumentExtractFlightRouteProvider,
    StudioMarkdownAnalysisFlightRouteProvider,
};
use crate::studio::router::handlers::graph::flight::StudioGraphNeighborsFlightRouteProvider;
use crate::studio::router::handlers::graph::topology_flight::StudioTopology3dFlightRouteProvider;
use crate::studio::router::handlers::repo::analysis::flight::StudioRepoDocCoverageFlightRouteProvider;
use crate::studio::router::handlers::repo::analysis::index_flight::StudioRepoIndexFlightRouteProvider;
use crate::studio::router::handlers::repo::analysis::index_status_flight::StudioRepoIndexStatusFlightRouteProvider;
use crate::studio::router::handlers::repo::analysis::overview_flight::StudioRepoOverviewFlightRouteProvider;
use crate::studio::router::handlers::repo::analysis::projected_page_index_tree_flight::StudioRepoProjectedPageIndexTreeFlightRouteProvider;
use crate::studio::router::handlers::repo::analysis::refine_doc_flight::StudioRefineDocFlightRouteProvider;
use crate::studio::router::handlers::repo::analysis::sync_flight::StudioRepoSyncFlightRouteProvider;
use crate::studio::search::handlers::ast::StudioAstSearchFlightRouteProvider;
use crate::studio::search::handlers::attachments::StudioAttachmentSearchFlightRouteProvider;
use crate::studio::search::handlers::autocomplete::StudioAutocompleteFlightRouteProvider;
use crate::studio::search::handlers::definition::StudioDefinitionFlightRouteProvider;
use crate::studio::vfs::{
    StudioVfsContentFlightRouteProvider, StudioVfsResolveFlightRouteProvider,
    StudioVfsScanFlightRouteProvider,
};
use xiuxian_wendao::search::queries::sql::provider::StudioSqlFlightRouteProvider;

pub(crate) fn build_studio_search_flight_service_with_repo_provider(
    expected_schema_version: impl Into<String>,
    repo_search_provider: Arc<dyn RepoSearchFlightRouteProvider>,
    state: impl Into<Arc<GatewayState>>,
    rerank_dimension: usize,
    rerank_weights: RerankScoreWeights,
) -> Result<WendaoFlightService, String> {
    let state = state.into();
    let mut route_providers = WendaoFlightRouteProviders::new(repo_search_provider);
    route_providers.search = Some(Arc::new(StudioSearchFlightRouteProvider::new(Arc::clone(
        &state,
    ))));
    route_providers.attachment_search = Some(Arc::new(
        StudioAttachmentSearchFlightRouteProvider::new(Arc::clone(&state.studio)),
    ));
    route_providers.ast_search = Some(Arc::new(StudioAstSearchFlightRouteProvider::new(
        Arc::clone(&state),
    )));
    route_providers.definition = Some(Arc::new(StudioDefinitionFlightRouteProvider::new(
        Arc::clone(&state.studio),
    )));
    route_providers.autocomplete = Some(Arc::new(StudioAutocompleteFlightRouteProvider::new(
        Arc::clone(&state.studio),
    )));
    route_providers.markdown_analysis = Some(Arc::new(
        StudioMarkdownAnalysisFlightRouteProvider::new(Arc::clone(&state)),
    ));
    route_providers.code_ast_analysis = Some(Arc::new(
        StudioCodeAstAnalysisFlightRouteProvider::new(Arc::clone(&state)),
    ));
    route_providers.document_extract = Some(Arc::new(
        StudioDocumentExtractFlightRouteProvider::new(state.as_ref()),
    ));
    route_providers.repo_overview = Some(Arc::new(StudioRepoOverviewFlightRouteProvider::new(
        Arc::clone(&state),
    )));
    route_providers.repo_index = Some(Arc::new(StudioRepoIndexFlightRouteProvider::new(
        Arc::clone(&state),
    )));
    route_providers.repo_index_status = Some(Arc::new(
        StudioRepoIndexStatusFlightRouteProvider::new(Arc::clone(&state)),
    ));
    route_providers.repo_sync = Some(Arc::new(StudioRepoSyncFlightRouteProvider::new(
        Arc::clone(&state),
    )));
    route_providers.repo_doc_coverage = Some(Arc::new(
        StudioRepoDocCoverageFlightRouteProvider::new(Arc::clone(&state)),
    ));
    route_providers.repo_projected_page_index_tree = Some(Arc::new(
        StudioRepoProjectedPageIndexTreeFlightRouteProvider::new(Arc::clone(&state)),
    ));
    route_providers.refine_doc = Some(Arc::new(StudioRefineDocFlightRouteProvider::new(
        Arc::clone(&state),
    )));
    route_providers.vfs_content = Some(Arc::new(StudioVfsContentFlightRouteProvider::new(
        Arc::clone(&state.studio),
    )));
    route_providers.vfs_scan = Some(Arc::new(StudioVfsScanFlightRouteProvider::new(Arc::clone(
        &state.studio,
    ))));
    route_providers.vfs_resolve = Some(Arc::new(StudioVfsResolveFlightRouteProvider::new(
        Arc::clone(&state.studio),
    )));
    route_providers.graph_neighbors = Some(Arc::new(StudioGraphNeighborsFlightRouteProvider::new(
        Arc::clone(&state),
    )));
    route_providers.topology_3d = Some(Arc::new(StudioTopology3dFlightRouteProvider::new(
        Arc::clone(&state),
    )));
    route_providers.sql = Some(Arc::new(StudioWebSqlFlightRouteProvider::new(
        StudioSqlFlightRouteProvider::new(state.studio.search_plane_service()),
    )));
    WendaoFlightService::new_with_route_providers_and_sql(
        expected_schema_version,
        route_providers,
        rerank_dimension,
        rerank_weights,
    )
}

#[derive(Clone)]
struct StudioWebSqlFlightRouteProvider {
    inner: StudioSqlFlightRouteProvider,
}

impl StudioWebSqlFlightRouteProvider {
    fn new(inner: StudioSqlFlightRouteProvider) -> Self {
        Self { inner }
    }
}

impl std::fmt::Debug for StudioWebSqlFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioWebSqlFlightRouteProvider")
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl SqlFlightRouteProvider for StudioWebSqlFlightRouteProvider {
    async fn sql_query_batches(&self, query_text: &str) -> Result<SqlFlightRouteResponse, String> {
        let response =
            xiuxian_wendao_runtime::transport::SqlFlightRouteProvider::sql_query_batches(
                &self.inner,
                query_text,
            )
            .await?;
        Ok(SqlFlightRouteResponse::new(response.batches).with_app_metadata(response.app_metadata))
    }
}

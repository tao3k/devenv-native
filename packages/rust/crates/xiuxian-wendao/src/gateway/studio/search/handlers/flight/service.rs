use std::sync::Arc;

use xiuxian_wendao_runtime::transport::{
    RepoSearchFlightRouteProvider, RerankScoreWeights, WendaoFlightRouteProviders,
    WendaoFlightService,
};

use super::provider::StudioSearchFlightRouteProvider;
use crate::gateway::studio::GatewayState;
use crate::gateway::studio::router::handlers::analysis::{
    StudioCodeAstAnalysisFlightRouteProvider, StudioDocumentExtractFlightRouteProvider,
    StudioMarkdownAnalysisFlightRouteProvider,
};
use crate::gateway::studio::router::handlers::graph::flight::StudioGraphNeighborsFlightRouteProvider;
use crate::gateway::studio::router::handlers::graph::topology_flight::StudioTopology3dFlightRouteProvider;
use crate::gateway::studio::router::handlers::repo::analysis::flight::StudioRepoDocCoverageFlightRouteProvider;
use crate::gateway::studio::router::handlers::repo::analysis::index_flight::StudioRepoIndexFlightRouteProvider;
use crate::gateway::studio::router::handlers::repo::analysis::index_status_flight::StudioRepoIndexStatusFlightRouteProvider;
use crate::gateway::studio::router::handlers::repo::analysis::overview_flight::StudioRepoOverviewFlightRouteProvider;
use crate::gateway::studio::router::handlers::repo::analysis::projected_page_index_tree_flight::StudioRepoProjectedPageIndexTreeFlightRouteProvider;
use crate::gateway::studio::router::handlers::repo::analysis::refine_doc_flight::StudioRefineDocFlightRouteProvider;
use crate::gateway::studio::router::handlers::repo::analysis::sync_flight::StudioRepoSyncFlightRouteProvider;
use crate::gateway::studio::search::handlers::ast::StudioAstSearchFlightRouteProvider;
use crate::gateway::studio::search::handlers::attachments::StudioAttachmentSearchFlightRouteProvider;
use crate::gateway::studio::search::handlers::autocomplete::StudioAutocompleteFlightRouteProvider;
use crate::gateway::studio::search::handlers::definition::StudioDefinitionFlightRouteProvider;
use crate::gateway::studio::vfs::{
    StudioVfsContentFlightRouteProvider, StudioVfsResolveFlightRouteProvider,
    StudioVfsScanFlightRouteProvider,
};
use crate::search::queries::sql::provider::StudioSqlFlightRouteProvider;

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
    route_providers.sql = Some(Arc::new(StudioSqlFlightRouteProvider::new(
        state.studio.search_plane_service(),
    )));
    WendaoFlightService::new_with_route_providers_and_sql(
        expected_schema_version,
        route_providers,
        rerank_dimension,
        rerank_weights,
    )
}

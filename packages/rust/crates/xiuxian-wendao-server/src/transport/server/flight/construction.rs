//! Constructors for the transport-owned Wendao Flight service.

use std::sync::Arc;

use arrow_array::RecordBatch as LanceRecordBatch;

use crate::transport::RerankScoreWeights;

use crate::transport::server::{
    RepoSearchFlightRouteProvider, StaticRepoSearchFlightRouteProvider, WendaoFlightRouteProviders,
};

use super::ServiceCore as WendaoFlightService;

impl WendaoFlightService {
    /// Create one transport-owned Wendao Flight service.
    ///
    /// # Errors
    ///
    /// Returns an error when the schema version is blank or the rerank route
    /// handler configuration is invalid.
    pub fn new(
        expected_schema_version: impl Into<String>,
        query_response_batch: LanceRecordBatch,
        rerank_dimension: usize,
    ) -> Result<Self, String> {
        Self::new_with_weights(
            expected_schema_version,
            query_response_batch,
            rerank_dimension,
            RerankScoreWeights::default(),
        )
    }

    /// Create one transport-owned Wendao Flight service with explicit rerank
    /// score weights.
    ///
    /// # Errors
    ///
    /// Returns an error when the schema version is blank or the rerank route
    /// handler configuration is invalid.
    pub fn new_with_weights(
        expected_schema_version: impl Into<String>,
        query_response_batch: LanceRecordBatch,
        rerank_dimension: usize,
        rerank_weights: RerankScoreWeights,
    ) -> Result<Self, String> {
        Self::new_with_provider(
            expected_schema_version,
            Arc::new(StaticRepoSearchFlightRouteProvider {
                batch: query_response_batch,
            }),
            rerank_dimension,
            rerank_weights,
        )
    }

    /// Create one transport-owned Wendao Flight service from a pluggable
    /// repo-search provider.
    ///
    /// # Errors
    ///
    /// Returns an error when the schema version is blank or the rerank route
    /// handler configuration is invalid.
    pub fn new_with_provider(
        expected_schema_version: impl Into<String>,
        repo_search_provider: Arc<dyn RepoSearchFlightRouteProvider>,
        rerank_dimension: usize,
        rerank_weights: RerankScoreWeights,
    ) -> Result<Self, String> {
        Self::new_with_route_providers(
            expected_schema_version,
            WendaoFlightRouteProviders::new(repo_search_provider),
            rerank_dimension,
            rerank_weights,
        )
    }

    /// Create one transport-owned Wendao Flight service from pluggable
    /// repo-search and generic search-family providers.
    ///
    /// # Errors
    ///
    /// Returns an error when the schema version is blank or the rerank route
    /// handler configuration is invalid.
    pub fn new_with_route_providers(
        expected_schema_version: impl Into<String>,
        route_providers: WendaoFlightRouteProviders,
        rerank_dimension: usize,
        rerank_weights: RerankScoreWeights,
    ) -> Result<Self, String> {
        Self::new_with_route_providers_and_sql(
            expected_schema_version,
            route_providers,
            rerank_dimension,
            rerank_weights,
        )
    }

    /// Create one transport-owned Wendao Flight service from pluggable
    /// route providers plus an optional SQL provider.
    ///
    /// # Errors
    ///
    /// Returns an error when the schema version is blank or the rerank route
    /// handler configuration is invalid.
    pub fn new_with_route_providers_and_sql(
        expected_schema_version: impl Into<String>,
        route_providers: WendaoFlightRouteProviders,
        rerank_dimension: usize,
        rerank_weights: RerankScoreWeights,
    ) -> Result<Self, String> {
        let expected_schema_version = expected_schema_version.into();
        if expected_schema_version.trim().is_empty() {
            return Err("wendao flight service schema version must not be blank".to_string());
        }
        Self::build(
            expected_schema_version,
            route_providers,
            rerank_weights,
            rerank_dimension,
        )
    }
}

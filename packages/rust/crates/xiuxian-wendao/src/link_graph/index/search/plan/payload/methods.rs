use crate::link_graph::index::LinkGraphIndex;
use crate::link_graph::runtime_config::resolve_link_graph_retrieval_policy_runtime;
use crate::link_graph::{LinkGraphPromotedOverlayTelemetry, LinkGraphSearchOptions};

use super::types::{PlannedPayloadBuildContext, PlannedPayloadSearchRequest};

impl LinkGraphIndex {
    /// Parse/execute search and return canonical external payload shape.
    #[must_use]
    pub fn search_planned_payload(
        &self,
        query: &str,
        limit: usize,
        base_options: LinkGraphSearchOptions,
    ) -> crate::link_graph::LinkGraphPlannedSearchPayload {
        self.search_planned_payload_with_agentic(query, limit, base_options, None, None)
    }

    /// Parse/execute search and return canonical external payload shape with
    /// optional provisional suggested-link injection policy.
    #[must_use]
    /// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
    pub fn search_planned_payload_with_agentic(
        &self,
        query: &str,
        limit: usize,
        base_options: LinkGraphSearchOptions,
        include_provisional: Option<bool>,
        provisional_limit: Option<usize>,
    ) -> crate::link_graph::LinkGraphPlannedSearchPayload {
        let runtime = resolve_link_graph_retrieval_policy_runtime();
        if matches!(
            runtime.semantic_ignition.backend,
            crate::link_graph::runtime_config::models::LinkGraphSemanticIgnitionBackend::Disabled
        ) {
            return self.search_planned_payload_with_agentic_sync_internal(
                PlannedPayloadSearchRequest {
                    query: query.to_string(),
                    limit,
                    base_options,
                    include_provisional,
                    provisional_limit,
                    build_context: PlannedPayloadBuildContext {
                        promoted_overlay: None,
                        query_vector_override: None,
                    },
                },
            );
        }
        self.search_planned_payload_with_agentic_runtime_bridge_with_query_vector(
            PlannedPayloadSearchRequest {
                query: query.to_string(),
                limit,
                base_options,
                include_provisional,
                provisional_limit,
                build_context: PlannedPayloadBuildContext {
                    promoted_overlay: None,
                    query_vector_override: None,
                },
            },
        )
    }

    /// Parse/execute search and return canonical external payload shape with
    /// one optional precomputed query vector for semantic ignition and Julia
    /// rerank integration.
    #[must_use]
    /// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
    pub fn search_planned_payload_with_agentic_query_vector(
        &self,
        query: &str,
        query_vector: &[f32],
        limit: usize,
        base_options: LinkGraphSearchOptions,
        include_provisional: Option<bool>,
        provisional_limit: Option<usize>,
    ) -> crate::link_graph::LinkGraphPlannedSearchPayload {
        let runtime = resolve_link_graph_retrieval_policy_runtime();
        let query_vector_override = (!query_vector.is_empty()).then(|| query_vector.to_vec());
        if matches!(
            runtime.semantic_ignition.backend,
            crate::link_graph::runtime_config::models::LinkGraphSemanticIgnitionBackend::Disabled
        ) {
            return self.search_planned_payload_with_agentic_sync_internal(
                PlannedPayloadSearchRequest {
                    query: query.to_string(),
                    limit,
                    base_options,
                    include_provisional,
                    provisional_limit,
                    build_context: PlannedPayloadBuildContext {
                        promoted_overlay: None,
                        query_vector_override,
                    },
                },
            );
        }
        self.search_planned_payload_with_agentic_runtime_bridge_with_query_vector(
            PlannedPayloadSearchRequest {
                query: query.to_string(),
                limit,
                base_options,
                include_provisional,
                provisional_limit,
                build_context: PlannedPayloadBuildContext {
                    promoted_overlay: None,
                    query_vector_override,
                },
            },
        )
    }

    /// Parse/execute search and return canonical external payload shape on the
    /// async path, including optional semantic ignition enrichment.
    /// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
    pub async fn search_planned_payload_with_agentic_async(
        &self,
        query: &str,
        limit: usize,
        base_options: LinkGraphSearchOptions,
        include_provisional: Option<bool>,
        provisional_limit: Option<usize>,
    ) -> crate::link_graph::LinkGraphPlannedSearchPayload {
        self.search_planned_payload_with_agentic_async_with_query_vector(
            query,
            &[],
            limit,
            base_options,
            include_provisional,
            provisional_limit,
        )
        .await
    }

    /// Parse/execute search and return canonical external payload shape on the
    /// async path with one optional precomputed query vector for semantic
    /// ignition and Julia rerank integration.
    /// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
    pub async fn search_planned_payload_with_agentic_async_with_query_vector(
        &self,
        query: &str,
        query_vector: &[f32],
        limit: usize,
        base_options: LinkGraphSearchOptions,
        include_provisional: Option<bool>,
        provisional_limit: Option<usize>,
    ) -> crate::link_graph::LinkGraphPlannedSearchPayload {
        let (overlay, overlay_stats) = self.with_promoted_edges_overlay_with_stats();
        let promoted_overlay = Some(LinkGraphPromotedOverlayTelemetry {
            applied: overlay_stats.applied,
            source: overlay_stats.source.to_string(),
            scanned_rows: overlay_stats.scanned_rows,
            promoted_rows: overlay_stats.promoted_rows,
            added_edges: overlay_stats.added_edges,
        });

        if let Some(overlay) = overlay {
            return overlay
                .search_planned_payload_with_agentic_core_async(PlannedPayloadSearchRequest {
                    query: query.to_string(),
                    limit,
                    base_options,
                    include_provisional,
                    provisional_limit,
                    build_context: PlannedPayloadBuildContext {
                        promoted_overlay,
                        query_vector_override: (!query_vector.is_empty())
                            .then(|| query_vector.to_vec()),
                    },
                })
                .await;
        }
        self.search_planned_payload_with_agentic_core_async(PlannedPayloadSearchRequest {
            query: query.to_string(),
            limit,
            base_options,
            include_provisional,
            provisional_limit,
            build_context: PlannedPayloadBuildContext {
                promoted_overlay,
                query_vector_override: (!query_vector.is_empty()).then(|| query_vector.to_vec()),
            },
        })
        .await
    }

    fn search_planned_payload_with_agentic_sync_internal(
        &self,
        request: PlannedPayloadSearchRequest,
    ) -> crate::link_graph::LinkGraphPlannedSearchPayload {
        let (overlay, overlay_stats) = self.with_promoted_edges_overlay_with_stats();
        let promoted_overlay = Some(LinkGraphPromotedOverlayTelemetry {
            applied: overlay_stats.applied,
            source: overlay_stats.source.to_string(),
            scanned_rows: overlay_stats.scanned_rows,
            promoted_rows: overlay_stats.promoted_rows,
            added_edges: overlay_stats.added_edges,
        });
        let PlannedPayloadSearchRequest {
            query,
            limit,
            base_options,
            include_provisional,
            provisional_limit,
            build_context,
        } = request;
        let request = PlannedPayloadSearchRequest {
            query,
            limit,
            base_options,
            include_provisional,
            provisional_limit,
            build_context: PlannedPayloadBuildContext {
                promoted_overlay,
                query_vector_override: build_context.query_vector_override,
            },
        };

        if let Some(overlay) = overlay {
            return overlay.search_planned_payload_with_agentic_core_sync(request);
        }
        self.search_planned_payload_with_agentic_core_sync(request)
    }
}

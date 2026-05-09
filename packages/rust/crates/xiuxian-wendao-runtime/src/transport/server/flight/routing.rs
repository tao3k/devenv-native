//! Route dispatch and cache-key derivation for the Wendao Flight service.

use std::sync::Arc;

use tonic::Status;

use crate::transport::query_contract::{
    ANALYSIS_CODE_AST_ROUTE, ANALYSIS_DOCUMENT_EXTRACT_ROUTE,
    ANALYSIS_DOCUMENT_EXTRACT_STATUS_ROUTE, ANALYSIS_MARKDOWN_ROUTE, ANALYSIS_REFINE_DOC_ROUTE,
    ANALYSIS_REPO_DOC_COVERAGE_ROUTE, ANALYSIS_REPO_INDEX_ROUTE, ANALYSIS_REPO_INDEX_STATUS_ROUTE,
    ANALYSIS_REPO_OVERVIEW_ROUTE, ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE,
    ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE, ANALYSIS_REPO_SYNC_ROUTE,
    GRAPH_NEIGHBORS_ROUTE, QUERY_SQL_ROUTE, REPO_SEARCH_ROUTE, SEARCH_AST_ROUTE,
    SEARCH_ATTACHMENTS_ROUTE, SEARCH_AUTOCOMPLETE_ROUTE, SEARCH_DEFINITION_ROUTE,
    TOPOLOGY_3D_ROUTE, VFS_CONTENT_ROUTE, VFS_RESOLVE_ROUTE, VFS_SCAN_ROUTE,
};

use crate::transport::server::{
    is_search_family_route, join_sorted_set, validate_attachment_search_request_metadata,
    validate_autocomplete_request_metadata, validate_code_ast_analysis_request_metadata,
    validate_definition_request_metadata, validate_document_extract_request_metadata,
    validate_document_extract_status_request_metadata, validate_graph_neighbors_request_metadata,
    validate_markdown_analysis_request_metadata, validate_refine_doc_request_metadata,
    validate_repo_doc_coverage_request_metadata, validate_repo_index_request_metadata,
    validate_repo_index_status_request_metadata, validate_repo_overview_request_metadata,
    validate_repo_projected_page_index_tree_request_metadata,
    validate_repo_projected_retrieval_context_request_metadata,
    validate_repo_search_request_metadata, validate_repo_sync_request_metadata,
    validate_search_request_metadata, validate_sql_request_metadata,
    validate_vfs_content_request_metadata, validate_vfs_resolve_request_metadata,
};

use super::ServiceCore as WendaoFlightService;
use super::payload::FlightRoutePayload;

fn document_extract_cache_key(
    route: &str,
    metadata: &tonic::metadata::MetadataMap,
) -> Result<String, Status> {
    let request = validate_document_extract_request_metadata(metadata)?;
    Ok(format!(
        "{route}|{:?}|{:?}|{}|{}|{:?}|{}",
        request.source_path,
        request.output_dir,
        request.force,
        request.error_row,
        request.mode,
        request.wait_ms
    ))
}

fn document_extract_status_cache_key(
    route: &str,
    metadata: &tonic::metadata::MetadataMap,
) -> Result<String, Status> {
    let job_id = validate_document_extract_status_request_metadata(metadata)?;
    Ok(format!("{route}|{job_id:?}"))
}

impl WendaoFlightService {
    pub(super) fn route_request_cache_key(
        route: &str,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<String, Status> {
        if route == REPO_SEARCH_ROUTE {
            let request = validate_repo_search_request_metadata(metadata)?;
            Ok(format!(
                "{route}|{repo_id:?}|{query_text:?}|{limit}|{}|{}|{}|{}|{}",
                join_sorted_set(&request.language_filters),
                join_sorted_set(&request.path_prefixes),
                join_sorted_set(&request.title_filters),
                join_sorted_set(&request.tag_filters),
                join_sorted_set(&request.filename_filters),
                repo_id = request.repo_id,
                query_text = request.query_text,
                limit = request.limit,
            ))
        } else if route == SEARCH_ATTACHMENTS_ROUTE {
            let (query_text, limit, ext_filters, kind_filters, case_sensitive) =
                validate_attachment_search_request_metadata(metadata)?;
            Ok(format!(
                "{route}|{query_text:?}|{limit}|{}|{}|{case_sensitive}",
                join_sorted_set(&ext_filters),
                join_sorted_set(&kind_filters),
            ))
        } else if route == SEARCH_AST_ROUTE {
            let (query_text, limit, intent, repo_hint) =
                validate_search_request_metadata(metadata)?;
            Ok(format!(
                "{route}|{query_text:?}|{limit}|{intent:?}|{repo_hint:?}"
            ))
        } else if route == SEARCH_DEFINITION_ROUTE {
            let (query_text, source_path, source_line) =
                validate_definition_request_metadata(metadata)?;
            Ok(format!(
                "{route}|{query_text:?}|{source_path:?}|{source_line:?}"
            ))
        } else if route == SEARCH_AUTOCOMPLETE_ROUTE {
            let (prefix, limit) = validate_autocomplete_request_metadata(metadata)?;
            Ok(format!("{route}|{prefix:?}|{limit}"))
        } else if route == QUERY_SQL_ROUTE {
            let query_text = validate_sql_request_metadata(metadata)?;
            Ok(format!("{route}|{query_text:?}"))
        } else if route == VFS_RESOLVE_ROUTE {
            let path = validate_vfs_resolve_request_metadata(metadata)?;
            Ok(format!("{route}|{path:?}"))
        } else if route == VFS_CONTENT_ROUTE {
            let path = validate_vfs_content_request_metadata(metadata)?;
            Ok(format!("{route}|{path:?}"))
        } else if route == VFS_SCAN_ROUTE {
            Ok(route.to_string())
        } else if route == GRAPH_NEIGHBORS_ROUTE {
            let (node_id, direction, hops, limit) =
                validate_graph_neighbors_request_metadata(metadata)?;
            Ok(format!("{route}|{node_id:?}|{direction:?}|{hops}|{limit}"))
        } else if route == TOPOLOGY_3D_ROUTE {
            Ok(route.to_string())
        } else if route == ANALYSIS_MARKDOWN_ROUTE {
            let path = validate_markdown_analysis_request_metadata(metadata)?;
            Ok(format!("{route}|{path:?}"))
        } else if route == ANALYSIS_CODE_AST_ROUTE {
            let (path, repo_id, line_hint) = validate_code_ast_analysis_request_metadata(metadata)?;
            Ok(format!("{route}|{path:?}|{repo_id:?}|{line_hint:?}"))
        } else if route == ANALYSIS_REPO_OVERVIEW_ROUTE {
            let repo_id = validate_repo_overview_request_metadata(metadata)?;
            Ok(format!("{route}|{repo_id:?}"))
        } else if route == ANALYSIS_REPO_INDEX_ROUTE {
            let (repo_id, refresh, request_id) = validate_repo_index_request_metadata(metadata)?;
            Ok(format!("{route}|{repo_id:?}|{refresh}|{request_id:?}"))
        } else if route == ANALYSIS_REPO_INDEX_STATUS_ROUTE {
            let repo_id = validate_repo_index_status_request_metadata(metadata);
            Ok(format!("{route}|{repo_id:?}"))
        } else if route == ANALYSIS_REPO_SYNC_ROUTE {
            let (repo_id, mode) = validate_repo_sync_request_metadata(metadata)?;
            Ok(format!("{route}|{repo_id:?}|{mode:?}"))
        } else if route == ANALYSIS_REPO_DOC_COVERAGE_ROUTE {
            let (repo_id, module_id) = validate_repo_doc_coverage_request_metadata(metadata)?;
            Ok(format!("{route}|{repo_id:?}|{module_id:?}"))
        } else if route == ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE {
            let (repo_id, page_id) =
                validate_repo_projected_page_index_tree_request_metadata(metadata)?;
            Ok(format!("{route}|{repo_id:?}|{page_id:?}"))
        } else if route == ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE {
            let (repo_id, page_id, node_id, related_limit) =
                validate_repo_projected_retrieval_context_request_metadata(metadata)?;
            Ok(format!(
                "{route}|{repo_id:?}|{page_id:?}|{node_id:?}|{related_limit}"
            ))
        } else if route == ANALYSIS_DOCUMENT_EXTRACT_ROUTE {
            document_extract_cache_key(route, metadata)
        } else if route == ANALYSIS_DOCUMENT_EXTRACT_STATUS_ROUTE {
            document_extract_status_cache_key(route, metadata)
        } else if route == ANALYSIS_REFINE_DOC_ROUTE {
            let (repo_id, entity_id, user_hints) = validate_refine_doc_request_metadata(metadata)?;
            Ok(format!("{route}|{repo_id:?}|{entity_id:?}|{user_hints:?}"))
        } else if is_search_family_route(route) {
            let (query_text, limit, intent, repo_hint) =
                validate_search_request_metadata(metadata)?;
            Ok(format!(
                "{route}|{query_text:?}|{limit}|{intent:?}|{repo_hint:?}"
            ))
        } else {
            Err(Status::invalid_argument(format!(
                "unexpected routed Flight request: {route}"
            )))
        }
    }

    pub(super) async fn read_route_payload(
        &self,
        route: &str,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<FlightRoutePayload, Status> {
        if route == REPO_SEARCH_ROUTE {
            self.read_repo_search_payload(metadata).await
        } else if route == SEARCH_ATTACHMENTS_ROUTE {
            self.read_attachment_search_payload(route, metadata).await
        } else if route == SEARCH_AST_ROUTE {
            self.read_ast_search_payload(route, metadata).await
        } else if route == SEARCH_DEFINITION_ROUTE {
            self.read_definition_payload(route, metadata).await
        } else if route == SEARCH_AUTOCOMPLETE_ROUTE {
            self.read_autocomplete_payload(route, metadata).await
        } else if route == QUERY_SQL_ROUTE {
            self.read_sql_payload(route, metadata).await
        } else if route == VFS_RESOLVE_ROUTE {
            self.read_vfs_resolve_payload(route, metadata).await
        } else if route == VFS_CONTENT_ROUTE {
            self.read_vfs_content_payload(route, metadata).await
        } else if route == VFS_SCAN_ROUTE {
            self.read_vfs_scan_payload(route).await
        } else if route == GRAPH_NEIGHBORS_ROUTE {
            self.read_graph_neighbors_payload(route, metadata).await
        } else if route == TOPOLOGY_3D_ROUTE {
            self.read_topology_3d_payload(route).await
        } else if route == ANALYSIS_MARKDOWN_ROUTE {
            self.read_markdown_analysis_payload(route, metadata).await
        } else if route == ANALYSIS_CODE_AST_ROUTE {
            self.read_code_ast_analysis_payload(route, metadata).await
        } else if route == ANALYSIS_REPO_OVERVIEW_ROUTE {
            self.read_repo_overview_payload(route, metadata).await
        } else if route == ANALYSIS_REPO_INDEX_ROUTE {
            self.read_repo_index_payload(route, metadata).await
        } else if route == ANALYSIS_REPO_INDEX_STATUS_ROUTE {
            self.read_repo_index_status_payload(route, metadata).await
        } else if route == ANALYSIS_REPO_SYNC_ROUTE {
            self.read_repo_sync_payload(route, metadata).await
        } else if route == ANALYSIS_REPO_DOC_COVERAGE_ROUTE {
            self.read_repo_doc_coverage_payload(route, metadata).await
        } else if route == ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE {
            self.read_repo_projected_page_index_tree_payload(route, metadata)
                .await
        } else if route == ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE {
            self.read_repo_projected_retrieval_context_payload(route, metadata)
                .await
        } else if route == ANALYSIS_DOCUMENT_EXTRACT_ROUTE {
            self.read_document_extract_payload(route, metadata).await
        } else if route == ANALYSIS_DOCUMENT_EXTRACT_STATUS_ROUTE {
            self.read_document_extract_status_payload(route, metadata)
                .await
        } else if route == ANALYSIS_REFINE_DOC_ROUTE {
            self.read_refine_doc_payload(route, metadata).await
        } else if is_search_family_route(route) {
            self.read_search_family_payload(route, metadata).await
        } else {
            Err(Status::invalid_argument(format!(
                "unexpected routed Flight request: {route}"
            )))
        }
    }

    async fn read_repo_search_payload(
        &self,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<FlightRoutePayload, Status> {
        let request = validate_repo_search_request_metadata(metadata)?;
        self.repo_search_provider
            .repo_search_batch(&request)
            .await
            .map_err(Status::internal)
            .and_then(FlightRoutePayload::try_new)
    }

    async fn read_attachment_search_payload(
        &self,
        route: &str,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<FlightRoutePayload, Status> {
        let (query_text, limit, ext_filters, kind_filters, case_sensitive) =
            validate_attachment_search_request_metadata(metadata)?;
        let provider = self.attachment_search_provider.as_ref().ok_or_else(|| {
            Status::unimplemented(format!(
                "attachment-search Flight route `{route}` is not configured for this runtime host"
            ))
        })?;
        provider
            .attachment_search_batch(
                query_text.as_str(),
                limit,
                &ext_filters,
                &kind_filters,
                case_sensitive,
            )
            .await
            .map_err(Status::internal)
            .and_then(|response| {
                FlightRoutePayload::try_with_app_metadata(response.batch, response.app_metadata)
            })
    }

    async fn read_ast_search_payload(
        &self,
        route: &str,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<FlightRoutePayload, Status> {
        let (query_text, limit, _intent, _repo_hint) = validate_search_request_metadata(metadata)?;
        let provider = self.ast_search_provider.as_ref().ok_or_else(|| {
            Status::unimplemented(format!(
                "AST-search Flight route `{route}` is not configured for this runtime host"
            ))
        })?;
        provider
            .ast_search_batch(query_text.as_str(), limit)
            .await
            .map_err(Status::internal)
            .and_then(|response| {
                FlightRoutePayload::try_with_app_metadata(response.batch, response.app_metadata)
            })
    }

    async fn read_definition_payload(
        &self,
        route: &str,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<FlightRoutePayload, Status> {
        let (query_text, source_path, source_line) =
            validate_definition_request_metadata(metadata)?;
        let provider = self.definition_provider.as_ref().ok_or_else(|| {
            Status::unimplemented(format!(
                "definition Flight route `{route}` is not configured for this runtime host"
            ))
        })?;
        provider
            .definition_batch(query_text.as_str(), source_path.as_deref(), source_line)
            .await
            .and_then(|response| {
                FlightRoutePayload::try_with_app_metadata(response.batch, response.app_metadata)
            })
    }

    async fn read_autocomplete_payload(
        &self,
        route: &str,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<FlightRoutePayload, Status> {
        let (prefix, limit) = validate_autocomplete_request_metadata(metadata)?;
        let provider = self.autocomplete_provider.as_ref().ok_or_else(|| {
            Status::unimplemented(format!(
                "autocomplete Flight route `{route}` is not configured for this runtime host"
            ))
        })?;
        provider
            .autocomplete_batch(prefix.as_str(), limit)
            .await
            .and_then(|response| {
                FlightRoutePayload::try_with_app_metadata(response.batch, response.app_metadata)
            })
    }

    async fn read_sql_payload(
        &self,
        route: &str,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<FlightRoutePayload, Status> {
        let query_text = validate_sql_request_metadata(metadata)?;
        let provider = self.sql_provider.as_ref().ok_or_else(|| {
            Status::unimplemented(format!(
                "SQL Flight route `{route}` is not configured for this runtime host"
            ))
        })?;
        let response = provider
            .sql_query_batches(query_text.as_str())
            .await
            .map_err(Status::internal)?;
        FlightRoutePayload::from_engine_batches_with_app_metadata(
            &response.batches,
            response.app_metadata,
        )
    }

    async fn read_vfs_resolve_payload(
        &self,
        route: &str,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<FlightRoutePayload, Status> {
        let path = validate_vfs_resolve_request_metadata(metadata)?;
        let provider = self.vfs_resolve_provider.as_ref().ok_or_else(|| {
            Status::unimplemented(format!(
                "VFS resolve Flight route `{route}` is not configured for this runtime host"
            ))
        })?;
        provider
            .resolve_vfs_navigation_batch(path.as_str())
            .await
            .and_then(|response| {
                FlightRoutePayload::try_with_app_metadata(response.batch, response.app_metadata)
            })
    }

    async fn read_vfs_content_payload(
        &self,
        route: &str,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<FlightRoutePayload, Status> {
        let path = validate_vfs_content_request_metadata(metadata)?;
        let provider = self.vfs_content_provider.as_ref().ok_or_else(|| {
            Status::unimplemented(format!(
                "VFS content Flight route `{route}` is not configured for this runtime host"
            ))
        })?;
        provider
            .read_vfs_content_batch(path.as_str())
            .await
            .and_then(|response| {
                FlightRoutePayload::try_with_app_metadata(response.batch, response.app_metadata)
            })
    }

    async fn read_graph_neighbors_payload(
        &self,
        route: &str,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<FlightRoutePayload, Status> {
        let (node_id, direction, hops, limit) =
            validate_graph_neighbors_request_metadata(metadata)?;
        let provider = self.graph_neighbors_provider.as_ref().ok_or_else(|| {
            Status::unimplemented(format!(
                "graph-neighbors Flight route `{route}` is not configured for this runtime host"
            ))
        })?;
        provider
            .graph_neighbors_batch(node_id.as_str(), direction.as_str(), hops, limit)
            .await
            .and_then(|response| {
                FlightRoutePayload::try_with_app_metadata(response.batch, response.app_metadata)
            })
    }

    async fn read_vfs_scan_payload(&self, route: &str) -> Result<FlightRoutePayload, Status> {
        let provider = self.vfs_scan_provider.as_ref().ok_or_else(|| {
            Status::unimplemented(format!(
                "VFS scan Flight route `{route}` is not configured for this runtime host"
            ))
        })?;
        provider.scan_vfs_batch().await.and_then(|response| {
            FlightRoutePayload::try_with_app_metadata(response.batch, response.app_metadata)
        })
    }

    async fn read_topology_3d_payload(&self, route: &str) -> Result<FlightRoutePayload, Status> {
        let provider = self.topology_3d_provider.as_ref().ok_or_else(|| {
            Status::unimplemented(format!(
                "topology-3d Flight route `{route}` is not configured for this runtime host"
            ))
        })?;
        provider.topology_3d_batch().await.and_then(|response| {
            FlightRoutePayload::try_with_app_metadata(response.batch, response.app_metadata)
        })
    }

    async fn read_markdown_analysis_payload(
        &self,
        route: &str,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<FlightRoutePayload, Status> {
        let path = validate_markdown_analysis_request_metadata(metadata)?;
        let provider = self.markdown_analysis_provider.as_ref().ok_or_else(|| {
            Status::unimplemented(format!(
                "markdown analysis Flight route `{route}` is not configured for this runtime host"
            ))
        })?;
        provider
            .markdown_analysis_batch(path.as_str())
            .await
            .map_err(Status::internal)
            .and_then(|response| {
                FlightRoutePayload::try_with_app_metadata(response.batch, response.app_metadata)
            })
    }

    async fn read_code_ast_analysis_payload(
        &self,
        route: &str,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<FlightRoutePayload, Status> {
        let (path, repo_id, line_hint) = validate_code_ast_analysis_request_metadata(metadata)?;
        let provider = self.code_ast_analysis_provider.as_ref().ok_or_else(|| {
            Status::unimplemented(format!(
                "code-AST analysis Flight route `{route}` is not configured for this runtime host"
            ))
        })?;
        provider
            .code_ast_analysis_batch(path.as_str(), repo_id.as_str(), line_hint)
            .await
            .map_err(Status::internal)
            .and_then(|response| {
                FlightRoutePayload::try_with_app_metadata(response.batch, response.app_metadata)
            })
    }

    async fn read_repo_overview_payload(
        &self,
        route: &str,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<FlightRoutePayload, Status> {
        let repo_id = validate_repo_overview_request_metadata(metadata)?;
        let provider = self.repo_overview_provider.as_ref().ok_or_else(|| {
            Status::unimplemented(format!(
                "repo overview Flight route `{route}` is not configured for this runtime host"
            ))
        })?;
        provider
            .repo_overview_batch(repo_id.as_str())
            .await
            .map_err(Status::internal)
            .and_then(|response| {
                FlightRoutePayload::try_with_app_metadata(response.batch, response.app_metadata)
            })
    }

    async fn read_repo_index_payload(
        &self,
        route: &str,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<FlightRoutePayload, Status> {
        let (repo_id, refresh, _request_id) = validate_repo_index_request_metadata(metadata)?;
        let provider = self.repo_index_provider.as_ref().ok_or_else(|| {
            Status::unimplemented(format!(
                "repo index Flight route `{route}` is not configured for this runtime host"
            ))
        })?;
        provider
            .repo_index_batch(repo_id.as_deref(), refresh)
            .await
            .map_err(Status::internal)
            .and_then(|response| {
                FlightRoutePayload::try_with_app_metadata(response.batch, response.app_metadata)
            })
    }

    async fn read_repo_index_status_payload(
        &self,
        route: &str,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<FlightRoutePayload, Status> {
        let repo_id = validate_repo_index_status_request_metadata(metadata);
        let provider = self.repo_index_status_provider.as_ref().ok_or_else(|| {
            Status::unimplemented(format!(
                "repo index status Flight route `{route}` is not configured for this runtime host"
            ))
        })?;
        provider
            .repo_index_status_batch(repo_id.as_deref())
            .await
            .map_err(Status::internal)
            .and_then(|response| {
                FlightRoutePayload::try_with_app_metadata(response.batch, response.app_metadata)
            })
    }

    async fn read_repo_sync_payload(
        &self,
        route: &str,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<FlightRoutePayload, Status> {
        let (repo_id, mode) = validate_repo_sync_request_metadata(metadata)?;
        let provider = self.repo_sync_provider.as_ref().ok_or_else(|| {
            Status::unimplemented(format!(
                "repo sync Flight route `{route}` is not configured for this runtime host"
            ))
        })?;
        provider
            .repo_sync_batch(repo_id.as_str(), mode.as_str())
            .await
            .map_err(Status::internal)
            .and_then(|response| {
                FlightRoutePayload::try_with_app_metadata(response.batch, response.app_metadata)
            })
    }

    async fn read_repo_doc_coverage_payload(
        &self,
        route: &str,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<FlightRoutePayload, Status> {
        let (repo_id, module_id) = validate_repo_doc_coverage_request_metadata(metadata)?;
        let provider = self.repo_doc_coverage_provider.as_ref().ok_or_else(|| {
            Status::unimplemented(format!(
                "repo doc coverage Flight route `{route}` is not configured for this runtime host"
            ))
        })?;
        provider
            .repo_doc_coverage_batch(repo_id.as_str(), module_id.as_deref())
            .await
            .map_err(Status::internal)
            .and_then(|response| {
                FlightRoutePayload::try_with_app_metadata(response.batch, response.app_metadata)
            })
    }

    async fn read_repo_projected_page_index_tree_payload(
        &self,
        route: &str,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<FlightRoutePayload, Status> {
        let (repo_id, page_id) =
            validate_repo_projected_page_index_tree_request_metadata(metadata)?;
        let provider = self
            .repo_projected_page_index_tree_provider
            .as_ref()
            .ok_or_else(|| {
                Status::unimplemented(format!(
                    "repo projected page-index tree Flight route `{route}` is not configured for this runtime host"
                ))
            })?;
        provider
            .repo_projected_page_index_tree_batch(repo_id.as_str(), page_id.as_str())
            .await
            .and_then(|response| {
                FlightRoutePayload::try_with_app_metadata(response.batch, response.app_metadata)
            })
    }

    async fn read_repo_projected_retrieval_context_payload(
        &self,
        route: &str,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<FlightRoutePayload, Status> {
        let (repo_id, page_id, node_id, related_limit) =
            validate_repo_projected_retrieval_context_request_metadata(metadata)?;
        let provider = self
            .repo_projected_retrieval_context_provider
            .as_ref()
            .ok_or_else(|| {
                Status::unimplemented(format!(
                    "repo projected retrieval-context Flight route `{route}` is not configured for this runtime host"
                ))
            })?;
        provider
            .repo_projected_retrieval_context_batch(
                repo_id.as_str(),
                page_id.as_str(),
                node_id.as_deref(),
                related_limit,
            )
            .await
            .and_then(|response| {
                FlightRoutePayload::try_with_app_metadata(response.batch, response.app_metadata)
            })
    }

    async fn read_refine_doc_payload(
        &self,
        route: &str,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<FlightRoutePayload, Status> {
        let (repo_id, entity_id, user_hints) = validate_refine_doc_request_metadata(metadata)?;
        let provider = self.refine_doc_provider.as_ref().ok_or_else(|| {
            Status::unimplemented(format!(
                "refine-doc Flight route `{route}` is not configured for this runtime host"
            ))
        })?;
        provider
            .refine_doc_batch(repo_id.as_str(), entity_id.as_str(), user_hints.as_deref())
            .await
            .and_then(|response| {
                FlightRoutePayload::try_with_app_metadata(response.batch, response.app_metadata)
            })
    }

    async fn read_document_extract_payload(
        &self,
        route: &str,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<FlightRoutePayload, Status> {
        let request = validate_document_extract_request_metadata(metadata)?;
        let provider = self.document_extract_provider.as_ref().ok_or_else(|| {
            Status::unimplemented(format!(
                "document extract Flight route `{route}` is not configured for this runtime host"
            ))
        })?;
        provider
            .document_extract_batch_for_request(&request)
            .await
            .map_err(Status::internal)
            .and_then(|response| {
                FlightRoutePayload::from_engine_batches_with_app_metadata(
                    &response.batches,
                    response.app_metadata,
                )
            })
    }

    async fn read_document_extract_status_payload(
        &self,
        route: &str,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<FlightRoutePayload, Status> {
        let job_id = validate_document_extract_status_request_metadata(metadata)?;
        let provider = self.document_extract_provider.as_ref().ok_or_else(|| {
            Status::unimplemented(format!(
                "document extract Flight route `{route}` is not configured for this runtime host"
            ))
        })?;
        provider
            .document_extract_status_batch(job_id.as_str())
            .await
            .map_err(Status::internal)
            .and_then(|response| {
                FlightRoutePayload::from_engine_batches_with_app_metadata(
                    &response.batches,
                    response.app_metadata,
                )
            })
    }

    async fn read_search_family_payload(
        &self,
        route: &str,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<FlightRoutePayload, Status> {
        let (query_text, limit, intent, repo_hint) = validate_search_request_metadata(metadata)?;
        let provider = self.search_provider.as_ref().ok_or_else(|| {
            Status::unimplemented(format!(
                "search Flight route `{route}` is not configured for this runtime host"
            ))
        })?;
        provider
            .search_batch(
                route,
                query_text.as_str(),
                limit,
                intent.as_deref(),
                repo_hint.as_deref(),
            )
            .await
            .map_err(Status::internal)
            .and_then(|response| {
                FlightRoutePayload::try_with_app_metadata(response.batch, response.app_metadata)
            })
    }

    pub(super) async fn cached_route_payload(
        &self,
        route: &str,
        metadata: &tonic::metadata::MetadataMap,
        cache_key: &str,
    ) -> Result<Arc<FlightRoutePayload>, Status> {
        if route == ANALYSIS_DOCUMENT_EXTRACT_STATUS_ROUTE {
            return self.read_route_payload(route, metadata).await.map(Arc::new);
        }
        if let Some(cached) = self.route_payload_cache.get(cache_key).await {
            return Ok(cached);
        }
        let payload = self.read_route_payload(route, metadata).await?;
        Ok(self
            .route_payload_cache
            .insert(cache_key.to_string(), payload)
            .await)
    }
}

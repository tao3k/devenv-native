use super::rerank::{apply_openai_plugin_rerank, apply_vector_store_plugin_rerank};
use crate::link_graph::index::LinkGraphIndex;
use crate::link_graph::models::{
    LinkGraphJuliaRerankTelemetry, LinkGraphRetrievalPlanRecord, QuantumContext,
};
use crate::link_graph::runtime_config::models::{
    LinkGraphSemanticIgnitionBackend, LinkGraphSemanticIgnitionRuntimeConfig,
};
use crate::link_graph::runtime_config::resolve_link_graph_retrieval_policy_runtime;
use crate::link_graph::{
    LinkGraphPlannedSearchPayload, LinkGraphSemanticIgnitionTelemetry,
    OpenAiCompatibleSemanticIgnition, QuantumFusionOptions, QuantumSemanticIgnition,
    VectorStoreSemanticIgnition,
};
use xiuxian_db_store::VectorStore;
use xiuxian_wendao_core::capabilities::PluginCapabilityBinding;

type SemanticIgnitionOutcome = Result<
    (
        Option<String>,
        Vec<QuantumContext>,
        Option<LinkGraphJuliaRerankTelemetry>,
    ),
    String,
>;

struct QuantumContextRuntimeRequest<'a> {
    query_text: &'a str,
    query_vector: &'a [f32],
    retrieval_plan: &'a LinkGraphRetrievalPlanRecord,
    rerank_binding: Option<&'a PluginCapabilityBinding>,
}

impl LinkGraphIndex {
    pub(crate) async fn enrich_planned_payload_with_quantum_contexts(
        &self,
        payload: &mut LinkGraphPlannedSearchPayload,
        query_vector: &[f32],
    ) {
        let runtime = resolve_link_graph_retrieval_policy_runtime();
        let backend = runtime.semantic_ignition.backend;
        let rerank_binding = runtime.rerank_binding();
        let backend_label = semantic_ignition_backend_label(backend);
        if backend_label.is_empty() {
            return;
        }

        let Some(retrieval_plan) = payload.retrieval_plan.as_ref() else {
            record_semantic_ignition_error(
                payload,
                backend_label,
                "semantic ignition skipped because retrieval plan is missing".to_string(),
            );
            return;
        };

        let (vector_store_path, table_name) =
            match resolve_vector_store_requirements(&runtime.semantic_ignition) {
                Ok(parts) => parts,
                Err(error) => {
                    record_semantic_ignition_error(payload, backend_label, error);
                    return;
                }
            };

        let store = match VectorStore::new(vector_store_path, None).await {
            Ok(store) => store,
            Err(error) => {
                record_semantic_ignition_error(
                    payload,
                    backend_label,
                    format!("failed to open vector store: {error}"),
                );
                return;
            }
        };

        let outcome = match backend {
            LinkGraphSemanticIgnitionBackend::Disabled => return,
            LinkGraphSemanticIgnitionBackend::VectorStore => {
                self.quantum_contexts_from_vector_store_runtime(
                    store,
                    table_name,
                    QuantumContextRuntimeRequest {
                        query_text: payload.query.as_str(),
                        query_vector,
                        retrieval_plan,
                        rerank_binding: rerank_binding.as_ref(),
                    },
                )
                .await
            }
            LinkGraphSemanticIgnitionBackend::OpenAiCompatible => {
                self.quantum_contexts_from_openai_runtime(
                    store,
                    &runtime.semantic_ignition,
                    table_name,
                    QuantumContextRuntimeRequest {
                        query_text: payload.query.as_str(),
                        query_vector,
                        retrieval_plan,
                        rerank_binding: rerank_binding.as_ref(),
                    },
                )
                .await
            }
        };

        apply_semantic_ignition_outcome(payload, backend_label, outcome);
    }

    async fn quantum_contexts_from_vector_store_runtime(
        &self,
        store: VectorStore,
        table_name: &str,
        request: QuantumContextRuntimeRequest<'_>,
    ) -> SemanticIgnitionOutcome {
        let ignition = VectorStoreSemanticIgnition::new(store, table_name);
        let backend_name = ignition.backend_name().to_string();
        let mut contexts = self
            .quantum_contexts_from_retrieval_plan(
                &ignition,
                Some(request.query_text),
                request.query_vector,
                Some(request.retrieval_plan),
                None,
                &QuantumFusionOptions::default(),
            )
            .await
            .map_err(|error| error.to_string())?;
        let telemetry = apply_vector_store_plugin_rerank(
            &ignition,
            request.rerank_binding,
            request.query_text,
            request.query_vector,
            request.retrieval_plan,
            &mut contexts,
        )
        .await;
        Ok((Some(backend_name), contexts, telemetry))
    }

    async fn quantum_contexts_from_openai_runtime(
        &self,
        store: VectorStore,
        config: &LinkGraphSemanticIgnitionRuntimeConfig,
        table_name: &str,
        request: QuantumContextRuntimeRequest<'_>,
    ) -> SemanticIgnitionOutcome {
        let Some(embedding_base_url) = config.embedding_base_url.as_deref() else {
            return Err(
                "openai-compatible semantic ignition requires `link_graph.retrieval.semantic_ignition.embedding_base_url`"
                    .to_string(),
            );
        };
        let mut ignition =
            OpenAiCompatibleSemanticIgnition::new(store, table_name, embedding_base_url);
        if let Some(model) = config.embedding_model.as_deref() {
            ignition = ignition.with_embedding_model(model);
        }
        let backend_name = ignition.backend_name().to_string();
        let mut contexts = self
            .quantum_contexts_from_retrieval_plan(
                &ignition,
                Some(request.query_text),
                request.query_vector,
                Some(request.retrieval_plan),
                None,
                &QuantumFusionOptions::default(),
            )
            .await
            .map_err(|error| error.to_string())?;
        let telemetry = apply_openai_plugin_rerank(
            &ignition,
            request.rerank_binding,
            request.query_text,
            request.retrieval_plan,
            &mut contexts,
        )
        .await;
        Ok((Some(backend_name), contexts, telemetry))
    }
}

fn resolve_vector_store_requirements(
    config: &LinkGraphSemanticIgnitionRuntimeConfig,
) -> Result<(&str, &str), String> {
    let Some(vector_store_path) = config.vector_store_path.as_deref() else {
        return Err(
            "semantic ignition requires `link_graph.retrieval.semantic_ignition.vector_store_path`"
                .to_string(),
        );
    };
    let Some(table_name) = config.table_name.as_deref() else {
        return Err(
            "semantic ignition requires `link_graph.retrieval.semantic_ignition.table_name`"
                .to_string(),
        );
    };
    Ok((vector_store_path, table_name))
}

fn apply_semantic_ignition_outcome(
    payload: &mut LinkGraphPlannedSearchPayload,
    backend_label: &str,
    outcome: SemanticIgnitionOutcome,
) {
    match outcome {
        Ok((backend_name, contexts, julia_rerank)) => {
            payload.semantic_ignition = Some(LinkGraphSemanticIgnitionTelemetry {
                backend: backend_label.to_string(),
                backend_name,
                context_count: contexts.len(),
                error: None,
            });
            payload.julia_rerank = julia_rerank;
            payload.quantum_contexts = contexts;
        }
        Err(error) => record_semantic_ignition_error(payload, backend_label, error),
    }
}

fn record_semantic_ignition_error(
    payload: &mut LinkGraphPlannedSearchPayload,
    backend_label: &str,
    error: String,
) {
    payload.semantic_ignition = Some(LinkGraphSemanticIgnitionTelemetry {
        backend: backend_label.to_string(),
        backend_name: None,
        context_count: 0,
        error: Some(error),
    });
    payload.julia_rerank = None;
    payload.quantum_contexts.clear();
}

fn semantic_ignition_backend_label(backend: LinkGraphSemanticIgnitionBackend) -> &'static str {
    match backend {
        LinkGraphSemanticIgnitionBackend::Disabled => "",
        LinkGraphSemanticIgnitionBackend::VectorStore => "vector_store",
        LinkGraphSemanticIgnitionBackend::OpenAiCompatible => "openai_compatible",
    }
}

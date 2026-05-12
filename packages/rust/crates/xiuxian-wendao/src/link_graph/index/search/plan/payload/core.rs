use super::policy::{LinkGraphPolicyDecision, evaluate_link_graph_policy};
use super::types::{PlannedPayloadBuildContext, PlannedPayloadSearchRequest};
use crate::link_graph::agentic::{LinkGraphSuggestedLink, LinkGraphSuggestedLinkState};
use crate::link_graph::index::LinkGraphIndex;
use crate::link_graph::runtime_config::resolve_link_graph_agentic_runtime;
use crate::link_graph::valkey_suggested_link_recent_latest;
use crate::link_graph::{
    LinkGraphCcsAudit, LinkGraphDirection, LinkGraphDisplayHit, LinkGraphHit,
    LinkGraphPlannedSearchPayload,
};
use crate::parsers::link_graph::query::{ParsedLinkGraphQuery, parse_search_query};
use std::collections::HashMap;

impl LinkGraphIndex {
    pub(in crate::link_graph::index::search::plan) async fn search_planned_payload_with_agentic_core_async(
        &self,
        request: PlannedPayloadSearchRequest,
    ) -> LinkGraphPlannedSearchPayload {
        let query_vector_override = request.build_context.query_vector_override.clone();
        let mut payload = self.search_planned_payload_with_agentic_core_sync(request);
        self.enrich_planned_payload_with_quantum_contexts(
            &mut payload,
            query_vector_override.as_deref().unwrap_or(&[]),
        )
        .await;
        payload
    }

    pub(in crate::link_graph::index::search::plan) fn search_planned_payload_with_agentic_core_sync(
        &self,
        request: PlannedPayloadSearchRequest,
    ) -> LinkGraphPlannedSearchPayload {
        let PlannedPayloadSearchRequest {
            query,
            limit,
            base_options,
            include_provisional,
            provisional_limit,
            build_context,
        } = request;
        let parsed = parse_search_query(&query, base_options);
        let effective_limit = parsed.limit_override.unwrap_or(limit);

        if let Some(direct_id) = parsed.direct_id.as_deref() {
            let rows = self.execute_direct_id_lookup(direct_id, effective_limit, &parsed.options);
            let policy = evaluate_link_graph_policy(&rows, effective_limit);
            return self.build_planned_payload(
                build_context,
                parsed,
                rows,
                policy,
                Vec::new(),
                None,
            );
        }

        let (provisional_suggestions, provisional_error, provisional_doc_boosts) =
            self.resolve_provisional_search_inputs(&parsed, include_provisional, provisional_limit);

        let rows = self.execute_search_with_doc_boosts(
            &parsed.query,
            effective_limit,
            &parsed.options,
            (!provisional_doc_boosts.is_empty()).then_some(&provisional_doc_boosts),
        );

        let policy = evaluate_link_graph_policy(&rows, effective_limit);

        self.build_planned_payload(
            build_context,
            parsed,
            rows,
            policy,
            provisional_suggestions,
            provisional_error,
        )
    }

    fn resolve_provisional_search_inputs(
        &self,
        parsed: &ParsedLinkGraphQuery,
        include_provisional: Option<bool>,
        provisional_limit: Option<usize>,
    ) -> (
        Vec<LinkGraphSuggestedLink>,
        Option<String>,
        HashMap<String, f64>,
    ) {
        let agentic_runtime = resolve_link_graph_agentic_runtime();
        let include_provisional =
            include_provisional.unwrap_or(agentic_runtime.search_include_provisional_default);
        let provisional_limit = provisional_limit
            .unwrap_or(agentic_runtime.search_provisional_limit)
            .max(1);
        let (provisional_suggestions, provisional_error) = if include_provisional {
            match valkey_suggested_link_recent_latest(
                provisional_limit,
                Some(LinkGraphSuggestedLinkState::Provisional),
            ) {
                Ok(rows) => (rows, None),
                Err(err) => (Vec::new(), Some(err)),
            }
        } else {
            (Vec::new(), None)
        };
        let provisional_doc_boosts = if include_provisional {
            self.build_provisional_doc_boosts(
                &parsed.query,
                parsed.options.case_sensitive,
                &provisional_suggestions,
            )
        } else {
            HashMap::new()
        };

        (
            provisional_suggestions,
            provisional_error,
            provisional_doc_boosts,
        )
    }

    fn build_planned_payload(
        &self,
        build_context: PlannedPayloadBuildContext,
        parsed: ParsedLinkGraphQuery,
        rows: Vec<LinkGraphHit>,
        policy: LinkGraphPolicyDecision,
        provisional_suggestions: Vec<LinkGraphSuggestedLink>,
        provisional_error: Option<String>,
    ) -> LinkGraphPlannedSearchPayload {
        let hit_count = rows.len();
        let section_hit_count = rows
            .iter()
            .filter(|row| {
                row.best_section
                    .as_deref()
                    .map(str::trim)
                    .is_some_and(|value| !value.is_empty())
            })
            .count();
        let hits = rows
            .iter()
            .map(LinkGraphDisplayHit::from)
            .collect::<Vec<_>>();

        crate::link_graph::saliency::touch_search_hits_with_coactivation_async(
            &hits,
            &coactivated_neighbor_node_ids(self, &hits),
        );

        // Compute CCS audit before moving ownership
        let ccs_audit = Self::compute_ccs_audit(&parsed.options.style_anchors, &hits);

        LinkGraphPlannedSearchPayload {
            query: parsed.query,
            options: parsed.options,
            hits,
            hit_count,
            section_hit_count,
            requested_mode: policy.requested_mode,
            selected_mode: policy.selected_mode,
            reason: policy.reason,
            graph_hit_count: policy.graph_hit_count,
            source_hint_count: policy.source_hint_count,
            graph_confidence_score: policy.graph_confidence_score,
            graph_confidence_level: policy.graph_confidence_level,
            retrieval_plan: Some(policy.retrieval_plan),
            results: rows,
            provisional_suggestions,
            provisional_error,
            promoted_overlay: build_context.promoted_overlay,
            ccs_audit,
            semantic_ignition: None,
            julia_rerank: None,
            query_vector: build_context.query_vector_override,
            quantum_contexts: Vec::new(),
        }
    }

    fn compute_ccs_audit(
        style_anchors: &[String],
        hits: &[LinkGraphDisplayHit],
    ) -> Option<LinkGraphCcsAudit> {
        if style_anchors.is_empty() {
            return None;
        }

        // Extract evidence from search hits (titles, stems, sections)
        let evidence: Vec<String> = hits
            .iter()
            .flat_map(|hit| {
                let mut parts = vec![hit.title.clone(), hit.stem.clone()];
                if !hit.best_section.trim().is_empty() {
                    parts.push(hit.best_section.clone());
                }
                parts
            })
            .collect();

        let (ccs_score, passed, missing_anchors) =
            evaluate_ccs_audit(style_anchors, evidence.as_slice());

        // Build the CCS audit result for payload
        Some(LinkGraphCcsAudit {
            ccs_score,
            passed,
            compensated: false,
            missing_anchors,
        })
    }
}

#[cfg(feature = "zhenfa-router")]
fn evaluate_ccs_audit(anchors: &[String], evidence: &[String]) -> (f64, bool, Vec<String>) {
    use crate::zhenfa_router::{audit_search_payload, evaluate_alignment};

    let audit = audit_search_payload(evidence, anchors);
    let verdict = evaluate_alignment(anchors, evidence);
    (
        audit.ccs_score,
        audit.passed && verdict.is_aligned,
        audit.missing_anchors,
    )
}

#[cfg(not(feature = "zhenfa-router"))]
fn evaluate_ccs_audit(anchors: &[String], evidence: &[String]) -> (f64, bool, Vec<String>) {
    let missing_anchors = anchors
        .iter()
        .filter(|anchor| {
            let anchor_lower = anchor.to_lowercase();
            !evidence
                .iter()
                .any(|item| item.to_lowercase().contains(&anchor_lower))
        })
        .cloned()
        .collect::<Vec<_>>();
    let matches = anchors.len().saturating_sub(missing_anchors.len());

    let ccs_score = if anchors.is_empty() {
        1.0
    } else {
        usize_to_f64_saturating(matches) / usize_to_f64_saturating(anchors.len())
    };
    let is_aligned = (1.0 - ccs_score) < 0.05;
    let passed = ccs_score >= 0.70 && is_aligned;
    (ccs_score, passed, missing_anchors)
}

#[cfg(not(feature = "zhenfa-router"))]
fn usize_to_f64_saturating(value: usize) -> f64 {
    u32::try_from(value).map_or(f64::from(u32::MAX), f64::from)
}

fn coactivated_neighbor_node_ids(
    index: &LinkGraphIndex,
    hits: &[LinkGraphDisplayHit],
) -> Vec<crate::link_graph::saliency::SearchHitCoactivationLink> {
    use crate::link_graph::runtime_config::resolve_link_graph_coactivation_runtime;
    let runtime = resolve_link_graph_coactivation_runtime();
    if !runtime.enabled || runtime.max_neighbors_per_direction == 0 {
        return Vec::new();
    }

    let neighbor_limit = runtime.max_neighbors_per_direction.saturating_mul(2);
    hits.iter()
        .flat_map(|hit| {
            index
                .neighbors(&hit.stem, LinkGraphDirection::Outgoing, 1, neighbor_limit)
                .into_iter()
                .enumerate()
                .map(
                    |(rank, neighbor)| crate::link_graph::saliency::SearchHitCoactivationLink {
                        source_node_id: hit.stem.clone(),
                        neighbor_node_id: neighbor.stem,
                        pre_resolved_rank: rank,
                    },
                )
        })
        .collect()
}

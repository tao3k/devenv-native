//! Total structured candidate-index contract for `SearchStrategyFlow`.

use serde_json::{Value, json};

pub(crate) const STRUCTURED_INDEX_CANDIDATE_SOURCE: &str = "rust-structured-candidate-index";
pub(crate) const REGISTRY_METADATA_CANDIDATE_SOURCE: &str = "rust-registry-metadata";
pub(crate) const RUST_DUCKDB_STRUCTURED_INDEX_BACKEND: &str = "rust-duckdb-structured-index";
pub(crate) const NARROWED_CANDIDATE_BATCH_POLICY: &str = "narrowed-candidate-batch";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchStrategyFlowStructuredCandidateSurface {
    pub(crate) surface_id: &'static str,
    pub(crate) candidate_source: &'static str,
    pub(crate) candidate_count: usize,
    pub(crate) structured_surface_role: &'static str,
    pub(crate) rust_backend: &'static str,
    pub(crate) bridge_status: &'static str,
    pub(crate) julia_input_policy: &'static str,
    pub(crate) promotion_denominator: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct SearchStrategyFlowStructuredCandidateCounts {
    pub(crate) primary_markdown: usize,
    pub(crate) code_intelligence: usize,
    pub(crate) registry_authority: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchStrategyFlowStructuredCandidateIndexContract {
    pub(crate) surfaces: Vec<SearchStrategyFlowStructuredCandidateSurface>,
}

impl SearchStrategyFlowStructuredCandidateIndexContract {
    pub(crate) fn total_candidate_count(&self) -> usize {
        self.surfaces
            .iter()
            .map(|surface| surface.candidate_count)
            .sum()
    }

    pub(crate) fn pending_surface_count(&self) -> usize {
        self.surfaces
            .iter()
            .filter(|surface| surface.bridge_status.starts_with("pending-"))
            .count()
    }

    pub(crate) fn all_surfaces_share_rust_backend(&self) -> bool {
        self.surfaces
            .iter()
            .all(|surface| surface.rust_backend == RUST_DUCKDB_STRUCTURED_INDEX_BACKEND)
    }

    pub(crate) fn all_surfaces_use_narrowed_julia_batches(&self) -> bool {
        self.surfaces
            .iter()
            .all(|surface| surface.julia_input_policy == NARROWED_CANDIDATE_BATCH_POLICY)
    }

    pub(crate) fn all_surfaces_are_promotion_denominator(&self) -> bool {
        self.surfaces
            .iter()
            .all(|surface| surface.promotion_denominator)
    }
}

pub(crate) fn search_strategy_flow_total_structured_candidate_index_contract(
    counts: SearchStrategyFlowStructuredCandidateCounts,
) -> SearchStrategyFlowStructuredCandidateIndexContract {
    SearchStrategyFlowStructuredCandidateIndexContract {
        surfaces: vec![
            SearchStrategyFlowStructuredCandidateSurface {
                surface_id: "primary-markdown",
                candidate_source: super::types::MARKDOWN_HEADING_CANDIDATE_SOURCE,
                candidate_count: counts.primary_markdown,
                structured_surface_role: "primary-markdown-scenario",
                rust_backend: RUST_DUCKDB_STRUCTURED_INDEX_BACKEND,
                bridge_status: "measured-local-rust-trace",
                julia_input_policy: NARROWED_CANDIDATE_BATCH_POLICY,
                promotion_denominator: true,
            },
            SearchStrategyFlowStructuredCandidateSurface {
                surface_id: "code-intelligence-downlink",
                candidate_source: super::types::CODE_INTELLIGENCE_CANDIDATE_SOURCE,
                candidate_count: counts.code_intelligence,
                structured_surface_role: "code-intelligence-support-evidence",
                rust_backend: RUST_DUCKDB_STRUCTURED_INDEX_BACKEND,
                bridge_status: "measured-git-tracked-inventory",
                julia_input_policy: NARROWED_CANDIDATE_BATCH_POLICY,
                promotion_denominator: true,
            },
            SearchStrategyFlowStructuredCandidateSurface {
                surface_id: "registry-authority",
                candidate_source: REGISTRY_METADATA_CANDIDATE_SOURCE,
                candidate_count: counts.registry_authority,
                structured_surface_role: "registry-authority-index",
                rust_backend: RUST_DUCKDB_STRUCTURED_INDEX_BACKEND,
                bridge_status: "measured-registry-metadata-replay",
                julia_input_policy: NARROWED_CANDIDATE_BATCH_POLICY,
                promotion_denominator: true,
            },
        ],
    }
}

pub(crate) fn search_strategy_flow_total_structured_candidate_index_contract_json(
    counts: SearchStrategyFlowStructuredCandidateCounts,
    inventory_source: &str,
) -> Value {
    let contract = search_strategy_flow_total_structured_candidate_index_contract(counts);
    let total_candidate_count = contract.total_candidate_count();
    let surfaces = contract
        .surfaces
        .iter()
        .map(|surface| {
            json!({
                "surfaceId": surface.surface_id,
                "candidateSource": surface.candidate_source,
                "candidateCount": surface.candidate_count,
                "structuredSurfaceRole": surface.structured_surface_role,
                "rustBackend": surface.rust_backend,
                "bridgeStatus": surface.bridge_status,
                "juliaInputPolicy": surface.julia_input_policy,
                "promotionDenominator": surface.promotion_denominator,
            })
        })
        .collect::<Vec<_>>();
    json!({
        "contractSource": STRUCTURED_INDEX_CANDIDATE_SOURCE,
        "inventorySource": inventory_source,
        "totalCandidateCount": total_candidate_count,
        "pendingSurfaceCount": contract.pending_surface_count(),
        "rustBackend": RUST_DUCKDB_STRUCTURED_INDEX_BACKEND,
        "juliaInputPolicy": NARROWED_CANDIDATE_BATCH_POLICY,
        "allSurfacesShareRustBackend": contract.all_surfaces_share_rust_backend(),
        "allSurfacesUseNarrowedJuliaBatches": contract.all_surfaces_use_narrowed_julia_batches(),
        "allSurfacesArePromotionDenominator": contract.all_surfaces_are_promotion_denominator(),
        "surfaces": surfaces,
    })
}

pub(crate) fn search_strategy_flow_candidate_discovery_contract_json(
    counts: SearchStrategyFlowStructuredCandidateCounts,
    candidate_input_source: Option<&str>,
    candidate_input_count: usize,
    candidate_input_discovery: Option<&Value>,
) -> Value {
    let contract = search_strategy_flow_total_structured_candidate_index_contract(counts);
    let total_candidate_count = contract.total_candidate_count();
    let source = candidate_input_source.unwrap_or("unknown");
    let surface = contract
        .surfaces
        .iter()
        .find(|surface| surface.candidate_source == source);
    json!({
        "candidateInputSource": source,
        "candidateInputCount": candidate_input_count,
        "totalStructuredCandidateCount": total_candidate_count,
        "promotionDenominator": total_candidate_count,
        "selectionPolicy": NARROWED_CANDIDATE_BATCH_POLICY,
        "inputIsNarrowedBatch": candidate_input_count < total_candidate_count,
        "rustOwnsFullCandidateInventory": true,
        "juliaReceivesFullInventory": false,
        "structuredSurfaceId": surface.map(|surface| surface.surface_id),
        "structuredSurfaceCandidateCount": surface.map(|surface| surface.candidate_count),
        "structuredSurfaceRole": surface.map(|surface| surface.structured_surface_role),
        "inputSourceInStructuredContract": surface.is_some(),
        "discoveryReceipt": candidate_input_discovery.cloned().unwrap_or(Value::Null),
    })
}

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use anyhow::{Context, Result};

use super::{
    builder::{ReasoningPacketBuild, build_reasoning_packet_rows},
    types::{
        EpistemeOntologyStructuralFactsReasoningPacketExecutionFlags,
        EpistemeOntologyStructuralFactsReasoningPacketReport,
        EpistemeOntologyStructuralFactsReasoningPacketRequest,
        EpistemeOntologyStructuralFactsReasoningPacketSafetyFlags, ReasoningPacketOutputPaths,
        STRUCTURAL_FACTS_REASONING_PACKET_REPORT_SCHEMA_VERSION,
    },
    write::{write_json, write_packet_org, write_packet_tsv},
};

/// Compile structural facts rows into a deterministic Org reasoning packet.
///
/// # Errors
///
/// Returns an error when the structural facts artifact is missing, malformed,
/// internally inconsistent, filtered to zero rows, or output artifacts cannot
/// be written.
pub fn write_episteme_ontology_structural_facts_reasoning_packet(
    request: &EpistemeOntologyStructuralFactsReasoningPacketRequest,
    run_root: impl AsRef<Path>,
) -> Result<EpistemeOntologyStructuralFactsReasoningPacketReport> {
    let build = build_reasoning_packet_rows(request)?;
    let paths = ReasoningPacketOutputPaths::new(run_root.as_ref(), request.run_id.as_str());
    fs::create_dir_all(paths.run_dir.as_path())
        .with_context(|| format!("failed to create `{}`", paths.run_dir.display()))?;

    write_packet_tsv(paths.packet_tsv.as_path(), &build.rows)?;
    write_json(paths.packet_json.as_path(), &build.rows)?;
    let report = build_report(request, &paths, &build);
    write_packet_org(paths.packet_org.as_path(), &report, &build.rows)?;
    write_json(paths.report_json.as_path(), &report)?;
    Ok(report)
}

fn build_report(
    request: &EpistemeOntologyStructuralFactsReasoningPacketRequest,
    paths: &ReasoningPacketOutputPaths,
    build: &ReasoningPacketBuild,
) -> EpistemeOntologyStructuralFactsReasoningPacketReport {
    EpistemeOntologyStructuralFactsReasoningPacketReport {
        schema_version: STRUCTURAL_FACTS_REASONING_PACKET_REPORT_SCHEMA_VERSION,
        run_id: request.run_id.clone(),
        structural_facts_json: request.structural_facts_json.clone(),
        run_dir: paths.run_dir.clone(),
        reasoning_packet_tsv: paths.packet_tsv.clone(),
        reasoning_packet_json: paths.packet_json.clone(),
        reasoning_packet_org: paths.packet_org.clone(),
        reasoning_packet_report_json: paths.report_json.clone(),
        packet_row_count: build.rows.len(),
        selected_document_count: build
            .rows
            .iter()
            .map(|row| row.document_id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        skipped_by_filter_count: build.skipped_by_filter_count,
        skipped_by_limit_count: build.skipped_by_limit_count,
        category_counts: count_by(build.rows.iter().map(|row| row.category.as_str())),
        route_counts: count_by(build.rows.iter().map(|row| row.extraction_route.as_str())),
        execution: EpistemeOntologyStructuralFactsReasoningPacketExecutionFlags {
            source_text_read: false,
            llm_executed: false,
        },
        safety: EpistemeOntologyStructuralFactsReasoningPacketSafetyFlags {
            source_mutation_allowed: false,
            ontology_truth: false,
        },
    }
}

fn count_by<'a>(values: impl Iterator<Item = &'a str>) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for value in values {
        *counts.entry(value.to_string()).or_insert(0) += 1;
    }
    counts
}

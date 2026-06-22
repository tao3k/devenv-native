use std::{collections::BTreeSet, fs, path::Path};

use anyhow::{Context, Result};
use xiuxian_wendao_parsers::{EpistemeFileRow, parse_episteme_files_tsv};

use crate::source_contract::{read_source_manifest, source_contract_paths};

use super::{
    EpistemeOntologyCandidateGenerationRequest,
    identifiers::{sha256_text, source_revision, validate_run_id},
    io::read_to_string,
    mapping::extract_mapping_terms,
    model::{CacheEvidence, CacheOutputRow, CandidateGenerationInputs},
};

impl CandidateGenerationInputs {
    pub(super) fn load(request: &EpistemeOntologyCandidateGenerationRequest) -> Result<Self> {
        let paths = source_contract_paths(request.episteme_root.as_path())?;
        let manifest = read_source_manifest(request.episteme_root.as_path())?;
        let corpus_dir = paths.corpus_dir(request.episteme_root.as_path())?;
        let files_path = corpus_dir.join(&manifest.files);
        let files_raw = read_to_string(files_path.as_path())?;
        let files = parse_episteme_files_tsv(files_raw.as_str())
            .with_context(|| format!("failed to parse `{}`", files_path.display()))?;
        let mapping_ledger_path = paths.mapping_ledger_path(request.episteme_root.as_path());
        let mapping_ledger = read_to_string(mapping_ledger_path.as_path())?;
        let mapping_terms = extract_mapping_terms(mapping_ledger.as_str());
        let cache_evidence = read_cache_evidence_runs(request, &files)?;
        let source_revision = source_revision(
            &manifest.source_contract_id,
            &files,
            &mapping_ledger,
            &cache_evidence,
        );

        Ok(Self {
            run_id: request.run_id.clone(),
            domain: manifest.domain,
            primary_language: manifest.primary_language,
            source_manifest_path: paths.source_manifest_relative_path().to_string(),
            mapping_ledger_path: paths.mapping_ledger_relative_path().to_string(),
            files,
            mapping_terms,
            cache_evidence,
            source_revision,
        })
    }
}

fn read_cache_evidence_runs(
    request: &EpistemeOntologyCandidateGenerationRequest,
    files: &[EpistemeFileRow],
) -> Result<Vec<CacheEvidence>> {
    let file_ids = files
        .iter()
        .map(|file| file.file_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut rows = Vec::new();
    for run_id in &request.extraction_run_ids {
        validate_run_id(run_id)?;
        let outputs_dir = request.extraction_run_root.join(run_id).join("outputs");
        if !outputs_dir.is_dir() {
            continue;
        }
        collect_cache_evidence_for_run(outputs_dir.as_path(), run_id, &file_ids, &mut rows)?;
    }
    rows.sort_by(|left, right| {
        left.run_id
            .cmp(&right.run_id)
            .then_with(|| left.queue_id.cmp(&right.queue_id))
    });
    Ok(rows)
}

fn collect_cache_evidence_for_run(
    outputs_dir: &Path,
    run_id: &str,
    file_ids: &BTreeSet<&str>,
    rows: &mut Vec<CacheEvidence>,
) -> Result<()> {
    for entry in fs::read_dir(outputs_dir)
        .with_context(|| format!("failed to read `{}`", outputs_dir.display()))?
    {
        let entry = entry.with_context(|| format!("failed to read `{}`", outputs_dir.display()))?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let Some(row) = read_cache_output(path.as_path(), run_id)? else {
            continue;
        };
        if file_ids.contains(row.file_id.as_str()) {
            rows.push(row);
        }
    }
    Ok(())
}

fn read_cache_output(path: &Path, run_id: &str) -> Result<Option<CacheEvidence>> {
    let raw = read_to_string(path)?;
    let row = serde_json::from_str::<CacheOutputRow>(&raw)
        .with_context(|| format!("failed to parse `{}`", path.display()))?;
    if row.status != "succeeded" {
        return Ok(None);
    }
    if row.ontology_truth.unwrap_or(false) || row.raw_to_rdf_promotion_allowed.unwrap_or(false) {
        anyhow::bail!(
            "cache output `{}` must not be ontology truth or raw-to-RDF promotable",
            path.display()
        );
    }
    let extracted_text = row.extracted_text.unwrap_or_default();
    let text_sha256 = row
        .text_sha256
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| sha256_text(extracted_text.as_str()));
    let text_char_count = row
        .text_char_count
        .unwrap_or_else(|| extracted_text.chars().count());
    Ok(Some(CacheEvidence {
        run_id: run_id.to_string(),
        output_path: path.to_string_lossy().into_owned(),
        queue_id: row.queue_id,
        file_id: row.file_id,
        relative_path: row.relative_path,
        category: row.category,
        language: row.language,
        extraction_route: row.extraction_route,
        source_sha256: row.source_sha256,
        text_sha256,
        text_char_count,
        extracted_text,
    }))
}

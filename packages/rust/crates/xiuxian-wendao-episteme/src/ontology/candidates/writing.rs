use std::{io::Write, path::Path};

use anyhow::Result;

use super::{
    identifiers::{org_cell, org_uuid, tsv},
    io::create_file,
    model::{
        CandidateEvidenceRow, CandidateGenerationInputs, CandidateObjectRow, CandidateRelationRow,
        CandidateRows,
    },
};

pub(super) fn write_candidate_objects_tsv(path: &Path, rows: &[CandidateObjectRow]) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "candidate_id\tcandidate_kind\tstatus\tlabel\tsuggested_term_key\tsuggested_term_label\tsource_file_id\tsource_queue_id\tsource_path\tcategory\tlanguage\textraction_route\textraction_run_id\tsource_sha256\tevidence_sha256\ttext_char_count\treview_status\tpromotion_status\traw_to_rdf_promotion_allowed\tontology_truth"
    )?;
    for row in rows {
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            tsv(&row.candidate_id),
            row.candidate_kind,
            row.status,
            tsv(&row.label),
            tsv(&row.suggested_term_key),
            tsv(&row.suggested_term_label),
            tsv(&row.source_file_id),
            tsv(&row.source_queue_id),
            tsv(&row.source_path),
            tsv(&row.category),
            tsv(&row.language),
            tsv(&row.extraction_route),
            tsv(&row.extraction_run_id),
            tsv(&row.source_sha256),
            tsv(&row.evidence_sha256),
            tsv(&row.text_char_count),
            row.review_status,
            row.promotion_status,
            row.raw_to_rdf_promotion_allowed,
            row.ontology_truth
        )?;
    }
    Ok(())
}

pub(super) fn write_candidate_relations_tsv(
    path: &Path,
    rows: &[CandidateRelationRow],
) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "candidate_id\trelation_kind\tsource_candidate_id\ttarget_candidate_id\tsource_file_id\tsource_queue_id\textraction_run_id\tevidence_sha256\treview_status\tpromotion_status\tontology_truth"
    )?;
    for row in rows {
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            tsv(&row.candidate_id),
            row.relation_kind,
            tsv(&row.source_candidate_id),
            tsv(&row.target_candidate_id),
            tsv(&row.source_file_id),
            tsv(&row.source_queue_id),
            tsv(&row.extraction_run_id),
            tsv(&row.evidence_sha256),
            row.review_status,
            row.promotion_status,
            row.ontology_truth
        )?;
    }
    Ok(())
}

pub(super) fn write_candidate_evidence_tsv(
    path: &Path,
    rows: &[CandidateEvidenceRow],
) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "evidence_id\tevidence_kind\tsource_file_id\tsource_queue_id\tsource_path\tsource_sha256\textraction_run_id\tcache_output_path\tevidence_sha256\ttext_char_count\treview_status\tpromotion_status\tontology_truth"
    )?;
    for row in rows {
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            tsv(&row.evidence_id),
            row.evidence_kind,
            tsv(&row.source_file_id),
            tsv(&row.source_queue_id),
            tsv(&row.source_path),
            tsv(&row.source_sha256),
            tsv(&row.extraction_run_id),
            tsv(&row.cache_output_path),
            tsv(&row.evidence_sha256),
            tsv(&row.text_char_count),
            row.review_status,
            row.promotion_status,
            row.ontology_truth
        )?;
    }
    Ok(())
}

pub(super) fn write_review_ledger_org(
    path: &Path,
    inputs: &CandidateGenerationInputs,
    rows: &CandidateRows,
) -> Result<()> {
    let mut file = create_file(path)?;
    write_review_header(&mut file, inputs)?;
    write_source_contract_section(&mut file, inputs)?;
    write_candidate_summary_section(&mut file, inputs, rows)?;
    write_review_policy_section(&mut file)?;
    Ok(())
}

fn write_review_header(file: &mut impl Write, inputs: &CandidateGenerationInputs) -> Result<()> {
    writeln!(file, "#+TITLE: Ontology Candidate Review Ledger")?;
    writeln!(
        file,
        "#+PROPERTY: WENDAO_ONTOLOGY_GENERATION_RUN {}",
        inputs.run_id
    )?;
    writeln!(file)?;
    writeln!(file, "* Ontology candidate generation review")?;
    writeln!(file, ":PROPERTIES:")?;
    writeln!(file, ":ID: {}", org_uuid(inputs.run_id.as_str()))?;
    writeln!(file, ":WENDAO_KIND: ontology_mapping")?;
    writeln!(file, ":ONTOLOGY_KIND: corpus_mapping")?;
    writeln!(file, ":DOMAIN: {}", inputs.domain)?;
    writeln!(file, ":PROMOTION_STATE: candidate")?;
    writeln!(file, ":LIFECYCLE_STATE: candidate")?;
    writeln!(file, ":PRIMARY_LANGUAGE: {}", inputs.primary_language)?;
    writeln!(file, ":END:")?;
    writeln!(file)?;
    writeln!(
        file,
        "本账本由 Rust Episteme materializer 生成，用于审查私有语料的 ontology candidate。"
    )?;
    writeln!(
        file,
        "当前输出只包含候选对象、候选关系和证据哈希，不把原始文本或抽取文本直接提升为 RDF truth。"
    )?;
    writeln!(file)?;
    Ok(())
}

fn write_source_contract_section(
    file: &mut impl Write,
    inputs: &CandidateGenerationInputs,
) -> Result<()> {
    writeln!(file, "** Source contract")?;
    writeln!(file)?;
    writeln!(file, "| Field | Value |")?;
    writeln!(file, "| domain | {} |", org_cell(inputs.domain.as_str()))?;
    writeln!(
        file,
        "| source manifest | {} |",
        org_cell(inputs.source_manifest_path.as_str())
    )?;
    writeln!(
        file,
        "| mapping ledger | {} |",
        org_cell(inputs.mapping_ledger_path.as_str())
    )?;
    writeln!(
        file,
        "| source revision | {} |",
        org_cell(inputs.source_revision.as_str())
    )?;
    writeln!(file)?;
    Ok(())
}

fn write_candidate_summary_section(
    file: &mut impl Write,
    inputs: &CandidateGenerationInputs,
    rows: &CandidateRows,
) -> Result<()> {
    writeln!(file, "** Candidate summary")?;
    writeln!(file)?;
    writeln!(file, "| Metric | Count |")?;
    writeln!(file, "| source files | {} |", inputs.files.len())?;
    writeln!(file, "| mapping terms | {} |", inputs.mapping_terms.len())?;
    writeln!(
        file,
        "| extraction evidence rows | {} |",
        inputs.cache_evidence.len()
    )?;
    writeln!(file, "| candidate objects | {} |", rows.objects.len())?;
    writeln!(file, "| candidate relations | {} |", rows.relations.len())?;
    writeln!(
        file,
        "| candidate evidence rows | {} |",
        rows.evidence.len()
    )?;
    writeln!(file)?;
    Ok(())
}

fn write_review_policy_section(file: &mut impl Write) -> Result<()> {
    writeln!(file, "** Review policy")?;
    writeln!(file)?;
    writeln!(file, "| Decision | Status | Reason |")?;
    writeln!(
        file,
        "| Raw rows are evidence only | accepted | Extension source rows and cache text are not ontology truth |"
    )?;
    writeln!(
        file,
        "| Candidate TSV requires review | accepted | Generated classes and links are suggestions until promoted |"
    )?;
    writeln!(
        file,
        "| Direct RDF mutation | rejected | Promotion must be a later controlled review step |"
    )?;
    Ok(())
}

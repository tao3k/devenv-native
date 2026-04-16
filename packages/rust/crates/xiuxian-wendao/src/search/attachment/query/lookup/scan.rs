use std::collections::HashSet;

use xiuxian_vector_store::EngineRecordBatch;

use crate::duckdb::ParquetQueryEngine;
use crate::search::attachment::schema::{
    attachment_ext_column, attachment_name_column, attachment_name_folded_column, id_column,
    kind_column, projected_columns,
};
use crate::search::ranking::{StreamingRerankSource, StreamingRerankTelemetry, trim_ranked_vec};

use super::helpers::{sql_identifier, sql_string_literal, string_column};
use super::scoring::{candidate_score, compare_candidates};
use super::types::{
    AttachmentCandidate, AttachmentCandidateQuery, AttachmentSearchError, AttachmentSearchExecution,
};

pub(crate) async fn execute_attachment_search(
    engine: &ParquetQueryEngine,
    table_name: &str,
    candidate_query: &AttachmentCandidateQuery<'_>,
) -> Result<AttachmentSearchExecution, AttachmentSearchError> {
    let sql = build_attachment_stage1_sql(
        table_name,
        candidate_query.extensions,
        candidate_query.kinds,
    );
    let batches = engine.query_batches(sql.as_str()).await?;
    let mut telemetry = StreamingRerankTelemetry::new(candidate_query.window, None, None);
    let mut candidates = Vec::with_capacity(candidate_query.window.target);

    for batch in batches {
        collect_candidates(&batch, candidate_query, &mut candidates, &mut telemetry)?;
    }

    Ok(AttachmentSearchExecution {
        candidates,
        telemetry,
        source: StreamingRerankSource::Scan,
    })
}

pub(crate) fn build_attachment_stage1_sql(
    table_name: &str,
    normalized_extensions: &HashSet<String>,
    normalized_kinds: &HashSet<String>,
) -> String {
    let projections = projected_columns()
        .into_iter()
        .map(sql_identifier)
        .collect::<Vec<_>>()
        .join(", ");
    let quoted_table_name = sql_identifier(table_name);
    match filter_expression(normalized_extensions, normalized_kinds) {
        Some(filter) => format!("SELECT {projections} FROM {quoted_table_name} WHERE {filter}"),
        None => format!("SELECT {projections} FROM {quoted_table_name}"),
    }
}

fn collect_candidates(
    batch: &EngineRecordBatch,
    query: &AttachmentCandidateQuery<'_>,
    candidates: &mut Vec<AttachmentCandidate>,
    telemetry: &mut StreamingRerankTelemetry,
) -> Result<(), AttachmentSearchError> {
    telemetry.observe_batch(batch.num_rows());
    let id = string_column(batch, id_column())?;
    let source_path = string_column(batch, "source_path")?;
    let source_title = string_column(batch, "source_title")?;
    let source_stem = string_column(batch, "source_stem")?;
    let attachment_path = string_column(batch, "attachment_path")?;
    let attachment_name = string_column(batch, attachment_name_column())?;
    let source_path_folded = string_column(batch, "source_path_folded")?;
    let source_title_folded = string_column(batch, "source_title_folded")?;
    let source_stem_folded = string_column(batch, "source_stem_folded")?;
    let attachment_path_folded = string_column(batch, "attachment_path_folded")?;
    let attachment_name_folded = string_column(batch, attachment_name_folded_column())?;

    for row in 0..batch.num_rows() {
        let fields = if query.case_sensitive {
            [
                attachment_path.value(row),
                attachment_name.value(row),
                source_path.value(row),
                source_title.value(row),
                source_stem.value(row),
            ]
        } else {
            [
                attachment_path_folded.value(row),
                attachment_name_folded.value(row),
                source_path_folded.value(row),
                source_title_folded.value(row),
                source_stem_folded.value(row),
            ]
        };
        let score = candidate_score(query.normalized_query, query.query_tokens, &fields);
        if score <= 0.0 {
            continue;
        }

        telemetry.observe_match();
        candidates.push(AttachmentCandidate {
            id: id.value(row).to_string(),
            score,
            source_path: source_path.value(row).to_string(),
            attachment_path: attachment_path.value(row).to_string(),
        });
        telemetry.observe_working_set(candidates.len());
        if candidates.len() > query.window.threshold {
            let before_len = candidates.len();
            trim_ranked_vec(candidates, query.window.target, compare_candidates);
            telemetry.observe_trim(before_len, candidates.len());
        }
    }

    Ok(())
}

fn filter_expression(extensions: &HashSet<String>, kinds: &HashSet<String>) -> Option<String> {
    let extension_clause = disjunction(attachment_ext_column(), extensions);
    let kind_clause = disjunction(kind_column(), kinds);
    match (extension_clause, kind_clause) {
        (Some(left), Some(right)) => Some(format!("({left}) AND ({right})")),
        (Some(clause), None) | (None, Some(clause)) => Some(clause),
        (None, None) => None,
    }
}

fn disjunction(column: &str, values: &HashSet<String>) -> Option<String> {
    if values.is_empty() {
        return None;
    }

    let mut sorted = values.iter().cloned().collect::<Vec<_>>();
    sorted.sort_unstable();
    Some(format!(
        "{} IN ({})",
        sql_identifier(column),
        sorted
            .into_iter()
            .map(|value| sql_string_literal(value.as_str()))
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

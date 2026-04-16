use std::collections::HashSet;

use xiuxian_vector_store::EngineRecordBatch;

use crate::analyzers::{
    example_match_score, import_match_score, module_match_score, normalized_rank_score,
    symbol_match_score,
};
use crate::duckdb::ParquetQueryEngine;
use crate::search::ranking::{
    RetainedWindow, StreamingRerankSource, StreamingRerankTelemetry, trim_ranked_vec,
};
use crate::search::repo_entity::query::hydrate::{engine_float64_column, engine_string_column};
use crate::search::repo_entity::query::lookup::types::{
    EXAMPLE_BUCKETS, IMPORT_BUCKETS, MIN_RECALL_CANDIDATES, MODULE_BUCKETS, RECALL_TRIM_MULTIPLIER,
    RepoEntityCandidate, RepoEntityQuery, RepoEntitySearchError, RepoEntitySearchExecution,
    SYMBOL_BUCKETS,
};
use crate::search::repo_entity::schema::projected_columns;

pub(crate) async fn execute_repo_entity_search(
    query_engine: &ParquetQueryEngine,
    table_name: &str,
    query: &RepoEntityQuery<'_>,
) -> Result<RepoEntitySearchExecution, RepoEntitySearchError> {
    let sql = build_repo_entity_stage1_sql(table_name, query.language_filters, query.kind_filters);
    let batches = query_engine.query_batches(sql.as_str()).await?;
    let mut telemetry = StreamingRerankTelemetry::new(query.window, None, None);
    let mut candidates = Vec::with_capacity(query.window.target);

    for batch in batches {
        collect_candidates(&batch, query, &mut candidates, &mut telemetry)?;
    }

    Ok(RepoEntitySearchExecution {
        candidates,
        telemetry,
        source: StreamingRerankSource::Scan,
    })
}

fn collect_candidates(
    batch: &EngineRecordBatch,
    query: &RepoEntityQuery<'_>,
    candidates: &mut Vec<RepoEntityCandidate>,
    telemetry: &mut StreamingRerankTelemetry,
) -> Result<(), RepoEntitySearchError> {
    telemetry.observe_batch(batch.num_rows());
    let id = engine_string_column(batch, "id")?;
    let entity_kind = engine_string_column(batch, "entity_kind")?;
    let name = engine_string_column(batch, "name")?;
    let name_folded = engine_string_column(batch, "name_folded")?;
    let qualified_name_folded = engine_string_column(batch, "qualified_name_folded")?;
    let path = engine_string_column(batch, "path")?;
    let path_folded = engine_string_column(batch, "path_folded")?;
    let language = engine_string_column(batch, "language")?;
    let symbol_kind = engine_string_column(batch, "symbol_kind")?;
    let signature_folded = engine_string_column(batch, "signature_folded")?;
    let summary_folded = engine_string_column(batch, "summary_folded")?;
    let related_symbols_folded = engine_string_column(batch, "related_symbols_folded")?;
    let related_modules_folded = engine_string_column(batch, "related_modules_folded")?;
    let saliency_score = engine_float64_column(batch, "saliency_score")?;

    for row in 0..batch.num_rows() {
        let entity_kind_value = entity_kind.value(row);
        let language_value = language.value(row);
        let symbol_kind_value = symbol_kind.value(row);
        if !matches_language_filters(query.language_filters, language_value) {
            continue;
        }
        if !matches_kind_filters(query.kind_filters, entity_kind_value, symbol_kind_value) {
            continue;
        }

        let Some(normalized) = candidate_score(&CandidateScoreInputs {
            query_lower: query.query_lower,
            entity_kind: entity_kind_value,
            name_folded: name_folded.value(row),
            qualified_name_folded: qualified_name_folded.value(row),
            path_folded: path_folded.value(row),
            signature_folded: signature_folded.value(row),
            summary_folded: summary_folded.value(row),
            related_symbols_folded: related_symbols_folded.value(row),
            related_modules_folded: related_modules_folded.value(row),
            import_package_filter: query.import_package_filter,
            import_module_filter: query.import_module_filter,
        }) else {
            continue;
        };

        telemetry.observe_match();
        let score = normalized.max(saliency_score.value(row)).clamp(0.0, 1.0);
        candidates.push(RepoEntityCandidate {
            id: id.value(row).to_string(),
            score,
            entity_kind: entity_kind_value.to_string(),
            name: name.value(row).to_string(),
            path: path.value(row).to_string(),
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

struct CandidateScoreInputs<'a> {
    query_lower: &'a str,
    entity_kind: &'a str,
    name_folded: &'a str,
    qualified_name_folded: &'a str,
    path_folded: &'a str,
    signature_folded: &'a str,
    summary_folded: &'a str,
    related_symbols_folded: &'a str,
    related_modules_folded: &'a str,
    import_package_filter: Option<&'a str>,
    import_module_filter: Option<&'a str>,
}

fn build_repo_entity_stage1_sql(
    table_name: &str,
    language_filters: &HashSet<String>,
    kind_filters: &HashSet<String>,
) -> String {
    let projections = projected_columns().join(", ");
    let filters = [
        language_filter_expression(language_filters),
        kind_filter_expression(kind_filters),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    if filters.is_empty() {
        return format!("SELECT {projections} FROM {table_name}");
    }

    format!(
        "SELECT {projections} FROM {table_name} WHERE {}",
        filters.join(" AND ")
    )
}

fn language_filter_expression(language_filters: &HashSet<String>) -> Option<String> {
    if language_filters.is_empty() {
        return None;
    }

    let mut values = language_filters.iter().cloned().collect::<Vec<_>>();
    values.sort_unstable();
    Some(format!(
        "language IN ({})",
        values
            .into_iter()
            .map(|value| sql_string_literal(value.as_str()))
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

fn kind_filter_expression(kind_filters: &HashSet<String>) -> Option<String> {
    if kind_filters.is_empty() {
        return None;
    }

    let mut filters = kind_filters.iter().cloned().collect::<Vec<_>>();
    filters.sort_unstable();
    let clauses = filters
        .into_iter()
        .map(|value| match value.as_str() {
            "module" => "entity_kind = 'module'".to_string(),
            "example" => "entity_kind = 'example'".to_string(),
            "import" => "entity_kind = 'import'".to_string(),
            "symbol" => "entity_kind = 'symbol'".to_string(),
            other => format!(
                "(entity_kind = 'symbol' AND symbol_kind = {})",
                sql_string_literal(other)
            ),
        })
        .collect::<Vec<_>>();
    Some(format!("({})", clauses.join(" OR ")))
}

fn sql_string_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

pub(crate) fn retained_window(limit: usize) -> RetainedWindow {
    RetainedWindow::new(limit, RECALL_TRIM_MULTIPLIER, MIN_RECALL_CANDIDATES)
}

pub(crate) fn fixed_kind_filters(kind: &str) -> HashSet<String> {
    HashSet::from([kind.to_string()])
}

pub(crate) fn compare_candidates(
    left: &RepoEntityCandidate,
    right: &RepoEntityCandidate,
) -> std::cmp::Ordering {
    right
        .score
        .partial_cmp(&left.score)
        .unwrap_or(std::cmp::Ordering::Equal)
        .then_with(|| {
            candidate_kind_priority(right.entity_kind.as_str())
                .cmp(&candidate_kind_priority(left.entity_kind.as_str()))
        })
        .then_with(|| left.path.cmp(&right.path))
        .then_with(|| left.name.cmp(&right.name))
}

fn candidate_score(inputs: &CandidateScoreInputs<'_>) -> Option<f64> {
    match inputs.entity_kind {
        "module" => module_match_score(
            inputs.query_lower,
            inputs.qualified_name_folded,
            inputs.path_folded,
        )
        .map(|score| normalized_rank_score(score, MODULE_BUCKETS)),
        "symbol" => symbol_match_score(
            inputs.query_lower,
            inputs.name_folded,
            inputs.qualified_name_folded,
            inputs.path_folded,
            inputs.signature_folded,
        )
        .map(|score| normalized_rank_score(score, SYMBOL_BUCKETS)),
        "example" => {
            let related_symbols = split_folded_values(inputs.related_symbols_folded);
            let related_modules = split_folded_values(inputs.related_modules_folded);
            example_match_score(
                inputs.query_lower,
                inputs.name_folded,
                inputs.path_folded,
                inputs.summary_folded,
                related_symbols.as_slice(),
                related_modules.as_slice(),
            )
            .map(|score| normalized_rank_score(score, EXAMPLE_BUCKETS))
        }
        "import" => {
            let import = crate::analyzers::ImportRecord {
                repo_id: String::new(),
                module_id: String::new(),
                path: String::new(),
                import_name: inputs.name_folded.to_string(),
                target_package: inputs.summary_folded.to_string(),
                source_module: inputs.signature_folded.to_string(),
                kind: crate::analyzers::ImportKind::Symbol,
                line_start: None,
                resolved_id: None,
                attributes: std::collections::BTreeMap::new(),
            };
            import_match_score(
                inputs.import_package_filter,
                inputs.import_module_filter,
                &import,
            )
            .map(|score| normalized_rank_score(score, IMPORT_BUCKETS))
        }
        _ => None,
    }
}

fn split_folded_values(value: &str) -> Vec<String> {
    value
        .split('\n')
        .map(str::trim)
        .filter(|candidate| !candidate.is_empty())
        .map(str::to_string)
        .collect()
}

fn matches_language_filters(filters: &HashSet<String>, language: &str) -> bool {
    filters.is_empty() || filters.contains(language)
}

fn matches_kind_filters(
    kind_filters: &HashSet<String>,
    entity_kind: &str,
    symbol_kind: &str,
) -> bool {
    if kind_filters.is_empty() {
        return true;
    }

    match entity_kind {
        "symbol" => {
            kind_filters.contains("symbol")
                || (!symbol_kind.is_empty() && kind_filters.contains(symbol_kind))
        }
        "module" => kind_filters.contains("module"),
        "example" => kind_filters.contains("example"),
        "import" => kind_filters.contains("import"),
        _ => false,
    }
}

fn candidate_kind_priority(entity_kind: &str) -> u8 {
    match entity_kind {
        "symbol" => 3,
        "module" => 2,
        "example" | "import" => 1,
        _ => 0,
    }
}

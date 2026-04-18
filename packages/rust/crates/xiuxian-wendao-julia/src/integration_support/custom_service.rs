use std::fs;
use std::path::{Component, PathBuf};
use std::process::{Command, Stdio};

use super::common::{
    JuliaExampleServiceGuard, project_cache_dir, repo_root, reserve_service_port,
    wait_for_service_ready, wendaoarrow_script,
};

/// Row-level score override for the custom Julia rerank service.
pub struct WendaoArrowScoreRow<'a> {
    /// Stable document identifier expected by the rerank response.
    pub doc_id: &'a str,
    /// Analyzer score returned for the row.
    pub analyzer_score: f64,
    /// Final score returned for the row.
    pub final_score: f64,
}

const WENDAOARROW_CUSTOM_SERVICE_CONFIG_TOML: &str =
    include_str!("../../resources/integration_support/wendaoarrow_custom_service.toml");

/// Spawns a custom `WendaoArrow` scoring service from inline score rows.
///
/// # Panics
///
/// Panics when the generated Julia script cannot be written, the child process
/// cannot be spawned, or the service never becomes ready.
pub async fn spawn_wendaoarrow_custom_scoring_service(
    rows: &[WendaoArrowScoreRow<'_>],
) -> (String, JuliaExampleServiceGuard) {
    let port = reserve_service_port();
    let base_url = format!("http://127.0.0.1:{port}");
    let generated_dir = project_cache_dir().join(custom_service_cache_relative_dir());
    fs::create_dir_all(&generated_dir)
        .unwrap_or_else(|error| panic!("create WendaoArrow custom scoring cache dir: {error}"));
    let generated_script = generated_dir.join(format!("custom_scoring_flight_server_{port}.jl"));
    fs::write(&generated_script, processor_script(rows)).unwrap_or_else(|error| {
        panic!("write generated WendaoArrow custom scoring script: {error}")
    });

    let child = Command::new("julia")
        .arg(wendaoarrow_script("run_flight_example.jl"))
        .arg(&generated_script)
        .arg("--port")
        .arg(port.to_string())
        .current_dir(repo_root())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|error| panic!("spawn WendaoArrow custom scoring service: {error}"));
    let mut guard = JuliaExampleServiceGuard::new(child);

    wait_for_service_ready(base_url.as_str())
        .await
        .unwrap_or_else(|error| {
            guard.kill();
            panic!("wait for Julia custom scoring service readiness: {error}");
        });

    (base_url, guard)
}

fn custom_service_cache_relative_dir() -> PathBuf {
    let parsed: toml::Value = toml::from_str(WENDAOARROW_CUSTOM_SERVICE_CONFIG_TOML)
        .unwrap_or_else(|error| panic!("parse WendaoArrow custom service config TOML: {error}"));
    let raw_relative_dir = parsed
        .get("cache")
        .and_then(toml::Value::as_table)
        .and_then(|cache| cache.get("relative_dir"))
        .and_then(toml::Value::as_str)
        .map_or_else(
            || panic!("WendaoArrow custom service config must define `cache.relative_dir`"),
            str::trim,
        );
    assert!(
        !raw_relative_dir.is_empty(),
        "WendaoArrow custom service config `cache.relative_dir` must not be blank"
    );

    let relative_dir = PathBuf::from(raw_relative_dir);
    if relative_dir.is_absolute()
        || relative_dir.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        panic!(
            "WendaoArrow custom service config `cache.relative_dir` must stay relative to PRJ_CACHE_HOME, got `{}`",
            relative_dir.display()
        );
    }

    relative_dir
}

fn processor_script(rows: &[WendaoArrowScoreRow<'_>]) -> String {
    let mut mappings = String::new();
    for row in rows {
        mappings.push_str(
            format!(
                "\"{}\" => ({}, {}),\n",
                row.doc_id, row.analyzer_score, row.final_score
            )
            .as_str(),
        );
    }

    format!(
        r#"
using WendaoArrow
using Tables

const SCORE_MAP = Dict(
{mappings}
)

function processor(stream)
    doc_ids = String[]
    analyzer_scores = Float64[]
    final_scores = Float64[]
    seen_doc_ids = Dict{{String, Int}}()
    row_offset = 0

    for batch in stream
        WendaoArrow.require_columns(
            batch,
            ("doc_id", "vector_score");
            subject = "custom Julia rerank request",
        )
        row_count = WendaoArrow.require_column_lengths(
            batch,
            ("doc_id", "vector_score");
            subject = "custom Julia rerank request",
        )
        WendaoArrow.require_unique_string_column(
            batch,
            "doc_id";
            subject = "custom Julia rerank request",
            seen = seen_doc_ids,
            row_offset = row_offset,
        )

        columns = Tables.columntable(batch)
        sizehint!(doc_ids, length(doc_ids) + row_count)
        sizehint!(analyzer_scores, length(analyzer_scores) + row_count)
        sizehint!(final_scores, length(final_scores) + row_count)

        for (row_index, (raw_doc_id, raw_vector_score)) in enumerate(zip(columns.doc_id, columns.vector_score))
            doc_id = WendaoArrow.coerce_string(
                raw_doc_id;
                column = "doc_id",
                subject = "custom Julia rerank request",
                row_index = row_index,
            )
            WendaoArrow.coerce_float64(
                raw_vector_score;
                column = "vector_score",
                subject = "custom Julia rerank request",
                row_index = row_index,
            )
            analyzer_score, final_score = get(SCORE_MAP, doc_id, (0.0, 0.0))
            push!(doc_ids, doc_id)
            push!(analyzer_scores, analyzer_score)
            push!(final_scores, final_score)
        end

        row_offset += row_count
    end

    return WendaoArrow.normalize_scoring_response((
        doc_id = doc_ids,
        analyzer_score = analyzer_scores,
        final_score = final_scores,
    ); subject = "custom Julia rerank response")
end

config = WendaoArrow.config_from_args(ARGS)

WendaoArrow.serve_stream_flight(
    processor;
    descriptor = WendaoArrow.flight_descriptor(("rerank",)),
    host=config.host,
    port=config.port,
)
"#
    )
}

#[cfg(test)]
#[path = "../../tests/unit/integration_support/custom_service.rs"]
mod tests;

use std::fs;
use std::path::Path;

use tempfile::tempdir;
#[cfg(feature = "duckdb")]
use xiuxian_wendao_runtime::config::{
    DEFAULT_SEARCH_DUCKDB_MATERIALIZE_THRESHOLD_ROWS, DEFAULT_SEARCH_DUCKDB_PREFER_VIRTUAL_ARROW,
};
use xiuxian_wendao_sql::DataFusionLocalRelationEngine;

#[cfg(feature = "duckdb")]
use super::query_bounded_work_markdown_payload_with_engine;
use super::{
    BOUNDED_WORK_MARKDOWN_TABLE_NAME, bootstrap_bounded_work_markdown_query_engine,
    build_bounded_work_markdown_rows, query_bounded_work_markdown_payload,
    register_bounded_work_markdown_table,
};
#[cfg(feature = "duckdb")]
use crate::duckdb::{
    DuckDbDatabasePath, DuckDbLocalRelationEngine, DuckDbRegistrationStrategy,
    SearchDuckDbRuntimeConfig,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn write_bounded_work_fixture(
    root: &Path,
    blueprint_file: &str,
    blueprint_body: &str,
    plan_file: &str,
    plan_body: &str,
) -> std::io::Result<()> {
    fs::create_dir_all(root.join("blueprint"))?;
    fs::create_dir_all(root.join("plan"))?;
    fs::write(root.join(blueprint_file), blueprint_body)?;
    fs::write(root.join(plan_file), plan_body)?;
    Ok(())
}

#[tokio::test]
async fn registers_bounded_work_markdown_rows_into_sql_surface() -> TestResult {
    let temp_dir = tempdir()?;
    let root = temp_dir.path();
    write_bounded_work_fixture(
        root,
        "blueprint/blueprint.md",
        "# Blueprint\n\n:PROPERTIES:\n:OWNER: Codex\n:END:\n\n## Boundary\n- [ ] Keep scope bounded\n",
        "plan/tasks.md",
        "# Plan\n\n## Implement\n1. Build rows\n- [ ] Add test\n",
    )?;

    let rows = build_bounded_work_markdown_rows(root).map_err(std::io::Error::other)?;
    assert!(
        rows.iter().any(|row| {
            row.path == "blueprint/blueprint.md"
                && row.heading_path.is_empty()
                && row.title == "Blueprint"
        }),
        "expected a document root row titled from the parsed markdown document"
    );
    assert!(
        rows.iter().any(|row| {
            row.path == "blueprint/blueprint.md"
                && row.surface == "blueprint"
                && row.heading_path == "Blueprint/Boundary"
        }),
        "expected a normalized heading_path row for blueprint boundary"
    );
    assert!(
        rows.iter().any(|row| {
            row.path == "plan/tasks.md"
                && row.surface == "plan"
                && row.heading_path == "Plan/Implement"
        }),
        "expected a normalized heading_path row for plan implement section"
    );

    let query_engine = DataFusionLocalRelationEngine::new_with_information_schema();
    let registered_rows =
        register_bounded_work_markdown_table(&query_engine, root).map_err(std::io::Error::other)?;
    assert_eq!(registered_rows.len(), rows.len());

    let batches = query_engine
        .session()
        .sql(
            "select path, surface, heading_path, title, level, skeleton \
             from markdown order by path, heading_path",
        )
        .await?
        .collect()
        .await?;
    let rendered = format!("{batches:?}");
    assert!(rendered.contains("blueprint/blueprint.md"));
    assert!(rendered.contains("Blueprint/Boundary"));
    assert!(rendered.contains("plan/tasks.md"));
    assert!(rendered.contains("Plan/Implement"));

    let skeleton_batches = query_engine
        .session()
        .sql(
            "select skeleton from markdown \
             where path = 'plan/tasks.md' and heading_path = 'Plan/Implement'",
        )
        .await?
        .collect()
        .await?;
    let skeleton_rendered = format!("{skeleton_batches:?}");
    assert!(skeleton_rendered.contains("## Implement"));
    assert!(skeleton_rendered.contains("1. Build rows"));
    assert!(skeleton_rendered.contains("- [ ] Add test"));

    let table_batches = query_engine
        .session()
        .sql(&format!(
            "select count(*) as row_count from {BOUNDED_WORK_MARKDOWN_TABLE_NAME}"
        ))
        .await?
        .collect()
        .await?;
    assert!(format!("{table_batches:?}").contains("row_count"));
    Ok(())
}

#[tokio::test]
async fn bootstraps_bounded_work_markdown_query_engine() -> TestResult {
    let temp_dir = tempdir()?;
    let root = temp_dir.path();
    write_bounded_work_fixture(
        root,
        "blueprint/overview.md",
        "# Blueprint\n\n## Scope\n- [ ] Keep aligned\n",
        "plan/steps.md",
        "# Plan\n\n## Validate\n- [ ] Query markdown\n",
    )?;

    let (query_engine, rows) =
        bootstrap_bounded_work_markdown_query_engine(root).map_err(std::io::Error::other)?;
    assert!(
        !rows.is_empty(),
        "expected bounded-work bootstrap to register markdown rows"
    );

    let batches = query_engine
        .session()
        .sql(
            "select path, heading_path from markdown \
             where surface = 'plan' order by path, heading_path",
        )
        .await?
        .collect()
        .await?;
    let rendered = format!("{batches:?}");
    assert!(rendered.contains("plan/steps.md"));
    assert!(rendered.contains("Plan/Validate"));
    Ok(())
}

#[tokio::test]
async fn queries_bounded_work_markdown_payload() -> TestResult {
    let temp_dir = tempdir()?;
    let root = temp_dir.path();
    write_bounded_work_fixture(
        root,
        "blueprint/overview.md",
        "# Blueprint\n\n## Scope\n- [ ] Keep aligned\n",
        "plan/steps.md",
        "# Plan\n\n## Validate\n- [ ] Query markdown\n",
    )?;

    let payload = query_bounded_work_markdown_payload(
        root,
        "select path, heading_path from markdown where surface = 'blueprint' order by path, heading_path",
    )
    .await
    .map_err(std::io::Error::other)?;

    assert_eq!(
        payload.metadata.registered_tables,
        vec!["markdown".to_string()]
    );
    assert_eq!(payload.metadata.registered_table_count, 1);
    assert_eq!(payload.metadata.registered_column_count, 7);
    assert!(
        payload
            .metadata
            .registered_input_bytes
            .is_some_and(|bytes| bytes > 0)
    );
    assert!(payload.metadata.result_bytes.is_some_and(|bytes| bytes > 0));
    assert_eq!(
        payload
            .metadata
            .local_relation_materialization_state
            .as_deref(),
        Some("materialized")
    );
    assert_eq!(payload.metadata.local_temp_storage_peak_bytes, None);
    assert_eq!(
        payload.metadata.local_relation_engine.as_deref(),
        Some("datafusion")
    );
    assert_eq!(payload.metadata.duckdb_registration_strategy, None);
    assert_eq!(payload.metadata.registered_input_batch_count, Some(1));
    assert!(
        payload
            .metadata
            .registered_input_row_count
            .is_some_and(|count| count > 0)
    );
    assert!(payload.metadata.registration_time_ms.is_some());
    assert!(payload.metadata.local_query_execution_time_ms.is_some());
    assert!(
        payload
            .batches
            .iter()
            .flat_map(|batch| batch.rows.iter())
            .any(|row| row.get("path").and_then(serde_json::Value::as_str)
                == Some("blueprint/overview.md"))
    );
    assert!(
        payload
            .batches
            .iter()
            .flat_map(|batch| batch.rows.iter())
            .any(
                |row| row.get("heading_path").and_then(serde_json::Value::as_str)
                    == Some("Blueprint/Scope")
            )
    );
    Ok(())
}

#[cfg(feature = "duckdb")]
#[tokio::test]
async fn queries_bounded_work_markdown_payload_with_duckdb_local_relation_engine() -> TestResult {
    let temp_dir = tempdir()?;
    let root = temp_dir.path();
    write_bounded_work_fixture(
        root,
        "blueprint/overview.md",
        "# Blueprint\n\n## Scope\n- [ ] Keep aligned\n",
        "plan/steps.md",
        "# Plan\n\n## Validate\n- [ ] Query markdown\n",
    )?;

    let query_engine = DuckDbLocalRelationEngine::from_runtime(SearchDuckDbRuntimeConfig {
        enabled: true,
        database_path: DuckDbDatabasePath::InMemory,
        temp_directory: root.join(".cache/duckdb-markdown/tmp"),
        threads: 2,
        materialize_threshold_rows: DEFAULT_SEARCH_DUCKDB_MATERIALIZE_THRESHOLD_ROWS,
        prefer_virtual_arrow: DEFAULT_SEARCH_DUCKDB_PREFER_VIRTUAL_ARROW,
    })
    .map_err(std::io::Error::other)?;

    let payload = query_bounded_work_markdown_payload_with_engine(
        root,
        "select path, heading_path from markdown where surface = 'plan' order by path, heading_path",
        &query_engine,
    )
    .await
    .map_err(std::io::Error::other)?;
    assert_eq!(
        query_engine.registered_strategy("markdown")?,
        Some(DuckDbRegistrationStrategy::VirtualArrow)
    );

    assert_eq!(
        payload.metadata.registered_tables,
        vec!["markdown".to_string()]
    );
    assert_eq!(payload.metadata.registered_table_count, 1);
    assert_eq!(payload.metadata.result_batch_count, 1);
    assert!(
        payload
            .metadata
            .registered_input_bytes
            .is_some_and(|bytes| bytes > 0)
    );
    assert!(payload.metadata.result_bytes.is_some_and(|bytes| bytes > 0));
    assert_eq!(
        payload
            .metadata
            .local_relation_materialization_state
            .as_deref(),
        Some("virtual")
    );
    assert!(payload.metadata.local_temp_storage_peak_bytes.is_some());
    assert_eq!(
        payload.metadata.local_relation_engine.as_deref(),
        Some("duckdb")
    );
    assert_eq!(
        payload.metadata.duckdb_registration_strategy.as_deref(),
        Some("virtual_arrow")
    );
    assert_eq!(payload.metadata.registered_input_batch_count, Some(1));
    assert!(
        payload
            .metadata
            .registered_input_row_count
            .is_some_and(|count| count > 0)
    );
    assert!(payload.metadata.registration_time_ms.is_some());
    assert!(payload.metadata.local_query_execution_time_ms.is_some());
    assert!(
        payload
            .batches
            .iter()
            .flat_map(|batch| batch.rows.iter())
            .any(|row| row.get("path").and_then(serde_json::Value::as_str)
                == Some("plan/steps.md"))
    );
    assert!(
        payload
            .batches
            .iter()
            .flat_map(|batch| batch.rows.iter())
            .any(
                |row| row.get("heading_path").and_then(serde_json::Value::as_str)
                    == Some("Plan/Validate")
            )
    );
    Ok(())
}

#[cfg(feature = "duckdb")]
#[tokio::test]
async fn queries_bounded_work_markdown_payload_with_duckdb_materialized_local_relation_engine()
-> TestResult {
    let temp_dir = tempdir()?;
    let root = temp_dir.path();
    write_bounded_work_fixture(
        root,
        "blueprint/overview.md",
        "# Blueprint\n\n## Scope\n- [ ] Keep aligned\n",
        "plan/steps.md",
        "# Plan\n\n## Validate\n- [ ] Query markdown\n",
    )?;

    let query_engine = DuckDbLocalRelationEngine::from_runtime(SearchDuckDbRuntimeConfig {
        enabled: true,
        database_path: DuckDbDatabasePath::InMemory,
        temp_directory: root.join(".cache/duckdb-markdown-materialized/tmp"),
        threads: 2,
        materialize_threshold_rows: 0,
        prefer_virtual_arrow: true,
    })
    .map_err(std::io::Error::other)?;

    let payload = query_bounded_work_markdown_payload_with_engine(
        root,
        "select path, heading_path from markdown where surface = 'plan' order by path, heading_path",
        &query_engine,
    )
    .await
    .map_err(std::io::Error::other)?;
    assert_eq!(
        query_engine.registered_strategy("markdown")?,
        Some(DuckDbRegistrationStrategy::MaterializedAppender)
    );
    assert_eq!(
        payload
            .metadata
            .local_relation_materialization_state
            .as_deref(),
        Some("materialized")
    );
    assert!(payload.metadata.local_temp_storage_peak_bytes.is_some());
    assert_eq!(
        payload.metadata.duckdb_registration_strategy.as_deref(),
        Some("materialized_appender")
    );
    assert!(payload.metadata.result_bytes.is_some_and(|bytes| bytes > 0));
    assert!(
        payload
            .batches
            .iter()
            .flat_map(|batch| batch.rows.iter())
            .any(|row| row.get("path").and_then(serde_json::Value::as_str)
                == Some("plan/steps.md"))
    );
    Ok(())
}

use anyhow::{Context, Result};

pub(super) fn initialize_agent_org_tasks_table(
    connection: &xiuxian_db_store::duckdb_crate::Connection,
) -> Result<()> {
    connection
        .execute_batch(
            r"
DROP TABLE IF EXISTS agent_org_tasks;
CREATE TABLE agent_org_tasks (
    orgid VARCHAR PRIMARY KEY,
    source_path VARCHAR NOT NULL,
    source_line UBIGINT NOT NULL,
    source_column UBIGINT NOT NULL,
    source_range_start UBIGINT NOT NULL,
    source_range_end UBIGINT NOT NULL,
    level UBIGINT NOT NULL,
    outline_path_json VARCHAR NOT NULL,
    title VARCHAR NOT NULL,
    todo_state VARCHAR,
    is_done BOOLEAN NOT NULL,
    tags_json VARCHAR NOT NULL,
    effective_tags_json VARCHAR NOT NULL,
    scheduled VARCHAR,
    scheduled_repeater_json VARCHAR,
    deadline VARCHAR,
    deadline_repeater_json VARCHAR,
    closed VARCHAR,
    archived BOOLEAN NOT NULL,
    archive_location VARCHAR,
    properties_json VARCHAR NOT NULL
);
",
        )
        .with_context(|| "failed to initialize agent_org_tasks DuckDB table")
}

pub(super) fn initialize_agent_org_memory_objects_table(
    connection: &xiuxian_db_store::duckdb_crate::Connection,
) -> Result<()> {
    connection
        .execute_batch(
            r"
DROP TABLE IF EXISTS agent_org_memory_objects;
CREATE TABLE agent_org_memory_objects (
    orgid VARCHAR NOT NULL,
    source_path VARCHAR NOT NULL,
    source_line UBIGINT NOT NULL,
    source_range_start UBIGINT NOT NULL,
    source_range_end UBIGINT NOT NULL,
    object_index UBIGINT NOT NULL,
    kind VARCHAR NOT NULL,
    facet VARCHAR NOT NULL,
    source_kind VARCHAR NOT NULL,
    source_key VARCHAR NOT NULL,
    value VARCHAR NOT NULL,
    PRIMARY KEY (orgid, source_kind, source_key, object_index)
);
",
        )
        .with_context(|| "failed to initialize agent_org_memory_objects DuckDB table")
}

pub(super) fn initialize_agent_org_elements_table(
    connection: &xiuxian_db_store::duckdb_crate::Connection,
) -> Result<()> {
    connection
        .execute_batch(
            r"
DROP TABLE IF EXISTS agent_org_elements;
CREATE TABLE agent_org_elements (
    source_path VARCHAR NOT NULL,
    source_modified_unix_ms UBIGINT NOT NULL,
    ordinal UBIGINT NOT NULL,
    category VARCHAR NOT NULL,
    kind VARCHAR NOT NULL,
    affiliated_name VARCHAR,
    outline_path_json VARCHAR NOT NULL,
    context VARCHAR NOT NULL,
    summary_json VARCHAR NOT NULL,
    language VARCHAR,
    source_start_line UBIGINT NOT NULL,
    source_start_column UBIGINT NOT NULL,
    source_end_line UBIGINT NOT NULL,
    source_end_column UBIGINT NOT NULL,
    source_range_start UBIGINT NOT NULL,
    source_range_end UBIGINT NOT NULL,
    source_raw VARCHAR NOT NULL,
    PRIMARY KEY (source_path, ordinal)
);
",
        )
        .with_context(|| "failed to initialize agent_org_elements DuckDB table")
}

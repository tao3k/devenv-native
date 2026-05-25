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

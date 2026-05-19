//! `DuckDB` storage and materialization for Org agent tasks.

mod connection;
mod materialize;
mod query;
mod refresh;
mod schema;

pub(super) use connection::open_read_model_connection;
pub(super) use query::query_agent_org_task_rows;
pub(super) use refresh::refresh_agent_org_read_model;
